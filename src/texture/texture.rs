use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureId {
    Stone,
    Dirt,
    GrassTop,
    GrassSide,
    OakPlanks,
}

pub struct Texture {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

pub struct TextureRegistry {
    pub textures: HashMap<TextureId, Texture>,
}

impl TextureRegistry {
    pub fn from_paths(textures: Vec<(TextureId, &str)>) -> Self {
        let mut new = TextureRegistry {
            textures: HashMap::new(),
        };

        for t in textures {
            let (id, path) = t;

            let path = Path::new(path);

            let mut pixels = Vec::new();
            let width;
            let height;

            let image = image::open(path);
            match image {
                Ok(i) => {
                    pixels = i.clone().into_rgba8().into_raw();
                    width = i.width();
                    height = i.height();
                },
                Err(_) => {
                    pixels = vec![
                        255, 0, 255, 255,
                        0, 0, 0, 255,
                        0, 0, 0, 255,
                        255, 0, 255, 255,
                    ];
                    width = 2;
                    height = 2;
                }
            }
            

            new.textures.insert(
                id,
                Texture {
                    width,
                    height,
                    pixels: pixels.clone(),
                }
            );
        }

        new
    }
}

pub struct AtlasRegion {
    pub min: [f32; 2],
    pub max: [f32; 2],
}

pub struct TextureAtlas {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
    pub regions: HashMap<TextureId, AtlasRegion>,
}

impl TextureAtlas {
    pub fn new(textures: Vec<(TextureId, Texture)>) -> anyhow::Result<Self> {
        let mut new = Self {
            width: 1,
            height: 1,
            pixels: Vec::new(),
            regions: HashMap::new(),
        };

        if textures.len() == 0 {
            return Ok(new);
        }

        let texture_width = textures[0].1.width;
        let texture_height = textures[0].1.height;

        let texture_count = textures.len();

        let grid_size = (texture_count as f64).sqrt().ceil() as u32;
        let grid_size = grid_size.next_power_of_two();

        new.width = grid_size * texture_width;
        new.height = grid_size * texture_height;

        new.pixels = vec![0; (new.width * new.height * 4) as usize];

        for i in 0..texture_count {
            let t = &textures[i].1;
            let id = &textures[i].0;

            if t.width != texture_width || t.height != texture_height {
                anyhow::bail!("Could not generate texture atlas: all texture sizes not identical");
            }

            let row = i as u32 % grid_size;
            let column = i as u32 / grid_size;

            let min = [
                column * texture_width,
                row * texture_height,
            ];
            let max = [min[0] + texture_width, min[1] + texture_height];

            let mut pxl_idx: usize = 0;

            for x in min[0]..max[0] {
                for y in min[1]..max[1] {
                    let write_idx = ((y * new.width + x) * 4) as usize;

                    new.pixels[write_idx + 0] = t.pixels[pxl_idx + 0];
                    new.pixels[write_idx + 1] = t.pixels[pxl_idx + 1];
                    new.pixels[write_idx + 2] = t.pixels[pxl_idx + 2];
                    new.pixels[write_idx + 3] = t.pixels[pxl_idx + 3];

                    pxl_idx += 4;
                }
            }

            new.regions.insert(*id, AtlasRegion {
                min: [min[0] as f32, min[1] as f32],
                max: [max[0] as f32, max[1] as f32],
            });
        }

        Ok(new)
    }
}