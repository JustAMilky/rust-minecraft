use crate::block::block::*;

#[derive(Clone, Debug, PartialEq)]
pub enum BlockState {
    Air,
    Stone,
    Dirt,
    Grass,
}

impl BlockState {
    pub fn get_block(&self) -> &dyn Block {
        match self {
            BlockState::Air => &Air,
            BlockState::Stone => &Stone,
            BlockState::Dirt => &Dirt,
            BlockState::Grass => &Grass,
        }
    }
}