use glam::IVec3;

use crate::chunk::chunk::*;

#[derive(Debug)]
pub enum JobType {
    GenTerrain,
    GenMesh,
}

pub struct ChunkJob {
    pub type_: JobType,
    pub position: IVec3,
    pub chunk: Chunk,
}

pub struct ChunkJobResult {
    pub type_: JobType,
    pub position: IVec3,
    pub chunk: Chunk,
}