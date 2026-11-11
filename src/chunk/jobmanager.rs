use std::collections::HashMap;
use std::sync::Arc;
use crossbeam_channel::{Receiver, Sender};
use glam::IVec3;

use crate::block::blockstate::BlockState;
use crate::chunk::job::*;
use crate::chunk::chunk::{CHUNK_SIZE, gen_chunk_mesh};
use crate::chunk::job::ChunkJobResult;
use crate::texture::texture::TextureAtlas;

pub struct ChunkJobManager {
    job_sender: Sender<ChunkJob>,
    result_receiver: Receiver<ChunkJobResult>,
}

impl ChunkJobManager {
    pub fn new(num_threads: usize, atlas: &TextureAtlas) -> Self {
        let (job_sender, job_receiver) = crossbeam_channel::unbounded();
        let (result_sender, result_receiver) = crossbeam_channel::unbounded();

        for i in 0..num_threads {
            let job_receiver = job_receiver.clone();
            let result_sender = result_sender.clone();

            let atlas = atlas.clone();

            std::thread::spawn(move || {
                loop {
                    let mut job: ChunkJob = match job_receiver.recv() {
                        Ok(job) => job,
                        Err(_) => break,
                    };

                    match job.type_ {
                        JobType::GenTerrain => {
                            for rel_x in 0..CHUNK_SIZE {
                                for rel_y in 0..CHUNK_SIZE {
                                    for rel_z in 0..CHUNK_SIZE {
                                        let block_pos = IVec3::new(rel_x, rel_y, rel_z)
                                            + job.position * CHUNK_SIZE;

                                        if block_pos.y < 0 {
                                            job.chunk.set_block_at(IVec3::new(rel_x, rel_y, rel_z), BlockState::Stone);
                                        }
                                    }
                                }
                            }

                            job.chunk.generated = true;
                            job.chunk.needs_remesh = true;

                            result_sender.send(ChunkJobResult {
                                type_: job.type_,
                                position: job.position,
                                chunk: job.chunk,
                            }).expect("Failed to send chunk job result");
                        },
                        JobType::GenMesh => {
                            job.chunk.mesh = gen_chunk_mesh(job.position, &job.chunk, HashMap::new(), &atlas);

                            job.chunk.meshed = true;
                            job.chunk.needs_remesh = false;

                            result_sender.send(ChunkJobResult {
                                type_: job.type_,
                                position: job.position,
                                chunk: job.chunk,
                            }).expect("Failed to send chunk job result");
                        },
                    }
                }
            });
        }

        Self {
            job_sender,
            result_receiver,
        }
    }

    pub fn submit_job(&self, job: ChunkJob) {
        self.job_sender.send(job).expect("Failed to submit job");
    }

    pub fn get_result(&self) -> Option<ChunkJobResult> {
        match self.result_receiver.try_recv() {
            Ok(Result) => Some(Result),
            Err(_) => None,
        }
    }
}