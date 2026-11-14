use std::{collections::HashMap, hash::Hash, time::Duration};

use glam::IVec3;

use crate::{chunk::chunk::*, utils::direction::Direction};

#[derive(Debug)]
pub enum JobType {
    GenTerrain,
    GenMesh,
}

pub struct ChunkJob {
    pub request_id: u64,
    pub type_: JobType,
    pub position: IVec3,
    pub chunk: Chunk,
    pub neighbours: HashMap<Direction, Chunk>,
}

pub enum ChunkJobResult {
    Succeeded(JobResultData),
    TerrainCanceled(IVec3),
    MeshCanceled(IVec3),
}

pub struct JobResultData {
    pub type_: JobType,
    pub position: IVec3,
    pub chunk: Chunk,
}