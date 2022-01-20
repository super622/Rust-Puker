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
    cell::{Ref, RefMut},
};
use glam::f32::Vec2;
use crate::{
    assets::*,
    entities::*,
    consts::*,
    utils::*,
};

pub trait Model: std::fmt::Debug {
    fn update(&mut self, ctx: &mut Context, assets: &mut Assets, _config: &Config, _delta_time: f32) -> GameResult;

    fn draw(&self, ctx: &mut Context, assets: &mut Assets, _config: &Config) -> GameResult;

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

    fn draw_bcircle(&self, ctx: &mut Context, screen: (f32, f32)) -> GameResult {
        let (sw, sh)= screen;
        let bcircle = self.get_bcircle(sw, sh);
        let mut text = Text::new(format!("{:?}", bcircle.0.0));

        let mesh = Mesh::new_circle(ctx, DrawMode::stroke(2.0), bcircle.0, bcircle.1, 0.5, Color::BLUE)?;
        graphics::draw(ctx, &mesh, DrawParam::default())?;

        text.fragments_mut().iter_mut().map(|x| x.scale = Some(PxScale::from(24.0))).count();
        graphics::draw(ctx, &text, DrawParam::default().dest([bcircle.0.0.x + bcircle.1, bcircle.0.0.y]))?;

        Ok(())
    }

    fn get_bcircle(&self, sw: f32, sh: f32) -> (Vec2Wrap, f32) {
        let width = sw / (ROOM_WIDTH as f32) * self.get_scale().x;
        let height = sh / (ROOM_HEIGHT as f32) * self.get_scale().y;
        (Vec2::new(self.get_pos().x, self.get_pos().y).into(), f32::max(width, height) / 2.)
    }    

    fn get_bbox(&self, sw: f32, sh: f32) -> graphics::Rect {
        let width = sw / (ROOM_WIDTH as f32) * self.get_scale().x;
        let height = sh / (ROOM_HEIGHT as f32) * self.get_scale().y;
        Rect::new(self.get_pos().x - width / 2., self.get_pos().y - height / 2., width, height)
    }

    fn scale_to_screen(&self, sw: f32, sh: f32, image: Rect) -> Vec2 {
        let bbox = self.get_bbox(sw, sh);
        Vec2::new(bbox.w / image.w, bbox.h / image.h)
    }

    fn get_pos(&self) -> Vec2;

    fn get_scale(&self) -> Vec2;

    fn get_velocity(&self) -> Vec2;

    fn get_translation(&self) -> Vec2;

    fn get_forward(&self) -> Vec2;

    fn set_pos(&mut self, _new_pos: Vec2) {}

    fn set_scale(&mut self, _new_scale: Vec2) {}

    fn set_velocity(&mut self, _new_velocity: Vec2) {}

    fn set_translation(&mut self, _new_translation: Vec2) {}

    fn set_forward(&mut self, _new_forward: Vec2) {}

    fn velocity_lerp(&mut self, speed: f32, _delta_time: f32, decay: f32) {
        self.set_translation(self.get_translation().clamp_length_max(1.));
        self.set_velocity(self.get_velocity() - self.get_velocity() * _delta_time * decay + self.get_translation() * 50. * _delta_time);
        if self.get_velocity().length() < 0.01 { self.set_velocity(self.get_velocity().clamp_length_min(0.)); }
        if self.get_velocity().length() > speed && speed > 0. { self.set_velocity(self.get_velocity().clamp_length_max(speed)); }
    }

    fn as_any(&self) -> &dyn Any;
    
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub trait Stationary: std::fmt::Debug {
    fn update(&mut self, _config: &Config, _delta_time: f32) -> GameResult;

    fn draw(&self, ctx: &mut Context, assets: &mut Assets, _config: &Config) -> GameResult;

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

    fn draw_bcircle(&self, ctx: &mut Context, screen: (f32, f32)) -> GameResult {
        let (sw, sh)= screen;
        let bcircle = self.get_bcircle(sw, sh);
        let mut text = Text::new(format!("{:?}", bcircle.0.0));

        let mesh = Mesh::new_circle(ctx, DrawMode::stroke(2.0), bcircle.0, bcircle.1, 0.5, Color::BLUE)?;
        graphics::draw(ctx, &mesh, DrawParam::default())?;

        text.fragments_mut().iter_mut().map(|x| x.scale = Some(PxScale::from(24.0))).count();
        graphics::draw(ctx, &text, DrawParam::default().dest([bcircle.0.0.x + bcircle.1, bcircle.0.0.y]))?;

        Ok(())
    }

    fn get_bbox(&self, sw: f32, sh: f32) -> graphics::Rect {
        let width = sw / (ROOM_WIDTH as f32) * self.get_scale().x;
        let height = sh / (ROOM_HEIGHT as f32) * self.get_scale().y;
        Rect::new(self.get_pos().x - width / 2., self.get_pos().y - height / 2., width, height)
    }

    fn get_bcircle(&self, sw: f32, sh: f32) -> (Vec2Wrap, f32) {
        let width = sw / (ROOM_WIDTH as f32) * self.get_scale().x;
        let height = sh / (ROOM_HEIGHT as f32) * self.get_scale().y;
        (Vec2::new(self.get_pos().x, self.get_pos().y).into(), f32::max(width, height) / 2.)
    }    

    fn scale_to_screen(&self, sw: f32, sh: f32, image: Rect) -> Vec2 {
        let bbox = self.get_bbox(sw, sh);
        Vec2::new(bbox.w / image.w, bbox.h / image.h)
    }

    fn get_pos(&self) -> Vec2;

    fn get_scale(&self) -> Vec2;

    fn as_any(&self) -> &dyn Any;
    
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub trait Scene {
    fn update(&mut self, ctx: &mut Context, assets: &mut Assets, _delta_time: f32) -> GameResult;

    fn draw(&mut self, ctx: &mut Context, assets: &mut Assets) -> GameResult;

    fn key_down_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymod: input::keyboard::KeyMods, _repeat: bool);

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymod: input::keyboard::KeyMods);

