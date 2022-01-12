use ggez::{
    graphics::{self, Color, DrawMode, DrawParam, Rect, Mesh, Text, PxScale},
    Context,
    GameResult,
    mint::{Point2},
    event::{KeyCode, MouseButton},
    input::{self, keyboard, mouse},
    audio::Source,
};
use glam::f32::{Vec2};
use std::{
    rc::Rc,
    cell::RefCell,
    str::FromStr,
};

use crate::{
    entities::*,
    assets::*,
    utils::*,
    dungeon::*,
    consts::*,
    traits::*,
};

pub struct PlayScene {
    config: Rc<RefCell<Config>>,
    player: Player,
    dungeon: Dungeon,
    cur_room: (usize, usize),
}

impl PlayScene {
    pub fn new(config: &Rc<RefCell<Config>>) -> Self {
        let config = Rc::clone(config);
        let player = Player {
            props: ActorProps {
                pos: Vec2::ZERO.into(),
                scale: Vec2::splat(PLAYER_SCALE),
                translation: Vec2::ZERO,
                forward: Vec2::ZERO,
                velocity: Vec2::ZERO,
            },
            speed: PLAYER_SPEED,
            health: PLAYER_HEALTH,
            state: ActorState::Base,
            shoot_rate: PLAYER_SHOOT_RATE,
            shoot_range: PLAYER_SHOOT_RANGE,
            shoot_timeout: PLAYER_SHOOT_TIMEOUT,
            color: Color::WHITE,
        };
        let dungeon = Dungeon::generate_dungeon((config.borrow().screen_width, config.borrow().screen_height));
        let cur_room = Dungeon::get_start_room_coords();

        PlayScene {
            config,
            player,
            dungeon,
            cur_room,
        }
    }

