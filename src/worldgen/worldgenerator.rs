use random::Source;
use std::collections::HashMap;

use glam::{IVec2, IVec3};
use noise::{NoiseFn, Perlin};

use crate::{
    blockdata::blockstate::BlockState,
    chunkdata::chunk::{CHUNK_SIZE, Chunk},
    utils::posinrange::pos_in_range,
    worldgen::structure::{
        structure::{Structure, StructureStart},
        structurereg::tree,
    },
};

#[derive(Clone)]
pub struct WorldGeneratorSettings {
    pub seed: u32,
    pub magnitude: f64,
    pub weirdness: f64,
    pub ridges: f64,
}

pub struct WorldGenerator {
    settings: WorldGeneratorSettings,
    random: random::Default,
    perlin: Perlin,
    cave_a: Perlin,
    cave_b: Perlin,
    terrain_noise_cache: HashMap<IVec3, f64>,
    surface_block_cache: HashMap<IVec3, BlockState>,
}

impl WorldGenerator {
    pub fn new(settings: &WorldGeneratorSettings) -> Self {
        Self {
            settings: settings.clone(),
            perlin: Perlin::new(settings.seed),
            random: random::default(settings.seed as u64),
            cave_a: Perlin::new(settings.seed.wrapping_add(1)),
            cave_b: Perlin::new(settings.seed.wrapping_add(2)),
            terrain_noise_cache: HashMap::new(),
            surface_block_cache: HashMap::new(),
        }
    }

    pub fn get_terrain_noise(&mut self, block_pos: IVec3) -> f64 {
        if let Some(noise) = self.terrain_noise_cache.get(&block_pos) {
            return *noise;
        }

        let x = block_pos.x as f64;
        let y = block_pos.y as f64;
        let z = block_pos.z as f64;

        let shape_noise = self.perlin.get([x * 0.008, y * 0.008, z * 0.008]);

        let weirdness_noise = self.perlin.get([x * 0.03, y * 0.03 + 128.0, z * 0.03]);

        let ridges_noise = (1.0
            - self
                .perlin
                .get([x * 0.013, y * 0.013 + 256.0, z * 0.013])
                .abs())
        .powf(0.8);

        let surface = shape_noise * self.settings.magnitude
            + weirdness_noise * self.settings.weirdness
            + ridges_noise * self.settings.ridges;
        let noise = y - surface;

        self.terrain_noise_cache.insert(block_pos, noise);

        noise
    }

    pub fn clear_caches(&mut self) {
        self.terrain_noise_cache.clear();
        self.surface_block_cache.clear();
    }

    pub fn get_surface_height(&mut self, x: i32, z: i32) -> i32 {
        let mut y =
            (self.settings.magnitude + self.settings.weirdness + self.settings.ridges) as i32;

        while self.get_terrain_noise(IVec3::new(x, y, z)) < 0.0 {
            y += 1;
        }

        while self.get_terrain_noise(IVec3::new(x, y - 1, z)) >= 0.0 {
            y -= 1;
        }

        y - 1
    }

    pub fn add_chunk_structure_starts(&mut self, chunk: &mut Chunk, pos: &IVec3) {
        println!("{}", self.random.read_f64());
        /* let world_x = x + pos.x * CHUNK_SIZE;
        let world_z = z + pos.z * CHUNK_SIZE;

        if self.is_tree_pos(IVec2::new(world_x, world_z)) {
            let y = self.get_surface_height(world_x, world_z);

            let tree_pos = IVec3::new(world_x, y + 1, world_z);

            chunk.structure_starts.push(StructureStart::new(
                *pos,
                tree(3, tree_pos).expect("Could not create tree"),
            ));
        } */
    }

    pub fn generate_chunk(&mut self, chunk: &mut Chunk, pos: &IVec3) {
        self.add_chunk_structure_starts(chunk, pos);

        for rel_x in 0..CHUNK_SIZE {
            for rel_y in 0..CHUNK_SIZE {
                for rel_z in 0..CHUNK_SIZE {
                    let rel_pos = IVec3::new(rel_x, rel_y, rel_z);
                    let block_pos = rel_pos + pos * CHUNK_SIZE;

                    let mut block = self.get_terrain_block(block_pos);

                    for s in &chunk.structure_starts {
                        let local_pos = block_pos - s.structure.pos;

                        let new_block = s.structure.get_block(local_pos).clone();
                        if pos_in_range(&local_pos, &s.structure.min, &s.structure.max)
                            && new_block != BlockState::Air
                        {
                            block = new_block;
                        }
                    }

                    if !matches!(block, BlockState::Air) {
                        chunk.set_block_at(rel_pos, block);
                    }
                }
            }
        }
    }

    pub fn get_terrain_block(&mut self, block_world_pos: IVec3) -> BlockState {
        let cave_point = [
            block_world_pos.x as f64 * 0.01,
            block_world_pos.y as f64 * 0.01,
            block_world_pos.z as f64 * 0.01,
        ];

        let in_cave =
            self.cave_a.get(cave_point).abs() < 0.03 && self.cave_b.get(cave_point).abs() < 0.03;

        if in_cave {
            BlockState::Air
        } else {
            if self.get_terrain_noise(block_world_pos) >= 0.0 {
                BlockState::Air
            } else {
                if let Some(block) = self.surface_block_cache.get(&block_world_pos) {
                    block.clone()
                } else {
                    let mut block = BlockState::Stone;

                    if self.get_terrain_noise(IVec3::new(
                        block_world_pos.x,
                        block_world_pos.y + 5,
                        block_world_pos.z,
                    )) < 0.0
                    {
                        block
                    } else {
                        for i in 0..=4 {
                            let pos = IVec3::new(
                                block_world_pos.x,
                                block_world_pos.y + i + 1,
                                block_world_pos.z,
                            );

                            let terrain_noise = self.get_terrain_noise(pos);

                            if terrain_noise >= 0.0 {
                                block = match i {
                                    0 => BlockState::Grass,
                                    1..=4 => BlockState::Dirt,
                                    _ => unreachable!(),
                                };
                                self.surface_block_cache.insert(pos, block.clone());
                                break;
                            }
                        }

                        block
                    }
                }
            }
        }
    }
}
