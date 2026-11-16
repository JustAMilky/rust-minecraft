use std::collections::HashMap;
use winit::event_loop::EventLoop;

use crate::texture::*;

mod application;
mod blockdata;
mod camera;
mod chunkdata;
mod gamestate;
mod gui;
mod modeldata;
mod render;
mod texture;
mod utils;
mod worldgen;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let texture_paths = vec![
        (TextureId::Stone, "assets/textures/block/stone.png"),
        (TextureId::Dirt, "assets/textures/block/dirt.png"),
        (
            TextureId::GrassSide,
            "assets/textures/block/grass_block_side.png",
        ),
        (
            TextureId::GrassTop,
            "assets/textures/block/grass_block_top.png",
        ),
        (TextureId::OakPlanks, "assets/textures/block/oak_planks.png"),
    ];

    let texture_registry = TextureRegistry::from_paths(texture_paths);

    let texture_atlas = TextureAtlas::new(texture_registry.textures.into_iter().collect())
        .expect("Failed to generate texture atlas");

    let mut model_registry = modeldata::model::ModelRegistry {
        models: HashMap::new(),
    };

    modeldata::modelregistry::populate_model_registry(&mut model_registry);

    let event_loop = EventLoop::with_user_event().build()?;

    let mut app = application::app::App::new(texture_atlas);
    event_loop.run_app(&mut app)?;

    Ok(())
}
