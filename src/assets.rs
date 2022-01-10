use ggez::{
    graphics::{Image},
    Context,
    GameResult,
};

#[derive(Clone,Debug)]
pub struct Assets {
    pub player_base: Image,
    pub player_shoot_north: Image, 
    pub player_shoot_south: Image, 
    pub player_shoot_west: Image, 
    pub player_shoot_east: Image, 

    pub shot_base: Image,

    pub enemy_mask_base: Image,

    pub door_base: Image,
    pub door_open: Image,

    pub floor: Image,
    pub wall: Image,
}

impl Assets {
    pub fn new(ctx: &mut Context) -> GameResult<Self> {
        let player_base = Image::new(ctx, "/player_base.png")?;
        let player_shoot_north = Image::new(ctx, "/player_shoot_north.png")?;
        let player_shoot_south = Image::new(ctx, "/player_shoot_south.png")?;
        let player_shoot_west = Image::new(ctx, "/player_shoot_west.png")?;
        let player_shoot_east = Image::new(ctx, "/player_shoot_east.png")?;

        let shot_base = Image::new(ctx, "/shot_base.png")?;

        let enemy_mask_base = Image::new(ctx, "/enemy_mask_base.png")?;

        let door_base = Image::new(ctx, "/door_base.png")?;
        let door_open = Image::new(ctx, "/door_open.png")?;

        let floor = Image::new(ctx, "/floor.png")?;
        let wall = Image::new(ctx, "/wall.png")?;

        Ok(Self {
            player_base,
            player_shoot_north,
            player_shoot_south,
            player_shoot_west, 
            player_shoot_east, 

            shot_base,

            enemy_mask_base,

            door_base,
            door_open,

            floor,
            wall,
        })
    }
}

// pub trait Sprite: Debug {
//     fn draw(&mut self, center: Point2<f32>, ctx: &mut Context) -> GameResult<()>;
//     fn width(&self, ctx: &mut Context) -> f32;
//     fn height(&self, ctx: &mut Context) -> f32;
// }

// impl Sprite for TextSprite {
//     fn draw(&mut self, top_left: Point2<f32>, ctx: &mut Context) -> GameResult<()> {
//         graphics::draw(ctx, &self.text, graphics::DrawParam::default().dest(top_left))
//     }

//     fn width(&self, ctx: &mut Context) -> f32 { self.text.width(ctx) }
//     fn height(&self, ctx: &mut Context) -> f32 { self.text.height(ctx) }
// }
