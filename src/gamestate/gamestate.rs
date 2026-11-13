use std::backtrace::Backtrace;
use std::collections::{HashSet, VecDeque};
use std::ops::Index;
use std::panic;
use std::sync::Arc;
use std::{f32::consts::PI, time::Instant};
use crossbeam_channel::{Receiver, RecvTimeoutError, Sender};
use egui::mutex::Mutex;
use winit::{
    event_loop::ActiveEventLoop, keyboard::KeyCode
};
use glam::{IVec3, Vec3};
use std::collections::HashMap;

use crate::camera::camera::Camera;
use crate::app::input::InputManager;
use crate::chunk::job::JobType::{GenMesh, GenTerrain};
use crate::chunk::{self, chunk::*};
use crate::chunk::job::*;
use crate::chunk::jobmanager::ChunkJobManager;
use crate::renderer::renderer::Renderer;
use crate::texture::texture::TextureAtlas;
use crate::utils::direction::{self, Direction};
use crate::worldgen::worldgenerator::WorldGeneratorSettings;

const MAX_GEN_IN_FLIGHT: usize = 16;
const MAX_MESH_IN_FLIGHT: usize = 16;

pub fn pos_in_range(pos: &IVec3, min: &IVec3, max: &IVec3) -> bool {
    pos.x >= min.x && pos.x <= max.x &&
    pos.y >= min.y && pos.y <= max.y &&
    pos.z >= min.z && pos.z <= max.z
}

pub struct ChunkManagerRequest {
    lat_render_dist: u32,
    vert_render_dist: u32,
    cam_chunk_pos: IVec3,
    loaded: HashSet<IVec3>,
}

pub struct ChunkManagerResult {
    to_load: Vec<IVec3>,
    to_unload: Vec<IVec3>,
}

pub struct GameState {
    pub camera: Camera,
    last_frame_tick: Instant,

    pub lateral_render_distance: u32,
    pub vertical_render_distance: u32,

    chunks: Arc<Mutex<HashMap<IVec3, Chunk>>>,
    loaded: HashSet<IVec3>,

    pub prev_cam_chunk_pos: IVec3,
    pub needs_chunk_load_refresh: bool,

    request_sender: Sender<ChunkManagerRequest>,
    result_receiver: Receiver<ChunkManagerResult>,
    latest_request_id: u64,

