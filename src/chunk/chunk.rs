use glam::IVec3;
use crate::block::blockstate::BlockState;

const CHUNK_SIZE: i32 = 16;

pub struct Chunk {
    generated: bool,
    meshing: bool,
    meshed: bool,
    needs_remesh: bool,

    blocks: Vec<BlockState>,
}

impl Chunk {
    fn index(pos: IVec3) -> usize {
        (pos.x + CHUNK_SIZE * (pos.y + CHUNK_SIZE * pos.z)) as usize
    }

    fn get_block_at(&self, pos: IVec3) -> BlockState {
        self.blocks[Chunk::index(pos)].clone()
    }

    fn set_block_at(&mut self, pos: IVec3, block: BlockState) {
        if !(pos.x < CHUNK_SIZE && pos.y < CHUNK_SIZE && pos.z < CHUNK_SIZE) {
            return;
        }

        self.blocks[Chunk::index(pos)] = block.clone();
    }
}