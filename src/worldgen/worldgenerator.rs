use std::collections::HashMap;

use glam::{IVec2, IVec3};
use noise::{NoiseFn, Perlin, Worley, core::worley::ReturnType};
use wgpu::naga::Block;

use crate::{block::{self, blockstate::BlockState}, chunk::{self, chunk::CHUNK_SIZE}};

#[derive(Clone)]
pub struct WorldGeneratorSettings {
    pub seed: u32,
}

pub struct WorldGenerator {
    settings: WorldGeneratorSettings,
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

        let shape = self.perlin.get([
            x * 0.008,
            y * 0.008,
            z * 0.008,
        ]);

        let mut weirdness = 0.0;

        for i in 1..8 {
            let octave = i as f64;
            weirdness += self.perlin.get([
                x * 0.025 / octave,
                y * 0.025 / octave,
                z * 0.025 / octave,
            ]) / octave;
        }

        let surface = shape * 32.0 + weirdness * 32.0;
        let noise = y - surface;

        self.terrain_noise_cache.insert(block_pos, noise);

        noise
    }

    pub fn get_block(&mut self, block_world_pos: IVec3, chunk_pos: IVec3) -> BlockState {
        let cave_point = [
            block_world_pos.x as f64 * 0.01,
            block_world_pos.y as f64 * 0.01,
            block_world_pos.z as f64 * 0.01,
        ];

        let in_cave =
            self.cave_a.get(cave_point).abs() < 0.03 &&
            self.cave_b.get(cave_point).abs() < 0.03;
        
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
                    )) < 0.0 {
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