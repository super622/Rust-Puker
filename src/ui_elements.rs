use ggez::{
    graphics::{self, Font, Color, DrawMode, DrawParam, Rect, Mesh, MeshBuilder, Text, PxScale},
    Context,
    GameResult,
    mint::{Point2},
    input::{mouse},
};
use std::{
    any::Any,
};
use glam::f32::Vec2;

use crate::{
    assets::*,
    utils::*,
    traits::*,
    consts::*,
    player::*,
    dungeon::{Dungeon, RoomState},
};

#[derive(Clone)]
pub struct TextSprite {
    pub pos: Point2<f32>,
    pub text: String,
    pub font: Font,
    pub font_size: f32,
    pub color: Color,
    pub background: Color,
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

impl Default for TextSprite {
    fn default() -> Self {
        Self {
            pos: Point2 { x: 0.5, y: 0.5 },
            text: String::from(""),
            font: Font::default(),
            font_size: BUTTON_TEXT_FONT_SIZE,
            color: Color::BLACK,
            background: Color::from([0.; 4]),
        }
    }
}

impl UIElement for TextSprite {
    fn update(&mut self, _ctx: &mut Context, _conf: &Config) -> GameResult { Ok(()) }

    fn draw(&mut self, ctx: &mut Context, _assets: &Assets, conf: &Config) -> GameResult {
        let (sw, sh) = (conf.screen_width, conf.screen_height);
        let text = self.get_text(sh);
        let (tw, th) = (text.dimensions(ctx).w, text.dimensions(ctx).h); 
        let tl = self.top_left(ctx, sw, sh);

        let background = Mesh::new_rectangle(
            ctx,
            DrawMode::fill(),
            Rect::new(tl.x, tl.y, tw, th),
            self.background,
        )?;

        graphics::draw(ctx, &background, DrawParam::default())?;
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
        
    fn as_any(&self) -> &dyn Any { self }

    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

pub struct Border {
    pub stroke: f32,
    pub radius: f32,
    pub color: Color,
}

impl Default for Border {
    fn default() -> Self {
        Self {
            stroke: 2.,
            radius: 5.,
            color: Color::BLACK,
        }
    }
}

pub struct Button {
    pub pos: Point2<f32>,
    pub width: f32,
    pub height: f32,
    pub tag: Option<State>,
    pub text: Option<TextSprite>,
    pub color: Color,
    pub border: Border,
}

impl Default for Button {
    fn default() -> Self {
        Self {
            pos: Point2 { x: 0.5, y: 0.5 },
            width: 0.3,
            height: 0.1,
            tag: None,
            text: None,
            color: Color::WHITE,
            border: Border::default(),
        }
    }
}

impl UIElement for Button {
    fn update(&mut self, _ctx: &mut Context, _conf: &Config) -> GameResult { Ok(()) }

    fn draw(&mut self, ctx: &mut Context, _assets: &Assets, conf: &Config) -> GameResult {
        let (sw, sh) = (conf.screen_width, conf.screen_height);
        let (ww, wh) = (conf.window_width, conf.window_height);
        let (w, h) = (self.width(ctx, sw), self.height(ctx, sh));
        let tl = self.top_left(ctx, sw, sh);

        // let (tw, th);
        let mut text = None;
        if let Some(t) = &self.text {
            text = Some(t.clone());
            // tw = t.width(ctx, sh) as f32;
            // th = t.height(ctx, sh) as f32;
        }

        let rect = Rect::new(tl.x, tl.y, w, h);
        let color = match self.mouse_overlap(ctx, sw, sh, ww, wh) {
            true => {
                mouse::set_cursor_type(ctx, mouse::CursorIcon::Hand);
                if let Some(t) = &mut text { t.color = invert_color(&t.color); }
                invert_color(&self.color)
            },
            _ => self.color,
        };

        let btn = MeshBuilder::new()
            .rounded_rectangle(DrawMode::fill(), rect, self.border.radius, color)?
            .rounded_rectangle(DrawMode::stroke(self.border.stroke), rect, self.border.radius, self.border.color)?
            .build(ctx)?;

        graphics::draw(ctx, &btn, DrawParam::default())?;
        if let Some(t) = &mut text { 
            t.pos = self.pos;
            t.draw(ctx, _assets, conf)?; 
        }

        Ok(())
    }

    fn pos(&self, sw: f32, sh: f32) -> Point2<f32> { Point2 { x: sw * self.pos.x, y: sh * self.pos.y } }

    fn width(&self, _ctx: &mut Context, sw: f32) -> f32 { self.width * sw }

    fn height(&self, _ctx: &mut Context, sh: f32) -> f32 { self.height * sh }

    fn as_any(&self) -> &dyn Any { self }

    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

pub struct Minimap {
    pub pos: Point2<f32>,
    pub width: f32,
    pub height: f32,
    pub cur_room: (usize, usize),
    pub rooms_state: Vec<Vec<RoomState>>,
}

impl UIElement for Minimap {
    fn update(&mut self, _ctx: &mut Context, _conf: &Config) -> GameResult { Ok(()) }

    fn draw(&mut self, ctx: &mut Context, _assets: &Assets, conf: &Config) -> GameResult {
        let (sw, sh) = (conf.screen_width, conf.screen_height);
        let (mw, mh) = (self.width(ctx, sw), self.height(ctx, sh));
        let pos = self.pos(sw, sh);
        let map_rect = Mesh::new_rectangle(
            ctx,
            DrawMode::fill(),
            Rect::new(pos.x, pos.y, mw, mh),
            Color::new(0., 0., 0., 0.7),
        )?;          
        graphics::draw(ctx, &map_rect, DrawParam::default())?;

        let (rw, rh) = (mw / (DUNGEON_GRID_COLS as f32), mh / (DUNGEON_GRID_ROWS as f32));
        let mut room_rect;

        for r in 0..DUNGEON_GRID_ROWS {
            for c in 0..DUNGEON_GRID_COLS {
                room_rect = MeshBuilder::new()
                    .rectangle( 
                        DrawMode::fill(),
                        Rect::new(pos.x + (c as f32) * rw, pos.y + (r as f32) * rh, rw, rh),
                        Color::BLACK,
                    )?
                    .rounded_rectangle(
                        DrawMode::fill(),
                        Rect::new(pos.x + (c as f32) * rw, pos.y + (r as f32) * rh, rw, rh),
                        8.,
                        Color::WHITE,
                    )?
                    .build(ctx)?;

                match self.rooms_state[r][c] {
                    _ if self.cur_room == (r, c) => graphics::draw(ctx, &room_rect, DrawParam::default().color(Color::WHITE))?,
                    RoomState::Discovered => graphics::draw(ctx, &room_rect, DrawParam::default().color(Color::new(0.3, 0.3, 0.3, 1.)))?,
                    RoomState::Cleared => graphics::draw(ctx, &room_rect, DrawParam::default().color(Color::new(0.6, 0.6, 0.6, 1.)))?,
                    _ => (),
                }
            }
        }
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
    fn update(&mut self, _ctx: &mut Context, _conf: &Config) -> GameResult { Ok(()) }

    fn draw(&mut self, ctx: &mut Context, assets: &Assets, conf: &Config) -> GameResult {
        let (sw, sh) = (conf.screen_width, conf.screen_height);
        let pos = self.pos(sw, sh);
        let img_dims = assets.sprites.get("heart_full").unwrap().dimensions();
        let img_width = self.width(ctx, sw) / self.max_health;

        for i in 1..=(self.max_health as i32) {
            let index = i as f32;

            let draw_params = DrawParam::default()
                .dest([pos.x + index * img_width * 1.1, pos.y])
                .scale([img_width / img_dims.w, self.height(ctx, sh) / img_dims.h])
                .offset([0.5, 0.5]);

            let dif = self.health - index;

            if dif >= 0. { graphics::draw(ctx, assets.sprites.get("heart_full").unwrap(), draw_params)?; }
            else if dif >= -0.5 { graphics::draw(ctx, assets.sprites.get("heart_half").unwrap(), draw_params)?; }
            else { graphics::draw(ctx, assets.sprites.get("heart_empty").unwrap(), draw_params)?; }
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
    pub fn new(player: &Player, _dungeon: &Dungeon, cur_room: (usize, usize)) -> Self {
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
            Box::new(Minimap {
                pos: Point2 { x: MINIMAP_POS.0, y: MINIMAP_POS.1 },
                width: MINIMAP_SCALE,
                height: MINIMAP_SCALE,
                cur_room,
                rooms_state: vec![vec![RoomState::Undiscovered; DUNGEON_GRID_COLS]; DUNGEON_GRID_ROWS],
            }),
        ];

        Self {
            pos,
            width,
            height,
            ui_elements,
        }
    }

    pub fn update_vars(&mut self, player: &Player, dungeon: &Dungeon, cur_room: (usize, usize)) {
        for e in self.ui_elements.iter_mut() {
            if let Some(h) = e.as_any_mut().downcast_mut::<HealthBar>() {
                h.health = player.health;
                h.max_health = player.max_health;
            }
            else if let Some(m) = e.as_any_mut().downcast_mut::<Minimap>() {
                m.cur_room = cur_room;
                m.rooms_state = dungeon.get_grid().iter().map(|r| {
                    r.iter().map(|c| { 
                        if let Some(room) = c { room.state }
                        else { RoomState::Undiscovered }
                    }).collect()
                }).collect();
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

pub struct CheckBox {
    pub pos: Point2<f32>,
    pub width: f32,
    pub height: f32,
    pub checked: bool,
    pub color: Color,
}

impl Default for CheckBox {
    fn default() -> Self {
        Self {
            pos: Point2 { x: 0.5, y: 0.5 },
            width: 0.1,
            height: 0.1,
            checked: false,
            color: Color::BLACK,
        }
    }
}

impl UIElement for CheckBox {
    fn update(&mut self, _ctx: &mut Context, _conf: &Config) -> GameResult { Ok(()) }

    fn draw(&mut self, ctx: &mut Context, _assets: &Assets, conf: &Config) -> GameResult {
        let (sw, sh) = (conf.screen_width, conf.screen_height);
        let (w, h) = (self.width(ctx, sw), self.height(ctx, sh));
        let tl = self.top_left(ctx, sw, sh);

        let tick = &[Vec2::new(tl.x + w / 4., tl.y + h / 2.), 
                     Vec2::new(tl.x + w / 3., tl.y + h),
                     Vec2::new(tl.x + w * 2. / 3., tl.y),];

        let stroke = match self.checked {
            true => 5.,
            false => 0.,
        };

        let checkbox = MeshBuilder::new()
            .rectangle(DrawMode::fill(), Rect::new(tl.x, tl.y, w, h), Color::WHITE)?
            .rectangle(DrawMode::stroke(5.), Rect::new(tl.x, tl.y, w, h), self.color)?
            .line(tick, stroke, self.color)?
            .build(ctx)?;

        graphics::draw(ctx, &checkbox, DrawParam::default())?;

        Ok(())
    }

    fn pos(&self, sw: f32, sh: f32) -> Point2<f32> { Point2 { x: sw * self.pos.x, y: sh * self.pos.y } }

    fn width(&self, _ctx: &mut Context, sw: f32) -> f32 { sw * self.width }

    fn height(&self, _ctx: &mut Context, sh: f32) -> f32 { sh * self.height }

    fn as_any(&self) -> &dyn Any { self }
    
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

pub struct Slider {
    pub pos: Point2<f32>,
    pub width: f32,
    pub height: f32,
    pub lower: f32,
    pub upper: f32,
    pub steps: f32,
    pub value: f32,
    pub border: Border,
    // pub left_side_color: Color,
    // pub right_side_color: Color,
    pub decrease_button: Button,
    pub increase_button: Button,
    pub slider_button: Button,
}

impl Default for Slider {
    fn default() -> Self {
        Self {
            pos: Point2 { x: 0.5, y: 0.5 },
            width: 0.4,
            height: 0.1,
            lower: 0.,
            upper: 100.,
            steps: 100.,
            value: 50.,
            border: Border::default(),
            decrease_button: Button::default(),
            increase_button: Button::default(),
            slider_button: Button::default(),
        }
    }
}

impl Slider {
    pub fn increase(&mut self) {
        self.value = (self.value + (self.upper - self.lower) / self.steps).clamp(self.lower, self.upper);
    }

    pub fn decrease(&mut self) {
        self.value = (self.value - (self.upper - self.lower) / self.steps).clamp(self.lower, self.upper);
    }

    pub fn get_step_in_pixels(&self, sw: f32) -> f32 { (self.lower - self.upper) / self.steps * self.width * sw }
}

impl UIElement for Slider {
    fn update(&mut self, _ctx: &mut Context, _conf: &Config) -> GameResult { 
        let (sw, sh) = (_conf.screen_width, _conf.screen_height);
        let tl = self.top_left(_ctx, sw, sh);
        let prop = self.value / (self.upper - self.lower);

        self.slider_button = Button {
            pos: Point2 { x: (tl.x + self.width(_ctx, sw) * prop) / sw, y: self.pos.y },
            width: self.width * 0.1,
            height: self.height,
            ..Default::default()
        };
        self.decrease_button = Button {
            pos: Point2 { x: self.pos.x - self.width / 2. - self.slider_button.width / 2., y: self.pos.y },
            width: self.width * 0.1,
            height: self.height,
            text: self.decrease_button.text.clone(),
            ..Default::default()
        };
        self.increase_button = Button {
            pos: Point2 { x: self.pos.x + self.width / 2. + self.slider_button.width / 2., y: self.pos.y },
            width: self.width * 0.1,
            height: self.height,
            text: self.increase_button.text.clone(),
            ..Default::default()
        };

        Ok(()) 
    }

    fn draw(&mut self, _ctx: &mut Context, _assets: &Assets, _conf: &Config) -> GameResult {
        let (sw, sh) = (_conf.screen_width, _conf.screen_height);
        let (w, h) = (self.width(_ctx, sw), self.height(_ctx, sh));
        let tl = self.top_left(_ctx, sw, sh);

        let ruler = MeshBuilder::new()
            .rectangle(DrawMode::fill(), Rect::new(tl.x, tl.y, w, h), Color::RED)?
            .rectangle(DrawMode::fill(), Rect::new(tl.x, tl.y, w * self.value / (self.upper - self.lower), h), Color::GREEN)?
            .rounded_rectangle(DrawMode::stroke(self.border.stroke), Rect::new(tl.x, tl.y, w, h), self.border.radius, self.border.color)?
            .build(_ctx)?;

        graphics::draw(_ctx, &ruler, DrawParam::default())?;
        self.decrease_button.draw(_ctx, _assets, _conf)?;
        self.increase_button.draw(_ctx, _assets, _conf)?;
        self.slider_button.draw(_ctx, _assets, _conf)?;

        Ok(())
    }

    fn pos(&self, sw: f32, sh: f32) -> Point2<f32> { Point2 { x: sw * self.pos.x, y: sh * self.pos.y } }

    fn width(&self, _ctx: &mut Context, sw: f32) -> f32 { sw * self.width }

    fn height(&self, _ctx: &mut Context, sh: f32) -> f32 { sh * self.height }

    fn as_any(&self) -> &dyn Any { self }
    
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}
