use ggez::{
    mint::Point2,
    graphics::{Rect, Color},
    GameError,
    audio::SoundSource,
};
use glam::f32::*;
use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};
use crate::{
    traits::*,
    consts::*,
    assets::*,
};

pub struct Config {
    pub assets: Assets,
    pub screen_width: f32,
    pub screen_height: f32,
    pub window_width: f32,
    pub window_height: f32,
    pub volume: f32,
    pub draw_bcircle_model: bool,
    pub draw_bbox_stationary: bool,
    pub current_state: State,
    pub previous_state: State,
}

#[derive(Clone, Copy, Hash, Debug)]
pub enum State {
    Play,
    MainMenu,
    New,
    PauseMenu,
    Options,
    Quit,
    Dead,
}

impl Display for State {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl FromStr for State {
    type Err = Errors;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let state = match input {
            "Play" => State::Play,
            "MainMenu" => State::MainMenu,
            "New" => State::New,
            "PauseMenu" => State::PauseMenu,
            "Quit" => State::Quit,
            "Dead" => State::Dead,
            _ => return Err(Errors::StateParse(input.to_string())),
        };
        Ok(state)
    }
}

impl PartialEq for State {
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
    }
}
impl Eq for State {}

#[derive(Debug)]
pub enum Errors {
    UnknownRoomIndex(usize),
    UnknownGridCoords((usize, usize)),
    StateParse(String),
}

impl Display for Errors {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Into<GameError> for Errors {
    fn into(self) -> GameError {
        GameError::CustomError(self.to_string())
    }
}

#[derive(Clone, Copy, Hash, Debug)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

impl Display for Direction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Debug, Copy, PartialEq)]
pub enum ActorState {
    Base,
    Shoot,
    Dead,
    Damaged,
}

#[derive(Clone, Debug, Copy)]
pub enum ActorTag {
    Player,
    Enemy,
}

#[derive(Clone, Debug, Copy)]
pub struct ActorProps {
    pub pos: Vec2Wrap,
    pub scale: Vec2,
    pub translation: Vec2,
    pub forward: Vec2,
    pub velocity: Vec2,
}

impl Default for ActorProps {
    fn default() -> Self {
        Self {
            pos: Vec2::ZERO.into(),
            scale: Vec2::ONE,
            translation: Vec2::ZERO,
            forward: Vec2::ZERO,
            velocity: Vec2::ZERO,
        }
    }
}

#[derive(Clone, Debug, Copy, Default)]
pub struct Vec2Wrap(pub Vec2);

impl Into<Point2<f32>> for Vec2Wrap {
    fn into(self) -> Point2<f32> {
        Point2 {
            x: self.0.x,
            y: self.0.y
        }
    }
}

impl Into<Vec2> for Vec2Wrap {
    fn into(self) -> Vec2 {
        self.0
    }
}

impl From<Vec2> for Vec2Wrap {
    fn from(v: Vec2) -> Self {
        Vec2Wrap(v)
    }
}

impl From<[f32; 2]> for Vec2Wrap {
    fn from(a: [f32; 2]) -> Self {
        Vec2Wrap(Vec2::from_slice(&a))
    }
}

impl From<Point2<f32>> for Vec2Wrap {
    fn from(p: Point2<f32>) -> Self {
        Vec2Wrap(Vec2::from_slice(&[p.x, p.y]))
    }
}

pub fn circle_vs_circle(c1: &(Vec2Wrap, f32), c2: &(Vec2Wrap, f32)) -> bool {
    c1.0.0.distance(c2.0.0) < c1.1 + c2.1
}

