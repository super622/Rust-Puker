use ggez::{
    graphics::{self, Color},
    Context,
    GameResult,
    event::{self,KeyCode,MouseButton,EventHandler},
    conf::{Conf,WindowMode},
    ContextBuilder,
    filesystem,
    input::{self, keyboard, mouse},
    timer,
};
use std::{
    env,
    path,
    f32::consts::PI,
};
use glam::f32::{Vec2};

use puker::{
    entities::*,
    assets::*,
    utils::*,
    dungeon::*,
    consts::*,
};

struct MainState {
    screen_width: f32,
    screen_height: f32,
    assets: Assets,
    player: Player,
    dungeon: Dungeon,
    cur_room: (usize, usize),
}

impl MainState {
    fn new(ctx: &mut Context, conf: &Conf) -> GameResult<MainState> {
        input::mouse::set_cursor_grabbed(ctx, true)?;

        let assets = Assets::new(ctx)?;
        let screen_width = conf.window_mode.width;
        let screen_height = conf.window_mode.height;
        let player = Player {
            props: ActorProps {
                pos: Vec2::ZERO.into(),
                scale: Vec2::splat(PLAYER_SCALE),
                translation: Vec2::ZERO,
                forward: Vec2::ZERO,
            },
            speed: PLAYER_SPEED,
            health: PLAYER_HEALTH,
            state: ActorState::BASE,
            shoot_rate: PLAYER_SHOOT_RATE,
            shoot_range: PLAYER_SHOOT_RANGE,
            shoot_timeout: PLAYER_SHOOT_TIMEOUT,
            shots: Vec::new(),
            color: Color::WHITE,
        };
        let dungeon = Dungeon::generate_dungeon((screen_width, screen_height));
        let cur_room = Dungeon::get_start_room_coords();

        let s = MainState {
            screen_width, 
            screen_height,           
            assets,
            player,
            dungeon,
            cur_room,
        };

        Ok(s)
    }

    fn mouse_relative_forward(&self, mouse: Vec2) -> Vec2 {
        let ppos = self.player.props.pos;
        let m = screen_to_world_space(self.screen_width, self.screen_height, mouse);

        let dx = m.x - ppos.0.x;
        let dy = m.y - ppos.0.y;

        if f32::abs(dx) > f32::abs(dy) {
            return Vec2::new(f32::signum(dx), 0.);
        }
        Vec2::new(0., f32::signum(dy))
    }

