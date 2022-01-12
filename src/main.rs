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
    scenes: HashMap<State, Box<dyn Scene>>,
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
            current_state: State::Start,
        }));
        let mut scenes = HashMap::<State, Box<dyn Scene>>::new();
        scenes.insert(State::Menu, Box::new(MenuScene::new(&config)));
        scenes.insert(State::Start, Box::new(StartScene::new(&config)));
        // scenes.insert(State::Dead, Box::new(DeadScene::new(&config)));

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
               scene = self.config.borrow().current_state;
            }

            match scene {
                State::New | State::Continue => input::mouse::set_cursor_grabbed(ctx, true)?,
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
           scene = self.config.borrow().current_state;
        }

        self.scenes.get_mut(&scene).unwrap().draw(ctx, &self.assets)?;

        graphics::present(ctx)?;
        Ok(())
    }

    fn key_down_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymod: input::keyboard::KeyMods, _repeat: bool) {
        let scene;

        {
           scene = self.config.borrow().current_state;
        }

        self.scenes.get_mut(&scene).unwrap().key_down_event(_ctx, keycode, _keymod, _repeat);
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymod: input::keyboard::KeyMods) {
        let scene;

        {
           scene = self.config.borrow().current_state;
        }

        self.scenes.get_mut(&scene).unwrap().key_up_event(_ctx, keycode, _keymod);
    }

    fn mouse_button_down_event(&mut self, _ctx: &mut Context, _button: MouseButton, _x: f32, _y: f32) {
        let mut scene;

        {
           scene = self.config.borrow().current_state;
        }

        self.scenes.get_mut(&scene).unwrap().mouse_button_down_event(_ctx, _button, _x, _y);

        {
           scene = self.config.borrow().current_state;
        }
        
        match scene {
            State::New => { 
                self.scenes.insert(State::Play, Box::new(PlayScene::new(&self.config)));
                self.config.borrow_mut().current_state = State::Play;
            },
            State::Continue => { 
                self.config.borrow_mut().current_state = State::Play;
            },
            State::Quit => { 
                ggez::event::quit(_ctx);
            },
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