    pub world_generator_settings: WorldGeneratorSettings,
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
            lateral_render_distance: 8,
            vertical_render_distance: 8,
            chunks: Arc::new(Mutex::new(HashMap::new())),
            loaded: HashSet::new(),
            prev_cam_chunk_pos: IVec3::new(0, 0, 0),
            needs_chunk_load_refresh: true,
            request_sender,
            result_receiver,
            latest_request_id: 0,
            world_generator_settings: WorldGeneratorSettings {
                seed: 0,
            }
        };

        Self::setup_chunk_manager_thread(request_receiver, result_sender, &new.world_generator_settings, Arc::clone(&new.chunks), atlas);

        Ok(new)
    }

    pub fn get_chunk_neighbours(chunks: Arc<Mutex<HashMap<IVec3, Chunk>>>, chunk_pos: IVec3) -> HashMap<Direction, Chunk> {
        let mut neighbours = HashMap::new();

        for d in Direction::all() {
            let neighbour_pos = d.rel_vec() + chunk_pos;

            if let Some(c) = chunks.lock().get(&neighbour_pos) {
                neighbours.insert(
                    d,
                    c.clone(),
                );
            }
        }

        neighbours
    }

    pub fn setup_chunk_manager_thread(
            request_receiver: Receiver<ChunkManagerRequest>,
            result_sender: Sender<ChunkManagerResult>,
            world_generator_settings: &WorldGeneratorSettings,
            chunks: Arc<Mutex<HashMap<IVec3, Chunk>>>,
            atlas: &TextureAtlas,
        ) {
            let atlas = atlas.clone();
            let world_generator_settings = world_generator_settings.clone();

            let default_hook = panic::take_hook();
            panic::set_hook(Box::new(move |info| {
                default_hook(info);
                eprintln!("Backtrace:\n{}", Backtrace::force_capture());
            }));

            std::thread::spawn(move || {
                let mut active_request = false;

                let mut active_request_id = 0;

                let mut cam_chunk_pos: IVec3;

                let mut min: IVec3 = IVec3::new(0, 0, 0);
                let mut max: IVec3 = IVec3::new(0, 0, 0);

                let mut loaded: HashSet<IVec3> = HashSet::new();

                let mut visited: HashSet<IVec3> = HashSet::new();
                let mut frontier: HashSet<IVec3> = HashSet::new();

                let mut to_load: Vec<IVec3> = Vec::new();
                let mut to_unload: Vec<IVec3> = Vec::new();
                let mut to_remesh: VecDeque<IVec3> = VecDeque::new();

                let mut to_submit: VecDeque<IVec3> = VecDeque::new();
                let mut submitted: HashSet<IVec3> = HashSet::new();
                let mut mesh_submitted: HashSet<IVec3> = HashSet::new();

                let chunk_job_manager = ChunkJobManager::new(
                    8,
                    world_generator_settings,
                    Arc::clone(&chunks),
                    &atlas
                );

                loop {
                    match request_receiver.try_iter().last() {
                        Some(request) => {
                            // Reset the area loading

                            active_request_id += 1;

                            chunk_job_manager.set_request_id(active_request_id);

                            submitted = HashSet::new();
                            mesh_submitted = HashSet::new();

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

                            loaded = request.loaded;

                            visited = HashSet::new();
                            frontier = HashSet::new();
                            frontier.insert(cam_chunk_pos);

                            to_load = Vec::new();
                            to_unload = Vec::new();

                            to_submit = VecDeque::new();

                            for p in &loaded {
                                if !pos_in_range(p, &min, &max) {
                                    to_unload.push(*p);
                                }
                            }

                            active_request = true;
                        },
                        None => {
                            // Contunue the area loading as normal

                            if !active_request { continue; }

                            let mut new_frontier: HashSet<IVec3> = HashSet::new();

                            for p in frontier.drain() {
                                if visited.contains(&p) || !pos_in_range(&p, &min, &max) { continue; }

                                if !loaded.contains(&p) && !to_unload.contains(&p) {
                                    if let Some(c) = chunks.lock().get(&p) {
                                        if !loaded.contains(&p) && c.meshed {
                                            loaded.insert(p);
                                            to_load.push(p);
                                        }
                                    } else {
                                        to_submit.push_back(p);
                                    }
                                }

                                for d in Direction::all() {
                                    let neighbour = d.rel_vec() + p;

                                    visited.insert(p);
                                    new_frontier.insert(neighbour);
                                }
                            }

                            frontier = new_frontier;

                            if let Some(result) = chunk_job_manager.get_result() {
                                match result.type_ {
                                    JobType::GenMesh => {
                                        mesh_submitted.remove(&result.position);
                                        loaded.insert(result.position);
                                        to_load.push(result.position);
                                        chunks.lock().insert(result.position, result.chunk);
                                    },
                                    JobType::GenTerrain => {
                                        submitted.remove(&result.position);
                                        
                                        chunks.lock().insert(result.position, result.chunk);
                                        to_remesh.push_back(result.position);

                                        for d in Direction::all() {
                                            let n = result.position + d.rel_vec();

                                            if let Some(c) = chunks.lock().get_mut(&n) {
                                                if c.was_edge {
                                                    c.was_edge = false;
                                                    to_remesh.push_back(n);
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            if to_load.len() > 0 || to_unload.len() > 0 {
                                match result_sender.send(
                                    ChunkManagerResult {
                                        to_load,
                                        to_unload,
                                    }
                                ) {
                                    Err(e) => eprintln!("Failed to send generation result: {}", e),
                                    _ => {},
                                }

                                to_load = Vec::new();
                                to_unload = Vec::new();
                            }

                            if submitted.len() < MAX_GEN_IN_FLIGHT {
                                if let Some(p) = to_submit.pop_front() {
                                    if !submitted.contains(&p) {
                                        submitted.insert(p);
                                        chunk_job_manager.submit_job(
                                            ChunkJob {
                                                request_id: active_request_id,
                                                type_: GenTerrain,
                                                position: p,
                                                chunk: Chunk::new(),
                                                neighbours: HashMap::new(),
                                            }
                                        );
                                    }
                                }
                            }

                            if mesh_submitted.len() < MAX_MESH_IN_FLIGHT {
                                if let Some(&p) = to_remesh.front() {
                                    let mut neighbours_ready = true;
                                    let mut on_edge = false;
                                    for d in Direction::all() {
                                        let n = p + d.rel_vec();
                                        let n_out_of_range = !pos_in_range(&n, &min, &max);

                                        if n_out_of_range {
                                            on_edge = true;
                                        }

                                        if let Some(c) = chunks.lock().get(&n) {
                                            if !c.generated {
                                                neighbours_ready = false;
                                            }
                                        } else if !n_out_of_range {
                                            neighbours_ready = false;
                                        }
                                    }

                                    if neighbours_ready {
                                        to_remesh.pop_front();
                                        let neighbours = Self::get_chunk_neighbours(Arc::clone(&chunks), p);
                                        let mut chunk = chunks.lock().get(&p).unwrap().clone();

                                        chunk.was_edge = on_edge;

                                        mesh_submitted.insert(p);
                                        chunk_job_manager.submit_job(ChunkJob {
                                            request_id: active_request_id,
                                            type_: GenMesh,
                                            chunk,
                                            position: p,
                                            neighbours,
                                        });
                                    }
                                }
                            }
                        },
                    }
                }
            });
        }

    pub fn game_tick(&mut self) {

    }

    fn handle_movement(&mut self, input_man: &InputManager, dt: f32) {
        let move_speed = 25.0;
        
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
            self.latest_request_id += 1;
            self.request_sender.send(
                ChunkManagerRequest {
                    lat_render_dist: self.lateral_render_distance,
                    vert_render_dist: self.vertical_render_distance,
                    cam_chunk_pos,
                    loaded: self.loaded.clone(),
                }
            ).unwrap();

            self.needs_chunk_load_refresh = false;
        }

        for _ in 0..64 {
            match self.result_receiver.try_recv() {
                Ok(result) => {
                    for p in result.to_load {
                        renderer.upload_chunk_mesh(
                            p, 
                            &self.chunks.lock().get(&p).unwrap().mesh,
                            true,
                        );
                        self.loaded.insert(p);
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