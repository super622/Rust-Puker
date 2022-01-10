use ggez::{
    graphics::{self, Text, PxScale, DrawParam, Rect, DrawMode, Color, Mesh},
    GameResult,
    Context,
};
use crate::{
    utils::*,
    assets::*,
    consts::*,
};
use glam::f32::{Vec2};
use std::{
    collections::{VecDeque},
};

pub trait Model: std::fmt::Debug {
    fn update(&mut self, _delta_time: f32) -> GameResult;

    fn draw(&self, ctx: &mut Context, assets: &Assets, screen: (f32, f32)) -> GameResult;

    fn draw_bbox(&self, ctx: &mut Context, screen: (f32, f32)) -> GameResult {
        let (sw, sh) = screen;
        let mut bbox = self.get_bbox(sw, sh);
        let mut text = Text::new(format!("{:?}, {:?}", bbox.x, bbox.y));
        let screen_coords = world_to_screen_space(sw, sh, Vec2::new(bbox.x, bbox.y));
        bbox.x = screen_coords.x;
        bbox.y = screen_coords.y;

        let mesh = Mesh::new_rectangle(ctx, DrawMode::stroke(2.0), bbox, Color::BLUE)?;
        graphics::draw(ctx, &mesh, DrawParam::default())?;

        text.fragments_mut().iter_mut().map(|x| x.scale = Some(PxScale { x: 0.5, y: 0.5 }));
        graphics::draw(ctx, &text, DrawParam::default().dest([bbox.x, bbox.y - text.height(ctx)]))?;

        Ok(())
    }

    fn scale_to_screen(&self, sw: f32, sh: f32, image: Rect) -> Vec2 {
        let bbox = self.get_bbox(sw, sh);
        Vec2::new(bbox.w / image.w, bbox.h / image.h)
    }

    fn get_bbox(&self, sw: f32, sh: f32) -> graphics::Rect {
        let width = sw / ROOM_WIDTH * self.get_scale().x;
        let height = sh / ROOM_HEIGHT * self.get_scale().y;
        Rect::new(self.get_pos().x - width / 2., self.get_pos().y + height / 2., width, height)
    }

    fn get_pos(&self) -> Vec2;

    fn get_scale(&self) -> Vec2;

    fn get_velocity(&self, _delta_time: f32) -> Vec2;

    fn get_forward(&self) -> Vec2;
}

pub trait Stationary: std::fmt::Debug {
    fn draw(&self, ctx: &mut Context, assets: &Assets, screen: (f32, f32)) -> GameResult;

    fn draw_bbox(&self, ctx: &mut Context, screen: (f32, f32)) -> GameResult {
        let (sw, sh) = screen;
        let mut bbox = self.get_bbox(sw, sh);
        let mut text = Text::new(format!("{:?}, {:?}", bbox.x, bbox.y));
        let screen_coords = world_to_screen_space(sw, sh, Vec2::new(bbox.x, bbox.y));
        bbox.x = screen_coords.x;
        bbox.y = screen_coords.y;

        let mesh = Mesh::new_rectangle(ctx, DrawMode::stroke(2.0), bbox, Color::BLUE)?;
        graphics::draw(ctx, &mesh, DrawParam::default())?;

        text.fragments_mut().iter_mut().map(|x| x.scale = Some(PxScale { x: 0.5, y: 0.5 }));
        graphics::draw(ctx, &text, DrawParam::default().dest([bbox.x, bbox.y - text.height(ctx)]))?;

        Ok(())
    }

    fn scale_to_screen(&self, sw: f32, sh: f32, image: Rect) -> Vec2 {
        let bbox = self.get_bbox(sw, sh);
        Vec2::new(bbox.w / image.w, bbox.h / image.h)
    }

    fn get_bbox(&self, sw: f32, sh: f32) -> graphics::Rect {
        let width = sw / ROOM_WIDTH * self.get_scale().x;
        let height = sh / ROOM_HEIGHT * self.get_scale().y;
        Rect::new(self.get_pos().x - width / 2., self.get_pos().y + height / 2., width, height)
    }

    fn get_pos(&self) -> Vec2;

    fn get_scale(&self) -> Vec2;
}

pub trait Actor {
    fn get_health(&self) -> f32; 

    fn damage(&mut self, dmg: f32);
}

