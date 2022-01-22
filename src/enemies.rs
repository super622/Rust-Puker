use ggez::{
    graphics::{self, DrawParam, Color},
    GameResult,
    Context,
    audio::SoundSource,
};
use crate::{
    utils::*,
    assets::*,
    consts::*,
    traits::*,
    shots::*,
    player::*,
};
use glam::f32::{Vec2};
use std::{
    any::Any,
};
use rand::{thread_rng, Rng};

#[derive(Clone, Debug, Copy)]
pub enum EnemyTag {
    Shooter,
    Chaser,
    Wanderer,
}

#[derive(Clone, Debug)]
pub struct EnemyMask {
    pub props: ActorProps,
    pub tag: EnemyTag,
    pub state: ActorState,
    pub health: f32,
    pub shoot_rate: f32,
    pub shoot_range: f32,
    pub shoot_timeout: f32,
    pub animation_cooldown: f32,
    pub afterlock_cooldown: f32,
}

impl Actor for EnemyMask {
    fn update(&mut self, ctx: &mut Context, assets: &mut Assets, _conf: &Config, _grid: &[[i32; ROOM_WIDTH]], _player: Option<&Player>, _delta_time: f32) -> GameResult {
        self.afterlock_cooldown = f32::max(0., self.afterlock_cooldown - _delta_time);

        if self.afterlock_cooldown == 0. {
            self.velocity_lerp(_delta_time, 0., 5., 0.);
        }
        self.props.pos.0 += self.props.velocity;

        self.shoot_timeout = f32::max(0., self.shoot_timeout - _delta_time);
        self.animation_cooldown = f32::max(0., self.animation_cooldown - _delta_time);

        if self.animation_cooldown == 0. { self.state = ActorState::Base; }
        if self.health <= 0. {
            assets.audio.get_mut("enemy_death_sound").unwrap().play(ctx)?; 
            self.state = ActorState::Dead;
        }

        Ok(())
    }

    fn draw(&self, ctx: &mut Context, assets: &mut Assets, conf: &Config) -> GameResult {
        let (sw, sh) = (conf.screen_width, conf.screen_height);

        let tremble: f32 = (thread_rng().gen::<f32>() * 2. - 1.) * 0.1;
        let sign = thread_rng().gen_range(-1..2) as f32;

        let draw_params = DrawParam::default()
            .dest(self.props.pos)
            .scale(self.scale_to_screen(sw, sh, assets.sprites.get("enemy_mask_base").unwrap().dimensions()))
            .offset([0.5 + tremble, 0.5 + tremble * sign]);

        match self.state {
            ActorState::Damaged => graphics::draw(ctx, assets.sprites.get("enemy_mask_base").unwrap(), draw_params.color(Color::RED))?,
            _ => graphics::draw(ctx, assets.sprites.get("enemy_mask_base").unwrap(), draw_params)?,
        }

        if conf.draw_bcircle_model { self.draw_bcircle(ctx, (sw, sh))?; }

        Ok(())
    }

    fn get_pos(&self) -> Vec2 { self.props.pos.into() }

    fn get_scale(&self) -> Vec2 { self.props.scale }

    fn get_velocity(&self) -> Vec2 { self.props.velocity }

    fn get_translation(&self) -> Vec2 { self.props.translation }

    fn get_forward(&self) -> Vec2 { self.props.forward }

    fn set_pos(&mut self, new_pos: Vec2) { self.props.pos = new_pos.into(); }

    fn set_scale(&mut self, new_scale: Vec2) { self.props.scale = new_scale; }

    fn set_velocity(&mut self, new_velocity: Vec2) { self.props.velocity = new_velocity; } 

    fn set_translation(&mut self, new_translation: Vec2) { self.props.translation = new_translation; }

    fn set_forward(&mut self, new_forward: Vec2) { self.props.forward = new_forward; } 

    fn get_health(&self) -> f32 { self.health }

    fn damage(&mut self, dmg: f32) { 
        self.health -= dmg; 
        self.state = ActorState::Damaged;
        self.animation_cooldown = ANIMATION_COOLDOWN;
    }

    fn get_tag(&self) -> ActorTag { ActorTag::Enemy(self.tag) }