    fn handle_input(&mut self, ctx: &mut Context) -> GameResult {
        let room = self.dungeon.get_room_mut(self.cur_room)?;
        self.player.props.forward = Vec2::ZERO;
        self.player.props.translation = Vec2::ZERO;
        self.player.state = ActorState::Base;

        if keyboard::is_key_pressed(ctx, KeyCode::W) {
            self.player.props.translation.y += 1.;
        }
        if keyboard::is_key_pressed(ctx, KeyCode::S) {
            self.player.props.translation.y -= 1.;
        }
        if keyboard::is_key_pressed(ctx, KeyCode::A) {
            self.player.props.translation.x -= 1.;
        }
        if keyboard::is_key_pressed(ctx, KeyCode::D) {
            self.player.props.translation.x += 1.;
        }
        if keyboard::is_key_pressed(ctx, KeyCode::Up) {
            self.player.props.forward = Vec2::new(0., 1.);
            self.player.state = ActorState::Shoot;
            self.player.shoot(&mut room.shots);
        }
        if keyboard::is_key_pressed(ctx, KeyCode::Down) {
            self.player.props.forward = Vec2::new(0., -1.);
            self.player.state = ActorState::Shoot;
            self.player.shoot(&mut room.shots);
        }
        if keyboard::is_key_pressed(ctx, KeyCode::Left) {
            self.player.props.forward = Vec2::new(-1., 0.);
            self.player.state = ActorState::Shoot;
            self.player.shoot(&mut room.shots);
        }
        if keyboard::is_key_pressed(ctx, KeyCode::Right) {
            self.player.props.forward = Vec2::new(1., 0.);
            self.player.state = ActorState::Shoot;
            self.player.shoot(&mut room.shots);
        }
        if mouse::button_pressed(ctx, MouseButton::Left) {
            self.player.props.forward = mouse_relative_forward(self.config.borrow().screen_width, self.config.borrow().screen_height, self.player.props.pos.0, Vec2::new(mouse::position(ctx).x, mouse::position(ctx).y));
            self.player.state = ActorState::Shoot;
            self.player.shoot(&mut room.shots);
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
                        self.player.props.pos.0 *= -1.;
                    }
                    else {
                        self.player.props.pos.0 += cn * self.player.get_velocity().abs() * (1. - ct);
                    }
                }
                else {
                    self.player.props.pos.0 += cn * self.player.get_velocity().abs() * (1. - ct);
                }
            }
        }

        Ok(())
    }

    fn handle_shot_collisions(&mut self, _delta_time: f32) -> GameResult {
        let (sw, sh) = (self.config.borrow().screen_width, self.config.borrow().screen_height);
        let room = &mut self.dungeon.get_room_mut(self.cur_room)?;

        room.shots = room.shots.clone().into_iter().filter(|s| {
            match s.tag {
                ShotTag::Player => {
                    for enemy in room.enemies.iter_mut() {
                        if rect_vs_rect(&s.get_bbox(sw, sh), &enemy.get_bbox(sw, sh)) {
                            enemy.damage(s.damage);
                            return false;
                        }
                    }
                },
                ShotTag::Enemy => {
                    if rect_vs_rect(&s.get_bbox(sw, sh), &self.player.get_bbox(sw, sh)) {
                        self.player.damage(s.damage);
                        return false;
                    }
                },
            };

            for obst in room.obstacles.iter() {
                if rect_vs_rect(&s.get_bbox(sw, sh), &obst.get_bbox(sw, sh)) {
                    return false;
                }
            }
            true
        }).collect();

        Ok(())
    }

    fn handle_player_enemy_collisions(&mut self, _delta_time: f32) -> GameResult {
        todo!();
    }

    fn handle_player_detection(&mut self, _delta_time: f32) -> GameResult {
        let (sw, sh) = (self.config.borrow().screen_width, self.config.borrow().screen_height);
        let room = &mut self.dungeon.get_room_mut(self.cur_room)?;

        for e in room.enemies.iter_mut() {
            if e.get_pos().distance(self.player.get_pos()) <= ENEMY_SHOOT_RANGE * 0.6 {
                if let Some(enemy) = e.as_any_mut().downcast_mut::<EnemyMask>() {
                    enemy.state = ActorState::Base;
                    enemy.props.forward = self.player.get_pos() - enemy.get_pos();

                    let (mut cp, mut cn) = (Vec2::ZERO, Vec2::ZERO);
                    let mut ct = 0.;

                    if room.obstacles.iter()
                        .filter(|o| {
                            ray_vs_rect(&enemy.get_pos(), &enemy.get_forward(), &o.get_bbox(sw, sh), &mut cp, &mut cn, &mut ct) && ct < 1.
                        })
                        .count() == 0 {
                        enemy.state = ActorState::Shoot;
                        enemy.shoot(&mut room.shots);
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

        self.handle_wall_collisions(delta_time)?;

        self.handle_shot_collisions(delta_time)?;

        self.handle_player_detection(delta_time)?;

        self.dungeon.get_room_mut(self.cur_room)?.update(delta_time)?;

        self.player.update(delta_time)?;

        if self.player.state == ActorState::Dead {
            self.config.borrow_mut().current_state = State::Dead;
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context, assets: &Assets) -> GameResult {
        let screen_coords = (self.config.borrow().screen_width, self.config.borrow().screen_height);

        self.dungeon.get_room(self.cur_room)?.draw(ctx, assets, screen_coords, &self.config.borrow())?;

        self.player.draw(ctx, assets, screen_coords, &self.config.borrow())?;

        Ok(())
    }

    fn key_down_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymod: input::keyboard::KeyMods, _repeat: bool) {
        match keycode {
            KeyCode::Escape => self.config.borrow_mut().current_state = State::Menu,
            _ => (),
        }
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymod: input::keyboard::KeyMods) {}

    fn mouse_button_down_event(&mut self, _ctx: &mut Context, _button: MouseButton, _x: f32, _y: f32) {}
}

pub struct Button {
    pub pos: Point2<f32>,
    pub text: String,
    pub color: Color,
    pub width: f32,
    pub height: f32,
}

impl Button {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        let (tx, ty) = (self.pos.x - self.width / 2., self.pos.y - self.height / 2.);

        self.color = Color::WHITE;
        if self.mouse_overlap(ctx) {
            self.color = Color::RED;
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context, assets: &Assets) -> GameResult {
        let (tx, ty) = (self.pos.x - self.width / 2., self.pos.y - self.height / 2.);
        let mut text = Text::new(self.text.as_str());
        text.fragments_mut().iter_mut().map(|f| {
            f.font = Some(assets.freedom_font);
            f.scale = Some(PxScale::from(60.));
            f.color = Some(Color::BLACK);
        }).count();

        let btn = Mesh::new_rounded_rectangle(
            ctx,
            DrawMode::fill(),
            Rect::new(tx, ty, self.width, self.height),
            5.,
            self.color,
        )?;

        graphics::draw(ctx, &btn, DrawParam::default());
        graphics::draw(ctx, &text, DrawParam::default().dest([tx, ty]));

        Ok(())
    }

    fn mouse_overlap(&self, ctx: &mut Context) -> bool {
        let (tx, ty) = (self.pos.x - self.width / 2., self.pos.y - self.height / 2.);
        Rect::new(tx, ty, self.width, self.height).contains(mouse::position(ctx))
    }
}

pub struct StartScene {
    config: Rc<RefCell<Config>>,
    buttons: Vec<Button>,
}

impl StartScene {
    pub fn new(config: &Rc<RefCell<Config>>) -> Self {
        let (sw, sh) = (config.borrow().screen_width, config.borrow().screen_height); 

        let config = Rc::clone(config);
        let buttons = vec![
            Button {
                pos: Point2 { x: sw * 0.5, y: sh * 0.4},
                text: String::from("New"),
                color: Color::WHITE,
                width: sw * 0.2,
                height: sh * 0.1,
            },
            Button {
                pos: Point2 { x: sw * 0.5, y: sh * 0.6},
                text: String::from("Quit"),
                color: Color::WHITE,
                width: sw * 0.2,
                height: sh * 0.1,
            }
        ];

        StartScene {
            config,
            buttons,
        }
    }

    fn check_for_button_click(&self, ctx: &mut Context) -> Option<&Button> {
        for b in self.buttons.iter() {
            if b.mouse_overlap(ctx) {
                return Some(b);
            }
        }
        None
    }
}

impl Scene for StartScene {
    fn update(&mut self, ctx: &mut Context, delta_time: f32) -> GameResult {
        for b in self.buttons.iter_mut() {
            b.update(ctx)?;
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context, assets: &Assets) -> GameResult {
        for b in self.buttons.iter_mut() {
            b.draw(ctx, assets)?;
        }

        Ok(())
    }

    fn key_down_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymod: input::keyboard::KeyMods, _repeat: bool) {}

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymod: input::keyboard::KeyMods) {}

    fn mouse_button_down_event(&mut self, _ctx: &mut Context, _button: MouseButton, _x: f32, _y: f32) {
        if _button == MouseButton::Left {
            match self.check_for_button_click(_ctx) {
                Some(b) => self.config.borrow_mut().current_state = State::from_str(b.text.as_str()).unwrap(),
                None => (),
            }
        }
    }
}

pub struct MenuScene {
    config: Rc<RefCell<Config>>,
    buttons: Vec<Button>,
}

impl MenuScene {
    pub fn new(config: &Rc<RefCell<Config>>) -> Self {
        let (sw, sh) = (config.borrow().screen_width, config.borrow().screen_height); 

        let config = Rc::clone(config);
        let buttons = vec![
            Button {
                pos: Point2 { x: sw * 0.5, y: sh * 0.3},
                text: String::from("Continue"),
                color: Color::WHITE,
                width: sw * 0.4,
                height: sh * 0.1,
            },
            Button {
                pos: Point2 { x: sw * 0.5, y: sh * 0.5},
                text: String::from("New"),
                color: Color::WHITE,
                width: sw * 0.2,
                height: sh * 0.1,
            },
            Button {
                pos: Point2 { x: sw * 0.5, y: sh * 0.7},
                text: String::from("Quit"),
                color: Color::WHITE,
                width: sw * 0.2,
                height: sh * 0.1,
            }
        ];

        MenuScene {
            config,
            buttons,
        }
    }

    fn check_for_button_click(&self, ctx: &mut Context) -> Option<&Button> {
        for b in self.buttons.iter() {
            if b.mouse_overlap(ctx) {
                return Some(b);
            }
        }
        None
    }
}

impl Scene for MenuScene {
    fn update(&mut self, ctx: &mut Context, delta_time: f32) -> GameResult {
        for b in self.buttons.iter_mut() {
            b.update(ctx)?;
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context, assets: &Assets) -> GameResult {
        let (sw, sh) = (self.config.borrow().screen_width, self.config.borrow().screen_height);

        let curtain = Mesh::new_rectangle(
            ctx,
            DrawMode::fill(),
            Rect::new(0., 0., sw, sh),
            [0.1, 0.2, 0.3, 0.3].into()
        )?;

        graphics::draw(ctx, &curtain, DrawParam::default());

        for b in self.buttons.iter_mut() {
            b.draw(ctx, assets)?;
        }

        Ok(())
    }

    fn key_down_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymod: input::keyboard::KeyMods, _repeat: bool) {
        match keycode {
            KeyCode::Escape => self.config.borrow_mut().current_state = State::Continue,
            _ => (),
        }
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymod: input::keyboard::KeyMods) {}

    fn mouse_button_down_event(&mut self, _ctx: &mut Context, _button: MouseButton, _x: f32, _y: f32) {
        if _button == MouseButton::Left {
            match self.check_for_button_click(_ctx) {
                Some(b) => self.config.borrow_mut().current_state = State::from_str(b.text.as_str()).unwrap(),
                None => (),
            }
        }
    }
}

pub struct DeadScene {
    config: Rc<RefCell<Config>>,
    buttons: Vec<Button>,
}
