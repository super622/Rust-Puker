use ggez::{
    graphics::{self, Font, Color, DrawMode, DrawParam, Rect, Mesh, Text, PxScale},
    Context,
    GameResult,
    mint::{Point2},
    input,
};
use std::{
    any::Any,
};

use crate::{
    assets::*,
    utils::*,
    traits::*,
    consts::*,
    entities::{Player},
    dungeon::{Dungeon},
};

pub struct TextSprite {
    pub pos: Point2<f32>,
    pub text: String,
    pub font: Font,
    pub font_size: f32,
    pub color: Color,
}

impl TextSprite {
    pub fn get_text(&self, sh: f32) -> Text {
        let mut text = Text::new(self.text.as_str());
        text.fragments_mut().iter_mut().map(|f| {
            f.font = Some(self.font);
            f.scale = Some(PxScale::from(sh * self.font_size));
            f.color = Some(self.color);
        }).count();
        text
    }
}

impl UIElement for TextSprite {
    fn update(&mut self, _ctx: &mut Context, _conf: &Config) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context, _assets: &Assets, conf: &Config) -> GameResult {
        let (sw, sh) = (conf.screen_width, conf.screen_height);
        let text = self.get_text(sh);
        let tl = self.top_left(ctx, sw, sh);

        graphics::draw(ctx, &text, DrawParam::default().dest([tl.x, tl.y]))?;

        Ok(())
    }

    fn pos(&self, sw: f32, sh: f32) -> Point2<f32> { Point2 { x: sw * self.pos.x, y: sh * self.pos.y } }

    fn width(&self, ctx: &mut Context, sh: f32) -> f32 { self.get_text(sh).width(ctx) as f32 }

    fn height(&self, ctx: &mut Context, sh: f32) -> f32 { self.get_text(sh).height(ctx) as f32 }

    fn top_left(&self, ctx: &mut Context, sw: f32, sh: f32) -> Point2<f32> {
        let pos = self.pos(sw, sh);
        let (w, h) = (self.width(ctx, sh), self.height(ctx, sh));
        Point2 { x: pos.x - w / 2., y: pos.y - h / 2. }
    }
        
    fn mouse_overlap(&self, ctx: &mut Context, sw: f32, sh: f32) -> bool {
        let tl = self.top_left(ctx, sw, sh);
        let (w, h) = (self.width(ctx, sh), self.height(ctx, sh));
        Rect::new(tl.x, tl.y, w, h).contains(input::mouse::position(ctx))
    }

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
    fn update(&mut self, ctx: &mut Context, conf: &Config) -> GameResult {
        let (sw, sh) = (conf.screen_width, conf.screen_height);

        self.color = Color::WHITE;
        if self.mouse_overlap(ctx, sw, sh) {
            self.color = Color::RED;
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context, _assets: &Assets, conf: &Config) -> GameResult {
        let (sw, sh) = (conf.screen_width, conf.screen_height);
        let (tw, th) = (self.text.width(ctx, sh) as f32, self.text.height(ctx, sh) as f32);
        let tl = self.top_left(ctx, sw, sh);

        let btn = Mesh::new_rounded_rectangle(
            ctx,
            DrawMode::fill(),
            Rect::new(tl.x, tl.y, tw, th),
            5.,
            self.color,
        )?;

        graphics::draw(ctx, &btn, DrawParam::default())?;
        self.text.draw(ctx, _assets, conf)?;

        Ok(())
    }

    fn pos(&self, sw: f32, sh: f32) -> Point2<f32> { Point2 { x: sw * self.pos.x, y: sh * self.pos.y } }

    fn width(&self, ctx: &mut Context, sh: f32) -> f32 { self.text.width(ctx, sh) as f32 }

    fn height(&self, ctx: &mut Context, sh: f32) -> f32 { self.text.height(ctx, sh) as f32 }

    fn top_left(&self, ctx: &mut Context, sw: f32, sh: f32) -> Point2<f32> {
        let pos = self.pos(sw, sh);
        let (w, h) = (self.width(ctx, sh), self.height(ctx, sh));
        Point2 { x: pos.x - w / 2., y: pos.y - h / 2. }
    }
        
    fn mouse_overlap(&self, ctx: &mut Context, sw: f32, sh: f32) -> bool {
        let tl = self.top_left(ctx, sw, sh);
        let (w, h) = (self.width(ctx, sh), self.height(ctx, sh));
        Rect::new(tl.x, tl.y, w, h).contains(input::mouse::position(ctx))
    }

    fn as_any(&self) -> &dyn Any { self }

    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

pub struct Minimap {
    pub pos: Point2<f32>,
    pub width: f32,
    pub height: f32,
}

impl UIElement for Minimap {
    fn update(&mut self, _ctx: &mut Context, _conf: &Config) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, _ctx: &mut Context, _assets: &Assets, _conf: &Config) -> GameResult {
        Ok(())
    }

    fn pos(&self, sw: f32, sh: f32) -> Point2<f32> { Point2 { x: sw * self.pos.x, y: sh * self.pos.y } }

    fn width(&self, _ctx: &mut Context, sw: f32) -> f32 { sw * self.width }

    fn height(&self, _ctx: &mut Context, sh: f32) -> f32 { sh * self.height }

    fn as_any(&self) -> &dyn Any { self }
    
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

pub struct HealthBar {
    pub pos: Point2<f32>,
    pub width: f32,
    pub height: f32,
    pub health: f32,
    pub max_health: f32,
}

impl UIElement for HealthBar {
    fn update(&mut self, _ctx: &mut Context, _conf: &Config) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context, assets: &Assets, conf: &Config) -> GameResult {
        let (sw, sh) = (conf.screen_width, conf.screen_height);
        let pos = self.pos(sw, sh);
        let img_dims = &assets.heart_full.dimensions();
        let img_width = self.width(ctx, sw) / self.max_health;

        for i in 1..=(self.max_health as i32) {
            let index = i as f32;

            let draw_params = DrawParam::default()
                .dest([pos.x + index * img_width * 1.1, pos.y])
                .scale([img_width / img_dims.w, self.height(ctx, sh) / img_dims.h])
                .offset([0.5, 0.5]);

            let dif = self.health - index;

            if dif >= 0. { graphics::draw(ctx, &assets.heart_full, draw_params)?; }
            else if dif >= -0.5 { graphics::draw(ctx, &assets.heart_half, draw_params)?; }
            else { graphics::draw(ctx, &assets.heart_empty, draw_params)?; }
        }

        Ok(())
    }

    fn pos(&self, sw: f32, sh: f32) -> Point2<f32> { Point2 { x: sw * self.pos.x, y: sh * self.pos.y } }

    fn width(&self, _ctx: &mut Context, sw: f32) -> f32 { sw * self.width }

    fn height(&self, _ctx: &mut Context, sh: f32) -> f32 { sh * self.height }

    fn as_any(&self) -> &dyn Any { self }
    
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    
}