/// Detects if a ray is intersecting a given rectangle.
/// Long live OneLoneCoder and his tutorials.
///
pub fn ray_vs_rect(ray_origin: &Vec2, ray_dir: &Vec2, target: &Rect, contact_point: &mut Vec2, contact_normal: &mut Vec2, t_hit_near: &mut f32) -> bool {
    let target_pos = Vec2::new(target.x, target.y);
    let target_size = Vec2::new(target.w, target.h);

    let mut t_near = (target_pos - *ray_origin) / *ray_dir;
    let mut t_far = (target_pos + target_size - *ray_origin) / *ray_dir;

    if t_far.x.is_nan() || t_far.y.is_nan() { return false; }
    if t_near.x.is_nan() || t_near.y.is_nan() { return false; }

    if t_near.x > t_far.x { std::mem::swap(&mut t_near.x, &mut t_far.x)}
    if t_near.y > t_far.y { std::mem::swap(&mut t_near.y, &mut t_far.y)}

    if t_near.x > t_far.y || t_near.y > t_far.x { return false; }

    *t_hit_near = f32::max(t_near.x, t_near.y);
    let t_hit_far = f32::min(t_far.x, t_far.y);

    if t_hit_far < 0. { return false; }

    *contact_point = *ray_origin + *t_hit_near * *ray_dir;

    if t_near.x > t_near.y {
        if ray_dir.x < 0. { *contact_normal = Vec2::new(1., 0.); }
        else { *contact_normal = Vec2::new(-1., 0.); }
    }
    else if t_near.x < t_near.y {
        if ray_dir.y < 0. { *contact_normal = Vec2::new(0., 1.); }
        else { *contact_normal = Vec2::new(0., -1.); }
    }

    true
}

/// Detects intersection between moving rectangle and stationary rectangle.
/// Long live OneLoneCoder and his tutorials.
///
pub fn dynamic_rect_vs_rect(source: &Rect, source_vel: &Vec2, target: &Rect, contact_point: &mut Vec2, contact_normal: &mut Vec2, contact_time: &mut f32, _elapsed_time: f32) -> bool { 
    let source_pos = Vec2::new(source.x, source.y);
    let source_size = Vec2::new(source.w, source.h);

    if source_vel.x == 0. && source_vel.y == 0. { return false; }

    let expanded_target = Rect {
        x: target.x - source_size.x / 2.,
        y: target.y - source_size.y / 2.,
        w: target.w + source_size.x,
        h: target.h + source_size.y,
    };

    if ray_vs_rect(&(source_pos + source_size / 2.), &(*source_vel + _elapsed_time), &expanded_target, contact_point, contact_normal, contact_time) {
        if *contact_time <= 1. { 
            return true;
        }
    }

    false
}

/// Detects intersection between moving circle and stationary rectangle.
/// Long live OneLoneCoder and his tutorials.
///
pub fn dynamic_circle_vs_rect(source: (Vec2Wrap, f32), _source_vel: &Vec2, target: &Rect, contact_point: &mut Vec2, contact_normal: &mut Vec2, contact_time: &mut f32, _elapsed_time: f32) -> bool { 
    let source_pos = source.0.0;
    let source_r = source.1;

    contact_point.x = f32::max(target.x, f32::min(target.x + target.w, source_pos.x));
    contact_point.y = f32::max(target.y, f32::min(target.y + target.h, source_pos.y));

    *contact_normal = *contact_point - source_pos;
    *contact_time = source_r - contact_normal.length();

    if contact_time.is_nan() { *contact_time = 0. }

    if *contact_time > 0. { return true; }

    false
}

/// Detects intersection between static circles.
/// Long live OneLoneCoder and his tutorials.
///
pub fn static_circle_vs_circle(c1: &(Vec2Wrap, f32), c2: &(Vec2Wrap, f32), displace_vec: &mut Vec2) -> bool {
    if !circle_vs_circle(c1, c2) { return false; }

    let c1_pos = c1.0.0;
    let c1_r = c1.1;
    let c2_pos = c2.0.0;
    let c2_r = c2.1;

    let distance = c1_pos.distance(c2_pos);
    let overlap = 0.5 * (distance - c1_r - c2_r);
    *displace_vec = overlap * (c1_pos - c2_pos).normalize();

    true
}

