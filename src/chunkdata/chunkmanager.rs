use crossbeam_channel::{Receiver, Sender};
use egui::mutex::Mutex;
use glam::IVec3;
use std::backtrace::Backtrace;
use std::collections::{HashMap, HashSet, VecDeque};
use std::panic;
use std::sync::Arc;

use crate::chunkdata::chunk::*;
use crate::chunkdata::job::JobType::{GenMesh, GenTerrain};
use crate::chunkdata::job::*;
use crate::chunkdata::jobmanager::ChunkJobManager;
use crate::texture::TextureAtlas;
use crate::utils::{direction::Direction, posinrange::pos_in_range};
use crate::worldgen::worldgenerator::*;

const MAX_GEN_IN_FLIGHT: usize = 4;
const MAX_MESH_IN_FLIGHT: usize = 4;

pub struct ChunkManagerRequest {
    pub lat_render_dist: u32,
    pub vert_render_dist: u32,
    pub cam_chunk_pos: IVec3,
    pub loaded: HashSet<IVec3>,
}

pub struct ChunkManagerResult {
    pub to_load: Vec<IVec3>,
    pub to_unload: Vec<IVec3>,
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

        let chunk_job_manager = ChunkJobManager::new(4, 4, world_generator_settings, &atlas);

        loop {
            match request_receiver.try_iter().last() {
                Some(request) => {
                    // Reset the area loading

                    active_request_id += 1;

                    chunk_job_manager.set_request_id(active_request_id);

                    cam_chunk_pos = request.cam_chunk_pos;

                    min = IVec3::new(
                        cam_chunk_pos.x - request.lat_render_dist as i32 - 1,
                        cam_chunk_pos.y - request.vert_render_dist as i32 - 1,
                        cam_chunk_pos.z - request.lat_render_dist as i32 - 1,
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
                }
                None => {
                    // Contunue the area loading as normal

                    if !active_request {
                        continue;
                    }

                    let mut new_frontier: HashSet<IVec3> = HashSet::new();

                    for p in frontier.drain() {
                        if visited.contains(&p) || !pos_in_range(&p, &min, &max) {
                            continue;
                        }

                        if !loaded.contains(&p) && !to_unload.contains(&p) {
                            if let Some(c) = chunks.lock().get(&p) {
                                if !loaded.contains(&p) && c.meshed {
                                    loaded.insert(p);
                                    to_load.push(p);
                                }

                                if c.generated
                                    && (!c.meshed || c.needs_remesh)
                                    && !to_remesh.contains(&p)
                                    && !mesh_submitted.contains(&p)
                                {
                                    to_remesh.push_back(p);
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
                        match result {
                            ChunkJobResult::Succeeded(data) => match data.type_ {
                                JobType::GenMesh => {
                                    mesh_submitted.remove(&data.position);
                                    loaded.insert(data.position);
                                    to_load.push(data.position);
                                    chunks.lock().insert(data.position, data.chunk);
                                }
                                JobType::GenTerrain => {
                                    submitted.remove(&data.position);

                                    chunks.lock().insert(data.position, data.chunk);
                                    to_remesh.push_back(data.position);

                                    for d in Direction::all() {
                                        let n = data.position + d.rel_vec();

                                        if let Some(c) = chunks.lock().get_mut(&n)
                                            && c.was_edge
                                        {
                                            c.was_edge = false;
                                            to_remesh.push_back(n);
                                        }
                                    }
                                }
                            },
                            ChunkJobResult::TerrainCanceled(position) => {
                                submitted.remove(&position);
                            }
                            ChunkJobResult::MeshCanceled(position) => {
                                mesh_submitted.remove(&position);
                            }
                        }
                    }

                    if !to_load.is_empty() || !to_unload.is_empty() {
                        if let Err(e) =
                            result_sender.send(ChunkManagerResult { to_load, to_unload })
                        {
                            eprintln!("{}", e);
                        }

                        to_load = Vec::new();
                        to_unload = Vec::new();
                    }

                    if submitted.len() < MAX_GEN_IN_FLIGHT
                        && let Some(p) = to_submit.pop_front()
                        && !submitted.contains(&p)
                    {
                        submitted.insert(p);
                        chunk_job_manager.submit_job(ChunkJob {
                            request_id: active_request_id,
                            type_: GenTerrain,
                            position: p,
                            chunk: Chunk::new(),
                            neighbours: HashMap::new(),
                        });
                    }

                    if mesh_submitted.len() < MAX_MESH_IN_FLIGHT {
                        let ready_index = to_remesh.iter().position(|&p| {
                            Direction::all().into_iter().all(|d| {
                                let n = p + d.rel_vec();
                                let n_out_of_range = !pos_in_range(&n, &min, &max);

                                match chunks.lock().get(&n) {
                                    Some(c) => c.generated,
                                    None => n_out_of_range,
                                }
                            })
                        });

                        if let Some(index) = ready_index {
                            let p = to_remesh
                                .remove(index)
                                .expect("ready queue index must exist");
                            let on_edge = Direction::all()
                                .into_iter()
                                .any(|d| !pos_in_range(&(p + d.rel_vec()), &min, &max));

                            let neighbours = get_chunk_neighbours(Arc::clone(&chunks), p);
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
            }
        }
    });
}
