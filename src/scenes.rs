use ggez::{
    graphics::{self, Font, DrawMode, Color, DrawParam, Rect, Mesh},
    Context,
    GameResult,
    mint::{Point2},
    event::{KeyCode, MouseButton},
    input::{self, keyboard, mouse},
    audio::{SoundSource},
};
use glam::f32::{Vec2};
use std::{
    rc::Rc,
    cell::RefCell,
};

use crate::{
    entities::*,
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
            damaged_cooldown: 0.,
            animation_cooldown: 0.,
        };
        let dungeon = Dungeon::generate_dungeon((sw, sh));
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
        let room = self.dungeon.get_room_mut(self.cur_room)?;
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
            self.player.props.forward = mouse_relative_forward(self.player.props.pos.0, Vec2::new(mouse::position(ctx).x, mouse::position(ctx).y));
            self.player.shoot(&mut room.shots)?;
        }

        Ok(())
    }

    fn handle_wall_collisions(&mut self, delta_time: f32) -> GameResult {
        let (sw, sh) = (self.config.borrow().screen_width, self.config.borrow().screen_height);
        let (mut cp, mut cn) = (Vec2::ZERO, Vec2::ZERO);
        let mut ct = 0.;

        let room = &self.dungeon.get_room(self.cur_room)?;
        let mut collisions = Vec::<(usize, f32)>::new();

        for (i, obst) in room.obstacles.iter().enumerate() {
            if dynamic_rect_vs_rect(&self.player.get_bbox(sw, sh), &self.player.get_velocity(), &obst.get_bbox(sw, sh), &mut cp, &mut cn, &mut ct, delta_time) {
                collisions.push((i, ct));
            }
        }

        collisions.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        for (i, mut ct) in collisions.iter_mut() {
            if dynamic_rect_vs_rect(&self.player.get_bbox(sw, sh), &self.player.get_velocity(), &room.obstacles[*i].get_bbox(sw, sh), &mut cp, &mut cn, &mut ct, delta_time) {
                let obst = room.obstacles[*i].as_any();

                if let Some(door) = obst.downcast_ref::<Door>() {
                    if door.is_open {
                        self.cur_room = door.connects_to;
                        self.player.props.pos.0 = Vec2::new(sw, sh) - self.player.props.pos.0;
                    }
                    else {
                        self.player.props.pos.0 += cn * self.player.get_velocity().abs() * (1. - ct);
                        // self.player.props.velocity += cn * self.player.get_velocity().abs() * (1. - ct);
                    }
                }
                else {
                    self.player.props.pos.0 += cn * self.player.get_velocity().abs() * (1. - ct);
                    // self.player.props.velocity += cn * self.player.get_velocity().abs() * (1. - ct);
                }
            }
        }

        Ok(())
    }

    fn handle_shot_collisions(&mut self, ctx: &mut Context, assets: &mut Assets, _delta_time: f32) -> GameResult {
        let (sw, sh) = (self.config.borrow().screen_width, self.config.borrow().screen_height);
        let room = &mut self.dungeon.get_room_mut(self.cur_room)?;

        room.shots = room.shots.clone().into_iter().filter(|s| {
            match s.tag {
                ShotTag::Player => {
                    for enemy in room.enemies.iter_mut() {
                        if s.get_bbox(sw, sh).overlaps(&enemy.get_bbox(sw, sh)) {
                            let _ = assets.audio.get_mut("bubble_pop_sound").unwrap().play(ctx);
                            enemy.damage(s.damage);
                            return false;
                        }
                    }
                },
                ShotTag::Enemy => {
                    if s.get_bbox(sw, sh).overlaps(&self.player.get_bbox(sw, sh)) {
                        let _ = assets.audio.get_mut("bubble_pop_sound").unwrap().play(ctx);
                        self.player.damage(s.damage);
                        return false;
                    }
                },
            };

            for obst in room.obstacles.iter() {
                if s.get_bbox(sw, sh).overlaps(&obst.get_bbox(sw, sh)) {
                    let _ = assets.audio.get_mut("bubble_pop_sound").unwrap().play(ctx);
                    return false;
                }
            }
            true
        }).collect();

        Ok(())
    }

    fn handle_player_enemy_collisions(&mut self, _delta_time: f32) -> GameResult {
        let (sw, sh) = (self.config.borrow().screen_width, self.config.borrow().screen_height);
        let room = &mut self.dungeon.get_room_mut(self.cur_room)?;

        for e in room.enemies.iter_mut() {
            if e.get_bbox(sw, sh).overlaps(&self.player.get_bbox(sw, sh)) {
                self.player.damage(ENEMY_DAMAGE);
            }
        }

        Ok(())
    }

    fn handle_player_detection(&mut self, _delta_time: f32) -> GameResult {
        let (sw, sh) = (self.config.borrow().screen_width, self.config.borrow().screen_height);
        let room = &mut self.dungeon.get_room_mut(self.cur_room)?;

        for e in room.enemies.iter_mut() {
            if e.get_pos().distance(self.player.get_pos()) <= ENEMY_SHOOT_RANGE * 0.6 {
                if let Some(enemy) = e.as_any_mut().downcast_mut::<EnemyMask>() {
                    enemy.props.forward = self.player.get_pos() - enemy.get_pos();

                    let (mut cp, mut cn) = (Vec2::ZERO, Vec2::ZERO);
                    let mut ct = 0.;

                    if room.obstacles.iter()
                        .filter(|o| { ray_vs_rect(&enemy.get_pos(), &enemy.get_forward(), &o.get_bbox(sw, sh), &mut cp, &mut cn, &mut ct) && ct < 1. })
                        .count() == 0 {
                        enemy.shoot(&mut room.shots)?;
                    }
                }
            }
        }

        Ok(())
    }
}

