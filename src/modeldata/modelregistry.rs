use std::{collections::HashMap, vec};

use crate::{modeldata::model::*, texture::TextureId, utils::direction::Direction};

pub fn populate_model_registry(model_registry: &mut ModelRegistry) {
    model_registry.models =
        HashMap::from([(ModelId::Air, air_model()), (ModelId::Stone, stone_model())]);
}

pub fn air_model() -> BlockModel {
    BlockModel { elements: vec![] }
}

pub fn stone_model() -> BlockModel {
    BlockModel {
        elements: vec![ModelElement {
            from: [0.0, 0.0, 0.0],
            to: [16.0, 16.0, 16.0],
            faces: HashMap::from([
                (
                    Direction::Up,
                    ModelFace {
                        uv: [0.0, 0.0, 16.0, 16.0],
                        cullface: Direction::Up,
                        texture: "all".to_string(),
                    },
                ),
                (
                    Direction::Down,
                    ModelFace {
                        uv: [0.0, 0.0, 16.0, 16.0],
                        cullface: Direction::Down,
                        texture: "all".to_string(),
                    },
                ),
                (
                    Direction::North,
                    ModelFace {
                        uv: [0.0, 0.0, 16.0, 16.0],
                        cullface: Direction::North,
                        texture: "all".to_string(),
                    },
                ),
                (
                    Direction::South,
                    ModelFace {
                        uv: [0.0, 0.0, 16.0, 16.0],
                        cullface: Direction::South,
                        texture: "all".to_string(),
                    },
                ),
                (
                    Direction::East,
                    ModelFace {
                        uv: [0.0, 0.0, 16.0, 16.0],
                        cullface: Direction::East,
                        texture: "all".to_string(),
                    },
                ),
                (
                    Direction::West,
                    ModelFace {
                        uv: [0.0, 0.0, 16.0, 16.0],
                        cullface: Direction::West,
                        texture: "all".to_string(),
                    },
                ),
            ]),
            textures: HashMap::from([("all".to_string(), TextureId::Stone)]),
        }],
    }
}

pub fn dirt_model() -> BlockModel {
    let mut model = stone_model();
    model.elements[0].textures = HashMap::from([("all".to_string(), TextureId::Dirt)]);
    model
}

pub fn grass_model() -> BlockModel {
    BlockModel {
        elements: vec![ModelElement {
            from: [0.0, 0.0, 0.0],
            to: [16.0, 16.0, 16.0],
            faces: HashMap::from([
                (
                    Direction::Up,
                    ModelFace {
                        uv: [0.0, 0.0, 16.0, 16.0],
                        cullface: Direction::Up,
                        texture: "top".to_string(),
                    },
                ),
                (
                    Direction::Down,
                    ModelFace {
                        uv: [0.0, 0.0, 16.0, 16.0],
                        cullface: Direction::Down,
                        texture: "bottom".to_string(),
                    },
                ),
                (
                    Direction::North,
                    ModelFace {
                        uv: [0.0, 0.0, 16.0, 16.0],
                        cullface: Direction::North,
                        texture: "sides".to_string(),
                    },
                ),
                (
                    Direction::South,
                    ModelFace {
                        uv: [0.0, 0.0, 16.0, 16.0],
                        cullface: Direction::South,
                        texture: "sides".to_string(),
                    },
                ),
                (
                    Direction::East,
                    ModelFace {
                        uv: [0.0, 0.0, 16.0, 16.0],
                        cullface: Direction::East,
                        texture: "sides".to_string(),
                    },
                ),
                (
                    Direction::West,
                    ModelFace {
                        uv: [0.0, 0.0, 16.0, 16.0],
                        cullface: Direction::West,
                        texture: "sides".to_string(),
                    },
                ),
            ]),
            textures: HashMap::from([
                ("top".to_string(), TextureId::GrassTop),
                ("sides".to_string(), TextureId::GrassSide),
                ("bottom".to_string(), TextureId::Dirt),
            ]),
        }],
    }
}

pub fn leaves_model() -> BlockModel {
    let mut model = stone_model();
    model.elements[0].textures = HashMap::from([("all".to_string(), TextureId::Leaves)]);
    model
}

pub fn log_model() -> BlockModel {
    let mut model = grass_model();
    model.elements[0].textures = HashMap::from([
        ("top".to_string(), TextureId::LogTop),
        ("sides".to_string(), TextureId::LogSide),
        ("bottom".to_string(), TextureId::LogTop),
    ]);
    model
}
