use std::cmp::Reverse;
use std::collections::HashSet;
use std::{f32::consts::PI, time::Instant};
use winit::{
    event_loop::ActiveEventLoop, keyboard::KeyCode
};
use glam::{IVec3, Vec3};
use std::collections::HashMap;

use crate::camera::camera::Camera;
use crate::app::input::InputManager;
use crate::chunk::{self, chunk::*};
use crate::chunk::job::*;
use crate::chunk::jobmanager::ChunkJobManager;
use crate::renderer::renderer::Renderer;
use crate::texture::texture::TextureAtlas;

const MAX_RESULTS_PER_FRAME: usize = 32;
const MAX_LOADS_PER_FRAME: usize = 16;
const MAX_UNLOADS_PER_FRAME: usize = 16;

pub struct GameState {
    pub camera: Camera,
    last_frame_tick: Instant,

    pub lateral_render_distance: u32,
    pub vertical_render_distance: u32,

    pub chunks: HashMap<IVec3, Chunk>,
    chunk_job_manager: ChunkJobManager,
    awaiting_terrain_gen: HashSet<IVec3>,
    awaiting_mesh_gen: HashSet<IVec3>,
    chunks_to_load: Vec<IVec3>,
    chunks_to_unload: Vec<IVec3>,

    pub prev_cam_chunk_pos: IVec3,
    pub needs_chunk_load_refresh: bool,
}

impl GameState {
    pub async fn new(atlas: &TextureAtlas) -> anyhow::Result<Self> {
        Ok(
            Self {
                camera: Camera {
                    position: Vec3::new(0.0, 16.0, 0.0),
                    yaw: 0.0,
                    pitch: 0.0,
                    fov: 60.0_f32.to_radians(),
                    aspect_ratio: 16.0 / 9.0,
                },
                last_frame_tick: Instant::now(),
                lateral_render_distance: 8,
                vertical_render_distance: 6,
                chunks: HashMap::new(),
                chunk_job_manager: ChunkJobManager::new(64, atlas),
                awaiting_terrain_gen: HashSet::new(),
                awaiting_mesh_gen: HashSet::new(),
                chunks_to_load: Vec::new(),
                chunks_to_unload: Vec::new(),
                prev_cam_chunk_pos: IVec3::new(0, 0, 0),
                needs_chunk_load_refresh: true,
            }
        )
    }

    pub fn game_tick(&mut self) {

    }

    fn handle_movement(&mut self, input_man: &InputManager, dt: f32) {
        let move_speed = 8.0;
        
        if input_man.is_pressed(KeyCode::ArrowLeft) {
            self.camera.yaw += PI / 2.0 * dt;
        }
        if input_man.is_pressed(KeyCode::ArrowRight) {
            self.camera.yaw -= PI / 2.0 * dt;
        }

        if input_man.is_pressed(KeyCode::ArrowDown) {
            self.camera.pitch -= PI / 2.0 * dt;
        }
        if input_man.is_pressed(KeyCode::ArrowUp) {
            self.camera.pitch += PI / 2.0 * dt;
        }

        if input_man.is_pressed(KeyCode::KeyW) {
            self.camera.position.x += self.camera.yaw.sin() * move_speed * dt;
            self.camera.position.z += self.camera.yaw.cos() * move_speed * dt;
        }
        if input_man.is_pressed(KeyCode::KeyS) {
            self.camera.position.x -= self.camera.yaw.sin() * move_speed * dt;
            self.camera.position.z -= self.camera.yaw.cos() * move_speed * dt;
        }

        if input_man.is_pressed(KeyCode::KeyA) {
            self.camera.position.x += (self.camera.yaw + PI / 2.0).sin() * move_speed * dt;
            self.camera.position.z += (self.camera.yaw + PI / 2.0).cos() * move_speed * dt;
        }
        if input_man.is_pressed(KeyCode::KeyD) {
            self.camera.position.x -= (self.camera.yaw + PI / 2.0).sin() * move_speed * dt;
            self.camera.position.z -= (self.camera.yaw + PI / 2.0).cos() * move_speed * dt;
        }

        if input_man.is_pressed(KeyCode::Space) {
            self.camera.position.y += move_speed * dt;
        }
        if input_man.is_pressed(KeyCode::ShiftLeft) {
            self.camera.position.y -= move_speed * dt;
        }
    }

