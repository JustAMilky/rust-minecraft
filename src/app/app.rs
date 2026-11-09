use winit::{
    application::ApplicationHandler, event::*, event_loop::ActiveEventLoop, keyboard::PhysicalKey, window::Window
};
use std::sync::Arc;
use std::time::Instant;
use crate::gamestate::gamestate::GameState;
use crate::renderer::renderer::Renderer;
use crate::gui::gui::Gui;

pub struct App {
    window: Option<Arc<Window>>,
    state: Option<GameState>,
    renderer: Option<Renderer>,
    gui: Option<Gui>,

    fps: f32,
    last_frame: std::time::Instant,
}

impl App {
    pub fn new() -> Self {
        Self {
            window: None,
            state: None,
            renderer: None,
            gui: None,
            fps: 0.0,
            last_frame: Instant::now(),
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

        let renderer = pollster::block_on(
            Renderer::new(window.clone())
        ).expect("Failed to create renderer");

        let state = pollster::block_on(
            GameState::new()
        ).unwrap();

        let gui = Gui::new(
            &window,
            &renderer.device,
            renderer.config.clone(),
        ).expect("Failed to create GUI");

        self.window = Some(window);
        self.renderer = Some(renderer);
        self.state = Some(state);
        self.gui = Some(gui);
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
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
            self.state.as_ref(),
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

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => renderer.resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let delta = now.duration_since(self.last_frame).as_secs_f32();

                self.last_frame = now;
                self.fps = 1.0 / delta.max(0.000001);

                gui.update(self.fps);

                self.state
                    .as_mut()
                    .expect("Unable to update game state: No game state")
                    .update();

                match renderer.render(window, gui, 0.0) {
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