use ggez::{
    graphics::{self, DrawParam},
    GameResult,
    Context,
};
use crate::{
    utils::*,
    traits::*,
};
use std::{
    any::Any,
};
use glam::f32::Vec2;

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

impl Actor for Shot {
    fn update(&mut self, _ctx: &mut Context, _conf: &mut Config, _delta_time: f32) -> GameResult {
        self.props.velocity = (self.props.velocity + self.props.translation * 100. * _delta_time).clamp_length_max(self.speed);
        self.props.pos.0 += self.props.velocity;

        Ok(())
    }

    fn draw(&self, ctx: &mut Context, conf: &mut Config) -> GameResult {
        let (sw, sh) = (conf.screen_width, conf.screen_height);
        let draw_params = DrawParam::default()
            .dest(self.props.pos)
            .scale(self.scale_to_screen(sw, sh, conf.assets.sprites.get("shot_puke_base").unwrap().dimensions()) * f32::min(self.damage, 1.5))
            .offset([0.5, 0.5]);

        match self.tag {
            ShotTag::Player => graphics::draw(ctx, conf.assets.sprites.get("shot_puke_base").unwrap(), draw_params)?,
            ShotTag::Enemy => graphics::draw(ctx, conf.assets.sprites.get("shot_blood_base").unwrap(), draw_params)?,
        }

        if conf.draw_bcircle_model { self.draw_bcircle(ctx, (sw, sh))?; }

        Ok(())
    }

    fn get_pos(&self) -> Vec2 { self.props.pos.into() }

    fn get_scale(&self) -> Vec2 { self.props.scale }

    fn get_velocity(&self) -> Vec2 { self.props.velocity }

    fn get_translation(&self) -> Vec2 { self.props.translation }

    fn get_forward(&self) -> Vec2 { self.props.forward }

    fn get_health(&self) -> f32 { 0. }

    fn get_state(&self) -> ActorState { ActorState::Base }

    fn get_tag(&self) -> ActorTag { ActorTag::Player }

    fn as_any(&self) -> &dyn Any { self }

    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}
