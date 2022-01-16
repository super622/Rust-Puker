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

pub enum CollectableTag {
    RedHeart(f32),
}

pub struct Collectable {
    pub props: ActorProps,
    pub tag: CollectableTag,
}

impl Model for Collectable {
    fn update(&mut self, ctx: &mut Context, assets: &mut Assets, _conf: &Config, _delta_time: f32) -> GameResult {
        self.props.translation = self.props.translation.clamp_length_max(1.);
        self.props.velocity -= self.props.velocity * _delta_time / 0.1;
        self.props.velocity += self.props.translation * 100. * _delta_time;
        self.props.pos.0 += self.props.velocity;
        if self.props.velocity.length() < 0.01 { self.props.velocity = self.props.velocity.clamp_length_min(0.); }
        if self.props.velocity.length() > self.speed { self.props.velocity = self.props.velocity.clamp_length_max(self.speed); }

        Ok(())
    }

    fn draw(&self, ctx: &mut Context, assets: &mut Assets, conf: &Config) -> GameResult {
        let (sw, sh) = (conf.screen_width, conf.screen_height);
        let draw_params = DrawParam::default()
            .dest(self.props.pos)
            .scale(self.scale_to_screen(sw, sh, assets.sprites.get("player_base").unwrap().dimensions()))
            .offset([0.5, 0.5]);

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

pub enum PowerUpTag {
    SpeedBoost(f32),
    ShootRateBoost(f32),
    DamageBoost(f32),
}

pub struct PowerUp {
    pub pos: Vec2Wrap,
    pub scale: Vec2,
    pub tag: PowerUpTag,
}
