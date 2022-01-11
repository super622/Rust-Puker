use ggez::{
    graphics::{self, Rect, DrawParam, PxScale, DrawMode, Mesh, Text, Color},
    GameResult,
    Context,
};
use std::{
    any::Any,
};
use glam::f32::Vec2;
use crate::{
    assets::*,
    entities::*,
    consts::*,
    utils::*,
};

pub trait Model: std::fmt::Debug {
    fn update(&mut self, _delta_time: f32) -> GameResult;

    fn draw(&self, ctx: &mut Context, assets: &Assets, screen: (f32, f32), _config: &Config) -> GameResult;

    fn draw_bbox(&self, ctx: &mut Context, screen: (f32, f32)) -> GameResult {
        let (sw, sh) = screen;
        let mut bbox = self.get_bbox(sw, sh);
        let mut text = Text::new(format!("{:?}, {:?}", bbox.x, bbox.y));
        let screen_coords = world_to_screen_space(sw, sh, Vec2::new(bbox.x, bbox.y));
        bbox.x = screen_coords.x;
        bbox.y = screen_coords.y;

        let mesh = Mesh::new_rectangle(ctx, DrawMode::stroke(2.0), bbox, Color::BLUE)?;
        graphics::draw(ctx, &mesh, DrawParam::default())?;

        text.fragments_mut().iter_mut().map(|x| x.scale = Some(PxScale { x: 0.5, y: 0.5 })).count();
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

    fn get_velocity(&self) -> Vec2;

    fn get_translation(&self) -> Vec2;

    fn get_forward(&self) -> Vec2;

    fn as_any(&self) -> &dyn Any;
    
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub trait Stationary: std::fmt::Debug {
    fn draw(&self, ctx: &mut Context, assets: &Assets, screen: (f32, f32), _config: &Config) -> GameResult;

    fn draw_bbox(&self, ctx: &mut Context, screen: (f32, f32)) -> GameResult {
        let (sw, sh) = screen;
        let mut bbox = self.get_bbox(sw, sh);
        let mut text = Text::new(format!("{:?}, {:?}", bbox.x, bbox.y));
        let screen_coords = world_to_screen_space(sw, sh, Vec2::new(bbox.x, bbox.y));
        bbox.x = screen_coords.x;
        bbox.y = screen_coords.y;

        let mesh = Mesh::new_rectangle(ctx, DrawMode::stroke(2.0), bbox, Color::BLUE)?;
        graphics::draw(ctx, &mesh, DrawParam::default())?;

        text.fragments_mut().iter_mut().map(|x| x.scale = Some(PxScale { x: 0.5, y: 0.5 })).count();
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

    fn as_any(&self) -> &dyn Any;
    
    fn as_any_mut(&mut self) -> &mut dyn Any;
}


pub trait Actor: Model {
    fn get_health(&self) -> f32; 

    fn damage(&mut self, dmg: f32);
}

pub trait Shooter: Model {
    fn shoot(&mut self) -> GameResult;

    fn get_shots(&self) -> &Vec<Shot>;

    fn get_shots_mut(&mut self) -> &mut Vec<Shot>;
}

pub trait Scene {
    fn update(&mut self, ctx: &mut Context, delta_time: f32) -> GameResult;

    fn draw(&mut self, ctx: &mut Context, assets: &Assets) -> GameResult;
}
