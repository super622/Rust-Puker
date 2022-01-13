use ggez::{
    graphics::{self, Rect, DrawParam, PxScale, DrawMode, Mesh, Text, Color},
    GameResult,
    Context,
    mint::{Point2},
    event::{KeyCode, MouseButton},
    input,
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
    fn update(&mut self, _config: &Config, _delta_time: f32) -> GameResult;

    fn draw(&self, ctx: &mut Context, assets: &Assets, _config: &Config) -> GameResult;

    fn draw_bbox(&self, ctx: &mut Context, screen: (f32, f32)) -> GameResult {
        let (sw, sh) = screen;
        let bbox = self.get_bbox(sw, sh);
        let mut text = Text::new(format!("{:?}, {:?}", bbox.x, bbox.y));

        let mesh = Mesh::new_rectangle(ctx, DrawMode::stroke(2.0), bbox, Color::BLUE)?;
        graphics::draw(ctx, &mesh, DrawParam::default())?;

        text.fragments_mut().iter_mut().map(|x| x.scale = Some(PxScale::from(24.0))).count();
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
        Rect::new(self.get_pos().x - width / 2., self.get_pos().y - height / 2., width, height)
    }

    fn get_pos(&self) -> Vec2;

    fn get_scale(&self) -> Vec2;

    fn get_velocity(&self) -> Vec2;

    fn get_translation(&self) -> Vec2;

    fn get_forward(&self) -> Vec2;

    fn resize_event(&mut self, _config: &Config);

    fn as_any(&self) -> &dyn Any;
    
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub trait Stationary: std::fmt::Debug {
    fn update(&mut self, _config: &Config, _delta_time: f32) -> GameResult;

    fn draw(&self, ctx: &mut Context, assets: &Assets, _config: &Config) -> GameResult;

    fn draw_bbox(&self, ctx: &mut Context, screen: (f32, f32)) -> GameResult {
        let (sw, sh) = screen;
        let bbox = self.get_bbox(sw, sh);
        let mut text = Text::new(format!("{:?}, {:?}", bbox.x, bbox.y));

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
        Rect::new(self.get_pos().x - width / 2., self.get_pos().y - height / 2., width, height)
    }

    fn get_pos(&self) -> Vec2;

    fn get_scale(&self) -> Vec2;

    fn resize_event(&mut self, conf: &Config);

    fn as_any(&self) -> &dyn Any;
    
    fn as_any_mut(&mut self) -> &mut dyn Any;
}


pub trait Actor: Model {
    fn get_health(&self) -> f32; 

    fn damage(&mut self, dmg: f32);
}

pub trait Shooter: Model {
    fn shoot(&mut self, shots: &mut Vec<Shot>) -> GameResult;
}

pub trait Scene {
    fn update(&mut self, ctx: &mut Context, _delta_time: f32) -> GameResult;

    fn draw(&mut self, ctx: &mut Context, assets: &mut Assets) -> GameResult;

    fn key_down_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymod: input::keyboard::KeyMods, _repeat: bool);

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymod: input::keyboard::KeyMods);

    fn mouse_button_down_event(&mut self, _ctx: &mut Context, _button: MouseButton, _x: f32, _y: f32);

    fn get_ui_elements(&self) -> Option<&Vec<Box<dyn UIElement>>> { None }

    fn get_ui_elements_mut(&mut self) -> Option<&mut Vec<Box<dyn UIElement>>> { None }  

    fn check_for_element_click(&self, ctx: &mut Context, sw: f32, sh: f32) -> Option<&dyn UIElement> {
        match self.get_ui_elements() {
            Some(ue) => {
                for e in ue.iter() {
                    if e.mouse_overlap(ctx, sw, sh) {
                        return Some(&**e);
                    }
                }
                None
            },
            None => None
        }
    }

    fn resize_event(&mut self, conf: &Config) {}
}

pub trait UIElement {
    fn update(&mut self, ctx: &mut Context, _conf: &Config) -> GameResult;

    fn draw(&mut self, ctx: &mut Context, _assets: &Assets, _conf: &Config) -> GameResult;

    fn pos(&self, sw: f32, sh: f32) -> Point2<f32>;

    fn width(&self, ctx: &mut Context, sw: f32) -> f32;

    fn height(&self, ctx: &mut Context, sh: f32) -> f32;

    fn top_left(&self, ctx: &mut Context, sw: f32, sh: f32) -> Point2<f32> {
        let pos = self.pos(sw, sh);
        let (w, h) = (self.width(ctx, sw), self.height(ctx, sh));
        Point2 { x: pos.x - w / 2., y: pos.y - h / 2. }
    }
        
    fn mouse_overlap(&self, ctx: &mut Context, sw: f32, sh: f32) -> bool {
        let tl = self.top_left(ctx, sw, sh);
        let (w, h) = (self.width(ctx, sw), self.height(ctx, sh));
        Rect::new(tl.x, tl.y, w, h).contains(input::mouse::position(ctx))
    }

    fn as_any(&self) -> &dyn Any;
    
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
