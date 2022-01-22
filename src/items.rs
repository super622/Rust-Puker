use crate::{
    assets::*,
    player::*,
    traits::*,
    utils::*,
    consts::*,
};
use ggez::{
    graphics::{self, DrawParam},
    Context,
    GameResult,
    audio::SoundSource,
};
use glam::f32::Vec2;
use std::any::Any;

#[derive(Debug, PartialEq)]
pub enum CollectableTag {
    Consumed,
    RedHeart(f32),
    SpeedBoost(f32),
    ShootRateBoost(f32),
    DamageBoost(f32),
}

#[derive(Debug)]
pub enum CollectableState {
    Base,
    Consumed,
}

#[derive(Debug)]
pub struct Collectable {
    pub props: ActorProps,
    pub tag: CollectableTag,
}

impl Actor for Collectable {
    fn update(&mut self, _ctx: &mut Context, _assets: &mut Assets, _conf: &Config, _grid: &[[i32; ROOM_WIDTH]], _player: Option<&Player>, _delta_time: f32) -> GameResult {
        self.velocity_lerp(_delta_time, 0., 2., 0.);

        self.props.pos.0 += self.props.velocity;

        Ok(())
    }

    fn draw(&self, ctx: &mut Context, assets: &mut Assets, conf: &Config) -> GameResult {
        let (sw, sh) = (conf.screen_width, conf.screen_height);

        let sprite = match self.tag {
            CollectableTag::RedHeart(a) => {
                if a == 1. {
                    assets.sprites.get("heart_full_collectable").unwrap()
                } else {
                    assets.sprites.get("heart_half_collectable").unwrap()
                }
            },
            CollectableTag::SpeedBoost(_) => assets.sprites.get("speed_boost").unwrap(),
            CollectableTag::ShootRateBoost(_) => assets.sprites.get("shoot_rate_boost").unwrap(),
            CollectableTag::DamageBoost(_) => assets.sprites.get("damage_boost").unwrap(),
            _ => unreachable!(),
        };

        let draw_params = DrawParam::default()
            .dest(self.props.pos)
            .scale(self.scale_to_screen(sw, sh, sprite.dimensions()))
            .offset([0.5, 0.5]);

        graphics::draw(ctx, sprite, draw_params)?;

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

    fn get_health(&self) -> f32 { 0. }

    fn get_tag(&self) -> ActorTag { ActorTag::Player }

    fn as_any(&self) -> &dyn Any { self }

    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

impl Collectable {
    pub fn affect_player(&mut self, ctx: &mut Context, assets: &mut Assets, player: &mut Player) -> GameResult<bool> {
        let result = match self.tag {
            CollectableTag::RedHeart(h) => {
                if player.health < player.max_health {
                    player.health = f32::min(player.health + h, player.max_health);
                    true
                } else { false }
            }
            CollectableTag::ShootRateBoost(b) => {
                player.shoot_rate = f32::min(player.shoot_rate * b, PLAYER_MAX_SHOOT_RATE);
                true
            }
            CollectableTag::SpeedBoost(b) => {
                player.speed = f32::min(player.speed * b, PLAYER_MAX_SPEED);
                true
            }
            CollectableTag::DamageBoost(b) => {
                player.damage = f32::min(player.damage * b, PLAYER_MAX_DAMAGE);
                true
            }
            _ => false,
        };

        if result { 
            match self.tag {
                CollectableTag::RedHeart(_) => assets.audio.get_mut("heal_sound").unwrap().play(ctx)?,
                _ => assets.audio.get_mut("power_up_sound").unwrap().play(ctx)?,
            }
            self.tag = CollectableTag::Consumed; 
        }

        Ok(result)
    }
}