pub trait Shooter {
    fn shoot(&mut self) -> GameResult;
}

#[derive(Clone, Debug)]
pub enum ActorState {
    BASE,
    SHOOT,
}

#[derive(Clone, Debug)]
pub enum ActorTag {
    PLAYER,
    ENEMY,
}

#[derive(Clone, Debug, Copy)]
pub struct ActorProps {
    pub pos: Vec2Wrap,
    pub scale: Vec2,
    pub translation: Vec2,
    pub forward: Vec2,
}

#[derive(Clone, Debug)]
pub struct Player {
    pub props: ActorProps,
    pub speed: f32,
    pub state: ActorState,
    pub health: f32,
    pub shoot_rate: f32,
    pub shoot_range: f32,
    pub shoot_timeout: f32,
    pub shots: Vec<Shot>,
    pub color: Color,
}

impl Model for Player {
    fn update(&mut self, _delta_time: f32) -> GameResult {
        self.props.pos.0 += self.get_velocity(_delta_time);
        self.shoot_timeout = f32::max(0., self.shoot_timeout - _delta_time);

        let mut shots_gone = VecDeque::<usize>::new();

        for (i, shot) in self.shots.iter_mut().enumerate() {
            shot.update(_delta_time)?;
            if shot.props.pos.0.distance(shot.spawn_pos.0) > self.shoot_range { shots_gone.push_back(i); } 
        }

        for shot in shots_gone {
            self.shots.remove(shot);
        }

        match self.state {
            ActorState::SHOOT => {
                if self.shoot_timeout == 0. {
                    self.shoot()?;
                }
            },
            _ => (),
        }

        Ok(())
    }

    fn draw(&self, ctx: &mut Context, assets: &Assets, screen: (f32, f32)) -> GameResult {
        let (sw, sh) = screen;
        let pos: Vec2Wrap = world_to_screen_space(sw, sh, self.props.pos.into()).into();
        let draw_params = DrawParam::default()
            .dest(pos)
            .scale(self.scale_to_screen(sw, sh, assets.player_base.dimensions()))
            .offset([0.5, 0.5])
            .color(self.color);

        for shot in self.shots.iter() {
            shot.draw(ctx, assets, screen)?;
        }

        match self.state {
            ActorState::BASE => graphics::draw(ctx, &assets.player_base, draw_params)?,
            ActorState::SHOOT => {
                match self.props.forward.to_array() {
                    [ 1., 0.] => graphics::draw(ctx, &assets.player_shoot_east, draw_params)?,
                    [-1., 0.] => graphics::draw(ctx, &assets.player_shoot_west, draw_params)?,
                    [0.,  1.] => graphics::draw(ctx, &assets.player_shoot_north, draw_params)?,
                    [0., -1.] => graphics::draw(ctx, &assets.player_shoot_south, draw_params)?,
                    _ => graphics::draw(ctx, &assets.player_base, draw_params)?,
                }
            },
            _ => ()
        }

        self.draw_bbox(ctx, screen)?;

        Ok(())
    }

    fn get_pos(&self) -> Vec2 { self.props.pos.into() }

    fn get_scale(&self) -> Vec2 { self.props.scale }

    fn get_velocity(&self, delta_time: f32) -> Vec2 { self.props.translation * self.speed * delta_time }

    fn get_forward(&self) -> Vec2 { self.props.forward }
}

impl Actor for Player {
    fn get_health(&self) -> f32 { self.health }

    fn damage(&mut self, dmg: f32) { self.health -= dmg; }
}

impl Shooter for Player {
    fn shoot(&mut self) -> GameResult {
        self.shoot_timeout = 1. / self.shoot_rate;
        let shot_dir = (self.props.forward + 0.5 * (self.props.translation * Vec2::new(self.props.forward.y, self.props.forward.x).abs())).normalize();

        let shot = Shot {
            props: ActorProps {
                pos: self.props.pos,
                forward: shot_dir,
                translation: shot_dir,
                scale: Vec2::splat(SHOT_SCALE),
            },
            spawn_pos: self.props.pos,
            speed: SHOT_SPEED,
            damage: PLAYER_DAMAGE,
        };

        self.shots.push(shot);

        Ok(())
    }
}

#[derive(Clone, Debug, Copy)]
pub struct Shot {
    pub props: ActorProps,
    pub speed: f32,
    pub spawn_pos: Vec2Wrap,
    pub damage: f32,
}

