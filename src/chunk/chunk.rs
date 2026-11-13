use std::collections::{HashMap, HashSet};

use glam::IVec3;
use crate::{block::blockstate::BlockState, renderer::mesh::Mesh, texture::texture::TextureAtlas, utils::direction::Direction};

pub const CHUNK_SIZE: i32 = 16;

#[derive(Clone)]
pub struct Chunk {
    pub generated: bool,
    pub meshed: bool,
    pub needs_remesh: bool,
    pub was_edge: bool,

    pub all_air: bool,

    blocks: Vec<BlockState>,
    pub mesh: Mesh,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            generated: false,
            meshed: false,
            needs_remesh: false,
            all_air: true,
            blocks: vec![BlockState::Air; CHUNK_SIZE.pow(3).try_into().unwrap()],
            mesh: Mesh::new(),
            was_edge: false,
        }
    }

    pub fn index(pos: IVec3) -> usize {
        (pos.x + CHUNK_SIZE * (pos.y + CHUNK_SIZE * pos.z)) as usize
    }

    pub fn get_block_at(&self, pos: IVec3) -> BlockState {
        self.blocks[Chunk::index(pos)].clone()
    }

    pub fn set_block_at(&mut self, pos: IVec3, block: BlockState) {
        if pos.x < 0 || pos.x >= CHUNK_SIZE ||
           pos.y < 0 || pos.y >= CHUNK_SIZE ||
           pos.z < 0 || pos.z >= CHUNK_SIZE {
            return;
        }

        let index = Chunk::index(pos);
        let was_air = matches!(&self.blocks[index], BlockState::Air);
        let is_air = matches!(&block, BlockState::Air);
        self.blocks[index] = block;

        if was_air && !is_air {
            self.all_air = false;
        } else if !was_air && is_air {
            self.all_air = self.blocks.iter().all(|b| matches!(b, BlockState::Air));
        }
    }
}

pub fn gen_chunk_mesh(chunk_pos: IVec3, chunk: &Chunk, neighbours: HashMap<Direction, Chunk>, atlas: &TextureAtlas) -> Mesh {
    let mut mesh = Mesh::new();

    let size = CHUNK_SIZE as usize;
    let chunk_origin = chunk_pos * CHUNK_SIZE;

    for (index, block) in chunk.blocks.iter().enumerate() {
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
                    BlockState::Stone
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
    use super::{Chunk, gen_chunk_mesh};
    use crate::block::blockstate::BlockState;
    use crate::texture::texture::{AtlasRegion, TextureAtlas, TextureId};
    use glam::IVec3;
    use std::collections::HashMap;

    #[test]
    fn all_air_tracks_block_updates() {
        let mut chunk = Chunk::new();
        let first = IVec3::new(0, 0, 0);
        let second = IVec3::new(1, 0, 0);

        assert!(chunk.all_air);

        chunk.set_block_at(first, BlockState::Stone);
        chunk.set_block_at(second, BlockState::Stone);
        assert!(!chunk.all_air);

        chunk.set_block_at(first, BlockState::Air);
        assert!(!chunk.all_air);

        chunk.set_block_at(second, BlockState::Air);
        assert!(chunk.all_air);
    }

    #[test]
    fn adjacent_blocks_have_unit_scale_and_keep_all_faces() {
        let mut chunk = Chunk::new();
        chunk.set_block_at(IVec3::new(0, 0, 0), BlockState::Stone);
        chunk.set_block_at(IVec3::new(1, 0, 0), BlockState::Stone);
        let atlas = TextureAtlas {
            width: 16,
            height: 16,
            pixels: vec![0; 16 * 16 * 4],
            regions: HashMap::from([(
                TextureId::Stone,
                AtlasRegion {
                    min: [0.0, 0.0],
                    max: [16.0, 16.0],
                },
            )]),
        };

        let mesh = gen_chunk_mesh(IVec3::ZERO, &chunk, HashMap::new(), &atlas);

        assert_eq!(mesh.vertices.len(), 48);
        assert_eq!(mesh.indicies.len(), 72);
        assert_eq!(
            mesh.vertices
                .iter()
                .map(|vertex| vertex.position[0])
                .fold((f32::INFINITY, f32::NEG_INFINITY), |(min, max), x| {
                    (min.min(x), max.max(x))
                }),
            (0.0, 2.0)
        );
        assert!(mesh.vertices.iter().all(|vertex| {
            (0.0..=1.0).contains(&vertex.position[1])
                && (0.0..=1.0).contains(&vertex.position[2])
        }));
    }

    #[test]
    fn air_blocks_generate_no_geometry() {
        let chunk = Chunk::new();
        let atlas = TextureAtlas {
            width: 1,
            height: 1,
            pixels: vec![0; 4],
            regions: HashMap::new(),
        };

        let mesh = gen_chunk_mesh(IVec3::ZERO, &chunk, HashMap::new(), &atlas);

        assert!(mesh.vertices.is_empty());
        assert!(mesh.indicies.is_empty());
    }

}