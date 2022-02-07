use ggez::{
    graphics::{self, MeshBuilder, Rect, DrawMode, DrawParam, Color},
    GameResult,
    Context,
    audio::SoundSource,
};
use crate::{
    utils::*,
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

#[derive(Clone, Debug)]
pub struct EnemyMask {
    pub props: ActorProps,
    pub state: ActorState,
    pub health: f32,
    pub damage: f32,
    pub shoot_rate: f32,
    pub shoot_range: f32,
    pub shoot_timeout: f32,
    pub animation_cooldown: f32,
    pub afterlock_cooldown: f32,
}

impl Default for EnemyMask {
    fn default() -> Self {
        Self {
            props: ActorProps::default(),
            health: ENEMY_HEALTH,
            damage: ENEMY_DAMAGE,
            state: ActorState::Base,
            shoot_rate: ENEMY_SHOOT_RATE,
            shoot_range: ENEMY_SHOOT_RANGE,
            shoot_timeout: 0.,
            animation_cooldown: 0.,
            afterlock_cooldown: ENEMY_AFTERLOCK_COOLDOWN,
        }
    }
}

impl Actor for EnemyMask {
    fn update(&mut self, ctx: &mut Context, conf: &mut Config, _delta_time: f32) -> GameResult {
        self.afterlock_cooldown = f32::max(0., self.afterlock_cooldown - _delta_time);

        self.velocity_lerp(_delta_time, 0., 5., 0.);
        self.props.pos.0 += self.props.velocity;

        self.shoot_timeout = f32::max(0., self.shoot_timeout - _delta_time);
        self.animation_cooldown = f32::max(0., self.animation_cooldown - _delta_time);

        if self.animation_cooldown == 0. { self.state = ActorState::Base; }
        if self.health <= 0. {
            conf.assets.audio.get_mut("enemy_death_sound").unwrap().play(ctx)?; 
            self.state = ActorState::Dead;
        }

        Ok(())
    }

    fn draw(&self, ctx: &mut Context, conf: &mut Config) -> GameResult {
        let (sw, sh) = (conf.screen_width, conf.screen_height);

        let tremble: f32 = (thread_rng().gen::<f32>() * 2. - 1.) * 0.1;
        let sign = thread_rng().gen_range(-1..2) as f32;

        let draw_params = DrawParam::default()
            .dest(self.props.pos)
            .scale(self.scale_to_screen(sw, sh, conf.assets.sprites.get("enemy_mask_base").unwrap().dimensions()))
            .offset([0.5 + tremble, 0.5 + tremble * sign]);

        match self.state {
            ActorState::Damaged => graphics::draw(ctx, conf.assets.sprites.get("enemy_mask_base").unwrap(), draw_params.color(Color::RED))?,
            _ => graphics::draw(ctx, conf.assets.sprites.get("enemy_mask_base").unwrap(), draw_params)?,
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

    fn get_state(&self) -> ActorState { self.state }

    fn damage(&mut self, dmg: f32) { 
        self.health -= dmg; 
        self.state = ActorState::Damaged;
        self.animation_cooldown = ANIMATION_COOLDOWN;
    }

    fn get_damage(&self) -> f32 { return self.damage }

    fn get_tag(&self) -> ActorTag { ActorTag::Enemy }

    fn act(&mut self, sw: f32, sh: f32, _grid: &[[i32; ROOM_WIDTH]], _obstacles: &Vec<Box<dyn Stationary>>, _shots: &mut Vec<Shot>, _player: &Player) -> GameResult {
        if self.afterlock_cooldown == 0. {
            self.shoot(sw, sh, _obstacles, _shots, _player)?;
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn Any { self }

    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

impl Shooter for EnemyMask {
    fn shoot(&mut self, sw: f32, sh: f32, obstacles: &Vec<Box<dyn Stationary>>, shots: &mut Vec<Shot>, player: &Player) -> GameResult {
        let (mut cp, mut cn) = (Vec2::ZERO, Vec2::ZERO);
        let mut ct = 0.;

        if self.shoot_timeout != 0. 
            || self.afterlock_cooldown != 0. 
            || self.get_pos().distance(player.get_pos()) > self.shoot_range * 0.8
            || obstacles.iter()
                .filter(|o| { ray_vs_rect(&self.get_pos(), &(player.get_pos() - self.get_pos()), &o.get_bbox(sw, sh), &mut cp, &mut cn, &mut ct) && ct < 1. })
                .count() != 0 {
            return Ok(())
        }

        self.props.forward = player.get_pos() - self.get_pos();
        self.state = ActorState::Shoot;
        self.shoot_timeout = 1. / self.shoot_rate;
        self.animation_cooldown = ANIMATION_COOLDOWN;

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
            damage: self.damage,
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
    pub speed: f32,
    pub state: ActorState,
    pub health: f32,
    pub damage: f32,
    pub animation_cooldown: f32,
    pub afterlock_cooldown: f32,
}

impl Default for EnemyBlueGuy {
    fn default() -> Self {
        Self {
            props: ActorProps::default(),
            speed: ENEMY_SPEED,
            health: ENEMY_HEALTH,
            damage: ENEMY_DAMAGE,
            state: ActorState::Base,
            animation_cooldown: 0.,
            afterlock_cooldown: ENEMY_AFTERLOCK_COOLDOWN,
        }
    }
}

impl Actor for EnemyBlueGuy {
    fn update(&mut self, ctx: &mut Context, conf: &mut Config, _delta_time: f32) -> GameResult {
        self.afterlock_cooldown = f32::max(0., self.afterlock_cooldown - _delta_time);
        
        self.velocity_lerp(_delta_time, self.speed, 10., 20.);
        self.props.pos.0 += self.props.velocity;

        self.animation_cooldown = f32::max(0., self.animation_cooldown - _delta_time);

        if self.animation_cooldown == 0. { self.state = ActorState::Base; }
        if self.health <= 0. {
            conf.assets.audio.get_mut("enemy_death_sound").unwrap().play(ctx)?; 
            self.state = ActorState::Dead;
        }

        Ok(())
    }

    fn draw(&self, ctx: &mut Context, conf: &mut Config) -> GameResult {
        let (sw, sh) = (conf.screen_width, conf.screen_height);
        let mut angle = self.props.forward.angle_between(Vec2::Y + self.props.pos.0 - self.props.pos.0);
        if angle.is_nan() { angle = 0.; }

        let draw_params = DrawParam::default()
            .dest(self.props.pos)
            .scale(self.scale_to_screen(sw, sh, conf.assets.sprites.get("enemy_blue_guy_base").unwrap().dimensions()))
            .rotation(-angle)
            .offset([0.5, 0.5]);

        match self.state {
            ActorState::Damaged => graphics::draw(ctx, conf.assets.sprites.get("enemy_blue_guy_base").unwrap(), draw_params.color(Color::RED))?,
            _ => graphics::draw(ctx, conf.assets.sprites.get("enemy_blue_guy_base").unwrap(), draw_params)?,
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

    fn get_state(&self) -> ActorState { self.state }

    fn damage(&mut self, dmg: f32) { 
        self.health -= dmg; 
        self.state = ActorState::Damaged;
        self.animation_cooldown = ANIMATION_COOLDOWN;
    }

    fn get_damage(&self) -> f32 { return self.damage }

    fn get_tag(&self) -> ActorTag { ActorTag::Enemy }

    fn act(&mut self, sw: f32, sh: f32, _grid: &[[i32; ROOM_WIDTH]], _obstacles: &Vec<Box<dyn Stationary>>, _shots: &mut Vec<Shot>, _player: &Player) -> GameResult { 
        if self.afterlock_cooldown == 0. {
            self.chase(sw, sh, _obstacles, _grid, _player);
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn Any { self }

    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

impl Chaser for EnemyBlueGuy {
    fn chase(&mut self, sw: f32, sh: f32, obstacles: &Vec<Box<dyn Stationary>>, grid: &[[i32; ROOM_WIDTH]], player: &Player) {
        if self.afterlock_cooldown == 0. {
            let (mut cp, mut cn) = (Vec2::ZERO, Vec2::ZERO);
            let mut ct = 0.;
            let mut target = player.get_pos();

            if obstacles.iter()
                .filter(|o| { ray_vs_rect(&self.get_pos(), &(player.get_pos() - self.get_pos()), &o.get_bbox(sw, sh), &mut cp, &mut cn, &mut ct) && ct < 1. })
                .count() != 0 {
                target = self.find_path(grid, sw, sh);
            }

            self.props.translation = (target - self.get_pos()).normalize_or_zero();
            self.props.forward = self.props.translation;
        }
    }
}

#[derive(Clone, Debug, Copy)]
pub struct EnemySlime {
    pub props: ActorProps,
    pub speed: f32,
    pub state: ActorState,
    pub health: f32,
    pub damage: f32,
    pub animation_cooldown: f32,
    pub afterlock_cooldown: f32,
    pub change_direction_cooldown: f32,
}

impl Default for EnemySlime {
    fn default() -> Self {
        Self {
            props: ActorProps::default(),
            speed: ENEMY_SPEED * 0.5,
            health: ENEMY_HEALTH * 1.5,
            damage: ENEMY_DAMAGE,
            state: ActorState::Base,
            animation_cooldown: 0.,
            afterlock_cooldown: ENEMY_AFTERLOCK_COOLDOWN,
            change_direction_cooldown: ENEMY_WANDERER_CHANGE_DIRECTION_COOLDOWN,
        }
    }
}
    
impl Actor for EnemySlime {
    fn update(&mut self, ctx: &mut Context, conf: &mut Config, _delta_time: f32) -> GameResult {
        self.afterlock_cooldown = f32::max(0., self.afterlock_cooldown - _delta_time);
        self.change_direction_cooldown = f32::max(0., self.change_direction_cooldown - _delta_time);

        self.velocity_lerp(_delta_time, self.speed, 10., 10.);
        self.props.pos.0 += self.props.velocity;

        self.animation_cooldown = f32::max(0., self.animation_cooldown - _delta_time);

        if self.animation_cooldown == 0. { self.state = ActorState::Base; }
        if self.health <= 0. {
            conf.assets.audio.get_mut("enemy_death_sound").unwrap().play(ctx)?; 
            self.state = ActorState::Dead;
        }

        Ok(())
    }

    fn draw(&self, ctx: &mut Context, conf: &mut Config) -> GameResult {
        let (sw, sh) = (conf.screen_width, conf.screen_height);

        let sprite = match self.props.forward {
            v if v == -Vec2::Y => conf.assets.sprites.get("enemy_slime_north").unwrap(),
            v if v == Vec2::X => conf.assets.sprites.get("enemy_slime_east").unwrap(),
            v if v == -Vec2::X => conf.assets.sprites.get("enemy_slime_west").unwrap(),
            _ => conf.assets.sprites.get("enemy_slime_south").unwrap(),
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

    fn get_state(&self) -> ActorState { self.state }

    fn damage(&mut self, dmg: f32) { 
        self.health -= dmg; 
        self.state = ActorState::Damaged;
        self.animation_cooldown = ANIMATION_COOLDOWN;
    }

    fn get_damage(&self) -> f32 { return self.damage }

    fn get_tag(&self) -> ActorTag { ActorTag::Enemy }

    fn act(&mut self, sw: f32, sh: f32, _grid: &[[i32; ROOM_WIDTH]], _obstacles: &Vec<Box<dyn Stationary>>, _shots: &mut Vec<Shot>, _player: &Player) -> GameResult {
        if self.afterlock_cooldown == 0. {
            self.wander(sw, sh, _grid);
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn Any { self }

    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

impl Wanderer for EnemySlime {
    fn get_change_direction_cooldown(&self) -> f32 { self.change_direction_cooldown }

    fn set_change_direction_cooldown(&mut self, cd: f32) { self.change_direction_cooldown = cd; }
}

#[derive(Clone, Debug, Copy)]
pub struct BossWeirdBall {
    pub props: ActorProps,
    pub speed: f32,
    pub state: ActorState,
    pub health: f32,
    pub max_health: f32,
    pub damage: f32,
    pub shoot_rate: f32,
    pub shoot_range: f32,
    pub shoot_timeout: f32,
    pub animation_cooldown: f32,
    pub afterlock_cooldown: f32,
    pub change_direction_cooldown: f32,
}

impl Default for BossWeirdBall {
    fn default() -> Self {
        Self {
            props: ActorProps::default(),
            speed: ENEMY_SPEED * 0.5,
            health: BOSS_HEALTH,
            max_health: BOSS_HEALTH,
            damage: ENEMY_DAMAGE * 2.,
            shoot_rate: ENEMY_SHOOT_RATE,
            shoot_range: ENEMY_SHOOT_RANGE,
            shoot_timeout: 0.,
            state: ActorState::Base,
            animation_cooldown: 0.,
            afterlock_cooldown: ENEMY_AFTERLOCK_COOLDOWN,
            change_direction_cooldown: ENEMY_WANDERER_CHANGE_DIRECTION_COOLDOWN,
        }
    }
}

impl Actor for BossWeirdBall {
    fn update(&mut self, ctx: &mut Context, conf: &mut Config, _delta_time: f32) -> GameResult {
        self.afterlock_cooldown = f32::max(0., self.afterlock_cooldown - _delta_time);
        self.change_direction_cooldown = f32::max(0., self.change_direction_cooldown - _delta_time);
        self.shoot_timeout = f32::max(0., self.shoot_timeout - _delta_time);
        self.animation_cooldown = f32::max(0., self.animation_cooldown - _delta_time);

        self.velocity_lerp(_delta_time, self.speed, 10., 10.);
        self.props.pos.0 += self.props.velocity;

        if self.animation_cooldown == 0. { self.state = ActorState::Base; }
        if self.health <= 0. {
            conf.assets.audio.get_mut("boss_death_sound").unwrap().play(ctx)?; 
            self.state = ActorState::Dead;
        }

        Ok(())
    }

    fn draw(&self, ctx: &mut Context, conf: &mut Config) -> GameResult {
        let (sw, sh) = (conf.screen_width, conf.screen_height);

        let sprite = match self.state {
            ActorState::Shoot => { 
                match self.props.forward.x as i32 {
                    1 => conf.assets.sprites.get("boss_weird_ball_shoot_cardinals").unwrap(),
                    _ => conf.assets.sprites.get("boss_weird_ball_shoot_diagonals").unwrap(),
                }
            },
            _ => conf.assets.sprites.get("boss_weird_ball_base").unwrap(),
        };

        let draw_params = DrawParam::default()
            .dest(self.props.pos)
            .scale(self.scale_to_screen(sw, sh, sprite.dimensions()))
            .offset([0.5, 0.5]);

        match self.state {
            ActorState::Damaged => graphics::draw(ctx, sprite, draw_params.color(Color::RED))?,
            _ => graphics::draw(ctx, sprite, draw_params)?,
        }

        self.draw_health_bar(ctx, sw, sh)?;

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

    fn get_state(&self) -> ActorState { self.state }

    fn damage(&mut self, dmg: f32) { 
        self.health -= dmg; 
        self.state = ActorState::Damaged;
        self.animation_cooldown = ANIMATION_COOLDOWN;
    }

    fn get_damage(&self) -> f32 { return self.damage }

    fn get_tag(&self) -> ActorTag { ActorTag::Enemy }

    fn act(&mut self, sw: f32, sh: f32, _grid: &[[i32; ROOM_WIDTH]], _obstacles: &Vec<Box<dyn Stationary>>, _shots: &mut Vec<Shot>, _player: &Player) -> GameResult {
        if self.afterlock_cooldown == 0. {
            self.wander(sw, sh, _grid);
            self.shoot(sw, sh, _obstacles, _shots, _player)?;
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn Any { self }

    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

impl Wanderer for BossWeirdBall {
    fn get_change_direction_cooldown(&self) -> f32 { self.change_direction_cooldown }

    fn set_change_direction_cooldown(&mut self, cd: f32) { self.change_direction_cooldown = cd; }
}

impl BossWeirdBall {
    fn draw_health_bar(&self, ctx: &mut Context, sw: f32, sh: f32) -> GameResult {
        let bbox = self.get_bbox(sw, sh);
        let (hbw, hbh) = (bbox.w * 1.4, sh * 0.01);
        let (hbx, hby) = (bbox.x - (hbw - bbox.w) * 0.5, bbox.y - hbh * 2.);

        let bar = MeshBuilder::new()
            .rectangle(DrawMode::fill(), Rect::new(hbx, hby, hbw / self.max_health * self.health, hbh), Color::RED)?
            .rectangle(DrawMode::stroke(3.), Rect::new(hbx, hby, hbw, hbh), Color::BLACK)?
            .build(ctx)?;

        graphics::draw(ctx, &bar, DrawParam::default())?;

        Ok(())
    }
}

impl Shooter for BossWeirdBall {
    fn shoot(&mut self, _sw: f32, _sh: f32, _obstacles: &Vec<Box<dyn Stationary>>, shots: &mut Vec<Shot>, _player: &Player) -> GameResult {
        if self.shoot_timeout != 0. 
            || self.afterlock_cooldown != 0. {
            return Ok(())
        }

        self.props.forward = match thread_rng().gen_bool(0.5) {
            true => Vec2::X,
            false => Vec2::Y,
        };
        self.state = ActorState::Shoot;
        self.shoot_timeout = 1. / self.shoot_rate;
        self.animation_cooldown = ANIMATION_COOLDOWN;

        let mut shot_dir = match self.props.forward.x as i32 {
            1 => Vec2::X,
            _ => Vec2::ONE,
        }.normalize();

        for _ in 0..4 {
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
                tag: ShotTag::Enemy,
            };

            shots.push(shot);
            shot_dir = shot_dir.perp();
        }

        Ok(())
    }

    fn get_range(&self) -> f32 { self.shoot_range }

    fn get_rate(&self) -> f32 { self.shoot_rate }
}