impl Model for Shot {
    fn update(&mut self, _delta_time: f32) -> GameResult {
        self.props.pos.0 += self.get_velocity(_delta_time);

        Ok(())
    }

    fn draw(&self, ctx: &mut Context, assets: &Assets, screen: (f32, f32)) -> GameResult {
        let (sw, sh) = screen;
        let pos: Vec2Wrap = world_to_screen_space(sw, sh, self.props.pos.into()).into();
        let draw_params = DrawParam::default()
            .dest(pos)
            .scale(self.scale_to_screen(sw, sh, assets.shot_base.dimensions()))
            .offset([0.5, 0.5]);

        graphics::draw(ctx, &assets.shot_base, draw_params)?;

        self.draw_bbox(ctx, screen)?;

        Ok(())
    }

    fn get_pos(&self) -> Vec2 { self.props.pos.into() }

    fn get_scale(&self) -> Vec2 { self.props.scale }

    fn get_velocity(&self, _delta_time: f32) -> Vec2 { self.props.translation * self.speed * _delta_time }

    fn get_forward(&self) -> Vec2 { self.props.forward }
}

#[derive(Clone, Debug)]
pub struct EnemyMask {
    pub props: ActorProps,
    pub speed: f32,
    pub state: ActorState,
    pub health: f32,
    pub shoot_rate: f32,
    pub shoot_range: f32,
    pub shoot_timeout: f32,
    pub shots: Vec<Shot>,
    pub color: Color,
}

impl Model for EnemyMask {
    fn update(&mut self, _delta_time: f32) -> GameResult {
        self.props.pos.0 += self.get_velocity(_delta_time);
        self.shoot_timeout = f32::max(0., self.shoot_timeout - _delta_time);

        let mut shots_gone = VecDeque::<usize>::new();

        for (i, shot) in self.shots.iter_mut().enumerate() {
            shot.update(_delta_time)?;
            if shot.props.pos.0.distance(shot.spawn_pos.0) > self.shoot_range { shots_gone.push_back(i); } 
        }

        for shot in shots_gone {
            self.shots.remove(shot);
        }

        match self.state {
            ActorState::SHOOT => {
                if self.shoot_timeout == 0. {
                    self.shoot()?;
                }
            },
            _ => (),
        }

        Ok(())
    }

    fn draw(&self, ctx: &mut Context, assets: &Assets, screen: (f32, f32)) -> GameResult {
        let (sw, sh) = screen;
        let pos: Vec2Wrap = world_to_screen_space(sw, sh, self.props.pos.into()).into();
        let draw_params = DrawParam::default()
            .dest(pos)
            .scale(self.scale_to_screen(sw, sh, assets.enemy_mask_base.dimensions()))
            .color(self.color)
            .offset([0.5, 0.5]);

        match self.state {
            ActorState::BASE => graphics::draw(ctx, &assets.enemy_mask_base, draw_params)?,
            _ => ()
        }

        self.draw_bbox(ctx, screen)?;

        for shot in self.shots.iter() {
            shot.draw(ctx, assets, screen)?;
        }

        Ok(())
    }

    fn get_pos(&self) -> Vec2 { self.props.pos.into() }

    fn get_scale(&self) -> Vec2 { self.props.scale }

    fn get_velocity(&self, _delta_time: f32) -> Vec2 { self.props.translation * self.speed * _delta_time }

    fn get_forward(&self) -> Vec2 { self.props.forward }
}

impl Actor for EnemyMask {
    fn get_health(&self) -> f32 { self.health }

    fn damage(&mut self, dmg: f32) { self.health -= dmg; }
}

impl Shooter for EnemyMask {
    fn shoot(&mut self) -> GameResult {
        self.shoot_timeout = 1. / self.shoot_rate;
        let shot_dir = (self.props.forward + 0.5 * (self.props.translation * Vec2::new(self.props.forward.y, self.props.forward.x).abs())).normalize();

        let shot = Shot {
            props: ActorProps {
                pos: self.props.pos,
                forward: shot_dir,
                translation: shot_dir,
                scale: Vec2::ONE,
            },
            spawn_pos: self.props.pos,
            speed: SHOT_SPEED,
            damage: ENEMY_DAMAGE,
        };

        self.shots.push(shot);

        Ok(())
    }
}