pub struct Overlay {
    pos: Point2<f32>,
    width: f32,
    height: f32,
    ui_elements: Vec<Box<dyn UIElement>>,
}

impl Overlay {
    pub fn new(player: &Player, dungeon: &Dungeon) -> Self {
        let pos = Point2 { x: 0., y: 0.};
        let (width, height) = (1.0, 1.0);
        let ui_elements: Vec<Box<dyn UIElement>> = vec![
            Box::new(HealthBar {
                pos: Point2 { x: HEALTH_BAR_POS.0, y: HEALTH_BAR_POS.1 },
                width: HEALTH_BAR_SCALE.0,
                height: HEALTH_BAR_SCALE.1,
                health: player.health,
                max_health: player.max_health,
            }),
        ];

        Self {
            pos,
            width,
            height,
            ui_elements,
        }
    }

    pub fn update_vars(&mut self, player: &Player, dungeon: &Dungeon) {
        for e in self.ui_elements.iter_mut() {
            if let Some(h) = e.as_any_mut().downcast_mut::<HealthBar>() {
                h.health = player.health;
                h.max_health = player.max_health;
            }
        }
    }
}

impl UIElement for Overlay {
    fn update(&mut self, _ctx: &mut Context, _conf: &Config) -> GameResult {
        for e in self.ui_elements.iter_mut() { e.update(_ctx, _conf)?; }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context, _assets: &Assets, _conf: &Config) -> GameResult {
        for e in self.ui_elements.iter_mut() { e.draw(ctx, _assets, _conf)?; }

        Ok(())
    }

    fn pos(&self, sw: f32, sh: f32) -> Point2<f32> { Point2 { x: sw * self.pos.x, y: sh * self.pos.y } }

    fn width(&self, _ctx: &mut Context, sw: f32) -> f32 { sw * self.width }

    fn height(&self, _ctx: &mut Context, sh: f32) -> f32 { sh * self.height }

    fn as_any(&self) -> &dyn Any { self }
    
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}
