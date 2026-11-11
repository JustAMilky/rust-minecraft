use crate::block::block::*;

#[derive(Clone, Debug)]
pub enum BlockState {
    Air,
    Stone,
}

impl BlockState {
    pub fn get_block(&self) -> &dyn Block {
        match self {
            BlockState::Air => &Air,
            BlockState::Stone => &Stone,
        }
    }
}