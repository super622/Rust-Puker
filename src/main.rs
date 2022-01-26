use ggez::{
    conf::{Conf, ModuleConf, WindowMode, WindowSetup},
    event::{self, EventHandler, KeyCode, MouseButton},
    filesystem,
    graphics,
    input,
    timer,
    Context, ContextBuilder, GameResult,
    audio::SoundSource,
};
use std::{cell::RefCell, collections::HashMap, env, path, rc::Rc};

use puker::{assets::*, consts::*, scenes::*, traits::*, utils::*};

struct MainState {
    config: Rc<RefCell<Config>>,
    scenes: HashMap<State, Box<dyn Scene>>,
    assets: Assets,
}

impl MainState {
    fn new(ctx: &mut Context, conf: &Conf) -> GameResult<MainState> {
        let mut assets = Assets::new(ctx)?;
        let config = Rc::new(RefCell::new(Config {
            screen_width: conf.window_mode.width,
            screen_height: conf.window_mode.height,
            window_width: conf.window_mode.width,
            window_height: conf.window_mode.height,
            volume: 0.5,
            draw_bcircle_model: true,
            draw_bbox_stationary: false,
            current_state: State::MainMenu,
            previous_state: State::MainMenu,
        }));
        let mut scenes = HashMap::<State, Box<dyn Scene>>::new();
        scenes.insert(State::PauseMenu, Box::new(PauseMenuScene::new(&config, &assets)));
        scenes.insert(State::MainMenu, Box::new(MainMenuScene::new(&config, &assets)));
        scenes.insert(State::Dead, Box::new(DeadScene::new(&config, &assets)));
        scenes.insert(State::Options, Box::new(OptionsScene::new(&config, &assets)));
              
        for (_, s) in assets.audio.iter_mut() {
            s.set_volume(0.3);
        }

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

            let scene = self.config.borrow().current_state;

            match scene {
                State::Play => input::mouse::set_cursor_grabbed(ctx, true)?,
                _ => input::mouse::set_cursor_grabbed(ctx, false)?,
            }

            self.scenes
                .get_mut(&scene)
                .unwrap()
                .update(ctx, &mut self.assets, delta_time)?;
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());

        let scene = self.config.borrow().current_state;

        match scene {
            State::PauseMenu | State::Dead | State::Options => {
                match self.scenes.get_mut(&State::Play) {
                    Some(s) => s.draw(ctx, &mut self.assets)?,
                    None => (),
                }
            },
            _ => (),
        }

        self.scenes
            .get_mut(&scene)
            .unwrap()
            .draw(ctx, &mut self.assets)?;

        graphics::present(ctx)?;
        Ok(())
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: KeyCode,
        _keymod: input::keyboard::KeyMods,
        _repeat: bool,
    ) {
        let scene = self.config.borrow().current_state;

        self.scenes
            .get_mut(&scene)
            .unwrap()
            .key_down_event(_ctx, keycode, _keymod, _repeat);
    }

    fn key_up_event(
        &mut self,
        _ctx: &mut Context,
        keycode: KeyCode,
        _keymod: input::keyboard::KeyMods,
    ) {
        let scene = self.config.borrow().current_state;

        self.scenes
            .get_mut(&scene)
            .unwrap()
            .key_up_event(_ctx, keycode, _keymod);
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        _button: MouseButton,
        _x: f32,
        _y: f32,
    ) {
        let mut scene = self.config.borrow().current_state;

        self.scenes
            .get_mut(&scene)
            .unwrap()
            .mouse_button_down_event(_ctx, _button, _x, _y);

        scene = self.config.borrow().current_state;

        match scene {
            State::New => {
                self.scenes
                    .insert(State::Play, Box::new(PlayScene::new(&self.config)));
                self.config.borrow_mut().current_state = State::Play;
            }
            State::Quit => {
                ggez::event::quit(_ctx);
            }
            _ => (),
        }
    }

    fn mouse_motion_event(&mut self, _ctx: &mut Context, _x: f32, _y: f32, _dx: f32, _dy: f32) {
        input::mouse::set_cursor_type(_ctx, input::mouse::CursorIcon::Default);
        let scene = self.config.borrow().current_state;

        self.scenes
            .get_mut(&scene)
            .unwrap()
            .mouse_motion_event(_ctx, _x, _y, _dx, _dy);
    }

    fn resize_event(&mut self, _ctx: &mut Context, _width: f32, _height: f32) {
        let mut conf = self.config.borrow_mut();
        conf.window_width = _width;
        conf.window_height = _height;
    }
}

fn main() -> GameResult {
    let conf = Conf {
        window_mode: WindowMode {
            width: DEFAULT_SCREEN_WIDTH,
            height: DEFAULT_SCREEN_HEIGHT,
            resizable: true,
            ..Default::default()
        },
        window_setup: WindowSetup {
            title: "puker".to_owned(),
            ..Default::default()
        },
        modules: ModuleConf {
            gamepad: false,
            ..Default::default()
        },
        ..Default::default()
    };

    let (mut ctx, event_loop) = ContextBuilder::new("puker", "Window")
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
