use std::collections::HashMap;

use crate::utils::direction::Direction;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModelId {
    Air,
    Stone,
}

pub struct ModelRegistry {
    pub models: HashMap<ModelId, BlockModel>,
}

pub struct BlockModel {
    pub elements: Vec<ModelElement>,
    pub textures: HashMap<String, String>,
}

pub struct ModelElement {
    pub from: [f32; 3],
    pub to: [f32; 3],
    pub faces: HashMap<Direction, ModelFace>,
}

pub struct ModelFace {
    pub direction: Direction,
    pub uv: [f32; 4],
    pub texture: String,
    pub cullface: Direction,
}