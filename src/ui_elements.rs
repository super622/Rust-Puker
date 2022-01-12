use ggez::{
    graphics::{self, Font, Color, DrawMode, DrawParam, Rect, Mesh, Text, PxScale},
    Context,
    GameResult,
    mint::{Point2},
};
use std::{
    any::Any,
};

use crate::{
    assets::*,
    utils::*,
    traits::*,
    entities::Player,
    dungeon::Dungeon,
};

pub struct TextSprite {
    pub pos: Point2<f32>,
    pub text: String,
    pub font: Font,
    pub font_size: f32,
    pub color: Color,
}

impl TextSprite {
    pub fn get_text(&self) -> Text {
        let mut text = Text::new(self.text.as_str());
        text.fragments_mut().iter_mut().map(|f| {
            f.font = Some(self.font);
            f.scale = Some(PxScale::from(self.font_size));
            f.color = Some(self.color);
        }).count();
        text
    }
}

impl UIElement for TextSprite {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context, _assets: &Assets) -> GameResult {
        let text = self.get_text();
        let tl = self.top_left(ctx);

        graphics::draw(ctx, &text, DrawParam::default().dest([tl.x, tl.y]))?;

        Ok(())
    }

    fn pos(&self) -> Point2<f32> { self.pos }

    fn width(&self, ctx: &mut Context) -> f32 { self.get_text().width(ctx) as f32 }

    fn height(&self, ctx: &mut Context) -> f32 { self.get_text().height(ctx) as f32 }

    fn as_any(&self) -> &dyn Any { self }

    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}


pub struct Button {
    pub pos: Point2<f32>,
    pub tag: State,
    pub text: TextSprite,
    pub color: Color,
}

impl UIElement for Button {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        self.color = Color::WHITE;
        if self.mouse_overlap(ctx) {
            self.color = Color::RED;
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context, _assets: &Assets) -> GameResult {
        let (tw, th) = (self.text.width(ctx) as f32, self.text.height(ctx) as f32);
        let tl = self.top_left(ctx);

        let btn = Mesh::new_rounded_rectangle(
            ctx,
            DrawMode::fill(),
            Rect::new(tl.x, tl.y, tw, th),
            5.,
            self.color,
        )?;

        graphics::draw(ctx, &btn, DrawParam::default())?;
        self.text.draw(ctx, _assets)?;

        Ok(())
    }

    fn pos(&self) -> Point2<f32> { self.pos }

    fn width(&self, ctx: &mut Context) -> f32 { self.text.width(ctx) as f32 }

    fn height(&self, ctx: &mut Context) -> f32 { self.text.height(ctx) as f32 }

    fn as_any(&self) -> &dyn Any { self }

    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

pub struct Minimap {
    pub pos: Point2<f32>,
    pub player: &'static Player,
    pub dungeon: &'static Dungeon,
    pub cur_room: (usize, usize),
    pub width: f32,
    pub height: f32,
}

impl UIElement for Minimap {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context, assets: &Assets) -> GameResult {
        Ok(())
    }

    fn pos(&self) -> Point2<f32> { self.pos }

    fn width(&self, _ctx: &mut Context) -> f32 { self.width }

    fn height(&self, _ctx: &mut Context) -> f32 { self.height }

    fn as_any(&self) -> &dyn Any { self }
    
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

pub struct HealthBar {
    pub pos: Point2<f32>,
    pub health: f32,
}
