use ggez::{
    graphics::{self, DrawParam, Color},
    GameResult,
    Context,
};
use crate::{
    utils::*,
    assets::*,
    consts::*,
    traits::*,
};
use glam::f32::{Vec2};
use std::{
    any::Any,
};

#[derive(Clone, Debug, Copy, PartialEq)]
pub enum ActorState {
    Base,
    Shoot,
    Dead,
}

#[derive(Clone, Debug, Copy)]
pub struct ActorProps {
    pub pos: Vec2Wrap,
    pub scale: Vec2,
    pub translation: Vec2,
    pub forward: Vec2,
    pub velocity: Vec2,
}

#[derive(Clone, Debug, Copy)]
pub struct Player {
    pub props: ActorProps,
    pub speed: f32,
    pub state: ActorState,
    pub health: f32,
    pub shoot_rate: f32,
    pub shoot_range: f32,
    pub shoot_timeout: f32,
    pub color: Color,
}

impl Model for Player {
    fn update(&mut self, _delta_time: f32) -> GameResult {
        self.props.velocity = self.props.translation * PLAYER_SPEED * _delta_time;
        self.props.pos.0 += self.props.velocity;
        self.shoot_timeout = f32::max(0., self.shoot_timeout - _delta_time);

        if self.health <= 0. { self.state = ActorState::Dead; }

        Ok(())
    }

    fn draw(&self, ctx: &mut Context, assets: &Assets, screen: (f32, f32), config: &Config) -> GameResult {
        let (sw, sh) = screen;
        let pos: Vec2Wrap = world_to_screen_space(sw, sh, self.props.pos.into()).into();
        let draw_params = DrawParam::default()
            .dest(pos)
            .scale(self.scale_to_screen(sw, sh, assets.player_base.dimensions()))
            .offset([0.5, 0.5])
            .color(self.color);

        match self.state {
            ActorState::Shoot => {
                if self.props.forward == Vec2::X { graphics::draw(ctx, &assets.player_shoot_east, draw_params)?; }
                else if self.props.forward == -Vec2::X { graphics::draw(ctx, &assets.player_shoot_west, draw_params)?; }
                else if self.props.forward == Vec2::Y { graphics::draw(ctx, &assets.player_shoot_north, draw_params)?; }
                else if self.props.forward == -Vec2::Y { graphics::draw(ctx, &assets.player_shoot_south, draw_params)?; }
                else { graphics::draw(ctx, &assets.player_base, draw_params)?; }
            },
            _ => graphics::draw(ctx, &assets.player_base, draw_params)?,
        }

        if config.draw_bbox_model {
            self.draw_bbox(ctx, screen)?;
        }

        Ok(())
    }

    fn get_pos(&self) -> Vec2 { self.props.pos.into() }

    fn get_scale(&self) -> Vec2 { self.props.scale }

    fn get_velocity(&self) -> Vec2 { self.props.velocity }

    fn get_translation(&self) -> Vec2 { self.props.translation }

    fn get_forward(&self) -> Vec2 { self.props.forward }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Actor for Player {
    fn get_health(&self) -> f32 { self.health }

    fn damage(&mut self, dmg: f32) { self.health -= dmg; }
}

impl Shooter for Player {
    fn shoot(&mut self, shots: &mut Vec<Shot>) -> GameResult {
        if self.shoot_timeout != 0. {
            return Ok(());
        }

        self.shoot_timeout = 1. / self.shoot_rate;
        let shot_dir = (self.props.forward + 0.5 * (self.props.translation * Vec2::new(self.props.forward.y, self.props.forward.x).abs())).normalize();

        let shot = Shot {
            props: ActorProps {
                pos: self.props.pos,
                scale: Vec2::splat(SHOT_SCALE),
                translation: shot_dir,
                forward: shot_dir,
                velocity: Vec2::ZERO,
            },
            spawn_pos: self.props.pos,
            speed: SHOT_SPEED,
            range: self.shoot_range,
            damage: PLAYER_DAMAGE,
            tag: ShotTag::Player,
        };

        shots.push(shot);

        Ok(())
    }
}

#[derive(Clone, Debug, Copy)]
pub enum ShotTag {
    Player,
    Enemy,
}

#[derive(Clone, Debug, Copy)]
pub struct Shot {
    pub props: ActorProps,
    pub speed: f32,
    pub range: f32,
    pub spawn_pos: Vec2Wrap,
    pub damage: f32,
    pub tag: ShotTag,
}

impl Model for Shot {
    fn update(&mut self, _delta_time: f32) -> GameResult {
        self.props.velocity = self.props.translation * SHOT_SPEED * _delta_time;
        self.props.pos.0 += self.props.velocity;

        Ok(())
    }