    fn mouse_button_down_event(&mut self, _ctx: &mut Context, _button: MouseButton, _x: f32, _y: f32);

    fn get_ui_elements(&self) -> Option<&Vec<Box<dyn UIElement>>> { None }

    fn get_ui_elements_mut(&mut self) -> Option<&mut Vec<Box<dyn UIElement>>> { None }  

    fn get_conf(&self) -> Option<Ref<Config>> { None }

    fn get_conf_mut(&mut self) -> Option<RefMut<Config>> { None }

    fn get_clicked(&self, ctx: &mut Context) -> Option<&dyn UIElement> {
        let (sw, sh, ww, wh);
        {
            match self.get_conf() {
                Some(conf) => {
                    sw = conf.screen_width;
                    sh = conf.screen_height;
                    ww = conf.window_width;
                    wh = conf.window_height;
                },
                None => return None,
            }
        }

        match self.get_ui_elements() {
            Some(ue) => {
                for e in ue.iter() {
                    if e.mouse_overlap(ctx, sw, sh, ww, wh) {
                        return Some(&**e);
                    }
                }
                None
            },
            None => None
        }
    }

    fn get_clicked_mut(&mut self, ctx: &mut Context) -> Option<&mut dyn UIElement> {
        let (sw, sh, ww, wh);
        {
            match self.get_conf() {
                Some(conf) => {
                    sw = conf.screen_width;
                    sh = conf.screen_height;
                    ww = conf.window_width;
                    wh = conf.window_height;
                },
                None => return None,
            }
        }

        match self.get_ui_elements_mut() {
            Some(ue) => {
                for e in ue.iter_mut() {
                    if e.mouse_overlap(ctx, sw, sh, ww, wh) {
                        return Some(&mut **e);
                    }
                }
                None
            },
            None => None
        }
    }
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
        
    fn mouse_overlap(&self, ctx: &mut Context, sw: f32, sh: f32, ww: f32, wh: f32) -> bool {
        let tl = self.top_left(ctx, sw, sh);
        let (w, h) = (self.width(ctx, sh), self.height(ctx, sh));
        Rect::new(tl.x, tl.y, w, h).contains(get_mouse_screen_coords(input::mouse::position(ctx), sw, sh, ww, wh))
    }

    fn as_any(&self) -> &dyn Any;
    
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub trait Actor: Model {
    fn get_health(&self) -> f32; 

    fn damage(&mut self, _dmg: f32) {}

    fn heal(&mut self, _heal: f32) {}

    fn get_tag(&self) -> ActorTag;
}

pub trait Chaser: Actor {
    fn chase(&mut self, target: Vec2);

    fn find_path(&mut self, grid: &[[i32; ROOM_WIDTH]], sw: f32, sh: f32) -> Vec2 {
        let (mut i, mut j) = ((self.get_pos().y / sh * (ROOM_HEIGHT as f32)) as usize, (self.get_pos().x / sw * (ROOM_WIDTH as f32)) as usize);

        if      i > 0               && grid[i - 1][j] > grid[i][j] { j += 1; }
        else if j > 0               && grid[i][j - 1] > grid[i][j] { i += 1; }
        else if j < ROOM_WIDTH - 1  && grid[i][j + 1] > grid[i][j] { i += 1; j += 2; }
        else if i < ROOM_HEIGHT - 1 && grid[i + 1][j] > grid[i][j] { i += 2; j += 1; }
        else { return self.get_pos() }

        Vec2::new((2. * (j as f32) - 1.) * sw / (ROOM_WIDTH as f32) / 2., (2. * (i as f32) - 1.) * sh / (ROOM_HEIGHT as f32) / 2.)
    }
}

pub trait Shooter: Actor {
    fn shoot(&mut self, target: &Vec2, shots: &mut Vec<Shot>) -> GameResult;

    fn get_range(&self) -> f32;  

    fn get_rate(&self) -> f32;  
}
