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
    utils::*,
    traits::*,
    consts::*,
    player::*,
    dungeon::{Dungeon, RoomState},
    items::*,
};

#[derive(Debug, Clone)]
pub enum UIElementTag {
    Blank,
    WindowMode,
    Volume,
    State,
    Text,
}

#[derive(Debug, Clone)]
pub struct TextSprite {
    pub pos: Point2<f32>,
    pub tag: UIElementTag,
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
            tag: UIElementTag::Text,
            text: String::from(""),
            font: Font::default(),
            font_size: BUTTON_TEXT_FONT_SIZE,
            color: Color::BLACK,
            background: Color::from([0.; 4]),
        }
    }
}

impl UIElement for TextSprite {
    fn update(&mut self, _ctx: &mut Context, _conf: &mut Config) -> GameResult { Ok(()) }

    fn draw(&mut self, ctx: &mut Context, conf: &mut Config) -> GameResult {
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

    fn top_left(&self, ctx: &mut Context, sw: f32, sh: f32) -> Vec2 {
        let pos = self.pos(sw, sh);
        let (w, h) = (self.width(ctx, sh), self.height(ctx, sh));
        Vec2::new(pos.x - w / 2., pos.y - h / 2.)
    }
        
    fn as_any(&self) -> &dyn Any { self }

    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum ButtonAction {
    Blank,
    ChangeState(Option<State>),
    ChangeVolume(i8),
}

#[derive(Debug, Clone)]
pub struct Button {
    pub pos: Point2<f32>,
    pub width: f32,
    pub height: f32,
    pub action: ButtonAction,
    pub tag: UIElementTag,
    pub text: Option<TextSprite>,
    pub color: Color,
    pub border: Border,
    pub clicked: bool,
}

impl Default for Button {
    fn default() -> Self {
        Self {
            pos: Point2 { x: 0.5, y: 0.5 },
            width: 0.3,
            height: 0.1,
            action: ButtonAction::Blank,
            tag: UIElementTag::Blank,
            text: None,
            color: Color::WHITE,
            border: Border::default(),
            clicked: false,
        }
    }
}

impl UIElement for Button {
    fn update(&mut self, _ctx: &mut Context, _conf: &mut Config) -> GameResult { Ok(()) }

    fn draw(&mut self, ctx: &mut Context, conf: &mut Config) -> GameResult {
        let (sw, sh) = (conf.screen_width, conf.screen_height);
        let (ww, wh) = (conf.window_width, conf.window_height);
        let (w, h) = (self.width(ctx, sw), self.height(ctx, sh));
        let (mx, my) = (mouse::position(ctx).x, mouse::position(ctx).y);
        let tl = self.top_left(ctx, sw, sh);

        let mut text = None;
        if let Some(t) = &self.text {
            text = Some(t.clone());
        }

        let rect = Rect::new(tl.x, tl.y, w, h);
        let mut color = match self.mouse_overlap(ctx, mx, my, sw, sh, ww, wh) {
            true => {
                mouse::set_cursor_type(ctx, mouse::CursorIcon::Hand);
                if let Some(t) = &mut text { t.color = invert_color(&t.color); }
                invert_color(&self.color)
            },
            _ => self.color,
        };
        if self.clicked { color = Color::MAGENTA }

        let btn = MeshBuilder::new()
            .rounded_rectangle(DrawMode::fill(), rect, self.border.radius, color)?
            .rounded_rectangle(DrawMode::stroke(self.border.stroke), rect, self.border.radius, self.border.color)?
            .build(ctx)?;

        graphics::draw(ctx, &btn, DrawParam::default())?;
        if let Some(t) = &mut text { 
            t.pos = self.pos;
            t.draw(ctx, conf)?; 
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
    fn update(&mut self, _ctx: &mut Context, _conf: &mut Config) -> GameResult { Ok(()) }

    fn draw(&mut self, ctx: &mut Context, conf: &mut Config) -> GameResult {
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
    fn update(&mut self, _ctx: &mut Context, _conf: &mut Config) -> GameResult { Ok(()) }

    fn draw(&mut self, ctx: &mut Context, conf: &mut Config) -> GameResult {
        let (sw, sh) = (conf.screen_width, conf.screen_height);
        let pos = self.pos(sw, sh);

        for i in 1..=(self.max_health as i32) {
            let index = i as f32;
            let dif = self.health - index;
            let sprite;
            if dif >= 0. { sprite = conf.assets.sprites.get("heart_full").unwrap(); }
            else if dif >= -0.5 { sprite = conf.assets.sprites.get("heart_half").unwrap(); }
            else { sprite = conf.assets.sprites.get("heart_empty").unwrap(); }

            let img_dims = sprite.dimensions();
            let img_width = self.width(ctx, sw) / self.max_health;

            let draw_params = DrawParam::default()
                .dest([pos.x + index * img_width * 1.1, pos.y])
                .scale([img_width / img_dims.w, self.height(ctx, sh) / img_dims.h])
                .offset([0.5, 0.5]);


            graphics::draw(ctx, sprite, draw_params)?;
        }

        Ok(())
    }

    fn pos(&self, sw: f32, sh: f32) -> Point2<f32> { Point2 { x: sw * self.pos.x, y: sh * self.pos.y } }

    fn width(&self, _ctx: &mut Context, sw: f32) -> f32 { sw * self.width }

    fn height(&self, _ctx: &mut Context, sh: f32) -> f32 { sh * self.height }

    fn as_any(&self) -> &dyn Any { self }
    
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

pub struct ItemHolder {
    pub pos: Point2<f32>,
    pub width: f32,
    pub height: f32,
    pub item_tag: Option<ItemTag>,
    pub charge: f32,
    pub border: Border,
}

impl UIElement for ItemHolder {
    fn update(&mut self, _ctx: &mut Context, _conf: &mut Config) -> GameResult { Ok(()) }

    fn draw(&mut self, ctx: &mut Context, conf: &mut Config) -> GameResult {
        let (sw, sh) = (conf.screen_width, conf.screen_height);
        let (w, h) = (self.width(ctx, sw), self.height(ctx, sh));
        let tl = self.top_left(ctx, sw, sh);

        let sprite = match self.item_tag {
            Some(tag) => match tag {
                ItemTag::Passive(p) => match p {
                    ItemPassive::IncreaseMaxHealth(_) => conf.assets.sprites.get("poop_item"),
                },
                ItemTag::Active(a) => match a {
                    ItemActive::Heal(_) => conf.assets.sprites.get("heart_item"),
                },
            },
            None => None,
        };
        let holder = MeshBuilder::new()
            .rectangle(DrawMode::fill(), Rect::new(tl.x, tl.y, w, h), Color::new(0.2, 0.2, 0.2, 1.0))?
            .rounded_rectangle(DrawMode::stroke(self.border.stroke * 2.), Rect::new(tl.x, tl.y, w, h), self.border.radius, self.border.color)?
            .build(ctx)?;

        graphics::draw(ctx, &holder, DrawParam::default())?;
        if let Some(s) = sprite { 
            let dims = s.dimensions();

            let mut draw_params = DrawParam::default()
                .dest(self.pos(sw, sh))
                .scale([w / dims.w, h / dims.h])
                .offset([0.5, 0.5]);

            if self.charge < ITEM_COOLDOWN {
                draw_params = draw_params.color(Color::new(1., 1., 1., 0.3));
            }

            graphics::draw(ctx, s, draw_params)?; 
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
            Box::new(ItemHolder {
                pos: Point2 { x: ITEM_HOLDER_POS.0, y: ITEM_HOLDER_POS.1 },
                width: ITEM_HOLDER_SCALE,
                height: ITEM_HOLDER_SCALE,
                item_tag: match player.item {
                    Some(i) => Some(i.tag),
                    None => None,
                },
                charge: match player.item {
                    Some(i) => ITEM_COOLDOWN - i.cooldown,
                    None => 0.,
                },
                border: Border::default(),
            }),
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
            else if let Some(i) = e.as_any_mut().downcast_mut::<ItemHolder>() {
                match player.item {
                    Some(item) => {
                        i.item_tag = Some(item.tag);
                        i.charge = ITEM_COOLDOWN - item.cooldown;
                    },
                    None => {
                        i.item_tag = None;
                        i.charge = 0.;
                    }
                };
            }
        }
    }
}

impl UIElement for Overlay {
    fn update(&mut self, _ctx: &mut Context, _conf: &mut Config) -> GameResult {
        for e in self.ui_elements.iter_mut() { e.update(_ctx, _conf)?; }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context, _conf: &mut Config) -> GameResult {
        for e in self.ui_elements.iter_mut() { e.draw(ctx, _conf)?; }

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
    pub tag: UIElementTag,
    pub checked: bool,
    pub color: Color,
}

impl Default for CheckBox {
    fn default() -> Self {
        Self {
            pos: Point2 { x: 0.5, y: 0.5 },
            width: 0.1,
            height: 0.1,
            tag: UIElementTag::Blank,
            checked: false,
            color: Color::BLACK,
        }
    }
}

impl UIElement for CheckBox {
    fn update(&mut self, _ctx: &mut Context, _conf: &mut Config) -> GameResult { Ok(()) }

    fn draw(&mut self, ctx: &mut Context, conf: &mut Config) -> GameResult {
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
    pub tag: UIElementTag,
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
    pub last_mx: f32,
}

impl Default for Slider {
    fn default() -> Self {
        Self {
            pos: Point2 { x: 0.5, y: 0.5 },
            width: 0.4,
            height: 0.1,
            tag: UIElementTag::Blank,
            lower: 0.,
            upper: 100.,
            steps: 100.,
            value: 50.,
            border: Border::default(),
            decrease_button: Button {
                text: Some(TextSprite {
                    text: String::from("<<"),
                    font_size: BUTTON_TEXT_FONT_SIZE * 0.5,
                    ..Default::default()
                }),
                ..Default::default()
            },                
            increase_button: Button {
                text: Some(TextSprite {
                    text: String::from(">>"),
                    font_size: BUTTON_TEXT_FONT_SIZE * 0.5,
                    ..Default::default()
                }),
                ..Default::default()
            },                
            slider_button: Button::default(),
            last_mx: 0.,
        }
    }
}

impl Slider {
    pub fn get_step(&self) -> f32 {
        (self.upper - self.lower) / self.steps
    }

    pub fn get_step_in_pixels(&self, sw: f32) -> f32 {
        self.width * sw * 0.7 / self.steps 
    }

    pub fn get_overlapped_mut(&mut self, ctx: &mut Context, mx: f32, my: f32, sw: f32, sh: f32, ww: f32, hh: f32) -> Option<&mut Button> {
        if self.decrease_button.mouse_overlap(ctx, mx, my, sw, sh, ww, hh) {
            Some(&mut self.decrease_button)
        }
        else if self.increase_button.mouse_overlap(ctx, mx, my, sw, sh, ww, hh) {
            Some(&mut self.increase_button)
        }
        else if self.slider_button.mouse_overlap(ctx, mx, my, sw, sh, ww, hh) {
            Some(&mut self.slider_button)
        }
        else {
            None
        }
    }
}

impl UIElement for Slider {
    fn update(&mut self, _ctx: &mut Context, _conf: &mut Config) -> GameResult { 
        let (sw, sh) = (_conf.screen_width, _conf.screen_height);
        let tl = self.top_left(_ctx, sw, sh);
        let step = self.get_step_in_pixels(sw);
        let lower = tl.x + self.width(_ctx, sw) * 0.15;
        let upper = tl.x + self.upper * step + self.width(_ctx, sw) * 0.15;

        self.last_mx = self.last_mx.clamp(lower, upper);

        self.slider_button.pos = Point2 { x: (lower + step * self.value) / sw, y: self.pos.y };
        self.slider_button.width = self.width * 0.1;
        self.slider_button.height = self.height;

        self.decrease_button.pos = Point2 { x: self.pos.x - self.width / 2. + self.width * 0.1 / 2., y: self.pos.y }; 
        self.decrease_button.width = self.width * 0.1;
        self.decrease_button.height = self.height;

        self.increase_button.pos = Point2 { x: self.pos.x + self.width / 2. - self.width * 0.1 / 2., y: self.pos.y }; 
        self.increase_button.width = self.width * 0.1;
        self.increase_button.height = self.height;

        Ok(()) 
    }

    fn draw(&mut self, _ctx: &mut Context, _conf: &mut Config) -> GameResult {
        let (sw, sh) = (_conf.screen_width, _conf.screen_height);
        let (w, h) = (self.width(_ctx, sw) * 0.7, self.height(_ctx, sh));
        let tl = self.top_left(_ctx, sw, sh) + Vec2::X * self.width(_ctx, sw) * 0.15;

        let ruler = MeshBuilder::new()
            .rectangle(DrawMode::fill(), Rect::new(tl.x, tl.y, w, h), Color::RED)?
            .rectangle(DrawMode::fill(), Rect::new(tl.x, tl.y, w * self.value / 100., h), Color::GREEN)?
            .rounded_rectangle(DrawMode::stroke(self.border.stroke), Rect::new(tl.x, tl.y, w, h), self.border.radius, self.border.color)?
            .build(_ctx)?;

        graphics::draw(_ctx, &ruler, DrawParam::default())?;
        self.decrease_button.draw(_ctx, _conf)?;
        self.increase_button.draw(_ctx, _conf)?;
        self.slider_button.draw(_ctx, _conf)?;

        Ok(())
    }

    fn pos(&self, sw: f32, sh: f32) -> Point2<f32> { Point2 { x: sw * self.pos.x, y: sh * self.pos.y } }

    fn width(&self, _ctx: &mut Context, sw: f32) -> f32 { sw * self.width }

    fn height(&self, _ctx: &mut Context, sh: f32) -> f32 { sh * self.height }

    fn as_any(&self) -> &dyn Any { self }
    
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}
