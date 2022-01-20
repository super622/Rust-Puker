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
};
use glam::f32::{Vec2};
use std::{
    any::Any,
};
use rand::{thread_rng, Rng};

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
    Enemy(EnemyTag),
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
    pub damage: f32,
    pub shoot_rate: f32,
    pub shoot_range: f32,
    pub shoot_timeout: f32,
    pub damaged_cooldown: f32,
    pub animation_cooldown: f32,
    pub afterlock_cooldown: f32,
}

impl Model for Player {
    fn update(&mut self, ctx: &mut Context, assets: &mut Assets, _conf: &Config, _delta_time: f32) -> GameResult {
        self.afterlock_cooldown = f32::max(0., self.afterlock_cooldown - _delta_time);

        if self.afterlock_cooldown == 0. {
            self.velocity_lerp(self.speed, 10., _delta_time);
        }

        self.props.pos.0 += self.props.velocity;

        self.shoot_timeout = f32::max(0., self.shoot_timeout - _delta_time);
        self.damaged_cooldown = f32::max(0., self.damaged_cooldown - _delta_time);
        self.animation_cooldown = f32::max(0., self.animation_cooldown - _delta_time);

        if self.animation_cooldown == 0. { self.state = ActorState::Base; }
        if self.health <= 0. {
            assets.audio.get_mut("player_death_sound").unwrap().play(ctx)?; 
            self.state = ActorState::Dead;
        }

        Ok(())
    }

    fn draw(&self, ctx: &mut Context, assets: &mut Assets, conf: &Config) -> GameResult {
        let (sw, sh) = (conf.screen_width, conf.screen_height);
        let draw_params = DrawParam::default()
            .dest(self.props.pos)
            .scale(self.scale_to_screen(sw, sh, assets.sprites.get("player_base").unwrap().dimensions()))
            .offset([0.5, 0.5]);

        match self.state {
            ActorState::Shoot => {
                if self.props.forward == Vec2::X { graphics::draw(ctx, assets.sprites.get("player_shoot_east").unwrap(), draw_params)?; }
                else if self.props.forward == -Vec2::X { graphics::draw(ctx, assets.sprites.get("player_shoot_west").unwrap(), draw_params)?; }
                else if self.props.forward == -Vec2::Y { graphics::draw(ctx, assets.sprites.get("player_shoot_north").unwrap(), draw_params)?; }
                else if self.props.forward == Vec2::Y { graphics::draw(ctx, assets.sprites.get("player_shoot_south").unwrap(), draw_params)?; }
                else { graphics::draw(ctx, assets.sprites.get("player_base").unwrap(), draw_params)?; }
            },
            ActorState::Damaged => {
                assets.audio.get_mut("player_damaged_sound").unwrap().play(ctx)?;
                graphics::draw(ctx, assets.sprites.get("player_damaged").unwrap(), draw_params.color(Color::RED))?;
            },
            ActorState::Dead => graphics::draw(ctx, assets.sprites.get("player_dead").unwrap(), draw_params)?,
            _ => graphics::draw(ctx, assets.sprites.get("player_base").unwrap(), draw_params)?,
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

    fn heal(&mut self, heal: f32) {
        self.health = f32::min(self.health + heal, self.max_health);
    }

    fn get_tag(&self) -> ActorTag { ActorTag::Player }
}

impl Player {
    pub fn shoot(&mut self, shots: &mut Vec<Shot>) -> GameResult {
        if self.shoot_timeout != 0. {
            return Ok(());
        }

        if self.state != ActorState::Shoot {
            self.state = ActorState::Shoot;
            self.animation_cooldown = ANIMATION_COOLDOWN / self.shoot_rate;
        }

        self.shoot_timeout = 1. / self.shoot_rate;
        let shot_dir = (self.props.forward + 0.5 * (self.props.velocity.clamp_length_max(0.5) * Vec2::new(self.props.forward.y, self.props.forward.x).abs())).normalize();

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
            damage: self.damage,
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
    fn update(&mut self, _ctx: &mut Context, _assets: &mut Assets, _conf: &Config, _delta_time: f32) -> GameResult {
        self.props.velocity = self.props.translation * SHOT_SPEED * _delta_time;
        self.props.pos.0 += self.props.velocity;

        Ok(())
    }

    fn draw(&self, ctx: &mut Context, assets: &mut Assets, conf: &Config) -> GameResult {
        let (sw, sh) = (conf.screen_width, conf.screen_height);
        let draw_params = DrawParam::default()
            .dest(self.props.pos)
            .scale(self.scale_to_screen(sw, sh, assets.sprites.get("shot_puke_base").unwrap().dimensions()))
            .offset([0.5, 0.5]);

        match self.tag {
            ShotTag::Player => graphics::draw(ctx, assets.sprites.get("shot_puke_base").unwrap(), draw_params)?,
            ShotTag::Enemy => graphics::draw(ctx, assets.sprites.get("shot_blood_base").unwrap(), draw_params)?,
        }

        if conf.draw_bcircle_model { self.draw_bcircle(ctx, (sw, sh))?; }

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

#[derive(Clone, Debug, Copy)]
pub enum EnemyTag {
    Shooter,
    Chaser,
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

impl Model for EnemyMask {
    fn update(&mut self, ctx: &mut Context, assets: &mut Assets, _conf: &Config, _delta_time: f32) -> GameResult {
        self.afterlock_cooldown = f32::max(0., self.afterlock_cooldown - _delta_time);

        if self.afterlock_cooldown == 0. {
            self.velocity_lerp(0., 5., _delta_time);
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

    fn get_tag(&self) -> ActorTag { ActorTag::Enemy(self.tag) }
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

impl Model for EnemyBlueGuy {
    fn update(&mut self, ctx: &mut Context, assets: &mut Assets, _conf: &Config, _delta_time: f32) -> GameResult {
        self.afterlock_cooldown = f32::max(0., self.afterlock_cooldown - _delta_time);
        
        if self.afterlock_cooldown == 0. {
            self.velocity_lerp(self.speed, 10., _delta_time);
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

    fn as_any(&self) -> &dyn Any { self }

    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

impl Actor for EnemyBlueGuy {
    fn get_health(&self) -> f32 { self.health }

    fn damage(&mut self, dmg: f32) { 
        self.health -= dmg; 
        self.state = ActorState::Damaged;
        self.animation_cooldown = ANIMATION_COOLDOWN;
    }

    fn get_tag(&self) -> ActorTag { ActorTag::Enemy(self.tag) }
}

impl Chaser for EnemyBlueGuy {
    fn chase(&mut self, target: Vec2) {
        if self.afterlock_cooldown == 0. {
            self.props.translation = (target - self.get_pos()).normalize_or_zero();
            self.props.forward = self.props.translation;
        }
    }
}
