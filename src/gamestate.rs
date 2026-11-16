use crossbeam_channel::{Receiver, Sender};
use egui::mutex::Mutex;
use glam::{IVec3, Vec3};
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use std::{f32::consts::PI, time::Instant};
use winit::{event_loop::ActiveEventLoop, keyboard::KeyCode};

use crate::application::input::InputManager;
use crate::camera::Camera;
use crate::chunkdata::chunk::*;
use crate::chunkdata::chunkmanager::*;
use crate::render::renderer::Renderer;
use crate::texture::TextureAtlas;
use crate::worldgen::worldgenerator::WorldGeneratorSettings;

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

    pub camera_fly_speed: f64,
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
            lateral_render_distance: 12,
            vertical_render_distance: 12,
            chunks: Arc::new(Mutex::new(HashMap::new())),
            loaded: HashSet::new(),
            prev_cam_chunk_pos: IVec3::new(0, 0, 0),
            needs_chunk_load_refresh: true,
            request_sender,
            result_receiver,
            latest_request_id: 0,
            world_generator_settings: WorldGeneratorSettings { seed: 0 },
            camera_fly_speed: 32.0,
        };

        setup_chunk_manager_thread(
            request_receiver,
            result_sender,
            &new.world_generator_settings,
            Arc::clone(&new.chunks),
            atlas,
        );

        Ok(new)
    }

    pub fn game_tick(&mut self) {}

    fn handle_movement(&mut self, input_man: &InputManager, dt: f32, fly_speed: f64) {
        if input_man.is_pressed(KeyCode::ArrowLeft) {
            self.camera.yaw += PI / 2.0 * dt;
        }
        if input_man.is_pressed(KeyCode::ArrowRight) {
            self.camera.yaw -= PI / 2.0 * dt;
        }

        if input_man.is_pressed(KeyCode::ArrowDown) {
            self.camera.pitch -= PI / 2.0 * dt;

            if self.camera.pitch <= -90_f32.to_radians() {
                self.camera.pitch = -90_f32.to_radians();
            }
        }
        if input_man.is_pressed(KeyCode::ArrowUp) {
            self.camera.pitch += PI / 2.0 * dt;

            if self.camera.pitch >= 90_f32.to_radians() {
                self.camera.pitch = 90_f32.to_radians();
            }
        }

        if input_man.is_pressed(KeyCode::KeyW) {
            self.camera.position.x += self.camera.yaw.sin() * fly_speed as f32 * dt;
            self.camera.position.z += self.camera.yaw.cos() * fly_speed as f32 * dt;
        }
        if input_man.is_pressed(KeyCode::KeyS) {
            self.camera.position.x -= self.camera.yaw.sin() * fly_speed as f32 * dt;
            self.camera.position.z -= self.camera.yaw.cos() * fly_speed as f32 * dt;
        }

        if input_man.is_pressed(KeyCode::KeyA) {
            self.camera.position.x += (self.camera.yaw + PI / 2.0).sin() * fly_speed as f32 * dt;
            self.camera.position.z += (self.camera.yaw + PI / 2.0).cos() * fly_speed as f32 * dt;
        }
        if input_man.is_pressed(KeyCode::KeyD) {
            self.camera.position.x -= (self.camera.yaw + PI / 2.0).sin() * fly_speed as f32 * dt;
            self.camera.position.z -= (self.camera.yaw + PI / 2.0).cos() * fly_speed as f32 * dt;
        }

        if input_man.is_pressed(KeyCode::Space) {
            self.camera.position.y += fly_speed as f32 * dt;
        }
        if input_man.is_pressed(KeyCode::ShiftLeft) {
            self.camera.position.y -= fly_speed as f32 * dt;
        }
    }

    pub fn frame_tick(&mut self, input_man: &InputManager, renderer: &mut Renderer) {
        let now = Instant::now();
        let dt = now.duration_since(self.last_frame_tick).as_secs_f32() / 1.0;
        self.last_frame_tick = now;

        self.handle_movement(input_man, dt, self.camera_fly_speed);

        let cam_chunk_pos = (self.camera.position
            / Vec3::new(CHUNK_SIZE as f32, CHUNK_SIZE as f32, CHUNK_SIZE as f32))
        .as_ivec3();

        if cam_chunk_pos != self.prev_cam_chunk_pos {
            self.prev_cam_chunk_pos = cam_chunk_pos;
            self.needs_chunk_load_refresh = true;
        }

        if self.needs_chunk_load_refresh {
            self.latest_request_id += 1;
            self.request_sender
                .send(ChunkManagerRequest {
                    lat_render_dist: self.lateral_render_distance,
                    vert_render_dist: self.vertical_render_distance,
                    cam_chunk_pos,
                    loaded: self.loaded.clone(),
                })
                .unwrap();

            self.needs_chunk_load_refresh = false;
        }

        for _ in 0..16 {
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
                }
                Err(_) => {
                    break;
                }
            }
        }
    }

    pub fn handle_key(&self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        if let (KeyCode::Escape, true) = (code, is_pressed) {
            event_loop.exit();
        }
    }
}
