use ggez::{
    graphics::{self},
    Context,
    GameResult,
    event::{self, KeyCode, MouseButton, EventHandler},
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
    cell::RefCell,
};

use puker::{
    assets::*,
    utils::*,
    consts::*,
    scenes::*,
    traits::*,
};

struct MainState {
    config: Rc<RefCell<Config>>,
    scenes: HashMap<SceneType, Box<dyn Scene>>,
    assets: Assets,
}

impl MainState {
    fn new(ctx: &mut Context, conf: &Conf) -> GameResult<MainState> {
        let assets = Assets::new(ctx)?;
        let config = Rc::new(RefCell::new(Config {
            screen_width: conf.window_mode.width,
            screen_height: conf.window_mode.height,
            draw_bbox_model: true,            
            draw_bbox_stationary: false,            
            current_scene: SceneType::Menu,
        }));
        let mut scenes = HashMap::<SceneType, Box<dyn Scene>>::new();
        scenes.insert(SceneType::Menu, Box::new(MenuScene::new(&config)));

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

            let mut scene;

            {
               scene = self.config.borrow().current_scene;
            }

            match scene {
                SceneType::Play => input::mouse::set_cursor_grabbed(ctx, true)?,
                _ => input::mouse::set_cursor_grabbed(ctx, false)?,
            }

            self.scenes.get_mut(&scene).unwrap().update(ctx, delta_time)?;
        }
        
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());

        let scene;

        {
           scene = self.config.borrow().current_scene;
        }

        if scene == SceneType::Menu && self.scenes.contains_key(&SceneType::Play) {
            self.scenes.get_mut(&SceneType::Play).unwrap().draw(ctx, &self.assets)?;
        }

        self.scenes.get_mut(&scene).unwrap().draw(ctx, &self.assets)?;

        graphics::present(ctx)?;
        Ok(())
    }

    fn key_down_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymod: input::keyboard::KeyMods, _repeat: bool) {
        let scene;

        {
           scene = self.config.borrow().current_scene;
        }

        self.scenes.get_mut(&scene).unwrap().key_down_event(_ctx, keycode, _keymod, _repeat);

        if self.config.borrow().current_scene == SceneType::Play && !self.scenes.contains_key(&SceneType::Play) { 
            self.config.borrow_mut().current_scene = SceneType::Menu;
        }
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymod: input::keyboard::KeyMods) {
        let scene;

        {
           scene = self.config.borrow().current_scene;
        }

        self.scenes.get_mut(&scene).unwrap().key_up_event(_ctx, keycode, _keymod);
    }

    fn mouse_button_down_event(&mut self, _ctx: &mut Context, _button: MouseButton, _x: f32, _y: f32) {
        let scene;

        {
           scene = self.config.borrow().current_scene;
        }

        self.scenes.get_mut(&scene).unwrap().mouse_button_down_event(_ctx, _button, _x, _y);
        
        if self.config.borrow().current_scene == SceneType::Play && !self.scenes.contains_key(&SceneType::Play) { 
            self.scenes.insert(SceneType::Play, Box::new(PlayScene::new(&self.config)));
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
