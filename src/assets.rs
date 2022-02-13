use ggez::{
    graphics,
    audio,
    Context,
    GameResult,
    filesystem,
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

        for f in filesystem::read_dir(&ctx, "/sprites")?.into_iter() {
            sprites.insert(f.file_stem().unwrap().to_str().unwrap().to_owned(), graphics::Image::new(ctx, format!("/sprites/{}", f.file_name().unwrap().to_str().unwrap().to_owned()))?);
        }

        for f in filesystem::read_dir(&ctx, "/fonts")?.into_iter() {
            fonts.insert(f.file_stem().unwrap().to_str().unwrap().to_owned(), graphics::Font::new(ctx, format!("/fonts/{}", f.file_name().unwrap().to_str().unwrap().to_owned()))?);
        }

        for f in filesystem::read_dir(&ctx, "/audio")?.into_iter() {
            audio.insert(f.file_stem().unwrap().to_str().unwrap().to_owned(), audio::Source::new(ctx, format!("/audio/{}", f.file_name().unwrap().to_str().unwrap().to_owned()))?);
        }

        Ok(Self {
            sprites,
            fonts,
            audio,
        })
    }
}
