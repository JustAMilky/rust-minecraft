use glam::Vec3;

use crate::render::vertex::Vertex;

#[derive(Clone)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indicies: Vec<u32>,
}

impl Mesh {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indicies: Vec::new(),
        }
    }

    pub fn translated(&self, offset: Vec3) -> Self {
        let mut new = Self::new();

        new.indicies = self.indicies.clone();

        for v in &self.vertices {
            new.vertices.push(Vertex {
                position: [
                    v.position[0] + offset.x,
                    v.position[1] + offset.y,
                    v.position[2] + offset.z,
                ],
                colour: v.colour,
                uv: v.uv,
            });
        }

        new
    }

    pub fn add_mesh(&mut self, to_add: &Mesh) {
        let vertex_offset = self.vertices.len() as u32;

        for i in to_add.indicies.clone().into_iter() {
            self.indicies.push(i + vertex_offset);
        }

        self.vertices.extend_from_slice(&to_add.vertices);
    }
}
