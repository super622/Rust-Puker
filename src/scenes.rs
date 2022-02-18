use ggez::{
    graphics::{self, DrawMode, Color, DrawParam, Rect, Mesh},
    Context,
    GameResult,
    mint::{Point2},
    event::{KeyCode, MouseButton},
    input::{self, keyboard, mouse},
    audio::{SoundSource},
    conf::FullscreenType,
};
use glam::f32::{Vec2};
use std::{
    rc::Rc,
    cell::{RefCell, Ref, RefMut},
    any::Any,
};

use crate::{
    player::*,
    shots::*,
    utils::*,
    dungeon::*,
    consts::*,
    traits::*,
    ui_elements::*,
    items::*,
};

pub struct PlayScene {
    config: Rc<RefCell<Config>>,
    player: Player,
    dungeon: Dungeon,
    cur_room: (usize, usize),
    overlay: Overlay,
}

impl PlayScene {
    pub fn new(config: &Rc<RefCell<Config>>) -> Self {
        let (sw, sh) = (config.borrow().screen_width, config.borrow().screen_height);

        let config = Rc::clone(config);
        let player = Player::default();
        let dungeon = Dungeon::generate_dungeon((sw, sh), 1);
        let cur_room = Dungeon::get_start_room_coords();
        let overlay = Overlay::new(&player, &dungeon, cur_room);

        Self {
            config,
            player,
            dungeon,
            cur_room,
            overlay,
        }
    }

    fn handle_input(&mut self, ctx: &mut Context) -> GameResult {
        let room = self.dungeon.get_room_mut(self.cur_room)?.unwrap();
        self.player.props.forward = Vec2::ZERO;
        self.player.props.translation = Vec2::ZERO;

        if keyboard::is_key_pressed(ctx, KeyCode::W) {
            self.player.props.translation.y -= 1.;
        }
        if keyboard::is_key_pressed(ctx, KeyCode::S) {
            self.player.props.translation.y += 1.;
        }
        if keyboard::is_key_pressed(ctx, KeyCode::A) {
            self.player.props.translation.x -= 1.;
        }
        if keyboard::is_key_pressed(ctx, KeyCode::D) {
            self.player.props.translation.x += 1.;
        }
        if keyboard::is_key_pressed(ctx, KeyCode::Up) {
            self.player.props.forward = Vec2::new(0., -1.);
            self.player.shoot(&mut room.shots);
        }
        if keyboard::is_key_pressed(ctx, KeyCode::Down) {
            self.player.props.forward = Vec2::new(0., 1.);
            self.player.shoot(&mut room.shots);
        }
        if keyboard::is_key_pressed(ctx, KeyCode::Left) {
            self.player.props.forward = Vec2::new(-1., 0.);
            self.player.shoot(&mut room.shots);
        }
        if keyboard::is_key_pressed(ctx, KeyCode::Right) {
            self.player.props.forward = Vec2::new(1., 0.);
            self.player.shoot(&mut room.shots);
        }
        if mouse::button_pressed(ctx, MouseButton::Left) {
            self.player.props.forward = mouse_relative_forward(self.player.props.pos.0, mouse::position(ctx), &self.config.borrow());
            self.player.shoot(&mut room.shots);
        }
        if keyboard::is_key_pressed(ctx, KeyCode::Space) {
            if self.player.use_item() {
                if let ItemTag::Active(act) = &self.player.item.unwrap().tag {
                    match act {
                        ItemActive::Heal(_) => self.config.borrow_mut().assets.audio.get_mut("heal_sound").unwrap().play(ctx)?,
                    }
                }
            }
        }

        Ok(())
    }

