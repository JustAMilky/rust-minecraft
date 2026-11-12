use std::{collections::HashMap, hash::Hash, time::Duration};

use glam::IVec3;

use crate::{chunk::chunk::*, utils::direction::Direction};

#[derive(Debug)]
pub enum JobType {
    GenTerrain,
    GenMesh,
}

pub struct ChunkJob {
    pub type_: JobType,
    pub request_id: u64,
    pub position: IVec3,
    pub chunk: Chunk,
    pub neighbours: HashMap<Direction, Chunk>,
}

pub struct ChunkJobResult {
    pub type_: JobType,
    pub request_id: u64,
    pub position: IVec3,
    pub chunk: Chunk,
}