    fn handle_input(&mut self, ctx: &mut Context) {
        self.player.props.forward = Vec2::ZERO;
        self.player.props.translation = Vec2::ZERO;
        self.player.state = ActorState::BASE;

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
            self.player.state = ActorState::SHOOT;
        }
        if keyboard::is_key_pressed(ctx, KeyCode::Down) {
            self.player.props.forward = Vec2::new(0., -1.);
            self.player.state = ActorState::SHOOT;
        }
        if keyboard::is_key_pressed(ctx, KeyCode::Left) {
            self.player.props.forward = Vec2::new(-1., 0.);
            self.player.state = ActorState::SHOOT;
        }
        if keyboard::is_key_pressed(ctx, KeyCode::Right) {
            self.player.props.forward = Vec2::new(1., 0.);
            self.player.state = ActorState::SHOOT;
        }
        if mouse::button_pressed(ctx, MouseButton::Left) {
            self.player.props.forward = self.mouse_relative_forward(Vec2::new(mouse::position(ctx).x, mouse::position(ctx).y));
            self.player.state = ActorState::SHOOT;
        }
    }

    fn handle_wall_collisions(&mut self, seconds: f32) -> GameResult {
        let (sw, sh) = (self.screen_width, self.screen_height);
        let (mut cp, mut cn) = (Vec2::ZERO, Vec2::ZERO);
        let mut ct = 0.;
        self.player.color = Color::WHITE;

        let room = &self.dungeon.get_room(self.cur_room)?;
        let mut collisions = Vec::<(usize, f32)>::new();

        for (i, obst) in room.obstacles.iter().enumerate() {
            if dynamic_rect_vs_rect(&self.player.get_bbox(sw, sh), &self.player.get_velocity(seconds), &obst.get_bbox(sw, sh), &mut cp, &mut cn, &mut ct, seconds) {
                collisions.push((i, ct));
            }
        }

        collisions.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        for (i, mut ct) in collisions.iter_mut() {
            if dynamic_rect_vs_rect(&self.player.get_bbox(sw, sh), &self.player.get_velocity(seconds), &room.obstacles[*i].get_bbox(sw, sh), &mut cp, &mut cn, &mut ct, seconds) {
                let obst = room.obstacles[*i].as_any();

                if let Some(door) = obst.downcast_ref::<Door>() {
                    if door.is_open {
                        self.cur_room = door.connects_to;
                        self.player.props.pos.0 *= -1.;
                    }
                    else {
                        self.player.props.translation += cn * self.player.get_velocity(seconds).abs() * (1. - ct);
                    }
                }
                else {
                    self.player.props.translation += cn * self.player.get_velocity(seconds).abs() * (1. - ct);
                }
            }
        }

        Ok(())
    }

    fn handle_shot_collisions(&mut self, _seconds: f32) -> GameResult {
        let (sw, sh) = (self.screen_width, self.screen_height);
        let mut enemies = &mut self.dungeon.get_room_mut(self.cur_room)?.enemies;

        self.player.shots = self.player.shots.clone().into_iter().filter(|s| {
            for enemy in enemies.into_iter() {
                if rect_vs_rect(&s.get_bbox(sw, sh), &enemy.get_bbox(sw, sh)) {
                    enemy.damage(s.damage);
                    return false;
                }
            }
            true
        }).collect();

        Ok(())
    }

    fn handle_player_enemy_collisions(&mut self, seconds: f32) -> GameResult {
        todo!();
    }
}

impl EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        const DESIRED_FPS: u32 = 60;

        while timer::check_update_time(ctx, DESIRED_FPS) {
            let seconds = 1.0 / (DESIRED_FPS as f32);

            self.handle_input(ctx);

            self.handle_wall_collisions(seconds)?;

            self.handle_shot_collisions(seconds)?;

            self.dungeon.get_room_mut(self.cur_room)?.update(seconds)?;

            self.player.update(seconds)?;
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());

        let screen_coords = (self.screen_width, self.screen_height);

        self.dungeon.get_room(self.cur_room)?.draw(ctx, &self.assets, screen_coords)?;

        self.player.draw(ctx, &self.assets, screen_coords)?;

        graphics::present(ctx)?;
        Ok(())
    }

    fn key_down_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymod: input::keyboard::KeyMods, _repeat: bool) {
        match keycode {
            _ => (),
        }
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymod: input::keyboard::KeyMods) {
        match keycode {
            _ => (),
        }
    }

    fn mouse_motion_event(&mut self, _ctx: &mut Context, x: f32, y: f32, _dx: f32, _dy: f32) {
        if input::mouse::button_pressed(_ctx, MouseButton::Left) {
            self.player.props.forward = self.mouse_relative_forward(Vec2::new(x, y));
        }
    }
}

fn main() -> GameResult {
    let conf = Conf::new()
        .window_mode(WindowMode {
            width: DEFAULT_SCREEN_WIDTH,
            height: DEFAULT_SCREEN_HEIGHT,
            ..Default::default()
        });

    let (mut ctx, event_loop) = ContextBuilder::new("PrimitiveIsaac", "Window")
        .default_conf(conf.clone())
        .build()
        .unwrap();

    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        filesystem::mount(&mut ctx, &path, true);
    }

    let state = MainState::new(&mut ctx, &conf)?;

    event::run(ctx, event_loop, state)
}