    fn handle_block_collisions(&mut self, ctx: &mut Context, delta_time: f32) -> GameResult {
        let (sw, sh) = (self.config.borrow().screen_width, self.config.borrow().screen_height);
        let (mut cp, mut cn) = (Vec2::ZERO, Vec2::ZERO);
        let mut ct = 0.;
        let room = self.dungeon.get_room_mut(self.cur_room)?.unwrap();
        let mut next_level = false;

        for o in room.obstacles.iter_mut() {
            let obst = o.as_any_mut().downcast_mut::<Block>().unwrap();

            if dynamic_circle_vs_rect(&self.player.get_bcircle(sw, sh), &obst.get_bbox(sw, sh), &mut cp, &mut cn, &mut ct, delta_time) {
                match obst.tag {
                    BlockTag::Door { is_open, dir, connects_to } => {
                        if is_open {
                            if (self.player.props.pos.0 - obst.pos.0).length() < obst.get_bcircle(sw, sh).1 {
                                self.cur_room = connects_to;
                                let (di, dj) = pos_to_room_coords(obst.get_pos(), sw, sh);
                                self.player.props.pos.0 = match dir {
                                        Direction::North => room_coords_to_pos(ROOM_HEIGHT - di - 2, dj, sw, sh),
                                        Direction::South => room_coords_to_pos(ROOM_HEIGHT - di, dj, sw, sh),
                                        Direction::West => room_coords_to_pos(di, ROOM_WIDTH - dj - 2, sw, sh),
                                        Direction::East => room_coords_to_pos(di, ROOM_WIDTH - dj, sw, sh),
                                };
                                self.player.props.velocity = Vec2::ZERO;
                                self.player.afterlock_cooldown = PLAYER_AFTERLOCK_COOLDOWN;
                            }
                        }
                        else { self.player.props.pos.0 -= cn.normalize() * ct; }
                    },
                    BlockTag::Spikes => {
                        if obst.get_bbox(sw, sh).contains(self.player.props.pos) { self.player.damage(1.); }
                    },
                    BlockTag::Hatch(is_open) => {
                        if is_open {
                            next_level = true;
                            self.config.borrow_mut().level += 1;
                            self.config.borrow_mut().current_state = State::Transition;
                        }
                    },
                    BlockTag::Pedestal(Some(mut item)) => {
                        self.config.borrow_mut().assets.audio.get_mut("wow_sound").unwrap().play(ctx)?;
                        match item.tag {
                            ItemTag::Passive(_) => {
                                item.affect_player(&mut self.player);
                                obst.tag = BlockTag::Pedestal(None);
                            },
                            ItemTag::Active(_) => {
                                let temp = Some(item); 
                                obst.tag = BlockTag::Pedestal(self.player.item);
                                self.player.item = temp;
                            }
                        }

                        self.player.props.pos.0 -= cn.normalize() * ct;
                    },
                    _ => self.player.props.pos.0 -= cn.normalize() * ct,
                }
            }

            match obst.tag {
                BlockTag::Hatch(_) => (),
                _ => {
                    for e in room.enemies.iter_mut() {
                        if dynamic_circle_vs_rect(&e.get_bcircle(sw, sh), &o.get_bbox(sw, sh), &mut cp, &mut cn, &mut ct, delta_time) {
                            e.set_pos(e.get_pos() - cn.normalize() * ct);
                        }
                    }

                    for d in room.drops.iter_mut() {
                        if dynamic_circle_vs_rect(&d.get_bcircle(sw, sh), &o.get_bbox(sw, sh), &mut cp, &mut cn, &mut ct, delta_time) {
                            d.set_velocity(d.get_velocity() - cn.normalize() * ct);
                        }
                    }
                },
            }
        }

        if next_level {
            self.dungeon = Dungeon::generate_dungeon((sw, sh), self.dungeon.get_level() + 1);
            self.cur_room = Dungeon::get_start_room_coords();
            self.player.props.pos = Vec2::new(sw / 2., sh / 2.).into();
        }

        Ok(())
    }

    fn handle_shot_collisions(&mut self, ctx: &mut Context, _delta_time: f32) -> GameResult {
        let (sw, sh) = (self.config.borrow().screen_width, self.config.borrow().screen_height);
        let room = self.dungeon.get_room_mut(self.cur_room)?.unwrap();

        room.shots = room.shots.clone().into_iter().filter(|s| {
            match s.tag {
                ShotTag::Player => {
                    for enemy in room.enemies.iter_mut() {
                        let (mut vel1, mut vel2) = (Vec2::ZERO, Vec2::ZERO);
                        if dynamic_circle_vs_circle(&s.get_bcircle(sw, sh), &s.get_velocity(), &enemy.get_bcircle(sw, sh), &enemy.get_velocity(), &mut vel1, &mut vel2, _delta_time) {
                            let _ = self.config.borrow_mut().assets.audio.get_mut("bubble_pop_sound").unwrap().play(ctx);
                            enemy.damage(s.damage);
                            enemy.set_velocity(enemy.get_velocity() + vel2);
                            return false;
                        }
                    }
                },
                ShotTag::Enemy => {
                    if circle_vs_circle(&s.get_bcircle(sw, sh), &self.player.get_bcircle(sw, sh)) {
                        let _ = self.config.borrow_mut().assets.audio.get_mut("bubble_pop_sound").unwrap().play(ctx);
                        self.player.damage(s.damage);
                        return false;
                    }
                },
            };

            for obst in room.obstacles.iter() {
                let (mut cp, mut cn) = (Vec2::ZERO, Vec2::ZERO);
                let mut ct = 0.;
                match obst.get_tag() {
                    BlockTag::Hatch(_) | BlockTag::Spikes => (),
                    _ => if dynamic_circle_vs_rect(&s.get_bcircle(sw, sh), &obst.get_bbox(sw, sh), &mut cp, &mut cn, &mut ct, _delta_time) {
                        let _ = self.config.borrow_mut().assets.audio.get_mut("bubble_pop_sound").unwrap().play(ctx);
                        return false;
                    },
                }
            }
            true
        }).collect();

        Ok(())
    }

