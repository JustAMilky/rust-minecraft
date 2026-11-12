use std::collections::HashMap;

use glam::{IVec2, IVec3};
use noise::{NoiseFn, Perlin};

use crate::block::blockstate::BlockState;

#[derive(Clone)]
pub struct WorldGeneratorSettings {
    pub seed: u32,
}

pub struct WorldGenerator {
    settings: WorldGeneratorSettings,
    perlin: Perlin,
    cave_a: Perlin,
    cave_b: Perlin,
    terrain_height_cache: HashMap<IVec2, i32>,
}

impl WorldGenerator {
    pub fn new(settings: &WorldGeneratorSettings) -> Self {
        Self {
            settings: settings.clone(),
            perlin: Perlin::new(settings.seed),
            cave_a: Perlin::new(settings.seed.wrapping_add(1)),
            cave_b: Perlin::new(settings.seed.wrapping_add(2)),
            terrain_height_cache: HashMap::new(),
        }
    }

    pub fn get_terrain_height(&mut self, pos: IVec2) -> i32 {
        match self.terrain_height_cache.get(&pos) {
            Some(h) => *h,
            None => {
                let terrain_height = (self.perlin.get([
                    pos.x as f64 * 0.01,
                    pos.y as f64 * 0.01,
                ]) * 16.0) as i32;

                self.terrain_height_cache.insert(
                    IVec2::new(pos.x, pos.y),
                    terrain_height,
                );

                terrain_height
            },
        }
    }

    pub fn get_block(&mut self, block_world_pos: IVec3) -> BlockState {
        let terrain_height = self.get_terrain_height(IVec2::new(block_world_pos.x, block_world_pos.z));

        let cave_point = [
            block_world_pos.x as f64 * 0.02,
            block_world_pos.y as f64 * 0.02,
            block_world_pos.z as f64 * 0.02,
        ];

        let in_cave = self.cave_a.get(cave_point).abs() < 0.05
            && self.cave_b.get(cave_point).abs() < 0.05;

        if in_cave
        {
            BlockState::Air
        } else if block_world_pos.y > terrain_height {
            BlockState::Air
        } else if block_world_pos.y > terrain_height - 1 {
            BlockState::Grass
        } else if block_world_pos.y > terrain_height - 5 {
            BlockState::Dirt
        } else {
            BlockState::Stone
        }
    }
}