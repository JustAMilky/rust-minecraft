use crossbeam_channel::{Receiver, Sender};

use crate::chunk::job::ChunkJob::{self, GenTerrain};

pub struct ChunkJobManager {
    job_sender: Sender<ChunkJob>,
    result_receiver: Receiver<ChunkJob>,
}

impl ChunkJobManager {
    pub fn new(num_threads: usize) -> Self {
        let (job_sender, job_receiver) = crossbeam_channel::unbounded();
        let (result_sender, result_receiver) = crossbeam_channel::unbounded();

        for _ in 0..num_threads {
            let job_receiver = job_receiver.clone();
            let result_sender = result_sender.clone();

            std::thread::spawn(move || {
                loop {
                    let job: ChunkJob = job_receiver.recv().unwrap();
                    
                    match job {
                        GenTerrain((chunk_pos, chunk)) => {

                        }
                    }

                    result_sender.send(job);
                }
            });
        }

        Self {
            job_sender,
            result_receiver,
        }
    }

    fn submit_job(&self, job: ChunkJob) {
        self.job_sender.send(job);
    }

    fn get_result(&self) -> Option<ChunkJob> {
        match self.result_receiver.try_recv() {
            Ok(job) => Some(job),
            Err(_) => None,
        }
    }
}