    fn handle_environment_collisions(&mut self, ctx: &mut Context, _delta_time: f32) -> GameResult {
        let (sw, sh) = (self.config.borrow().screen_width, self.config.borrow().screen_height);
        let room = self.dungeon.get_room_mut(self.cur_room)?.unwrap();

        for e in room.enemies.iter_mut() {
            if circle_vs_circle(&e.get_bcircle(sw, sh), &self.player.get_bcircle(sw, sh)) {
                resolve_environment_collision(&mut **e, &mut self.player, sw, sh, _delta_time);
                self.player.damage(e.get_damage());
            }
        }

        for d in room.drops.iter_mut() {
            if circle_vs_circle(&d.get_bcircle(sw, sh), &self.player.get_bcircle(sw, sh)) {
                if !d.affect_player(&mut self.player) {
                    resolve_environment_collision(d, &mut self.player, sw, sh, _delta_time);
                }
                else {
                    match d.tag {
                        CollectableTag::RedHeart(_) => self.config.borrow_mut().assets.audio.get_mut("heal_sound").unwrap().play(ctx)?,
                        _ => self.config.borrow_mut().assets.audio.get_mut("power_up_sound").unwrap().play(ctx)?,
                    }
                }
            }
        }

        Ok(())
    }
}

impl Scene for PlayScene {
    fn update(&mut self, ctx: &mut Context, delta_time: f32) -> GameResult {
        self.handle_input(ctx)?;

        self.handle_block_collisions(ctx, delta_time)?;

        self.handle_environment_collisions(ctx, delta_time)?;

        self.handle_shot_collisions(ctx, delta_time)?;

        self.dungeon.update_rooms_state(self.cur_room)?;
        self.dungeon.get_room_mut(self.cur_room)?.unwrap().update(ctx, &mut self.config.borrow_mut(), &mut self.player, delta_time)?;

        self.player.update(ctx, &mut self.config.borrow_mut(), delta_time)?;

        self.update_ui_vars(ctx)?;

        match self.player.state {
            ActorState::Dead => self.config.borrow_mut().current_state = State::Dead,
            _ => (),
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        self.dungeon.get_room(self.cur_room)?.unwrap().draw(ctx, &mut self.config.borrow_mut())?;

        self.player.draw(ctx, &mut self.config.borrow_mut())?;

        self.overlay.draw(ctx, &mut self.config.borrow_mut())?;

        Ok(())
    }

    fn key_down_event(&mut self, _ctx: &mut Context, _keycode: KeyCode, _keymod: input::keyboard::KeyMods, _repeat: bool) {
        let cur = self.config.borrow().current_state;

        match _keycode {
            KeyCode::Escape => self.config.borrow_mut().current_state = State::PauseMenu,
            _ => (),
        };

        self.config.borrow_mut().previous_state = cur;
    }

    fn get_conf(&self) -> Option<Ref<Config>> { Some(self.config.borrow()) }

    fn get_conf_mut(&mut self) -> Option<RefMut<Config>> { Some(self.config.borrow_mut()) }

    fn update_ui_vars(&mut self, ctx: &mut Context) -> GameResult {
        self.overlay.update_vars(&self.player, &self.dungeon, self.cur_room);
        self.overlay.update(ctx, &mut self.config.borrow_mut())?;

        Ok(())
    }

    fn as_any(&self) -> &dyn Any { self }

    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

pub struct MainMenuScene {
    config: Rc<RefCell<Config>>,
    ui_elements: Vec<Box<dyn UIElement>>,
}

impl MainMenuScene {
    pub fn new(config: &Rc<RefCell<Config>>) -> Self {
        let config = Rc::clone(config);
        let ui_elements: Vec<Box<dyn UIElement>> = vec![
            Box::new(Button {
                pos: Point2 { x: 0.5, y: 0.3},
                action: ButtonAction::ChangeState(Some(State::New)),
                tag: UIElementTag::State,
                text: Some(TextSprite {
                    text: String::from("Play"),
                    font: *config.borrow().assets.fonts.get("enigma").unwrap(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            Box::new(Button {
                pos: Point2 { x: 0.5, y: 0.5},
                action: ButtonAction::ChangeState(Some(State::Options)),
                tag: UIElementTag::State,
                text: Some(TextSprite {
                    text: String::from("Options"),
                    font: *config.borrow().assets.fonts.get("enigma").unwrap(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            Box::new(Button {
                pos: Point2 { x: 0.5, y: 0.7},
                action: ButtonAction::ChangeState(Some(State::Quit)),
                tag: UIElementTag::State,
                text: Some(TextSprite {
                    text: String::from("Quit"),
                    font: *config.borrow().assets.fonts.get("enigma").unwrap(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
        ];

        Self {
            config,
            ui_elements,
        }
    }
}

impl Scene for MainMenuScene {
    fn update(&mut self, ctx: &mut Context, _delta_time: f32) -> GameResult {
        for e in self.ui_elements.iter_mut() {
            e.update(ctx, &mut self.config.borrow_mut())?;
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let (sw, sh) = (self.config.borrow().screen_width, self.config.borrow().screen_height);
        let logo_rect = self.config.borrow().assets.sprites.get("puker_logo").unwrap().dimensions();
        let draw_param = DrawParam::default()
            .scale([sw / logo_rect.w, sh / logo_rect.h]);

        graphics::draw(ctx, self.config.borrow().assets.sprites.get("puker_logo").unwrap(), draw_param)?;

        for e in self.ui_elements.iter_mut() {
            e.draw(ctx, &mut self.config.borrow_mut())?;
        }

        Ok(())
    }

    fn get_conf(&self) -> Option<Ref<Config>> { Some(self.config.borrow()) }

    fn get_conf_mut(&mut self) -> Option<RefMut<Config>> { Some(self.config.borrow_mut()) }

    fn get_ui_elements(&self) -> Option<&Vec<Box<dyn UIElement>>> { Some(&self.ui_elements) }

    fn get_ui_elements_mut(&mut self) -> Option<&mut Vec<Box<dyn UIElement>>> { Some(&mut self.ui_elements) }  

    fn mouse_button_down_event(&mut self, ctx: &mut Context, _button: MouseButton, _x: f32, _y: f32) {
        let (sw, sh) = (self.config.borrow().screen_width, self.config.borrow().screen_height);
        let (ww, wh) = (self.config.borrow().window_width, self.config.borrow().window_height);

        if _button == MouseButton::Left {
            for e in self.ui_elements.iter() {
                if e.mouse_overlap(ctx, _x, _y, sw, sh, ww, wh) {
                    if let Some(b) = e.as_any().downcast_ref::<Button>() {
                        match b.action.clone() {
                            ButtonAction::ChangeState(s) => change_scene(&mut self.config.borrow_mut(), s), 
                            _ => (),
                        }
                    }
                }
            }
        }
    }

    fn as_any(&self) -> &dyn Any { self }

    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

pub struct PauseMenuScene {
    config: Rc<RefCell<Config>>,
    ui_elements: Vec<Box<dyn UIElement>>,
}

impl PauseMenuScene {
    pub fn new(config: &Rc<RefCell<Config>>) -> Self {
        let config = Rc::clone(config);
        let ui_elements: Vec<Box<dyn UIElement>> = vec![
            Box::new(Button {
                pos: Point2 { x: 0.5, y: 0.2},
                action: ButtonAction::ChangeState(Some(State::Play)),
                tag: UIElementTag::State,
                text: Some(TextSprite {
                    text: String::from("Continue"),
                    font: *config.borrow().assets.fonts.get("enigma").unwrap(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            Box::new(Button {
                pos: Point2 { x: 0.5, y: 0.4},
                action: ButtonAction::ChangeState(Some(State::New)),
                tag: UIElementTag::State,
                text: Some(TextSprite {
                    text: String::from("New Game"),
                    font: *config.borrow().assets.fonts.get("enigma").unwrap(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            Box::new(Button {
                pos: Point2 { x: 0.5, y: 0.6},
                action: ButtonAction::ChangeState(Some(State::Options)),
                tag: UIElementTag::State,
                text: Some(TextSprite {
                    text: String::from("Options"),
                    font: *config.borrow().assets.fonts.get("enigma").unwrap(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            Box::new(Button {
                pos: Point2 { x: 0.5, y: 0.8},
                action: ButtonAction::ChangeState(Some(State::MainMenu)),
                tag: UIElementTag::State,
                text: Some(TextSprite {
                    pos: Point2 { x: 0.5, y: 0.8},
                    text: String::from("Main Menu"),
                    font: *config.borrow().assets.fonts.get("enigma").unwrap(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
        ];

        Self {
            config,
            ui_elements,
        }
    }
}

impl Scene for PauseMenuScene {
    fn update(&mut self, ctx: &mut Context, _delta_time: f32) -> GameResult {
        for e in self.ui_elements.iter_mut() {
            e.update(ctx, &mut self.config.borrow_mut())?;
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let (sw, sh) = (self.config.borrow().screen_width, self.config.borrow().screen_height);

        let curtain = Mesh::new_rectangle(
            ctx,
            DrawMode::fill(),
            Rect::new(0., 0., sw, sh),
            [0.1, 0.2, 0.3, 0.3].into()
        )?;

        graphics::draw(ctx, &curtain, DrawParam::default())?;

        for e in self.ui_elements.iter_mut() {
            e.draw(ctx, &mut self.config.borrow_mut())?;
        }

        Ok(())
    }

    fn key_down_event(&mut self, _ctx: &mut Context, _keycode: KeyCode, _keymod: input::keyboard::KeyMods, _repeat: bool) {
        let cur = self.config.borrow().current_state;

        match _keycode {
            KeyCode::Escape => self.config.borrow_mut().current_state = State::Play,
            _ => (),
        }

        self.config.borrow_mut().previous_state = cur;
    }

    fn get_conf(&self) -> Option<Ref<Config>> { Some(self.config.borrow()) }

    fn get_conf_mut(&mut self) -> Option<RefMut<Config>> { Some(self.config.borrow_mut()) }

    fn get_ui_elements(&self) -> Option<&Vec<Box<dyn UIElement>>> { Some(&self.ui_elements) }

    fn get_ui_elements_mut(&mut self) -> Option<&mut Vec<Box<dyn UIElement>>> { Some(&mut self.ui_elements) }  

    fn mouse_button_down_event(&mut self, ctx: &mut Context, _button: MouseButton, _x: f32, _y: f32) {
        let (sw, sh) = (self.config.borrow().screen_width, self.config.borrow().screen_height);
        let (ww, wh) = (self.config.borrow().window_width, self.config.borrow().window_height);

        if _button == MouseButton::Left {
            for e in self.ui_elements.iter() {
                if e.mouse_overlap(ctx, _x, _y, sw, sh, ww, wh) {
                    if let Some(b) = e.as_any().downcast_ref::<Button>() {
                        match b.action.clone() {
                            ButtonAction::ChangeState(s) => change_scene(&mut self.config.borrow_mut(), s), 
                            _ => (),
                        }
                    }
                }
            }
        }
    }

    fn as_any(&self) -> &dyn Any { self }

    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

pub struct DeadScene {
    config: Rc<RefCell<Config>>,
    ui_elements: Vec<Box<dyn UIElement>>,
}

impl DeadScene {
    pub fn new(config: &Rc<RefCell<Config>>) -> Self {
        let config = Rc::clone(config);
        let ui_elements: Vec<Box<dyn UIElement>> = vec![
            Box::new(TextSprite {
                pos: Point2 { x: 0.5, y: 0.3},
                text: String::from("YOU DIED"),
                font_size: BUTTON_TEXT_FONT_SIZE * 2.,
                color: Color::RED,
                ..Default::default()
            }),
            Box::new(Button {
                pos: Point2 { x: 0.5, y: 0.5},
                action: ButtonAction::ChangeState(Some(State::New)),
                tag: UIElementTag::State,
                text: Some(TextSprite {
                    text: String::from("Try Again"),
                    font: *config.borrow().assets.fonts.get("enigma").unwrap(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            Box::new(Button {
                pos: Point2 { x: 0.5, y: 0.7},
                action: ButtonAction::ChangeState(Some(State::Quit)),
                tag: UIElementTag::State,
                text: Some(TextSprite {
                    text: String::from("Quit"),
                    font: *config.borrow().assets.fonts.get("enigma").unwrap(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
        ];

        Self {
            config,
            ui_elements,
        }
    }
}

impl Scene for DeadScene {
    fn update(&mut self, ctx: &mut Context, _delta_time: f32) -> GameResult {
        for e in self.ui_elements.iter_mut() {
            e.update(ctx, &mut self.config.borrow_mut())?;
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let (sw, sh) = (self.config.borrow().screen_width, self.config.borrow().screen_height);

        let curtain = Mesh::new_rectangle(
            ctx,
            DrawMode::fill(),
            Rect::new(0., 0., sw, sh),
            [0.1, 0.2, 0.3, 0.3].into()
        )?;

        graphics::draw(ctx, &curtain, DrawParam::default())?;

        for e in self.ui_elements.iter_mut() {
            e.draw(ctx, &mut self.config.borrow_mut())?;
        }

        Ok(())
    }

    fn get_conf(&self) -> Option<Ref<Config>> { Some(self.config.borrow()) }

    fn get_conf_mut(&mut self) -> Option<RefMut<Config>> { Some(self.config.borrow_mut()) }

    fn get_ui_elements(&self) -> Option<&Vec<Box<dyn UIElement>>> { Some(&self.ui_elements) }

    fn get_ui_elements_mut(&mut self) -> Option<&mut Vec<Box<dyn UIElement>>> { Some(&mut self.ui_elements) }  

    fn mouse_button_down_event(&mut self, ctx: &mut Context, _button: MouseButton, _x: f32, _y: f32) {
        let (sw, sh) = (self.config.borrow().screen_width, self.config.borrow().screen_height);
        let (ww, wh) = (self.config.borrow().window_width, self.config.borrow().window_height);

        if _button == MouseButton::Left {
            for e in self.ui_elements.iter() {
                if e.mouse_overlap(ctx, _x, _y, sw, sh, ww, wh) {
                    if let Some(b) = e.as_any().downcast_ref::<Button>() {
                        match b.action.clone() {
                            ButtonAction::ChangeState(s) => change_scene(&mut self.config.borrow_mut(), s), 
                            _ => (),
                        }
                    }
                }
            }
        }
    }

    fn as_any(&self) -> &dyn Any { self }

    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

pub struct OptionsScene {
    config: Rc<RefCell<Config>>,
    ui_elements: Vec<Box<dyn UIElement>>,
}

impl OptionsScene {
    pub fn new(config: &Rc<RefCell<Config>>) -> Self {
        let config = Rc::clone(config);
        let ui_elements: Vec<Box<dyn UIElement>> = vec![
            Box::new(TextSprite {
                pos: Point2 { x: 0.5, y: 0.2},
                text: String::from("OPTIONS"),
                font_size: BUTTON_TEXT_FONT_SIZE * 2.,
                color: Color::RED,
                ..Default::default()
            }),
            Box::new(CheckBox {
                pos: Point2 { x: 0.3, y: 0.4 },
                tag: UIElementTag::WindowMode,
                ..Default::default()
            }),
            Box::new(TextSprite {
                pos: Point2 { x: 0.6, y: 0.4},
                text: String::from("Fullscreen"),
                color: Color::BLUE,
                background: Color::from([1.; 4]),
                ..Default::default()
            }),
            Box::new(Button {
                pos: Point2 { x: 0.5, y: 0.8},
                action: ButtonAction::ChangeState(None),
                tag: UIElementTag::State,
                text: Some(TextSprite {
                    text: String::from("Back"),
                    font: *config.borrow().assets.fonts.get("enigma").unwrap(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            Box::new(TextSprite {
                pos: Point2 { x: 0.7, y: 0.6},
                text: String::from("Volume"),
                color: Color::BLUE,
                background: Color::from([1.; 4]),
                ..Default::default()
            }),
            Box::new(Slider {
                pos: Point2 { x: 0.3, y: 0.6 },
                value: config.borrow().volume * 100.,
                tag: UIElementTag::Volume,
                border: Border {
                    stroke: 3.,
                    radius: 0.,
                    ..Default::default()
                },
                decrease_button: Button {
                    action: ButtonAction::ChangeVolume(-1),
                    tag: UIElementTag::Volume,
                    text: Some(TextSprite {
                        text: String::from("<<"),
                        font_size: BUTTON_TEXT_FONT_SIZE * 0.5,
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                increase_button: Button {
                    action: ButtonAction::ChangeVolume(1),
                    tag: UIElementTag::Volume,
                    text: Some(TextSprite {
                        text: String::from(">>"),
                        font_size: BUTTON_TEXT_FONT_SIZE * 0.5,
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                ..Default::default()
            }),
        ];

        Self {
            config,
            ui_elements,
        }
    }
}

impl Scene for OptionsScene {
    fn update(&mut self, ctx: &mut Context, _delta_time: f32) -> GameResult {
        for e in self.ui_elements.iter_mut() {
            e.update(ctx, &mut self.config.borrow_mut())?;
        }

        self.update_ui_vars(ctx)?;

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let (sw, sh) = (self.config.borrow().screen_width, self.config.borrow().screen_height);

        let curtain = Mesh::new_rectangle(
            ctx,
            DrawMode::fill(),
            Rect::new(0., 0., sw, sh),
            [0.1, 0.2, 0.3, 0.3].into()
        )?;

        graphics::draw(ctx, &curtain, DrawParam::default())?;

        for e in self.ui_elements.iter_mut() {
            e.draw(ctx, &mut self.config.borrow_mut())?;
        }

        Ok(())
    }

    fn key_down_event(&mut self, _ctx: &mut Context, _keycode: KeyCode, _keymod: input::keyboard::KeyMods, _repeat: bool) {
        let prev = self.config.borrow().previous_state;
        let cur = self.config.borrow().current_state;

        match _keycode {
            KeyCode::Escape => self.config.borrow_mut().current_state = prev,
            _ => (),
        };

        self.config.borrow_mut().previous_state = cur;
    }

    fn mouse_button_down_event(&mut self, ctx: &mut Context, _button: MouseButton, _x: f32, _y: f32) {
        let (sw, sh) = (self.config.borrow().screen_width, self.config.borrow().screen_height);
        let (ww, wh) = (self.config.borrow().window_width, self.config.borrow().window_height);
        let overlapped = self.get_overlapped_idx(ctx, _x, _y);

        if _button == MouseButton::Left {
            match overlapped {
                Some(i) => {
                    if let Some(b) = self.ui_elements[i].as_any_mut().downcast_mut::<Button>() {
                        b.clicked = true;
                        match b.action.clone() {
                            ButtonAction::ChangeState(s) => change_scene(&mut self.config.borrow_mut(), s), 
                            _ => (),
                        }
                    }
                    else if let Some(s) = self.ui_elements[i].as_any_mut().downcast_mut::<Slider>() {
                        let step = s.get_step();
                        s.last_mx = _x;

                        match s.get_overlapped_mut(ctx, _x, _y, sw, sh, ww, wh) {
                            Some(b) => {
                                b.clicked = true;
                                match b.action {
                                    ButtonAction::ChangeVolume(sign) => change_volume(&mut self.config.borrow_mut(), (sign as f32) * step),
                                    _ => (),
                                };
                            },
                            None => (),
                        }
                    }
                    else if let Some(c) = self.ui_elements[i].as_any().downcast_ref::<CheckBox>() {
                        let _ = match c.checked {
                            true => {
                                self.config.borrow_mut().window_mode = FullscreenType::Windowed;
                                graphics::set_fullscreen(ctx, FullscreenType::Windowed)
                            }
                            false => {
                                self.config.borrow_mut().window_mode = FullscreenType::True;
                                graphics::set_fullscreen(ctx, FullscreenType::True)
                            }
                        };
                    }
                },
                None => (),
            }
        }
    }

    fn mouse_button_up_event(&mut self, _ctx: &mut Context, _button: MouseButton, _x: f32, _y: f32) {
        if _button == MouseButton::Left {
            for e in self.ui_elements.iter_mut() {
                if let Some(b) = e.as_any_mut().downcast_mut::<Button>() {
                    b.clicked = false;
                }
                else if let Some(s) = e.as_any_mut().downcast_mut::<Slider>() {
                    s.slider_button.clicked = false;
                    s.increase_button.clicked = false;
                    s.decrease_button.clicked = false;
                }
            }
        }
    }

    fn mouse_motion_event(&mut self, _ctx: &mut Context, _x: f32, _y: f32, _dx: f32, _dy: f32) {
        let (sw, sh) = (self.config.borrow().screen_width, self.config.borrow().screen_height);
        let (ww, wh) = (self.config.borrow().window_width, self.config.borrow().window_height);
        let mouse = get_mouse_screen_coords(_x, _y, sw, sh, ww, wh);

        if mouse::button_pressed(_ctx, MouseButton::Left) {
            for e in self.ui_elements.iter_mut() {
                if let Some(s) = e.as_any_mut().downcast_mut::<Slider>() {
                    let dc = mouse.x - s.last_mx; 
                    let step = s.get_step_in_pixels(sw);

                    if s.slider_button.clicked && dc.abs() > step {
                        let steps = s.get_step() * dc.abs() / step;
                        s.last_mx = mouse.x;
                        match s.tag {
                            UIElementTag::Volume => change_volume(&mut self.config.borrow_mut(), dc.signum() * steps),
                            _ => (),
                        }
                    }
                }
            }
        }
    }

    fn get_conf(&self) -> Option<Ref<Config>> { Some(self.config.borrow()) }

    fn get_conf_mut(&mut self) -> Option<RefMut<Config>> { Some(self.config.borrow_mut()) }

    fn get_ui_elements(&self) -> Option<&Vec<Box<dyn UIElement>>> { Some(&self.ui_elements) }

    fn get_ui_elements_mut(&mut self) -> Option<&mut Vec<Box<dyn UIElement>>> { Some(&mut self.ui_elements) }  

    fn update_ui_vars(&mut self, _ctx: &mut Context) -> GameResult {
        for el in self.ui_elements.iter_mut() {
            if let Some(e) = el.as_any_mut().downcast_mut::<Slider>() {
                match e.tag {
                    UIElementTag::Volume => e.value = self.config.borrow().volume * 100.,
                    _ => (),
                }
            }
            else if let Some(e) = el.as_any_mut().downcast_mut::<CheckBox>() {
                match e.tag {
                    UIElementTag::WindowMode => e.checked = self.config.borrow().window_mode == FullscreenType::True,
                    _ => (),
                }
            }
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn Any { self }

    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

pub struct LevelTransitionScene {
    config: Rc<RefCell<Config>>,
    ui_elements: Vec<Box<dyn UIElement>>,
    cooldown: f32,
    level: usize,
}

impl LevelTransitionScene {
    pub fn new(config: &Rc<RefCell<Config>>) -> Self {
        let config = Rc::clone(config);
        let ui_elements: Vec<Box<dyn UIElement>> = vec![
            Box::new(TextSprite {
                pos: Point2 { x: 0.5, y: 0.2},
                font_size: BUTTON_TEXT_FONT_SIZE * 2.,
                color: Color::RED,
                ..Default::default()
            }),
        ];
        let cooldown = TRANSITION_SCENE_COOLDOWN;

        Self {
            config,
            ui_elements,
            cooldown,
            level: 0,
        }
    }
}

impl Scene for LevelTransitionScene {
    fn update(&mut self, ctx: &mut Context, _delta_time: f32) -> GameResult {
        for e in self.ui_elements.iter_mut() {
            e.update(ctx, &mut self.config.borrow_mut())?;
        }

        self.ui_elements[0].as_any_mut().downcast_mut::<TextSprite>().unwrap().text = format!("LEVEL {}", self.level);

        self.update_ui_vars(ctx)?;

        self.cooldown = f32::max(self.cooldown - _delta_time, 0.);

        if self.cooldown == 0. {
            self.config.borrow_mut().current_state = State::Play;    
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let (sw, sh) = (self.config.borrow().screen_width, self.config.borrow().screen_height);

        let curtain = Mesh::new_rectangle(
            ctx,
            DrawMode::fill(),
            Rect::new(0., 0., sw, sh),
            [0.1, 0.2, 0.3, self.cooldown / TRANSITION_SCENE_COOLDOWN].into()
        )?;

        graphics::draw(ctx, &curtain, DrawParam::default())?;

        for e in self.ui_elements.iter_mut() {
            e.draw(ctx, &mut self.config.borrow_mut())?;
        }

        Ok(())
    }

    fn get_conf(&self) -> Option<Ref<Config>> { Some(self.config.borrow()) }

    fn get_conf_mut(&mut self) -> Option<RefMut<Config>> { Some(self.config.borrow_mut()) }

    fn get_ui_elements(&self) -> Option<&Vec<Box<dyn UIElement>>> { Some(&self.ui_elements) }

    fn get_ui_elements_mut(&mut self) -> Option<&mut Vec<Box<dyn UIElement>>> { Some(&mut self.ui_elements) }  

    fn update_ui_vars(&mut self, _ctx: &mut Context) -> GameResult {
        if self.level < self.config.borrow().level {
            self.level = self.config.borrow().level;
            self.cooldown = TRANSITION_SCENE_COOLDOWN;
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn Any { self }

    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}
