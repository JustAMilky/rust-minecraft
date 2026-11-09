use winit::event_loop::EventLoop;
use std::collections::HashMap;

mod app;
mod renderer;
mod gamestate;
mod gui;
mod model;
mod block;
mod utils;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let mut model_registry = model::model::ModelRegistry {
        models: HashMap::new(),
    };

    model::modelregistry::populate_model_registry(&mut model_registry);

    let event_loop = EventLoop::with_user_event().build()?;

    let mut app = app::app::App::new();
    event_loop.run_app(&mut app)?;

    Ok(())
}