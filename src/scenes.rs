use ggez::{
    graphics::{self, Font, DrawMode, Color, DrawParam, Rect, Mesh},
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
};

use crate::{
    player::*,
    shots::*,
    assets::*,
    utils::*,
    dungeon::*,
    consts::*,
    traits::*,
    ui_elements::*,
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
        let player = Player {
            props: ActorProps {
                pos: Vec2::new(sw / 2., sh / 2.).into(),
                scale: Vec2::splat(PLAYER_SCALE),
                translation: Vec2::ZERO,
                forward: Vec2::ZERO,
                velocity: Vec2::ZERO,
            },
            speed: PLAYER_SPEED,
            health: PLAYER_HEALTH,
            max_health: PLAYER_HEALTH,
            state: ActorState::Base,
            shoot_rate: PLAYER_SHOOT_RATE,
            shoot_range: PLAYER_SHOOT_RANGE,
            shoot_timeout: PLAYER_SHOOT_TIMEOUT,
            damage: PLAYER_DAMAGE,
            damaged_cooldown: 0.,
            animation_cooldown: 0.,
            afterlock_cooldown: PLAYER_AFTERLOCK_COOLDOWN,
        };
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
            self.player.shoot(&mut room.shots)?;
        }
        if keyboard::is_key_pressed(ctx, KeyCode::Down) {
            self.player.props.forward = Vec2::new(0., 1.);
            self.player.shoot(&mut room.shots)?;
        }
        if keyboard::is_key_pressed(ctx, KeyCode::Left) {
            self.player.props.forward = Vec2::new(-1., 0.);
            self.player.shoot(&mut room.shots)?;
        }
        if keyboard::is_key_pressed(ctx, KeyCode::Right) {
            self.player.props.forward = Vec2::new(1., 0.);
            self.player.shoot(&mut room.shots)?;
        }
        if mouse::button_pressed(ctx, MouseButton::Left) {
            self.player.props.forward = mouse_relative_forward(self.player.props.pos.0, mouse::position(ctx), &self.config.borrow());
            self.player.shoot(&mut room.shots)?;
        }

        Ok(())
    }

    fn handle_block_collisions(&mut self, delta_time: f32) -> GameResult {
        let (sw, sh) = (self.config.borrow().screen_width, self.config.borrow().screen_height);
        let (mut cp, mut cn) = (Vec2::ZERO, Vec2::ZERO);
        let mut ct = 0.;
        let room = self.dungeon.get_room_mut(self.cur_room)?.unwrap();
        let mut next_level = false;

        for o in room.obstacles.iter() {
            let obst = o.as_any().downcast_ref::<Block>().unwrap();

            if dynamic_circle_vs_rect(self.player.get_bcircle(sw, sh), &self.player.get_velocity(), &o.get_bbox(sw, sh), &mut cp, &mut cn, &mut ct, delta_time) {
                match obst.tag {
                    BlockTag::Door { is_open, dir, connects_to } => {
                        if is_open {
                            if (self.player.props.pos.0 - obst.pos.0).length() < obst.get_bcircle(sw, sh).1 {
                                self.cur_room = connects_to;
                                let (di, dj) = pos_to_room_coords(obst.get_pos(), sw, sh);
                                self.player.props.pos.0 = match dir {
                                        Direction::North => room_coords_to_pos(ROOM_HEIGHT - di - 1, dj + 1, sw, sh),
                                        Direction::South => room_coords_to_pos(ROOM_HEIGHT - di + 1, dj + 1, sw, sh),
                                        Direction::West => room_coords_to_pos(di + 1, ROOM_WIDTH - dj - 1, sw, sh),
                                        Direction::East => room_coords_to_pos(di + 1, ROOM_WIDTH - dj + 1, sw, sh),
                                };
                                self.player.props.velocity = Vec2::ZERO;
                                self.player.afterlock_cooldown = PLAYER_AFTERLOCK_COOLDOWN;
                            }
                        }
                        else { self.player.props.pos.0 -= cn.normalize() * ct; }
                    },
                    BlockTag::Spikes => {
                        if o.get_bbox(sw, sh).contains(self.player.props.pos) { self.player.damage(1.); }
                    },
                    BlockTag::Hatch(is_open) => {
                        if is_open {
                            next_level = true;
                        }
                    },
                    _ => self.player.props.pos.0 -= cn.normalize() * ct,
                }
            }

            match obst.tag {
                BlockTag::Hatch(_) => (),
                _ => {
                    for e in room.enemies.iter_mut() {
                        if dynamic_circle_vs_rect(e.get_bcircle(sw, sh), &e.get_velocity(), &o.get_bbox(sw, sh), &mut cp, &mut cn, &mut ct, delta_time) {
                            e.set_pos(e.get_pos() - cn.normalize() * ct);
                        }
                    }

                    for d in room.drops.iter_mut() {
                        if dynamic_circle_vs_rect(d.get_bcircle(sw, sh), &d.get_velocity(), &o.get_bbox(sw, sh), &mut cp, &mut cn, &mut ct, delta_time) {
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

    fn handle_shot_collisions(&mut self, ctx: &mut Context, assets: &mut Assets, _delta_time: f32) -> GameResult {
        let (sw, sh) = (self.config.borrow().screen_width, self.config.borrow().screen_height);
        let room = self.dungeon.get_room_mut(self.cur_room)?.unwrap();

        room.shots = room.shots.clone().into_iter().filter(|s| {
            match s.tag {
                ShotTag::Player => {
                    for enemy in room.enemies.iter_mut() {
                        let (mut vel1, mut vel2) = (Vec2::ZERO, Vec2::ZERO);
                        if dynamic_circle_vs_circle(&s.get_bcircle(sw, sh), &s.get_velocity(), &enemy.get_bcircle(sw, sh), &enemy.get_velocity(), &mut vel1, &mut vel2, _delta_time) {
                            let _ = assets.audio.get_mut("bubble_pop_sound").unwrap().play(ctx);
                            enemy.damage(s.damage);
                            enemy.set_velocity(enemy.get_velocity() + vel2);
                            return false;
                        }
                    }
                },
                ShotTag::Enemy => {
                    if circle_vs_circle(&s.get_bcircle(sw, sh), &self.player.get_bcircle(sw, sh)) {
                        let _ = assets.audio.get_mut("bubble_pop_sound").unwrap().play(ctx);
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
                    _ => if dynamic_circle_vs_rect(s.get_bcircle(sw, sh), &s.get_velocity(), &obst.get_bbox(sw, sh), &mut cp, &mut cn, &mut ct, _delta_time) {
                        let _ = assets.audio.get_mut("bubble_pop_sound").unwrap().play(ctx);
                        return false;
                    },
                }
            }
            true
        }).collect();

        Ok(())
    }

    fn handle_environment_collisions(&mut self, ctx: &mut Context, assets: &mut Assets, _delta_time: f32) -> GameResult {
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
                if !d.affect_player(ctx, assets, &mut self.player)? {
                    resolve_environment_collision(d, &mut self.player, sw, sh, _delta_time);
                }
            }
        }

        Ok(())
    }
}

impl Scene for PlayScene {
    fn update(&mut self, ctx: &mut Context, assets: &mut Assets, delta_time: f32) -> GameResult {
        self.handle_input(ctx)?;

        self.handle_block_collisions(delta_time)?;

        self.handle_environment_collisions(ctx, assets, delta_time)?;

        self.handle_shot_collisions(ctx, assets, delta_time)?;

        self.dungeon.update_rooms_state(self.cur_room)?;
        self.dungeon.get_room_mut(self.cur_room)?.unwrap().update(ctx, assets, &self.config.borrow(), &self.player, delta_time)?;

        self.player.update(ctx, assets, &self.config.borrow(), delta_time)?;

        self.overlay.update_vars(&self.player, &self.dungeon, self.cur_room);
        self.overlay.update(ctx, &self.config.borrow())?;

        match self.player.state {
            ActorState::Dead => self.config.borrow_mut().current_state = State::Dead,
            _ => (),
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context, assets: &mut Assets) -> GameResult {
        self.dungeon.get_room(self.cur_room)?.unwrap().draw(ctx, assets, &self.config.borrow())?;

        self.player.draw(ctx, assets, &self.config.borrow())?;

        self.overlay.draw(ctx, assets, &self.config.borrow())?;

        Ok(())
    }

    fn key_down_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymod: input::keyboard::KeyMods, _repeat: bool) {
        match keycode {
            KeyCode::Escape => self.config.borrow_mut().current_state = State::PauseMenu,
            _ => (),
        }
    }

    fn get_conf(&self) -> Option<Ref<Config>> { Some(self.config.borrow()) }

    fn get_conf_mut(&mut self) -> Option<RefMut<Config>> { Some(self.config.borrow_mut()) }
}

pub struct MainMenuScene {
    config: Rc<RefCell<Config>>,
    ui_elements: Vec<Box<dyn UIElement>>,
}

impl MainMenuScene {
    pub fn new(config: &Rc<RefCell<Config>>, assets: &Assets) -> Self {
        let config = Rc::clone(config);
        let ui_elements: Vec<Box<dyn UIElement>> = vec![
            Box::new(Button {
                pos: Point2 { x: 0.5, y: 0.4},
                tag: State::New,
                text: TextSprite {
                    pos: Point2 { x: 0.5, y: 0.4},
                    text: String::from("Play"),
                    font: *assets.fonts.get("button_font").unwrap(),
                    font_size: BUTTON_TEXT_FONT_SIZE,
                    color: Color::BLACK,
                    background: Color::from([0.; 4]),
                },
                color: Color::WHITE,
                border: Border::default(),
            }),
            Box::new(Button {
                pos: Point2 { x: 0.5, y: 0.6},
                tag: State::Quit,
                text: TextSprite {
                    pos: Point2 { x: 0.5, y: 0.6},
                    text: String::from("Quit"),
                    font: *assets.fonts.get("button_font").unwrap(),
                    font_size: BUTTON_TEXT_FONT_SIZE,
                    color: Color::BLACK,
                    background: Color::from([0.; 4]),
                },
                color: Color::WHITE,
                border: Border::default(),
            }),
        ];

        Self {
            config,
            ui_elements,
        }
    }
}

impl Scene for MainMenuScene {
    fn update(&mut self, ctx: &mut Context, _assets: &mut Assets, _delta_time: f32) -> GameResult {
        for e in self.ui_elements.iter_mut() {
            e.update(ctx, &self.config.borrow())?;
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context, assets: &mut Assets) -> GameResult {
        let (sw, sh) = (self.config.borrow().screen_width, self.config.borrow().screen_height);
        let logo_rect = assets.sprites.get("puker_logo").unwrap().dimensions();
        let draw_param = DrawParam::default()
            .scale([sw / logo_rect.w, sh / logo_rect.h]);

        graphics::draw(ctx, assets.sprites.get("puker_logo").unwrap(), draw_param)?;

        for e in self.ui_elements.iter_mut() {
            e.draw(ctx, assets, &self.config.borrow())?;
        }

        Ok(())
    }

    fn mouse_button_down_event(&mut self, _ctx: &mut Context, _button: MouseButton, _x: f32, _y: f32) {
        if _button == MouseButton::Left {
            let result = self.get_clicked(_ctx);
            match result {
                Some(e) => {
                    if let Some(b) = e.as_any().downcast_ref::<Button>() {
                        self.config.borrow_mut().current_state = b.tag;
                    }
                },
                None => (),
            }
        }
    }

    fn get_conf(&self) -> Option<Ref<Config>> { Some(self.config.borrow()) }

    fn get_conf_mut(&mut self) -> Option<RefMut<Config>> { Some(self.config.borrow_mut()) }

    fn get_ui_elements(&self) -> Option<&Vec<Box<dyn UIElement>>> { Some(&self.ui_elements) }

    fn get_ui_elements_mut(&mut self) -> Option<&mut Vec<Box<dyn UIElement>>> { Some(&mut self.ui_elements) }  
}

pub struct PauseMenuScene {
    config: Rc<RefCell<Config>>,
    ui_elements: Vec<Box<dyn UIElement>>,
}

impl PauseMenuScene {
    pub fn new(config: &Rc<RefCell<Config>>, assets: &Assets) -> Self {
        let config = Rc::clone(config);
        let ui_elements: Vec<Box<dyn UIElement>> = vec![
            Box::new(Button {
                pos: Point2 { x: 0.5, y: 0.2},
                tag: State::Play,
                text: TextSprite {
                    pos: Point2 { x: 0.5, y: 0.2},
                    text: String::from("Continue"),
                    font: *assets.fonts.get("button_font").unwrap(),
                    font_size: BUTTON_TEXT_FONT_SIZE,
                    color: Color::BLACK,
                    background: Color::from([0.; 4]),
                },
                color: Color::WHITE,
                border: Border::default(),
            }),
            Box::new(Button {
                pos: Point2 { x: 0.5, y: 0.4},
                tag: State::New,
                text: TextSprite {
                    pos: Point2 { x: 0.5, y: 0.4},
                    text: String::from("New Game"),
                    font: *assets.fonts.get("button_font").unwrap(),
                    font_size: BUTTON_TEXT_FONT_SIZE,
                    color: Color::BLACK,
                    background: Color::from([0.; 4]),
                },
                color: Color::WHITE,
                border: Border::default(),
            }),
            Box::new(Button {
                pos: Point2 { x: 0.5, y: 0.6},
                tag: State::Options,
                text: TextSprite {
                    pos: Point2 { x: 0.5, y: 0.6},
                    text: String::from("Options"),
                    font: *assets.fonts.get("button_font").unwrap(),
                    font_size: BUTTON_TEXT_FONT_SIZE,
                    color: Color::BLACK,
                    background: Color::from([0.; 4]),
                },
                color: Color::WHITE,
                border: Border::default(),
            }),
            Box::new(Button {
                pos: Point2 { x: 0.5, y: 0.8},
                tag: State::Quit,
                text: TextSprite {
                    pos: Point2 { x: 0.5, y: 0.8},
                    text: String::from("Quit"),
                    font: *assets.fonts.get("button_font").unwrap(),
                    font_size: BUTTON_TEXT_FONT_SIZE,
                    color: Color::BLACK,
                    background: Color::from([0.; 4]),
                },
                color: Color::WHITE,
                border: Border::default(),
            }),
        ];

        Self {
            config,
            ui_elements,
        }
    }
}

impl Scene for PauseMenuScene {
    fn update(&mut self, ctx: &mut Context, _assets: &mut Assets, _delta_time: f32) -> GameResult {
        for e in self.ui_elements.iter_mut() {
            e.update(ctx, &self.config.borrow())?;
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context, assets: &mut Assets) -> GameResult {
        let (sw, sh) = (self.config.borrow().screen_width, self.config.borrow().screen_height);

        let curtain = Mesh::new_rectangle(
            ctx,
            DrawMode::fill(),
            Rect::new(0., 0., sw, sh),
            [0.1, 0.2, 0.3, 0.3].into()
        )?;

        graphics::draw(ctx, &curtain, DrawParam::default())?;

        for e in self.ui_elements.iter_mut() {
            e.draw(ctx, assets, &self.config.borrow())?;
        }

        Ok(())
    }

    fn key_down_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymod: input::keyboard::KeyMods, _repeat: bool) {
        match keycode {
            KeyCode::Escape => self.config.borrow_mut().current_state = State::Play,
            _ => (),
        }
    }

    fn mouse_button_down_event(&mut self, _ctx: &mut Context, _button: MouseButton, _x: f32, _y: f32) {
        if _button == MouseButton::Left {
            let result = self.get_clicked(_ctx);
            match result {
                Some(e) => {
                    if let Some(b) = e.as_any().downcast_ref::<Button>() {
                        self.config.borrow_mut().current_state = b.tag;
                    }
                },
                None => (),
            }
        }
    }

    fn get_conf(&self) -> Option<Ref<Config>> { Some(self.config.borrow()) }

    fn get_conf_mut(&mut self) -> Option<RefMut<Config>> { Some(self.config.borrow_mut()) }

    fn get_ui_elements(&self) -> Option<&Vec<Box<dyn UIElement>>> { Some(&self.ui_elements) }

    fn get_ui_elements_mut(&mut self) -> Option<&mut Vec<Box<dyn UIElement>>> { Some(&mut self.ui_elements) }  
}

pub struct DeadScene {
    config: Rc<RefCell<Config>>,
    ui_elements: Vec<Box<dyn UIElement>>,
}

impl DeadScene {
    pub fn new(config: &Rc<RefCell<Config>>, assets: &Assets) -> Self {
        let config = Rc::clone(config);
        let ui_elements: Vec<Box<dyn UIElement>> = vec![
            Box::new(TextSprite {
                pos: Point2 { x: 0.5, y: 0.3},
                text: String::from("YOU DIED"),
                font: Font::default(),
                font_size: BUTTON_TEXT_FONT_SIZE * 2.,
                color: Color::RED,
                background: Color::from([0.; 4]),
            }),
            Box::new(Button {
                pos: Point2 { x: 0.5, y: 0.5},
                tag: State::New,
                text: TextSprite {
                    pos: Point2 { x: 0.5, y: 0.5},
                    text: String::from("Try Again"),
                    font: *assets.fonts.get("button_font").unwrap(),
                    font_size: BUTTON_TEXT_FONT_SIZE,
                    color: Color::BLACK,
                    background: Color::from([0.; 4]),
                },
                color: Color::WHITE,
                border: Border::default(),
            }),
            Box::new(Button {
                pos: Point2 { x: 0.5, y: 0.7},
                tag: State::Quit,
                text: TextSprite {
                    pos: Point2 { x: 0.5, y: 0.7},
                    text: String::from("Quit"),
                    font: *assets.fonts.get("button_font").unwrap(),
                    font_size: BUTTON_TEXT_FONT_SIZE,
                    color: Color::BLACK,
                    background: Color::from([0.; 4]),
                },
                color: Color::WHITE,
                border: Border::default(),
            }),
        ];

        Self {
            config,
            ui_elements,
        }
    }
}

impl Scene for DeadScene {
    fn update(&mut self, ctx: &mut Context, _assets: &mut Assets, _delta_time: f32) -> GameResult {
        for e in self.ui_elements.iter_mut() {
            e.update(ctx, &self.config.borrow())?;
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context, assets: &mut Assets) -> GameResult {
        let (sw, sh) = (self.config.borrow().screen_width, self.config.borrow().screen_height);

        let curtain = Mesh::new_rectangle(
            ctx,
            DrawMode::fill(),
            Rect::new(0., 0., sw, sh),
            [0.1, 0.2, 0.3, 0.3].into()
        )?;

        graphics::draw(ctx, &curtain, DrawParam::default())?;

        for e in self.ui_elements.iter_mut() {
            e.draw(ctx, assets, &self.config.borrow())?;
        }

        Ok(())
    }

    fn mouse_button_down_event(&mut self, _ctx: &mut Context, _button: MouseButton, _x: f32, _y: f32) {
        if _button == MouseButton::Left {
            let result = self.get_clicked(_ctx);
            match result {
                Some(e) => {
                    if let Some(b) = e.as_any().downcast_ref::<Button>() {
                        self.config.borrow_mut().current_state = b.tag;
                    }
                },
                None => (),
            }
        }
    }

    fn get_conf(&self) -> Option<Ref<Config>> { Some(self.config.borrow()) }

    fn get_conf_mut(&mut self) -> Option<RefMut<Config>> { Some(self.config.borrow_mut()) }

    fn get_ui_elements(&self) -> Option<&Vec<Box<dyn UIElement>>> { Some(&self.ui_elements) }

    fn get_ui_elements_mut(&mut self) -> Option<&mut Vec<Box<dyn UIElement>>> { Some(&mut self.ui_elements) }  
}

pub struct OptionsScene {
    config: Rc<RefCell<Config>>,
    ui_elements: Vec<Box<dyn UIElement>>,
}

impl OptionsScene {
    pub fn new(config: &Rc<RefCell<Config>>, assets: &Assets) -> Self {
        let config = Rc::clone(config);
        let ui_elements: Vec<Box<dyn UIElement>> = vec![
            Box::new(TextSprite {
                pos: Point2 { x: 0.5, y: 0.2},
                text: String::from("OPTIONS"),
                font: Font::default(),
                font_size: BUTTON_TEXT_FONT_SIZE * 2.,
                color: Color::RED,
                background: Color::from([0.; 4]),
            }),
            Box::new(CheckBox {
                pos: Point2 { x: 0.3, y: 0.4 },
                width: 0.1,
                height: 0.1,
                checked: false,
                color: Color::BLACK,
            }),
            Box::new(TextSprite {
                pos: Point2 { x: 0.6, y: 0.4},
                text: String::from("Fullscreen"),
                font: Font::default(),
                font_size: BUTTON_TEXT_FONT_SIZE,
                color: Color::BLUE,
                background: Color::from([1.; 4]),
            }),
            Box::new(Button {
                pos: Point2 { x: 0.5, y: 0.8},
                tag: State::PauseMenu,
                text: TextSprite {
                    pos: Point2 { x: 0.5, y: 0.8},
                    text: String::from("Back"),
                    font: *assets.fonts.get("button_font").unwrap(),
                    font_size: BUTTON_TEXT_FONT_SIZE,
                    color: Color::BLACK,
                    background: Color::from([0.; 4]),
                },
                color: Color::WHITE,
                border: Border::default(),
            }),
        ];

        Self {
            config,
            ui_elements,
        }
    }
}

impl Scene for OptionsScene {
    fn update(&mut self, ctx: &mut Context, _assets: &mut Assets, _delta_time: f32) -> GameResult {
        for e in self.ui_elements.iter_mut() {
            e.update(ctx, &self.config.borrow())?;
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context, assets: &mut Assets) -> GameResult {
        let (sw, sh) = (self.config.borrow().screen_width, self.config.borrow().screen_height);

        let curtain = Mesh::new_rectangle(
            ctx,
            DrawMode::fill(),
            Rect::new(0., 0., sw, sh),
            [0.1, 0.2, 0.3, 0.3].into()
        )?;

        graphics::draw(ctx, &curtain, DrawParam::default())?;

        for e in self.ui_elements.iter_mut() {
            e.draw(ctx, assets, &self.config.borrow())?;
        }

        Ok(())
    }

    fn key_down_event(&mut self, _ctx: &mut Context, _keycode: KeyCode, _keymod: input::keyboard::KeyMods, _repeat: bool) {
        match _keycode {
            KeyCode::Escape => self.config.borrow_mut().current_state = State::PauseMenu,
            _ => (),
        }
    }

    fn get_conf(&self) -> Option<Ref<Config>> { Some(self.config.borrow()) }

    fn get_conf_mut(&mut self) -> Option<RefMut<Config>> { Some(self.config.borrow_mut()) }

    fn mouse_button_down_event(&mut self, _ctx: &mut Context, _button: MouseButton, _x: f32, _y: f32) {
        if _button == MouseButton::Left {
            let result = self.get_clicked_mut(_ctx);
            match result {
                Some(e) => {
                    if let Some(b) = e.as_any().downcast_ref::<Button>() {
                        self.config.borrow_mut().current_state = b.tag;
                    }
                    else if let Some(c) = e.as_any_mut().downcast_mut::<CheckBox>() {
                        let _ = match c.checked {
                            true => graphics::set_fullscreen(_ctx, FullscreenType::Windowed),
                            false => graphics::set_fullscreen(_ctx, FullscreenType::True),
                        };
                        c.checked = !c.checked;
                    }
                },
                None => (),
            }
        }
    }

    fn get_ui_elements(&self) -> Option<&Vec<Box<dyn UIElement>>> { Some(&self.ui_elements) }

    fn get_ui_elements_mut(&mut self) -> Option<&mut Vec<Box<dyn UIElement>>> { Some(&mut self.ui_elements) }  
}
