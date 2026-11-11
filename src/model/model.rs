use std::collections::HashMap;

use crate::renderer::mesh::Mesh;
use crate::renderer::vertex::Vertex;
use crate::utils::direction::Direction;
use crate::texture::texture::{AtlasRegion, TextureAtlas, TextureId};

const WHITE: [f32; 3] = [1.0, 1.0, 1.0];

fn map_face_uv(uv: [f32; 2], texture_region: &AtlasRegion, atlas: &TextureAtlas) -> [f32; 2] {
    [
        (texture_region.min[0] + uv[0] / 16.0 * (texture_region.max[0] - texture_region.min[0]))
            / atlas.width as f32,
        (texture_region.min[1] + uv[1] / 16.0 * (texture_region.max[1] - texture_region.min[1]))
            / atlas.height as f32,
    ]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModelId {
    Air,
    Stone,
}

pub struct ModelRegistry {
    pub models: HashMap<ModelId, BlockModel>,
}

pub struct BlockModel {
    pub elements: Vec<ModelElement>,
}

impl BlockModel {
    pub fn get_mesh(&self, cull_faces: Vec<Direction>, atlas: &TextureAtlas) -> Mesh {
        let mut mesh = Mesh::new();

        for e in &self.elements {
            mesh.add_mesh(&e.get_mesh(&cull_faces, &atlas));
        }

        mesh
    }
}

pub struct ModelElement {
    pub from: [f32; 3],
    pub to: [f32; 3],
    pub faces: HashMap<Direction, ModelFace>,
    pub textures: HashMap<String, TextureId>,
}

impl ModelElement {
    pub fn get_mesh(&self, cull_faces: &Vec<Direction>, atlas: &TextureAtlas) -> Mesh {
        let mut mesh = Mesh::new();

        for f in &self.faces {
            let (direction, face) = f;

            if cull_faces.contains(&face.cullface) {
                continue;
            }

            let texture = self.textures.get(&face.texture).unwrap();
            let texture_region = atlas
                .regions
                .get(texture)
                .expect("Texture referenced by model is missing from atlas");
            let uv = [
                map_face_uv([face.uv[0], face.uv[1]], texture_region, atlas),
                map_face_uv([face.uv[2], face.uv[1]], texture_region, atlas),
                map_face_uv([face.uv[2], face.uv[3]], texture_region, atlas),
                map_face_uv([face.uv[0], face.uv[3]], texture_region, atlas),
            ];

            mesh.add_mesh(&match direction {
                Direction::Down => {
                    Mesh {
                        vertices: vec![
                            Vertex { position: [self.from[0], self.from[1], self.from[2]], colour: WHITE, uv: uv[0] },
                            Vertex { position: [self.to[0],   self.from[1], self.from[2]], colour: WHITE, uv: uv[1] },
                            Vertex { position: [self.to[0],   self.from[1], self.to[2]],   colour: WHITE, uv: uv[2] },
                            Vertex { position: [self.from[0], self.from[1], self.to[2]],   colour: WHITE, uv: uv[3] },
                        ],
                        indicies: vec![0, 1, 2, 0, 2, 3],
                    }
                },

                Direction::Up => {
                    Mesh {
                        vertices: vec![
                            Vertex { position: [self.from[0], self.to[1],   self.from[2]], colour: WHITE, uv: uv[0] },
                            Vertex { position: [self.to[0],   self.to[1],   self.from[2]], colour: WHITE, uv: uv[1] },
                            Vertex { position: [self.to[0],   self.to[1],   self.to[2]],   colour: WHITE, uv: uv[2] },
                            Vertex { position: [self.from[0], self.to[1],   self.to[2]],   colour: WHITE, uv: uv[3] },
                        ],
                        indicies: vec![0, 2, 1, 0, 3, 2],
                    }
                },

                Direction::East => {
                    Mesh {
                        vertices: vec![
                            Vertex { position: [self.to[0], self.to[1],   self.from[2]], colour: WHITE, uv: uv[0] },
                            Vertex { position: [self.to[0], self.from[1], self.from[2]], colour: WHITE, uv: uv[1] },
                            Vertex { position: [self.to[0], self.from[1], self.to[2]],   colour: WHITE, uv: uv[2] },
                            Vertex { position: [self.to[0], self.to[1],   self.to[2]],   colour: WHITE, uv: uv[3] },
                        ],
                        indicies: vec![0, 1, 2, 0, 2, 3],
                    }
                },

                Direction::West => {
                    Mesh {
                        vertices: vec![
                            Vertex { position: [self.from[0], self.to[1],   self.from[2]], colour: WHITE, uv: uv[0] },
                            Vertex { position: [self.from[0], self.from[1], self.from[2]], colour: WHITE, uv: uv[1] },
                            Vertex { position: [self.from[0], self.from[1], self.to[2]],   colour: WHITE, uv: uv[2] },
                            Vertex { position: [self.from[0], self.to[1],   self.to[2]],   colour: WHITE, uv: uv[3] },
                        ],
                        indicies: vec![0, 2, 1, 0, 3, 2],
                    }
                },

                Direction::South => {
                    Mesh {
                        vertices: vec![
                            Vertex { position: [self.from[0], self.to[1],   self.to[2]], colour: WHITE, uv: uv[0] },
                            Vertex { position: [self.to[0],   self.to[1],   self.to[2]], colour: WHITE, uv: uv[1] },
                            Vertex { position: [self.to[0],   self.from[1], self.to[2]], colour: WHITE, uv: uv[2] },
                            Vertex { position: [self.from[0], self.from[1], self.to[2]], colour: WHITE, uv: uv[3] },
                        ],
                        indicies: vec![0, 1, 2, 0, 2, 3],
                    }
                },

                Direction::North => {
                    Mesh {
                        vertices: vec![
                            Vertex { position: [self.from[0], self.to[1],   self.from[2]], colour: WHITE, uv: uv[0] },
                            Vertex { position: [self.to[0],   self.to[1],   self.from[2]], colour: WHITE, uv: uv[1] },
                            Vertex { position: [self.to[0],   self.from[1], self.from[2]], colour: WHITE, uv: uv[2] },
                            Vertex { position: [self.from[0], self.from[1], self.from[2]], colour: WHITE, uv: uv[3] },
                        ],
                        indicies: vec![0, 2, 1, 0, 3, 2],
                    }
                },
            });
        }

        mesh
    }
}

pub struct ModelFace {
    pub direction: Direction,
    pub uv: [f32; 4],
    pub texture: String,
    pub cullface: Direction,
}