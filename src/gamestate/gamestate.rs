use std::{f32::consts::PI, time::Instant};
use winit::{
    event_loop::ActiveEventLoop, keyboard::KeyCode
};
use glam::{IVec3, Vec3};
use std::collections::HashMap;

use crate::camera::camera::Camera;
use crate::app::input::InputManager;
use crate::chunk::chunk::*;
use crate::chunk::jobmanager::ChunkJobManager;

pub struct GameState {
    pub camera: Camera,
    last_frame_tick: Instant,

    pub chunks: HashMap<IVec3, Chunk>,
    chunk_job_manager: ChunkJobManager,
}

impl GameState {
    pub async fn new() -> anyhow::Result<Self> {
        Ok(
            Self {
                camera: Camera {
                    position: Vec3::new(0.0, 0.0, -5.0),
                    yaw: 0.0,
                    pitch: 0.0,
                    fov: 60.0_f32.to_radians(),
                    aspect_ratio: 16.0 / 9.0,
                },
                last_frame_tick: Instant::now(),
                chunks: HashMap::new(),
                chunk_job_manager: ChunkJobManager::new(8),
            }
        )
    }

    pub fn game_tick(&mut self) {

    }

    pub fn frame_tick(&mut self, input_man: &InputManager) {
        let now = Instant::now();
        let dt = now.duration_since(self.last_frame_tick).as_secs_f32() / 1.0;
        self.last_frame_tick = now;

        let move_speed = 3.0;
        
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

    pub fn handle_key(&self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        match (code, is_pressed) {
            (KeyCode::Escape, true) => event_loop.exit(),
            _ => {}
        }
    }
}