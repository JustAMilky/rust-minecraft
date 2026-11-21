use glam::IVec3;

use crate::{blockdata::blockstate::BlockState, utils::posinrange::pos_in_range};

#[derive(Clone)]
pub struct Structure {
    pub min: IVec3,
    pub max: IVec3,
    pub pos: IVec3,
    pub blocks: Vec<BlockState>,
}

impl Structure {
    pub fn new(min: IVec3, max: IVec3, pos: IVec3) -> Self {
        let size = max - min + IVec3::ONE;

        Self {
            min,
            max,
            pos,
            blocks: vec![BlockState::Air; (size.x * size.y * size.z) as usize],
        }
    }

    fn index(&self, pos: IVec3) -> usize {
        let size = self.max - self.min + IVec3::ONE;
        let local = pos - self.min;

        (local.x + size.x * (local.y + size.y * local.z)) as usize
    }

    pub fn get_block(&self, pos: IVec3) -> &BlockState {
        if !pos_in_range(&pos, &self.min, &self.max) {
            return &BlockState::Air;
        }

        &self.blocks[self.index(pos)]
    }

    pub fn set_block(&mut self, pos: IVec3, block: BlockState) -> bool {
        if !pos_in_range(&pos, &self.min, &self.max) {
            return false;
        }

        let index = self.index(pos);
        self.blocks[index] = block;
        true
    }
}

#[derive(Clone)]
pub struct StructureStart {
    pub chunk_pos: IVec3,
    pub structure: Structure,
}

impl StructureStart {
    pub fn new(chunk_pos: IVec3, structure: Structure) -> Self {
        Self {
            chunk_pos,
            structure,
        }
    }
}
