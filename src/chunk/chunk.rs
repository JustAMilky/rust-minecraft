use std::collections::{HashMap, HashSet};

use glam::IVec3;
use crate::{block::blockstate::BlockState, renderer::mesh::Mesh, texture::texture::TextureAtlas, utils::direction::Direction};

pub const CHUNK_SIZE: i32 = 32;

#[derive(Clone)]
pub struct Chunk {
    pub generated: bool,
    pub meshed: bool,
    pub needs_remesh: bool,
    pub was_edge: bool,

    pub all_air: bool,

    palette: Vec<BlockState>,
    bits_per_entry: u8,
    blocks: Vec<u64>,
    pub mesh: Mesh,
}

impl Chunk {
    pub fn new() -> Self {
        let bits_per_entry = 2;
        let entries_per_word = 64 / bits_per_entry as usize;
        let block_count = CHUNK_SIZE.pow(3) as usize;

        Self {
            generated: false,
            meshed: false,
            needs_remesh: false,
            all_air: true,
            palette: vec![BlockState::Air],
            bits_per_entry,
            blocks: vec![0; block_count.div_ceil(entries_per_word)],
            mesh: Mesh::new(),
            was_edge: false,
        }
    }


    fn get_word_and_bit_offset(&self, block_index: usize) -> (usize, usize) {
        let entries_per_word = 64 / self.bits_per_entry as usize;

        let word_index = block_index / entries_per_word;
        let entry_index = block_index % entries_per_word;

        let bit_offset = entry_index * self.bits_per_entry as usize;

        (word_index, bit_offset)
    }

    pub fn index(pos: IVec3) -> usize {
        (pos.x + CHUNK_SIZE * (pos.y + CHUNK_SIZE * pos.z)) as usize
    }

    pub fn get_block_at_index(&self, index: usize) -> &BlockState {
        let (word_index, bit_offset) = self.get_word_and_bit_offset(index);

        let mask = (1u64 << self.bits_per_entry) - 1;

        let palette_index = (self.blocks[word_index] >> bit_offset) & mask;

        &self.palette[palette_index as usize]
    }

    pub fn get_block_at(&self, pos: IVec3) -> &BlockState {
        let block_index = Self::index(pos);
        self.get_block_at_index(block_index)
    }

    pub fn set_block_at(&mut self, pos: IVec3, block: BlockState) {
        let block_index = Self::index(pos);

        let palette_index = match self.palette.iter().position(|b| *b == block) {
            Some(index) => index,
            None => {
                self.palette.push(block);
                self.palette.len() - 1
            }
        };

        let new_bits = (usize::BITS - (self.palette.len() - 1).leading_zeros())
            .max(1) as u8;

        if new_bits > self.bits_per_entry {
            let old_bits = self.bits_per_entry;
            let old_entries_per_word = 64 / old_bits as usize;
            let new_entries_per_word = 64 / new_bits as usize;

            let block_count = CHUNK_SIZE.pow(3) as usize;
            let new_word_count = block_count.div_ceil(new_entries_per_word);

            let mut new_blocks = vec![0u64; new_word_count];

            for i in 0..block_count {
                let old_word = i / old_entries_per_word;
                let old_entry = i % old_entries_per_word;
                let old_offset = old_entry * old_bits as usize;
                let old_mask = (1u64 << old_bits) - 1;

                let index = (self.blocks[old_word] >> old_offset) & old_mask;

                let new_word = i / new_entries_per_word;
                let new_entry = i % new_entries_per_word;
                let new_offset = new_entry * new_bits as usize;

                new_blocks[new_word] |= index << new_offset;
            }

            self.blocks = new_blocks;
            self.bits_per_entry = new_bits;
        }

        let entries_per_word = 64 / self.bits_per_entry as usize;

        let word_index = block_index / entries_per_word;
        let entry_index = block_index % entries_per_word;
        let bit_offset = entry_index * self.bits_per_entry as usize;

        let mask = (1u64 << self.bits_per_entry) - 1;

        self.blocks[word_index] &= !(mask << bit_offset);
        self.blocks[word_index] |= (palette_index as u64) << bit_offset;
    }
}