impl Scene for PlayScene {
    fn update(&mut self, ctx: &mut Context, assets: &mut Assets, delta_time: f32) -> GameResult {
        self.handle_input(ctx)?;

        self.handle_wall_collisions(delta_time)?;

        self.handle_player_enemy_collisions(delta_time)?;

        self.handle_player_detection(delta_time)?;

        self.handle_shot_collisions(ctx, assets, delta_time)?;

        self.dungeon.get_room_mut(self.cur_room)?.update(&self.config.borrow(), delta_time)?;

        self.player.update(&self.config.borrow(), delta_time)?;

        self.overlay.update_vars(&self.player, &self.dungeon, self.cur_room);
        self.overlay.update(ctx, &self.config.borrow())?;

        match self.player.state {
            ActorState::Dead => self.config.borrow_mut().current_state = State::Dead,
            _ => (),
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context, assets: &mut Assets) -> GameResult {
        self.dungeon.get_room(self.cur_room)?.draw(ctx, assets, &self.config.borrow())?;

        self.player.draw(ctx, assets, &self.config.borrow())?;

        self.overlay.draw(ctx, assets, &self.config.borrow())?;

        Ok(())
    }

    fn key_down_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymod: input::keyboard::KeyMods, _repeat: bool) {
        match keycode {
            KeyCode::Escape => self.config.borrow_mut().current_state = State::Menu,
            _ => (),
        }
    }

    fn key_up_event(&mut self, _ctx: &mut Context, _keycode: KeyCode, _keymod: input::keyboard::KeyMods) {}

    fn mouse_button_down_event(&mut self, _ctx: &mut Context, _button: MouseButton, _x: f32, _y: f32) {}

    fn resize_event(&mut self, conf: &Config) {
        self.player.resize_event(&conf);
        self.dungeon.resize_event(&conf);
    }
}

pub struct StartScene {
    config: Rc<RefCell<Config>>,
    ui_elements: Vec<Box<dyn UIElement>>,
}

impl StartScene {
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
                },
                color: Color::WHITE,
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
                },
                color: Color::WHITE,
            }),
        ];

        Self {
            config,
            ui_elements,
        }
    }
}

impl Scene for StartScene {
    fn update(&mut self, ctx: &mut Context, assets: &mut Assets, _delta_time: f32) -> GameResult {
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

    fn key_down_event(&mut self, _ctx: &mut Context, _keycode: KeyCode, _keymod: input::keyboard::KeyMods, _repeat: bool) {}

    fn key_up_event(&mut self, _ctx: &mut Context, _keycode: KeyCode, _keymod: input::keyboard::KeyMods) {}

    fn mouse_button_down_event(&mut self, _ctx: &mut Context, _button: MouseButton, _x: f32, _y: f32) {
        let (sw, sh) = (self.config.borrow().screen_width, self.config.borrow().screen_height);

        if _button == MouseButton::Left {
            match self.check_for_element_click(_ctx, sw, sh) {
                Some(e) => {
                    if let Some(b) = e.as_any().downcast_ref::<Button>() {
                        self.config.borrow_mut().current_state = b.tag;
                    }
                },
                None => (),
            }
        }
    }

    fn get_ui_elements(&self) -> Option<&Vec<Box<dyn UIElement>>> { Some(&self.ui_elements) }

    fn get_ui_elements_mut(&mut self) -> Option<&mut Vec<Box<dyn UIElement>>> { Some(&mut self.ui_elements) }  
}

pub struct MenuScene {
    config: Rc<RefCell<Config>>,
    ui_elements: Vec<Box<dyn UIElement>>,
}

impl MenuScene {
    pub fn new(config: &Rc<RefCell<Config>>, assets: &Assets) -> Self {
        let config = Rc::clone(config);
        let ui_elements: Vec<Box<dyn UIElement>> = vec![
            Box::new(Button {
                pos: Point2 { x: 0.5, y: 0.3},
                tag: State::Play,
                text: TextSprite {
                    pos: Point2 { x: 0.5, y: 0.3},
                    text: String::from("Continue"),
                    font: *assets.fonts.get("button_font").unwrap(),
                    font_size: BUTTON_TEXT_FONT_SIZE,
                    color: Color::BLACK,
                },
                color: Color::WHITE,
            }),
            Box::new(Button {
                pos: Point2 { x: 0.5, y: 0.5},
                tag: State::New,
                text: TextSprite {
                    pos: Point2 { x: 0.5, y: 0.5},
                    text: String::from("New"),
                    font: *assets.fonts.get("button_font").unwrap(),
                    font_size: BUTTON_TEXT_FONT_SIZE,
                    color: Color::BLACK,
                },
                color: Color::WHITE,
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
                },
                color: Color::WHITE,
            }),
        ];

