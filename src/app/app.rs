use winit::{
    application::ApplicationHandler, event::*, event_loop::ActiveEventLoop, keyboard::PhysicalKey, window::Window
};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::gamestate::gamestate::GameState;
use crate::renderer::renderer::Renderer;
use crate::gui::gui::Gui;
use crate::texture::texture::{Texture, TextureAtlas};
use crate::app::input::InputManager;

pub struct App {
    window: Option<Arc<Window>>,
    state: Option<GameState>,
    renderer: Option<Renderer>,
    gui: Option<Gui>,

    fps: f32,
    tps: f32,
    ticks_since_tps_update: u32,
    last_tps_update: Instant,
    last_frame: std::time::Instant,
    last_tick: std::time::Instant,
    tick_interval: Duration,

    input_man: InputManager,

    texture_atlas: TextureAtlas,
}

impl App {
    pub fn new(texture_atlas: TextureAtlas) -> Self {
        Self {
            window: None,
            state: None,
            renderer: None,
            gui: None,
            fps: 0.0,
            tps: 0.0,
            ticks_since_tps_update: 0,
            last_tps_update: Instant::now(),
            last_frame: Instant::now(),
            last_tick: Instant::now(),
            tick_interval: Duration::from_millis(50),
            input_man: InputManager::new(),
            texture_atlas,
        }
    }
}

impl ApplicationHandler<()> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_title("Minecraft")
            .with_visible(true)
            .with_inner_size(winit::dpi::PhysicalSize::new(960, 540));

        let window = Arc::new(
            event_loop.create_window(window_attributes).unwrap()
        );

        let state = pollster::block_on(
            GameState::new(&self.texture_atlas)
        ).unwrap();

        self.state = Some(state);

        let renderer = pollster::block_on(
            Renderer::new(window.clone(), Texture {
                width: self.texture_atlas.width,
                height: self.texture_atlas.height,
                pixels: self.texture_atlas.pixels.clone(),
            }
        )
        ).expect("Failed to create renderer");

        let gui = Gui::new(
            &window,
            &renderer.device,
            renderer.config.clone(),
        ).expect("Failed to create GUI");

        self.renderer = Some(renderer);
        self.gui = Some(gui);
        self.window = Some(window);
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        let now = Instant::now();

        if now.duration_since(self.last_tick) >= self.tick_interval {
            self.last_tick = now;

            self.state
                .as_mut()
                .expect("Unable to execute game tick")
                .game_tick();
            self.ticks_since_tps_update += 1;
        }

        let tps_elapsed = now.duration_since(self.last_tps_update);
        if tps_elapsed >= Duration::from_secs(1) {
            self.tps = self.ticks_since_tps_update as f32 / tps_elapsed.as_secs_f32();
            self.ticks_since_tps_update = 0;
            self.last_tps_update = now;
        }

        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let (Some(window), Some(state), Some(renderer), Some(gui)) = (
            self.window.as_ref(),
            self.state.as_mut(),
            self.renderer.as_mut(),
            self.gui.as_mut(),
        ) else {
            return;
        };

        if window.id() != window_id {
            return;
        }

        let response = gui.state.on_window_event(window, &event);

        if response.repaint {
            window.request_redraw();
        }

        self.input_man.process_event(&event);

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
                std::process::exit(0);
            },
            WindowEvent::Resized(size) => {
                renderer.resize(size.width, size.height);
                if size.height > 0 {
                    state.camera.aspect_ratio = size.width as f32 / size.height as f32;
                }
            }
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let delta = now.duration_since(self.last_frame).as_secs_f32();

                self.last_frame = now;
                self.fps = 1.0 / delta.max(0.000001);

                gui.update(self.fps, self.tps);

                state.frame_tick(&self.input_man, renderer);

                match renderer.render(
                    window,
                    gui,
                    &state.camera,
                    
                    &mut state.lateral_render_distance,
                    &mut state.vertical_render_distance,
                    &mut state.needs_chunk_load_refresh,

                    &mut state.camera_fly_speed,
                ) {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!("{e}");
                        event_loop.exit();
                    }
                }
            },
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => state.handle_key(event_loop, code, key_state.is_pressed()),
            _ => {}
        };
    }
}