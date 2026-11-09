use std::{collections::HashMap, vec};

use crate::{model::model::*, utils::direction::Direction};

pub fn populate_model_registry(model_registry: &mut ModelRegistry) {
    model_registry.models = HashMap::from([
        (ModelId::Air, air_model()),
        (ModelId::Stone, stone_model()),
    ]);
}

fn air_model() -> BlockModel {
    BlockModel {
        elements: vec![],
        textures: HashMap::new(),
    }
}

fn stone_model() -> BlockModel {
    BlockModel {
        elements: vec![
            ModelElement {
                from: [0.0, 0.0, 0.0],
                to: [16.0, 16.0, 16.0],
                faces: HashMap::from([
                    (Direction::Up,    ModelFace { direction: Direction::Up,    uv: [0.0, 0.0, 16.0, 16.0], cullface: Direction::Up,    texture: "all".to_string() }),
                    (Direction::Down,  ModelFace { direction: Direction::Down,  uv: [0.0, 0.0, 16.0, 16.0], cullface: Direction::Down,  texture: "all".to_string() }),
                    (Direction::North, ModelFace { direction: Direction::North, uv: [0.0, 0.0, 16.0, 16.0], cullface: Direction::North, texture: "all".to_string() }),
                    (Direction::South, ModelFace { direction: Direction::South, uv: [0.0, 0.0, 16.0, 16.0], cullface: Direction::South, texture: "all".to_string() }),
                    (Direction::East,  ModelFace { direction: Direction::East,  uv: [0.0, 0.0, 16.0, 16.0], cullface: Direction::East,  texture: "all".to_string() }),
                    (Direction::West,  ModelFace { direction: Direction::West,  uv: [0.0, 0.0, 16.0, 16.0], cullface: Direction::West,  texture: "all".to_string() }),
                ])
            },
        ],
        textures: HashMap::from([
            ("all".to_string(), "stone".to_string()),
        ])
    }
}

fn _dirt_model() -> BlockModel {
    let mut model = stone_model();
    model.textures = HashMap::from([
        ("all".to_string(), "dirt".to_string()),
    ]);
    model
}