pub fn gen_chunk_mesh(chunk_pos: IVec3, chunk: &Chunk, neighbours: HashMap<Direction, Chunk>, atlas: &TextureAtlas) -> Mesh {
    let mut mesh = Mesh::new();

    let size = CHUNK_SIZE as usize;
    let chunk_origin = chunk_pos * CHUNK_SIZE;

    for index in 0..CHUNK_SIZE.pow(3) as usize {
        let block = chunk.get_block_at_index(index);
        if matches!(block, BlockState::Air) { continue; }

        let x = index % size;
        let y = (index / size) % size;
        let z = index / (size * size);

        let local_pos = IVec3::new(
            x as i32,
            y as i32,
            z as i32,
        );

        let mut cull_faces: HashSet<Direction> = HashSet::new();

        for d in Direction::all() {
            let neighbour = local_pos + d.rel_vec();

            if neighbour.x >= 0 && neighbour.x < CHUNK_SIZE &&
               neighbour.y >= 0 && neighbour.y < CHUNK_SIZE &&
               neighbour.z >= 0 && neighbour.z < CHUNK_SIZE {
                if chunk.get_block_at(neighbour).get_block().is_solid() {
                    cull_faces.insert(d);
                }
            } else {
                let neighbour_block = if let Some(neighbour_chunk) = neighbours.get(&d) {
                    match d {
                        Direction::Up => {
                            neighbour_chunk.get_block_at(
                                IVec3::new(
                                    local_pos.x,
                                    0,
                                    local_pos.z,
                                )
                            )
                        },
                        Direction::Down => {
                            neighbour_chunk.get_block_at(
                                IVec3::new(
                                    local_pos.x,
                                    CHUNK_SIZE - 1,
                                    local_pos.z,
                                )
                            )
                        },
                        Direction::North => {
                            neighbour_chunk.get_block_at(
                                IVec3::new(
                                    local_pos.x,
                                    local_pos.y,
                                    CHUNK_SIZE - 1,
                                )
                            )
                        },
                        Direction::South => {
                            neighbour_chunk.get_block_at(
                                IVec3::new(
                                    local_pos.x,
                                    local_pos.y,
                                    0,
                                )
                            )
                        },
                        Direction::East => {
                            neighbour_chunk.get_block_at(
                                IVec3::new(
                                    0,
                                    local_pos.y,
                                    local_pos.z,
                                )
                            )
                        },
                        Direction::West => {
                            neighbour_chunk.get_block_at(
                                IVec3::new(
                                    CHUNK_SIZE - 1,
                                    local_pos.y,
                                    local_pos.z,
                                )
                            )
                        },
                    }
                } else {
                    &BlockState::Stone
                };

                if neighbour_block.get_block().is_solid() {
                    cull_faces.insert(d);
                }
            }
        }

        if cull_faces.len() == 6 { continue; }

        let model = block.get_block().get_model(block);
        model.add_own_mesh_to_mesh(&mut mesh, cull_faces, (chunk_origin + local_pos).as_vec3(), atlas);
    }

    mesh
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_storage_is_packed_and_preserves_blocks_across_words() {
        let mut chunk = Chunk::new();
        let last = CHUNK_SIZE - 1;

        assert_eq!(chunk.blocks.len(), (CHUNK_SIZE.pow(3) as usize).div_ceil(32));

        chunk.set_block_at(IVec3::new(0, 0, 0), BlockState::Stone);
        chunk.set_block_at(IVec3::new(last, 0, 0), BlockState::Dirt);
        chunk.set_block_at(IVec3::new(0, 1, 0), BlockState::Grass);
        chunk.set_block_at(IVec3::new(last, last, last), BlockState::Stone);

        assert_eq!(chunk.get_block_at_index(0), &BlockState::Stone);
        assert_eq!(chunk.get_block_at_index(last as usize), &BlockState::Dirt);
        assert_eq!(chunk.get_block_at_index(CHUNK_SIZE as usize), &BlockState::Grass);
        assert_eq!(
            chunk.get_block_at_index(CHUNK_SIZE.pow(3) as usize - 1),
            &BlockState::Stone
        );
    }
}