use glam::IVec3;

use crate::chunk::chunk::*;

pub enum ChunkJob {
    GenTerrain((IVec3, Chunk)),
    GenMesh((IVec3, Chunk)),
}

pub struct ChunkJobResult {
    position: IVec3,
    chunk: Chunk,
}