    // TODO: handle this in a separate thread to remove stutters
    fn chunk_load_refresh(&mut self, cam_chunk_pos: IVec3) {
        let min = IVec3::new(
            cam_chunk_pos.x - self.lateral_render_distance as i32,
            cam_chunk_pos.y - self.vertical_render_distance as i32,
            cam_chunk_pos.z - self.lateral_render_distance as i32,
        );
        let max = IVec3::new(
            cam_chunk_pos.x + self.lateral_render_distance as i32,
            cam_chunk_pos.y + self.vertical_render_distance as i32,
            cam_chunk_pos.z + self.lateral_render_distance as i32,
        );

        let mut chunk_positions: Vec<_> = (min.x..=max.x)
            .flat_map(|x| (min.y..=max.y).flat_map(move |y| (min.z..=max.z).map(move |z| IVec3::new(x, y, z))))
            .collect();

        for p in self.chunks.keys() {
            if chunk_positions.contains(p) ||
               self.chunks_to_unload.contains(p) ||
               !self.chunks.get(p).unwrap().loaded {
                continue;
            }

            self.chunks_to_unload.push(p.clone());
        }

        chunk_positions.sort_by_key(|pos| {
            let delta = *pos - cam_chunk_pos;
            Reverse(delta.x * delta.x + delta.y * delta.y + delta.z * delta.z)
        });

        for p in chunk_positions {
            match self.chunks.get(&p) {
                Some(c) => {
                    if !c.loaded && !c.all_air {
                        self.chunks_to_load.push(p);
                    }
                },
                None => {
                    self.chunks_to_load.push(p);
                },
            }
        }
    }

    pub fn frame_tick(&mut self, input_man: &InputManager, renderer: &mut Renderer) {
        let now = Instant::now();
        let dt = now.duration_since(self.last_frame_tick).as_secs_f32() / 1.0;
        self.last_frame_tick = now;

        self.handle_movement(input_man, dt);

        let cam_chunk_pos = (self.camera.position / Vec3::new(CHUNK_SIZE as f32, CHUNK_SIZE as f32, CHUNK_SIZE as f32)).as_ivec3();

        if cam_chunk_pos != self.prev_cam_chunk_pos {
            self.prev_cam_chunk_pos = cam_chunk_pos;
            self.needs_chunk_load_refresh = true;
        }

        if self.needs_chunk_load_refresh {
            println!("Chunk load refresh!");
            self.chunk_load_refresh(cam_chunk_pos);
            self.needs_chunk_load_refresh = false;
        }


        // TODO: use maximum time spent processing this (and things like it) rather than hard cap
        for _ in 0..MAX_LOADS_PER_FRAME {
            if self.chunks_to_load.len() == 0 { break; }

            let chunk_pos = self.chunks_to_load.pop().unwrap();

            if self.chunks.contains_key(&chunk_pos) {
                let chunk = self.chunks.get_mut(&chunk_pos).unwrap();

                if !chunk.loaded {
                    chunk.loaded = true;
                    renderer.upload_chunk_mesh(chunk_pos, &chunk.mesh);
                    if !chunk.meshed || chunk.needs_remesh {
                        self.chunk_job_manager.submit_job(
                            ChunkJob {
                                type_: JobType::GenTerrain,
                                position: chunk_pos,
                                chunk: chunk.clone(),
                            }
                        );
                        self.awaiting_mesh_gen.insert(chunk_pos);
                    }
                }
            } else {
                if !self.awaiting_terrain_gen.contains(&chunk_pos) {
                    self.chunk_job_manager.submit_job(
                        ChunkJob {
                            type_: JobType::GenTerrain,
                            position: chunk_pos,
                            chunk: Chunk::new(),
                        }
                    );
                    self.awaiting_terrain_gen.insert(chunk_pos);
                }
            }
        }

        for _ in 0..MAX_UNLOADS_PER_FRAME {
            if self.chunks_to_unload.len() == 0 { break; }

            let chunk_pos = self.chunks_to_unload.pop().unwrap();

            if let Some(chunk) = self.chunks.get_mut(&chunk_pos) {
                chunk.loaded = false;
            }

            renderer.remove_chunk_mesh(chunk_pos);
        }

        for _ in 0..MAX_RESULTS_PER_FRAME {
            let Some(mut result) = self.chunk_job_manager.get_result() else {
                break;
            };

            match result.type_ {
                JobType::GenTerrain => {
                    self.awaiting_terrain_gen.remove(&result.position);

                    self.chunk_job_manager.submit_job(
                        ChunkJob {
                            type_: JobType::GenMesh,
                            position: result.position,
                            chunk: result.chunk.clone(),
                        }
                    );
                    self.awaiting_mesh_gen.insert(result.position);
                },
                JobType::GenMesh => {
                    self.awaiting_mesh_gen.remove(&result.position);
                    renderer.upload_chunk_mesh(result.position, &result.chunk.mesh);
                },
            }

            result.chunk.loaded = true;
            self.chunks.insert(result.position, result.chunk);
        }
    }

    pub fn handle_key(&self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        match (code, is_pressed) {
            (KeyCode::Escape, true) => event_loop.exit(),
            _ => {}
        }
    }
}