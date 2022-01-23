use ggez::{
    graphics,
    audio,
    Context,
    GameResult,
};
use std::collections::HashMap;

pub struct Assets {
    pub sprites: HashMap<String, graphics::Image>,
    pub fonts: HashMap<String, graphics::Font>,
    pub audio: HashMap<String, audio::Source>,
}

impl Assets {
    pub fn new(ctx: &mut Context) -> GameResult<Self> {
        let mut sprites = HashMap::new();
        let mut fonts = HashMap::new();
        let mut audio = HashMap::new();

        sprites.insert("player_base".to_string(), graphics::Image::new(ctx, "/images/player_base.png")?);
        sprites.insert("player_damaged".to_string(), graphics::Image::new(ctx, "/images/player_damaged.png")?); 
        sprites.insert("player_dead".to_string(), graphics::Image::new(ctx, "/images/player_dead.png")?); 
        sprites.insert("player_shoot_north".to_string(), graphics::Image::new(ctx, "/images/player_shoot_north.png")?);
        sprites.insert("player_shoot_south".to_string(), graphics::Image::new(ctx, "/images/player_shoot_south.png")?);
        sprites.insert("player_shoot_west".to_string(), graphics::Image::new(ctx, "/images/player_shoot_west.png")?);
        sprites.insert("player_shoot_east".to_string(), graphics::Image::new(ctx, "/images/player_shoot_east.png")?);
        sprites.insert("shot_puke_base".to_string(), graphics::Image::new(ctx, "/images/shot_puke_base.png")?);
        sprites.insert("shot_blood_base".to_string(), graphics::Image::new(ctx, "/images/shot_blood_base.png")?);
        sprites.insert("enemy_mask_base".to_string(), graphics::Image::new(ctx, "/images/enemy_mask_base.png")?);
        sprites.insert("enemy_blue_guy_base".to_string(), graphics::Image::new(ctx, "/images/enemy_blue_guy_base.png")?);
        sprites.insert("enemy_slime_east".to_string(), graphics::Image::new(ctx, "/images/enemy_slime_east.png")?);
        sprites.insert("enemy_slime_west".to_string(), graphics::Image::new(ctx, "/images/enemy_slime_west.png")?);
        sprites.insert("enemy_slime_north".to_string(), graphics::Image::new(ctx, "/images/enemy_slime_north.png")?);
        sprites.insert("enemy_slime_south".to_string(), graphics::Image::new(ctx, "/images/enemy_slime_south.png")?);
        sprites.insert("boss_weird_ball_base".to_string(), graphics::Image::new(ctx, "/images/boss_weird_ball_base.png")?);
        sprites.insert("boss_weird_ball_shoot_cardinals".to_string(), graphics::Image::new(ctx, "/images/boss_weird_ball_shoot_cardinals.png")?);
        sprites.insert("boss_weird_ball_shoot_diagonals".to_string(), graphics::Image::new(ctx, "/images/boss_weird_ball_shoot_diagonals.png")?);
        sprites.insert("door_closed".to_string(), graphics::Image::new(ctx, "/images/door_closed.png")?);
        sprites.insert("door_open".to_string(), graphics::Image::new(ctx, "/images/door_open.png")?);
        sprites.insert("floor".to_string(), graphics::Image::new(ctx, "/images/floor.png")?);
        sprites.insert("wall".to_string(), graphics::Image::new(ctx, "/images/wall.png")?);
        sprites.insert("stone".to_string(), graphics::Image::new(ctx, "/images/stone.png")?);
        sprites.insert("heart_full".to_string(), graphics::Image::new(ctx, "/images/heart_full.png")?);
        sprites.insert("heart_half".to_string(), graphics::Image::new(ctx, "/images/heart_half.png")?);
        sprites.insert("heart_empty".to_string(), graphics::Image::new(ctx, "/images/heart_empty.png")?);
        sprites.insert("puker_logo".to_string(), graphics::Image::new(ctx, "/images/puker_logo.png")?);
        sprites.insert("spikes".to_string(), graphics::Image::new(ctx, "/images/spikes.png")?);
        sprites.insert("heart_full_collectable".to_string(), graphics::Image::new(ctx, "/images/heart_full_collectable.png")?);
        sprites.insert("heart_half_collectable".to_string(), graphics::Image::new(ctx, "/images/heart_half_collectable.png")?);
        sprites.insert("speed_boost".to_string(), graphics::Image::new(ctx, "/images/speed_boost.png")?);
        sprites.insert("damage_boost".to_string(), graphics::Image::new(ctx, "/images/damage_boost.png")?);
        sprites.insert("shoot_rate_boost".to_string(), graphics::Image::new(ctx, "/images/shoot_rate_boost.png")?);
        sprites.insert("hatch_open".to_string(), graphics::Image::new(ctx, "/images/hatch_open.png")?);
        sprites.insert("hatch_closed".to_string(), graphics::Image::new(ctx, "/images/hatch_closed.png")?);

        fonts.insert("button_font".to_string(), graphics::Font::new(ctx, "/fonts/enigma.ttf")?);

        audio.insert("player_death_sound".to_string(), audio::Source::new(ctx, "/audio/player_death_sound.mp3")?);
        audio.insert("player_damaged_sound".to_string(), audio::Source::new(ctx, "/audio/player_damaged_sound.mp3")?);
        audio.insert("enemy_death_sound".to_string(), audio::Source::new(ctx, "/audio/enemy_death_sound.mp3")?);
        audio.insert("door_close_sound".to_string(), audio::Source::new(ctx, "/audio/door_close_sound.mp3")?);
        audio.insert("door_open_sound".to_string(), audio::Source::new(ctx, "/audio/door_open_sound.mp3")?);
        audio.insert("bubble_pop_sound".to_string(), audio::Source::new(ctx, "/audio/bubble_pop_sound.mp3")?);
        audio.insert("power_up_sound".to_string(), audio::Source::new(ctx, "/audio/power_up_sound.mp3")?);
        audio.insert("heal_sound".to_string(), audio::Source::new(ctx, "/audio/heal_sound.mp3")?);
        audio.insert("boss_death_sound".to_string(), audio::Source::new(ctx, "/audio/boss_death_sound.mp3")?);

        Ok(Self {
            sprites,
            fonts,
            audio,
        })
    }
}
