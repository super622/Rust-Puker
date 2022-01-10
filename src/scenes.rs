use ggez::{
    graphics::{self, Color},
    Context,
    GameResult,
    event::{self, KeyCode, MouseButton, EventHandler},
    conf::{Conf,WindowMode},
    ContextBuilder,
    filesystem,
    input::{self, keyboard, mouse},
    timer,
};
use glam::f32::{Vec2};
use std::{
    fmt::{Display, Formatter},
    str::FromStr,
    collections::{HashMap},
};

use crate::{
    entities::*,
    assets::*,
    utils::*,
    dungeon::*,
    consts::*,
};

#[derive(Clone, Copy, Hash, Debug)]
pub enum SceneType {
    PLAY,
    MENU,
    DEAD,
}

impl Display for SceneType {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            SceneType::PLAY => write!(f, "PLAY"),
            SceneType::MENU => write!(f, "MENU"),
            SceneType::DEAD => write!(f, "DEAD"),
        }
    }
}

impl FromStr for SceneType {
    type Err = Errors;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "PLAY" => Ok(SceneType::PLAY),
            "MENU" => Ok(SceneType::MENU),
            "DEAD" => Ok(SceneType::DEAD),
            _ => Err(Errors::SceneTypeParse(input.to_string())),
        }
    }
}

impl PartialEq for SceneType {
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
    }
}
impl Eq for SceneType {}

pub trait Scene {
    fn update(&mut self, ctx: &mut Context, delta_time: f32) -> GameResult;

    fn draw(&mut self, ctx: &mut Context, assets: &Assets) -> GameResult;
}

#[derive(Debug)]
pub struct PlayScene {
    config: Config,
    player: Player,
    dungeon: Dungeon,
    cur_room: (usize, usize),
}

impl PlayScene {
    pub fn new(ctx: &mut Context, conf: &Conf) -> GameResult<Self> {
        input::mouse::set_cursor_grabbed(ctx, true)?;

        let config = Config {
            screen_width: conf.window_mode.width,
            screen_height: conf.window_mode.height,
            draw_bbox_model: true,            
            draw_bbox_stationary: false,            
        };
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
        let dungeon = Dungeon::generate_dungeon((config.screen_width, config.screen_height));
        let cur_room = Dungeon::get_start_room_coords();

        let s = Self {
            config,
            player,
            dungeon,
            cur_room,
        };

        Ok(s)
    }

    fn mouse_relative_forward(&self, mouse: Vec2) -> Vec2 {
        let ppos = self.player.props.pos;
        let m = screen_to_world_space(self.config.screen_width, self.config.screen_height, mouse);

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

    fn handle_wall_collisions(&mut self, delta_time: f32) -> GameResult {
        let (sw, sh) = (self.config.screen_width, self.config.screen_height);
        let (mut cp, mut cn) = (Vec2::ZERO, Vec2::ZERO);
        let mut ct = 0.;
        self.player.color = Color::WHITE;

        let room = &self.dungeon.get_room(self.cur_room)?;
        let mut collisions = Vec::<(usize, f32)>::new();

        for (i, obst) in room.obstacles.iter().enumerate() {
            if dynamic_rect_vs_rect(&self.player.get_bbox(sw, sh), &self.player.get_velocity(delta_time), &obst.get_bbox(sw, sh), &mut cp, &mut cn, &mut ct, delta_time) {
                collisions.push((i, ct));
            }
        }

        collisions.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        for (i, mut ct) in collisions.iter_mut() {
            if dynamic_rect_vs_rect(&self.player.get_bbox(sw, sh), &self.player.get_velocity(delta_time), &room.obstacles[*i].get_bbox(sw, sh), &mut cp, &mut cn, &mut ct, delta_time) {
                let obst = room.obstacles[*i].as_any();

                if let Some(door) = obst.downcast_ref::<Door>() {
                    if door.is_open {
                        self.cur_room = door.connects_to;
                        self.player.props.pos.0 *= -1.;
                        self.player.shots = Vec::new();
                    }
                    else {
                        self.player.props.translation += cn * self.player.get_velocity(delta_time).abs() * (1. - ct);
                    }
                }
                else {
                    self.player.props.translation += cn * self.player.get_velocity(delta_time).abs() * (1. - ct);
                }
            }
        }

        Ok(())
    }

    fn handle_shot_collisions(&mut self, _delta_time: f32) -> GameResult {
        let (sw, sh) = (self.config.screen_width, self.config.screen_height);
        let room = &mut self.dungeon.get_room_mut(self.cur_room)?;

        self.player.shots = self.player.shots.clone().into_iter().filter(|s| {
            for enemy in room.enemies.iter_mut() {
                if rect_vs_rect(&s.get_bbox(sw, sh), &enemy.get_bbox(sw, sh)) {
                    enemy.damage(s.damage);
                    return false;
                }
            }
            for obst in room.obstacles.iter() {
                if rect_vs_rect(&s.get_bbox(sw, sh), &obst.get_bbox(sw, sh)) {
                    return false;
                }
            }
            true
        }).collect();

        room.enemies.iter_mut().map(|e| { 
            if let Some(enemy) = e.as_any_mut().downcast_mut::<EnemyMask>() {
                enemy.get_shots_mut().into_iter().filter(|s| {
                    if rect_vs_rect(&s.get_bbox(sw, sh), &self.player.get_bbox(sw, sh)) {
                        self.player.damage(s.damage);
                        return false;
                    }
                    true
                }).count();
            }
        }).count();

        Ok(())
    }

    fn handle_player_enemy_collisions(&mut self, _delta_time: f32) -> GameResult {
        todo!();
    }

    fn handle_player_detection(&mut self, _delta_time: f32) -> GameResult {
        let (sw, sh) = (self.config.screen_width, self.config.screen_height);
        let room = &mut self.dungeon.get_room_mut(self.cur_room)?;

        for e in room.enemies.iter_mut() {
            if e.get_pos().distance(self.player.get_pos()) <= ENEMY_SHOOT_RANGE {
                if let Some(enemy) = e.as_any_mut().downcast_mut::<EnemyMask>() {
                    enemy.state = ActorState::BASE;
                    enemy.props.forward = self.player.get_pos() - enemy.get_pos();

                    let (mut cp, mut cn) = (Vec2::ZERO, Vec2::ZERO);
                    let mut ct = 0.;

                    if room.obstacles.iter()
                        .filter(|o| {
                            ray_vs_rect(&enemy.get_pos(), &enemy.get_forward(), &o.get_bbox(sw, sh), &mut cp, &mut cn, &mut ct) && ct < 1.
                        })
                        .count() == 0 {
                        enemy.state = ActorState::SHOOT;
                    }
                }
            }
        }

        Ok(())
    }
}

impl Scene for PlayScene {
    fn update(&mut self, ctx: &mut Context, delta_time: f32) -> GameResult {

        self.handle_input(ctx);

        self.handle_wall_collisions(delta_time)?;

        self.handle_shot_collisions(delta_time)?;

        self.handle_player_detection(delta_time)?;

        self.dungeon.get_room_mut(self.cur_room)?.update(delta_time)?;

        self.player.update(delta_time)?;

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context, assets: &Assets) -> GameResult {
        let screen_coords = (self.config.screen_width, self.config.screen_height);

        self.dungeon.get_room(self.cur_room)?.draw(ctx, assets, screen_coords, &self.config)?;

        self.player.draw(ctx, assets, screen_coords, &self.config)?;

        Ok(())
    }
}