        Self {
            config,
            ui_elements,
        }
    }
}

impl Scene for MenuScene {
    fn update(&mut self, ctx: &mut Context, assets: &mut Assets, _delta_time: f32) -> GameResult {
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

    fn key_up_event(&mut self, _ctx: &mut Context, _keycode: KeyCode, _keymod: input::keyboard::KeyMods) {}

    fn mouse_button_down_event(&mut self, _ctx: &mut Context, _button: MouseButton, _x: f32, _y: f32) {
        let (sw, sh) = (self.config.borrow().screen_width, self.config.borrow().screen_height);

        if _button == MouseButton::Left {
            match self.check_for_element_click(_ctx, sw, sh) {
                Some(e) => {
                    if let Some(b) = e.as_any().downcast_ref::<Button>() {
                        self.config.borrow_mut().current_state = b.tag;
                    }
                },
                None => (),
            }
        }
    }

    fn get_ui_elements(&self) -> Option<&Vec<Box<dyn UIElement>>> { Some(&self.ui_elements) }

    fn get_ui_elements_mut(&mut self) -> Option<&mut Vec<Box<dyn UIElement>>> { Some(&mut self.ui_elements) }  
}

pub struct DeadScene {
    config: Rc<RefCell<Config>>,
    ui_elements: Vec<Box<dyn UIElement>>,
    played_death_sound: bool,
}

impl DeadScene {
    pub fn new(config: &Rc<RefCell<Config>>, assets: &Assets) -> Self {
        let config = Rc::clone(config);
        let ui_elements: Vec<Box<dyn UIElement>> = vec![
            Box::new(Button {
                pos: Point2 { x: 0.5, y: 0.5},
                tag: State::New,
                text: TextSprite {
                    pos: Point2 { x: 0.5, y: 0.5},
                    text: String::from("Try Again"),
                    font: *assets.fonts.get("button_font").unwrap(),
                    font_size: BUTTON_TEXT_FONT_SIZE,
                    color: Color::BLACK,
                },
                color: Color::WHITE,
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
                },
                color: Color::WHITE,
            }),
            Box::new(TextSprite {
                pos: Point2 { x: 0.5, y: 0.3},
                text: String::from("YOU DIED"),
                font: Font::default(),
                font_size: BUTTON_TEXT_FONT_SIZE * 2.,
                color: Color::RED,
            }),
        ];

        Self {
            config,
            ui_elements,
            played_death_sound: false,
        }
    }
}

impl Scene for DeadScene {
    fn update(&mut self, ctx: &mut Context, assets: &mut Assets, _delta_time: f32) -> GameResult {
        for e in self.ui_elements.iter_mut() {
            e.update(ctx, &self.config.borrow())?;
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context, assets: &mut Assets) -> GameResult {
        let (sw, sh) = (self.config.borrow().screen_width, self.config.borrow().screen_height);

        if !self.played_death_sound {
            assets.audio.get_mut("death_sound").unwrap().play(ctx)?;
            self.played_death_sound = true;
        }

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

    fn key_down_event(&mut self, _ctx: &mut Context, _keycode: KeyCode, _keymod: input::keyboard::KeyMods, _repeat: bool) {}

    fn key_up_event(&mut self, _ctx: &mut Context, _keycode: KeyCode, _keymod: input::keyboard::KeyMods) {}

    fn mouse_button_down_event(&mut self, _ctx: &mut Context, _button: MouseButton, _x: f32, _y: f32) {
        let (sw, sh) = (self.config.borrow().screen_width, self.config.borrow().screen_height);

        if _button == MouseButton::Left {
            match self.check_for_element_click(_ctx, sw, sh) {
                Some(e) => {
                    if let Some(b) = e.as_any().downcast_ref::<Button>() {
                        self.config.borrow_mut().current_state = b.tag;
                        self.played_death_sound = false;
                    }
                },
                None => (),
            }
        }
    }

    fn get_ui_elements(&self) -> Option<&Vec<Box<dyn UIElement>>> { Some(&self.ui_elements) }

    fn get_ui_elements_mut(&mut self) -> Option<&mut Vec<Box<dyn UIElement>>> { Some(&mut self.ui_elements) }  
}
