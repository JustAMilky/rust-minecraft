use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
use crossbeam_channel::{Receiver, Sender};
use egui::mutex::Mutex;
use glam::IVec3;

use crate::block::blockstate::BlockState;
use crate::chunk::job::*;
use crate::chunk::chunk::{CHUNK_SIZE, Chunk, gen_chunk_mesh};
use crate::chunk::job::ChunkJobResult;
use crate::texture::texture::TextureAtlas;
use crate::worldgen::worldgenerator::{WorldGenerator, WorldGeneratorSettings};

pub struct ChunkJobManager {
    terrain_job_sender: Sender<ChunkJob>,
    mesh_job_sender: Sender<ChunkJob>,
    result_receiver: Receiver<ChunkJobResult>,
    request_id: Arc<AtomicU64>,
}

impl ChunkJobManager {
    pub fn new(num_terrain_threads: usize, num_mesh_threads: usize, world_generator_settings: WorldGeneratorSettings, chunks: Arc<Mutex<HashMap<IVec3, Chunk>>>, atlas: &TextureAtlas) -> Self {
        let (terrain_job_sender, terrain_job_receiver) = crossbeam_channel::unbounded();
        let (mesh_job_sender, mesh_job_receiver) = crossbeam_channel::unbounded();
        let (result_sender, result_receiver) = crossbeam_channel::unbounded();
        let request_id = Arc::new(AtomicU64::new(0));

        for _ in 0..num_terrain_threads {
            let job_receiver = terrain_job_receiver.clone();
            let result_sender = result_sender.clone();

            let request_id = Arc::clone(&request_id);
            let world_generator_settings = world_generator_settings.clone();

            std::thread::spawn(move || {
                let mut world_generator = WorldGenerator::new(&world_generator_settings);

                loop {
                    let mut job: ChunkJob = match job_receiver.recv() {
                        Ok(job) => job,
                        Err(_) => break,
                    };

                    if job.request_id != request_id.load(Ordering::Relaxed) {
                        match result_sender.send(ChunkJobResult::TerrainCanceled(job.position)) {
                            Err(e) => eprintln!("Failed to send chunk job result: {}", e),
                            _ => {},
                        }
                    }

                    let job_type = "Terrain";
                    let started_at = Instant::now();

                    world_generator.clear_caches();

                    for rel_x in 0..CHUNK_SIZE {
                        if job.request_id != request_id.load(Ordering::Relaxed) {
                            break;
                        }
                        for rel_y in 0..CHUNK_SIZE {
                            for rel_z in 0..CHUNK_SIZE {
                                let rel_pos = IVec3::new(rel_x, rel_y, rel_z);
                                let block_pos = rel_pos + job.position * CHUNK_SIZE;

                                let block = world_generator.get_block(block_pos);

                                if !matches!(block, BlockState::Air) {
                                    job.chunk.set_block_at(rel_pos, block);
                                }
                            }
                        }
                    }

                    job.chunk.generated = true;
                    job.chunk.needs_remesh = true;

                    if job.request_id == request_id.load(Ordering::Relaxed) {
                        match result_sender.send(ChunkJobResult::Succeeded(
                                JobResultData {
                                    type_: job.type_,
                                    position: job.position,
                                    chunk: job.chunk,
                                }
                            )
                        ) {
                            Err(e) => eprintln!("Failed to send chunk job result: {}", e),
                            _ => {},
                        }
                    } else {
                        match result_sender.send(ChunkJobResult::TerrainCanceled(job.position)) {
                            Err(e) => eprintln!("Failed to send chunk job result: {}", e),
                            _ => {},
                        }
                    }
                }
            });
        }

        for _ in 0..num_mesh_threads {
            let job_receiver = mesh_job_receiver.clone();
            let result_sender = result_sender.clone();

            let atlas = atlas.clone();
            let request_id = Arc::clone(&request_id);

            std::thread::spawn(move || {
                loop {
                    let mut job: ChunkJob = match job_receiver.recv() {
                        Ok(job) => job,
                        Err(_) => break,
                    };

                    if job.request_id != request_id.load(Ordering::Relaxed) {
                        match result_sender.send(ChunkJobResult::MeshCanceled(job.position)) {
                            Err(e) => eprintln!("Failed to send chunk job result: {}", e),
                            _ => {},
                        }
                    }

                    let job_type = "Mesh";
                    let started_at = Instant::now();

                    job.chunk.mesh = gen_chunk_mesh(
                        job.position,
                        &job.chunk,
                        job.neighbours,
                        &atlas
                    );

                    job.chunk.meshed = true;
                    job.chunk.needs_remesh = false;

                    if job.request_id == request_id.load(Ordering::Relaxed) {
                        match result_sender.send(
                            ChunkJobResult::Succeeded(
                                JobResultData {
                                    type_: job.type_,
                                    position: job.position,
                                    chunk: job.chunk,
                                }
                            )
                        ) {
                            Err(e) => eprintln!("Failed to send chunk job result: {}", e),
                            _ => {},
                        }
                    } else {
                        match result_sender.send(ChunkJobResult::MeshCanceled(job.position)) {
                            Err(e) => eprintln!("Failed to send chunk job result: {}", e),
                            _ => {},
                        }
                    }
                }
            });
        }

        Self {
            terrain_job_sender,
            mesh_job_sender,
            result_receiver,
            request_id,
        }
    }

    pub fn set_request_id(&self, request_id: u64) {
        self.request_id.store(request_id, Ordering::Release);
    }

    pub fn submit_job(&self, job: ChunkJob) {
        match job.type_ {
            JobType::GenTerrain => {
                self.terrain_job_sender.send(job).expect("Failed to submit job");
            },
            JobType::GenMesh => {
                self.mesh_job_sender.send(job).expect("Failed to submit job");
            },
        }
    }

    pub fn get_result(&self) -> Option<ChunkJobResult> {
        match self.result_receiver.try_recv() {
            Ok(Result) => Some(Result),
            Err(_) => None,
        }
    }
}