    fn as_any(&self) -> &dyn Any { self }

    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

impl Shooter for EnemyMask {
    fn shoot(&mut self, target: &Vec2, shots: &mut Vec<Shot>) -> GameResult {
        if self.shoot_timeout != 0. || self.afterlock_cooldown != 0. {
            return Ok(());
        }

        self.props.forward = *target - self.get_pos();
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

    fn get_range(&self) -> f32 { self.shoot_range }

    fn get_rate(&self) -> f32 { self.shoot_rate }
}

#[derive(Clone, Debug)]
pub struct EnemyBlueGuy {
    pub props: ActorProps,
    pub tag: EnemyTag,
    pub speed: f32,
    pub state: ActorState,
    pub health: f32,
    pub animation_cooldown: f32,
    pub afterlock_cooldown: f32,
}

impl Actor for EnemyBlueGuy {
    fn update(&mut self, ctx: &mut Context, assets: &mut Assets, _conf: &Config, _grid: &[[i32; ROOM_WIDTH]], _player: Option<&Player>, _delta_time: f32) -> GameResult {
        self.afterlock_cooldown = f32::max(0., self.afterlock_cooldown - _delta_time);
        
        if self.afterlock_cooldown == 0. {
            self.velocity_lerp(_delta_time, self.speed, 10., 20.);
        }
        self.props.pos.0 += self.props.velocity;

        self.animation_cooldown = f32::max(0., self.animation_cooldown - _delta_time);

        if self.animation_cooldown == 0. { self.state = ActorState::Base; }
        if self.health <= 0. {
            assets.audio.get_mut("enemy_death_sound").unwrap().play(ctx)?; 
            self.state = ActorState::Dead;
        }

        Ok(())
    }

    fn draw(&self, ctx: &mut Context, assets: &mut Assets, conf: &Config) -> GameResult {
        let (sw, sh) = (conf.screen_width, conf.screen_height);
        let mut angle = self.props.forward.angle_between(Vec2::Y + self.props.pos.0 - self.props.pos.0);
        if angle.is_nan() { angle = 0.; }

        let draw_params = DrawParam::default()
            .dest(self.props.pos)
            .scale(self.scale_to_screen(sw, sh, assets.sprites.get("enemy_blue_guy_base").unwrap().dimensions()))
            .rotation(-angle)
            .offset([0.5, 0.5]);

        match self.state {
            ActorState::Damaged => graphics::draw(ctx, assets.sprites.get("enemy_blue_guy_base").unwrap(), draw_params.color(Color::RED))?,
            _ => graphics::draw(ctx, assets.sprites.get("enemy_blue_guy_base").unwrap(), draw_params)?,
        }

        if conf.draw_bcircle_model { self.draw_bcircle(ctx, (sw, sh))?; }

        Ok(())
    }

    fn get_pos(&self) -> Vec2 { self.props.pos.into() }

    fn get_scale(&self) -> Vec2 { self.props.scale }

    fn get_velocity(&self) -> Vec2 { self.props.velocity }

    fn get_translation(&self) -> Vec2 { self.props.translation }

    fn get_forward(&self) -> Vec2 { self.props.forward }

    fn set_pos(&mut self, new_pos: Vec2) { self.props.pos = new_pos.into(); }

    fn set_scale(&mut self, new_scale: Vec2) { self.props.scale = new_scale; }

    fn set_velocity(&mut self, new_velocity: Vec2) { self.props.velocity = new_velocity; } 

    fn set_translation(&mut self, new_translation: Vec2) { self.props.translation = new_translation; }

    fn set_forward(&mut self, new_forward: Vec2) { self.props.forward = new_forward; } 

    fn get_health(&self) -> f32 { self.health }

    fn damage(&mut self, dmg: f32) { 
        self.health -= dmg; 
        self.state = ActorState::Damaged;
        self.animation_cooldown = ANIMATION_COOLDOWN;
    }

    fn get_tag(&self) -> ActorTag { ActorTag::Enemy(self.tag) }

