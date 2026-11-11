use std::cmp::Reverse;
use std::collections::HashSet;
use std::time::Duration;
use std::{f32::consts::PI, time::Instant};
use crossbeam_channel::{Receiver, RecvTimeoutError, Sender};
use egui::MouseWheelSource::Unknown;
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
use crate::renderer::mesh::Mesh;
use crate::renderer::renderer::Renderer;
use crate::texture::texture::TextureAtlas;
use crate::utils::direction::Direction;

pub struct ChunkManagerRequest {
    lat_render_dist: u32,
    vert_render_dist: u32,
    cam_chunk_pos: IVec3,
    chunks: HashSet<IVec3>,
    loaded: HashSet<IVec3>,
}

pub struct ChunkManagerResult {
    position: IVec3,
    chunk: Option<Chunk>,
    to_load: Vec<IVec3>,
    to_unload: Vec<IVec3>,
}

pub struct GameState {
    pub camera: Camera,
    last_frame_tick: Instant,

    pub lateral_render_distance: u32,
    pub vertical_render_distance: u32,

    chunks: HashMap<IVec3, Chunk>,
    loaded: HashSet<IVec3>,

    pub prev_cam_chunk_pos: IVec3,
    pub needs_chunk_load_refresh: bool,

    request_sender: Sender<ChunkManagerRequest>,
    result_receiver: Receiver<ChunkManagerResult>
}

impl GameState {
    pub async fn new(atlas: &TextureAtlas) -> anyhow::Result<Self> {
        let (request_sender, request_receiver) = crossbeam_channel::unbounded();
        let (result_sender, result_receiver) = crossbeam_channel::unbounded();

        let new = Self {
            camera: Camera {
                position: Vec3::new(0.0, 16.0, 0.0),
                yaw: 0.0,
                pitch: 0.0,
                fov: 60.0_f32.to_radians(),
                aspect_ratio: 16.0 / 9.0,
            },
            last_frame_tick: Instant::now(),
            lateral_render_distance: 4,
            vertical_render_distance: 3,
            chunks: HashMap::new(),
            loaded: HashSet::new(),
            prev_cam_chunk_pos: IVec3::new(0, 0, 0),
            needs_chunk_load_refresh: true,
            request_sender,
            result_receiver,
        };

        Self::setup_chunk_manager_thread(request_receiver, result_sender, atlas);

        Ok(new)
    }