    fn draw(&self, ctx: &mut Context, assets: &Assets, screen: (f32, f32), config: &Config) -> GameResult {
        let (sw, sh) = screen;
        let pos: Vec2Wrap = world_to_screen_space(sw, sh, self.props.pos.into()).into();
        let draw_params = DrawParam::default()
            .dest(pos)
            .scale(self.scale_to_screen(sw, sh, assets.shot_base.dimensions()))
            .offset([0.5, 0.5]);

        graphics::draw(ctx, &assets.shot_base, draw_params)?;

        if config.draw_bbox_model {
            self.draw_bbox(ctx, screen)?;
        }

        Ok(())
    }

    fn get_pos(&self) -> Vec2 { self.props.pos.into() }

    fn get_scale(&self) -> Vec2 { self.props.scale }

    fn get_velocity(&self) -> Vec2 { self.props.velocity }

    fn get_translation(&self) -> Vec2 { self.props.translation }

    fn get_forward(&self) -> Vec2 { self.props.forward }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
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
    pub color: Color,
}

impl Model for EnemyMask {
    fn update(&mut self, _delta_time: f32) -> GameResult {
        self.props.velocity = self.props.translation * ENEMY_SPEED * _delta_time;
        self.props.pos.0 += self.props.velocity;
        self.shoot_timeout = f32::max(0., self.shoot_timeout - _delta_time);

        if self.health <= 0. { self.state = ActorState::Dead; }

        Ok(())
    }

    fn draw(&self, ctx: &mut Context, assets: &Assets, screen: (f32, f32), config: &Config) -> GameResult {
        let (sw, sh) = screen;
        let pos: Vec2Wrap = world_to_screen_space(sw, sh, self.props.pos.into()).into();
        let draw_params = DrawParam::default()
            .dest(pos)
            .scale(self.scale_to_screen(sw, sh, assets.enemy_mask_base.dimensions()))
            .color(self.color)
            .offset([0.5, 0.5]);

        match self.state {
            _ => graphics::draw(ctx, &assets.enemy_mask_base, draw_params)?,
        }

        if config.draw_bbox_model {
            self.draw_bbox(ctx, screen)?;
        }

        Ok(())
    }

    fn get_pos(&self) -> Vec2 { self.props.pos.into() }

    fn get_scale(&self) -> Vec2 { self.props.scale }

    fn get_velocity(&self) -> Vec2 { self.props.velocity }

    fn get_translation(&self) -> Vec2 { self.props.translation }

    fn get_forward(&self) -> Vec2 { self.props.forward }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Actor for EnemyMask {
    fn get_health(&self) -> f32 { self.health }

    fn damage(&mut self, dmg: f32) { self.health -= dmg; }
}

impl Shooter for EnemyMask {
    fn shoot(&mut self, shots: &mut Vec<Shot>) -> GameResult {
        if self.shoot_timeout != 0. {
            return Ok(());
        }

        self.shoot_timeout = 1. / self.shoot_rate;
        let shot_dir = self.props.forward.normalize();

        let shot = Shot {
            props: ActorProps {
                pos: self.props.pos,
                scale: Vec2::splat(SHOT_SCALE),
                translation: shot_dir,
                forward: shot_dir,
                velocity: Vec2::ZERO,
            },
            spawn_pos: self.props.pos,
            speed: SHOT_SPEED,
            range: self.shoot_range,
            damage: ENEMY_DAMAGE,
            tag: ShotTag::Enemy,
        };

        shots.push(shot);

        Ok(())
    }
}