    fn as_any(&self) -> &dyn Any { self }

    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

impl Chaser for EnemyBlueGuy {
    fn chase(&mut self, target: Vec2) {
        if self.afterlock_cooldown == 0. {
            self.props.translation = (target - self.get_pos()).normalize_or_zero();
            self.props.forward = self.props.translation;
        }
    }
}

#[derive(Clone, Debug, Copy)]
pub struct EnemySlime {
    pub props: ActorProps,
    pub tag: EnemyTag,
    pub speed: f32,
    pub state: ActorState,
    pub health: f32,
    pub animation_cooldown: f32,
    pub afterlock_cooldown: f32,
    pub change_direction_cooldown: f32,
}
    
impl Actor for EnemySlime {
    fn update(&mut self, ctx: &mut Context, assets: &mut Assets, _conf: &Config, _grid: &[[i32; ROOM_WIDTH]], _player: Option<&Player>, _delta_time: f32) -> GameResult {
        let (sw, sh) = (_conf.screen_width, _conf.screen_height);

        self.afterlock_cooldown = f32::max(0., self.afterlock_cooldown - _delta_time);
        self.change_direction_cooldown = f32::max(0., self.change_direction_cooldown - _delta_time);

        if self.afterlock_cooldown == 0. {
            self.wander(_grid, sw, sh);
        }
        self.velocity_lerp(_delta_time, self.speed, 10., 10.);
        self.props.pos.0 += self.props.velocity;

        self.animation_cooldown = f32::max(0., self.animation_cooldown - _delta_time);

        if self.animation_cooldown == 0. { self.state = ActorState::Base; }
        if self.health <= 0. {
            assets.audio.get_mut("enemy_death_sound").unwrap().play(ctx)?; 
            self.state = ActorState::Dead;
        }

        Ok(())
    }

    fn draw(&self, ctx: &mut Context, assets: &mut Assets, conf: &Config) -> GameResult {
        let (sw, sh) = (conf.screen_width, conf.screen_height);

        let sprite = match self.props.forward {
            v if v == -Vec2::Y => assets.sprites.get("enemy_slime_north").unwrap(),
            v if v == Vec2::X => assets.sprites.get("enemy_slime_east").unwrap(),
            v if v == -Vec2::X => assets.sprites.get("enemy_slime_west").unwrap(),
            _ => assets.sprites.get("enemy_slime_south").unwrap(),
        };

        let draw_params = DrawParam::default()
            .dest(self.props.pos)
            .scale(self.scale_to_screen(sw, sh, sprite.dimensions()))
            .offset([0.5, 0.5]);

        match self.state {
            ActorState::Damaged => graphics::draw(ctx, sprite, draw_params.color(Color::RED))?,
            _ => graphics::draw(ctx, sprite, draw_params)?,
        }

        if conf.draw_bcircle_model { self.draw_bcircle(ctx, (sw, sh))?; }

        Ok(())
    }

    fn get_pos(&self) -> Vec2 { self.props.pos.into() }

    fn get_scale(&self) -> Vec2 { self.props.scale }

    fn get_velocity(&self) -> Vec2 { self.props.velocity }

    fn get_translation(&self) -> Vec2 { self.props.translation }

    fn get_forward(&self) -> Vec2 { self.props.forward }

    fn set_pos(&mut self, new_pos: Vec2) { self.props.pos = new_pos.into(); }

    fn set_scale(&mut self, new_scale: Vec2) { self.props.scale = new_scale; }

    fn set_velocity(&mut self, new_velocity: Vec2) { self.props.velocity = new_velocity; } 

    fn set_translation(&mut self, new_translation: Vec2) { self.props.translation = new_translation; }

    fn set_forward(&mut self, new_forward: Vec2) { self.props.forward = new_forward; } 

    fn get_health(&self) -> f32 { self.health }

    fn damage(&mut self, dmg: f32) { 
        self.health -= dmg; 
        self.state = ActorState::Damaged;
        self.animation_cooldown = ANIMATION_COOLDOWN;
    }

    fn get_tag(&self) -> ActorTag { ActorTag::Enemy(self.tag) }

    fn as_any(&self) -> &dyn Any { self }

    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

impl Wanderer for EnemySlime {
    fn get_change_direction_cooldown(&self) -> f32 { self.change_direction_cooldown }

    fn set_change_direction_cooldown(&mut self, cd: f32) { self.change_direction_cooldown = cd; }
}
