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
    Damaged,
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
    pub max_health: f32,
    pub shoot_rate: f32,
    pub shoot_range: f32,
    pub shoot_timeout: f32,
    pub damaged_cooldown: f32,
    pub animation_cooldown: f32,
}

impl Model for Player {
    fn update(&mut self, _delta_time: f32) -> GameResult {
        self.props.velocity = self.props.translation * PLAYER_SPEED * _delta_time;
        self.props.pos.0 += self.props.velocity;
        self.shoot_timeout = f32::max(0., self.shoot_timeout - _delta_time);
        self.damaged_cooldown = f32::max(0., self.damaged_cooldown - _delta_time);
        self.animation_cooldown = f32::max(0., self.animation_cooldown - _delta_time);

        if self.animation_cooldown == 0. { self.state = ActorState::Base; }
        if self.health <= 0. { self.state = ActorState::Dead; }

        Ok(())
    }

    fn draw(&self, ctx: &mut Context, assets: &Assets, conf: &Config) -> GameResult {
        let (sw, sh) = (conf.screen_width, conf.screen_height);
        let draw_params = DrawParam::default()
            .dest(self.props.pos)
            .scale(self.scale_to_screen(sw, sh, assets.player_base.dimensions()))
            .offset([0.5, 0.5]);

        match self.state {
            ActorState::Shoot => {
                if self.props.forward == Vec2::X { graphics::draw(ctx, &assets.player_shoot_east, draw_params)?; }
                else if self.props.forward == -Vec2::X { graphics::draw(ctx, &assets.player_shoot_west, draw_params)?; }
                else if self.props.forward == -Vec2::Y { graphics::draw(ctx, &assets.player_shoot_north, draw_params)?; }
                else if self.props.forward == Vec2::Y { graphics::draw(ctx, &assets.player_shoot_south, draw_params)?; }
                else { graphics::draw(ctx, &assets.player_base, draw_params)?; }
            },
            ActorState::Damaged => graphics::draw(ctx, &assets.player_damaged, draw_params.color(Color::RED))?,
            ActorState::Dead => graphics::draw(ctx, &assets.player_dead, draw_params)?,
            _ => graphics::draw(ctx, &assets.player_base, draw_params)?,
        }

        if conf.draw_bbox_model { self.draw_bbox(ctx, (sw, sh))?; }

        Ok(())
    }

    fn get_pos(&self) -> Vec2 { self.props.pos.into() }

    fn get_scale(&self) -> Vec2 { self.props.scale }

    fn get_velocity(&self) -> Vec2 { self.props.velocity }

    fn get_translation(&self) -> Vec2 { self.props.translation }

    fn get_forward(&self) -> Vec2 { self.props.forward }

    fn as_any(&self) -> &dyn Any { self }

    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

impl Actor for Player {
    fn get_health(&self) -> f32 { self.health }

    fn damage(&mut self, dmg: f32) { 
        if self.damaged_cooldown <= 0. {
            self.health -= dmg; 
            self.state = ActorState::Damaged;
            self.damaged_cooldown = PLAYER_DAMAGED_COOLDOWN;
            self.animation_cooldown = ANIMATION_COOLDOWN / self.damaged_cooldown;
        }
    }
}

impl Shooter for Player {
    fn shoot(&mut self, shots: &mut Vec<Shot>) -> GameResult {
        if self.shoot_timeout != 0. {
            return Ok(());
        }

        if self.state != ActorState::Shoot {
            self.state = ActorState::Shoot;
            self.animation_cooldown = ANIMATION_COOLDOWN / self.shoot_rate;
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

    fn draw(&self, ctx: &mut Context, assets: &Assets, conf: &Config) -> GameResult {
        let (sw, sh) = (conf.screen_width, conf.screen_height);
        let draw_params = DrawParam::default()
            .dest(self.props.pos)
            .scale(self.scale_to_screen(sw, sh, assets.shot_puke_base.dimensions()))
            .offset([0.5, 0.5]);

        match self.tag {
            ShotTag::Player => graphics::draw(ctx, &assets.shot_puke_base, draw_params)?,
            ShotTag::Enemy => graphics::draw(ctx, &assets.shot_blood_base, draw_params)?,
        }

        if conf.draw_bbox_model { self.draw_bbox(ctx, (sw, sh))?; }

        Ok(())
    }

    fn get_pos(&self) -> Vec2 { self.props.pos.into() }

    fn get_scale(&self) -> Vec2 { self.props.scale }

    fn get_velocity(&self) -> Vec2 { self.props.velocity }

    fn get_translation(&self) -> Vec2 { self.props.translation }

    fn get_forward(&self) -> Vec2 { self.props.forward }

    fn as_any(&self) -> &dyn Any { self }

    fn as_any_mut(&mut self) -> &mut dyn Any { self }
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
    pub animation_cooldown: f32,
}

impl Model for EnemyMask {
    fn update(&mut self, _delta_time: f32) -> GameResult {
        self.props.velocity = self.props.translation * ENEMY_SPEED * _delta_time;
        self.props.pos.0 += self.props.velocity;
        self.shoot_timeout = f32::max(0., self.shoot_timeout - _delta_time);
        self.animation_cooldown = f32::max(0., self.animation_cooldown - _delta_time);

        if self.animation_cooldown == 0. { self.state = ActorState::Base; }
        if self.health <= 0. { self.state = ActorState::Dead; }

        Ok(())
    }

    fn draw(&self, ctx: &mut Context, assets: &Assets, conf: &Config) -> GameResult {
        let (sw, sh) = (conf.screen_width, conf.screen_height);
        let draw_params = DrawParam::default()
            .dest(self.props.pos)
            .scale(self.scale_to_screen(sw, sh, assets.enemy_mask_base.dimensions()))
            .offset([0.5, 0.5]);

        match self.state {
            ActorState::Damaged => graphics::draw(ctx, &assets.enemy_mask_base, draw_params.color(Color::RED))?,
            _ => graphics::draw(ctx, &assets.enemy_mask_base, draw_params)?,
        }

        if conf.draw_bbox_model { self.draw_bbox(ctx, (sw, sh))?; }

        Ok(())
    }

    fn get_pos(&self) -> Vec2 { self.props.pos.into() }

    fn get_scale(&self) -> Vec2 { self.props.scale }

    fn get_velocity(&self) -> Vec2 { self.props.velocity }

    fn get_translation(&self) -> Vec2 { self.props.translation }

    fn get_forward(&self) -> Vec2 { self.props.forward }

    fn as_any(&self) -> &dyn Any { self }

    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

impl Actor for EnemyMask {
    fn get_health(&self) -> f32 { self.health }

    fn damage(&mut self, dmg: f32) { 
        self.health -= dmg; 
        self.state = ActorState::Damaged;
        self.animation_cooldown = ANIMATION_COOLDOWN;
    }
}

impl Shooter for EnemyMask {
    fn shoot(&mut self, shots: &mut Vec<Shot>) -> GameResult {
        if self.shoot_timeout != 0. {
            return Ok(());
        }

        self.state = ActorState::Shoot;

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