/// Detects intersection between dynamic circles.
/// Long live OneLoneCoder and his tutorials.
///
pub fn dynamic_circle_vs_circle(c1: &(Vec2Wrap, f32), c1_vel: &Vec2, c2: &(Vec2Wrap, f32), c2_vel: &Vec2, vel1: &mut Vec2, vel2: &mut Vec2, _elapsed_time: f32) -> bool { 
    if !circle_vs_circle(c1, c2) { return false; }

    let c1_pos = c1.0.0;
    let c1_r = c1.1;
    let c1_mass = c1_r * 100.;
    let c2_pos = c2.0.0;
    let c2_r = c2.1;
    let c2_mass = c2_r * 100.;

    let norm = (c1_pos - c2_pos).normalize();
    let tan = Vec2::new(-norm.y, norm.x);

    let dp_tan = Vec2::new(c1_vel.dot(tan), c2_vel.dot(tan));
    let dp_norm = Vec2::new(c1_vel.dot(norm), c2_vel.dot(norm));
      
    let m1 = (dp_norm.x * (c1_mass - c2_mass) + 2.0 * c2_mass * dp_norm.y) / (c1_mass + c2_mass);
    let m2 = (dp_norm.y * (c2_mass - c1_mass) + 2.0 * c1_mass * dp_norm.x) / (c1_mass + c2_mass);

    *vel1 = tan * dp_tan.x + norm * m1;
    *vel2 = tan * dp_tan.y + norm * m2;

    true
}

pub fn mouse_relative_forward(target: Vec2, mouse: Point2<f32>, conf: &Config) -> Vec2 {
    let (sw, sh) = (conf.screen_width, conf.screen_height);
    let (ww, wh) = (conf.window_width, conf.window_height);
    let (mx, my) = (mouse.x, mouse.y);

    let m = get_mouse_screen_coords(mx, my, sw, sh, ww, wh);

    let dx = m.x - target.x;
    let dy = m.y - target.y;

    if f32::abs(dx) > f32::abs(dy) {
        return Vec2::new(f32::signum(dx), 0.);
    }
    Vec2::new(0., f32::signum(dy))
}

pub fn get_mouse_screen_coords(mx: f32, my: f32, sw: f32, sh: f32, ww: f32, wh: f32) -> Vec2 {
    Vec2::new(mx * sw / ww, my * sh / wh)
}

pub fn resolve_environment_collision(e1: &mut dyn Actor, e2: &mut dyn Actor, sw: f32, sh: f32, _delta_time: f32) {
    let mut displace_vec = Vec2::ZERO;
    if static_circle_vs_circle(&e1.get_bcircle(sw, sh), &e2.get_bcircle(sw, sh), &mut displace_vec) {
        e1.set_pos(e1.get_pos() - displace_vec);
        e2.set_pos(e2.get_pos() + displace_vec);
    }

    let (mut vel1, mut vel2) = (Vec2::ZERO, Vec2::ZERO);
    if dynamic_circle_vs_circle(&e1.get_bcircle(sw, sh), &e1.get_velocity(), &e2.get_bcircle(sw, sh), &e2.get_velocity(), &mut vel1, &mut vel2, _delta_time) {
        e1.set_velocity(e1.get_velocity() + vel1);
        e2.set_velocity(e2.get_velocity() - vel2);
    }
}

pub fn pos_to_room_coords(pos: Vec2, sw: f32, sh: f32) -> (usize, usize) {
    ((pos.y / sh * (ROOM_HEIGHT as f32)).floor() as usize, (pos.x / sw * (ROOM_WIDTH as f32)).floor() as usize)
}

pub fn room_coords_to_pos(i: usize, j: usize, sw: f32, sh: f32) -> Vec2 {
    Vec2::new((2. * (j as f32) - 1.) * sw / (ROOM_WIDTH as f32) / 2., (2. * (i as f32) - 1.) * sh / (ROOM_HEIGHT as f32) / 2.)
}

pub fn invert_color(c: &Color) -> Color {
    Color::from_rgb_u32(!c.to_rgb_u32())
}

pub fn change_scene(conf: &mut Config, new_state: Option<State>) {
    let cur = conf.current_state;
    let prev = conf.previous_state;

    match new_state {
        Some(s) => conf.current_state = s,
        None => conf.current_state = prev,
    }

    conf.previous_state = cur;
}

pub fn change_volume(conf: &mut Config, change: f32) {
    conf.volume = (conf.volume + change * 0.01).clamp(0., 1.);
    for (_, s) in conf.assets.audio.iter_mut() {
        s.set_volume(conf.volume);
    }
    println!("{:?}", conf.volume);
}