    pub fn setup_chunk_manager_thread(
            request_receiver: Receiver<ChunkManagerRequest>,
            result_sender: Sender<ChunkManagerResult>,
            atlas: &TextureAtlas,
        ) {
            let atlas = atlas.clone();

            std::thread::spawn(move || {
                let mut active_request = false;

                let mut cam_chunk_pos: IVec3;

                let mut min: IVec3 = IVec3::new(0, 0, 0);
                let mut max: IVec3 = IVec3::new(0, 0, 0);

                let mut chunks: HashSet<IVec3> = HashSet::new();
                let mut loaded: HashSet<IVec3> = HashSet::new();

                let mut visited: HashSet<IVec3> = HashSet::new();
                let mut frontier: HashSet<IVec3> = HashSet::new();

                let mut to_load: Vec<IVec3> = Vec::new();
                let mut to_unload: Vec<IVec3>;

                let chunk_job_manager = ChunkJobManager::new(8, &atlas);

                loop {
                    match request_receiver.try_recv() {
                        Ok(request) => {
                            // Reset the area loading

                            cam_chunk_pos = request.cam_chunk_pos;

                            min = IVec3::new(
                                cam_chunk_pos.x - request.lat_render_dist as i32,
                                cam_chunk_pos.y - request.vert_render_dist as i32,
                                cam_chunk_pos.z - request.lat_render_dist as i32,
                            );
                            max = IVec3::new(
                                cam_chunk_pos.x + request.lat_render_dist as i32,
                                cam_chunk_pos.y + request.vert_render_dist as i32,
                                cam_chunk_pos.z + request.lat_render_dist as i32,
                            );

                            chunks = request.chunks;
                            loaded = request.loaded;

                            visited = HashSet::new();
                            frontier = HashSet::new();
                            frontier.insert(cam_chunk_pos);

                            to_load = Vec::new();
                            to_unload = Vec::new();

                            for p in &loaded {
                                if p.x < min.x || p.x > max.x ||
                                   p.y < min.y || p.y > max.y ||
                                   p.z < min.z || p.z > max.z
                                {
                                    to_unload.push(*p);
                                }
                            }

                            result_sender.send(
                                ChunkManagerResult {
                                    position: IVec3::new(0, 0, 0),
                                    chunk: None,
                                    to_load: Vec::new(),
                                    to_unload,
                                }
                            ).expect("Failed to send unload result");

                            active_request = true;
                        },
                        Err(_) => {
                            // Contunue the area loading as normal

                            if !active_request { continue; }

                            let mut new_frontier: HashSet<IVec3> = HashSet::new();

                            for p in frontier.drain() {
                                for d in Direction::all() {
                                    let neighbour = d.rel_vec() + p;

                                    if visited.contains(&neighbour) ||
                                       neighbour.x < min.x || neighbour.x > max.x ||
                                       neighbour.y < min.y || neighbour.y > max.y ||
                                       neighbour.z < min.z || neighbour.z > max.z
                                    { continue; }

                                    if chunks.contains(&neighbour) {
                                        if !loaded.contains(&neighbour) {
                                            loaded.insert(neighbour);
                                            to_load.push(neighbour);
                                        }
                                    } else {
                                        chunk_job_manager.submit_job(
                                            ChunkJob {
                                                type_: JobType::GenTerrain,
                                                position: neighbour,
                                                chunk: Chunk::new(),
                                            }
                                        );
                                    }

                                    visited.insert(p);
                                    new_frontier.insert(neighbour);
                                }
                            }

                            frontier = new_frontier;
                        },
                    }

                    if let Some(result) = chunk_job_manager.get_result() {
                        match result.type_ {
                            JobType::GenMesh => {
                                result_sender.send(
                                    ChunkManagerResult {
                                        position: result.position,
                                        chunk: Some(result.chunk),
                                        to_load,
                                        to_unload: Vec::new(),
                                    }
                                ).expect("Failed to send generation result");
                                to_load = Vec::new();
                            },
                            JobType::GenTerrain => {
                                chunk_job_manager.submit_job(
                                    ChunkJob {
                                        type_: JobType::GenMesh,
                                        position: result.position,
                                        chunk: result.chunk,
                                    }
                                );
                            }
                        }
                    } else {
                        if to_load.len() > 0 {
                            result_sender.send(
                                ChunkManagerResult {
                                    position: IVec3::new(0, 0, 0),
                                    chunk: None,
                                    to_load,
                                    to_unload: Vec::new(),
                                }
                            ).expect("Failed to send loading result");
                            to_load = Vec::new();
                        }
                    }
                }
            });
        }

    pub fn game_tick(&mut self) {

    }

    fn handle_movement(&mut self, input_man: &InputManager, dt: f32) {
        let move_speed = 10.0;
        
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
            self.request_sender.send(
                ChunkManagerRequest {
                    lat_render_dist: self.lateral_render_distance,
                    vert_render_dist: self.vertical_render_distance,
                    cam_chunk_pos,
                    chunks: self.chunks.keys().copied().collect(),
                    loaded: self.loaded.clone(),
                }
            ).unwrap();

            self.needs_chunk_load_refresh = false;
        }

        for _ in 0..512 {
            match self.result_receiver.try_recv() {
                Ok(result) => {
                    match result.chunk {
                        Some(chunk) => {
                            renderer.upload_chunk_mesh(
                                result.position, 
                                &chunk.mesh,
                            );

                            self.chunks.insert(result.position, chunk);
                            self.loaded.insert(result.position);
                        },
                        None => {},
                    }

                    for p in result.to_load {
                        if !self.loaded.contains(&p) {
                            renderer.upload_chunk_mesh(p, &self.chunks.get(&p).expect("Chunk to load not found").mesh);
                            self.loaded.insert(p);
                        }
                    }

                    for p in result.to_unload {
                        if self.loaded.contains(&p) {
                            renderer.remove_chunk_mesh(p);
                            self.loaded.remove(&p);
                        }
                    }
                },
                Err(_) => { break; },
            }
    }
    }

    pub fn handle_key(&self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        match (code, is_pressed) {
            (KeyCode::Escape, true) => event_loop.exit(),
            _ => {}
        }
    }
}