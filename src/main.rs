use ggez::{
    graphics::{self},
    Context,
    GameResult,
    event::{self, KeyCode, EventHandler},
    conf::{Conf,WindowMode},
    ContextBuilder,
    filesystem,
    input::{self, keyboard, mouse},
    timer,
};
use std::{
    env,
    path,
    collections::{HashMap},
    rc::Rc,
};

use puker::{
    assets::*,
    utils::*,
    consts::*,
    scenes::*,
    traits::*,
};

struct MainState {
    config: Rc<Config>,
    scenes: HashMap<SceneType, Box<dyn Scene>>,
    assets: Assets,
}

impl MainState {
    fn new(ctx: &mut Context, conf: &Conf) -> GameResult<MainState> {
        input::mouse::set_cursor_grabbed(ctx, true)?;

        let assets = Assets::new(ctx)?;
        let config = Rc::new(Config {
            screen_width: conf.window_mode.width,
            screen_height: conf.window_mode.height,
            draw_bbox_model: true,            
            draw_bbox_stationary: false,            
            current_scene: SceneType::Play,
        });
        let mut scenes = HashMap::<SceneType, Box<dyn Scene>>::new();
        scenes.insert(SceneType::Play, Box::new(PlayScene::new(ctx, &config)?));

        let s = MainState {
            config,
            scenes,
            assets,
        };

        Ok(s)
    }
}

impl EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        const DESIRED_FPS: u32 = 60;

        while timer::check_update_time(ctx, DESIRED_FPS) {
            let delta_time = 1.0 / (DESIRED_FPS as f32);

            self.scenes.get_mut(&self.config.current_scene).unwrap().update(ctx, delta_time)?;
        }
        
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());

        self.scenes.get_mut(&self.config.current_scene).unwrap().draw(ctx, &self.assets)?;

        graphics::present(ctx)?;
        Ok(())
    }

    fn key_down_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymod: input::keyboard::KeyMods, _repeat: bool) {
        match keycode {
            _ => (),
        }
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymod: input::keyboard::KeyMods) {
        match keycode {
            _ => (),
        }
    }
}

fn main() -> GameResult {
    let conf = Conf::new()
        .window_mode(WindowMode {
            width: DEFAULT_SCREEN_WIDTH,
            height: DEFAULT_SCREEN_HEIGHT,
            ..Default::default()
        });

    let (mut ctx, event_loop) = ContextBuilder::new("PrimitiveIsaac", "Window")
        .default_conf(conf.clone())
        .build()
        .unwrap();

    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        filesystem::mount(&mut ctx, &path, true);
    }

    let state = MainState::new(&mut ctx, &conf)?;

    event::run(ctx, event_loop, state)
}
