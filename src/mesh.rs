use crate::geometry::{Triangle, Vertex};
use glam::Vec3A;
use image::ImageReader;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};
use win_loop::anyhow::{bail, Context, Result};

const DEFAULT_NORMAL: Vec3A = Vec3A::ZERO;
const DEFAULT_TEXTURE: Vec3A = Vec3A::ZERO;

#[derive(Debug, Clone)]
pub struct Texture {
    pub width: usize,
    pub height: usize,
    pixels: Vec<u8>,
}

impl Texture {
    fn from_color(r: u8, g: u8, b: u8) -> Self {
        Self {
            width: 1,
            height: 1,
            pixels: vec![r, g, b, 255],
        }
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> [u8; 4] {
        let index = (x.min(self.width - 1) + self.width * y.min(self.height - 1)) * 4;
        self.pixels[index..index + 4].try_into().unwrap()
    }

    pub fn try_from_path(path: impl AsRef<Path>) -> Result<Self> {
        let image = ImageReader::open(path)?
            .with_guessed_format()?
            .decode()?
            .flipv()
            .to_rgba8();
        let texture = Texture {
            width: image.width() as usize,
            height: image.height() as usize,
            pixels: image.into_raw(),
        };
        Ok(texture)
    }
}

impl Default for Texture {
    fn default() -> Self {
        Self::from_color(255, 255, 255)
    }
}

pub fn load_mtl_file(path: impl AsRef<Path>, map: &mut HashMap<String, Texture>) -> Result<()> {
    let path = path.as_ref();

    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut material_name = None;

    for line in reader.lines() {
        let line = line?;
        if let Some('#') = line.chars().next() {
            continue; // skip comments
        }
        let mut words = line.split_whitespace();
        let Some(command) = words.next() else {
            continue;
        };

        match command {
            "newmtl" => {
                let name = words
                    .next()
                    .with_context(|| "No material specified")?
                    .to_string();
                material_name = Some(name);
            }
            "map_Kd" => {
                let name = material_name
                    .take()
                    .with_context(|| "No material name specified")?;
                let image_string = words.next().with_context(|| "No path specified")?;
                let image_path = path.with_file_name(image_string);
                let image = Texture::try_from_path(image_path)?;
                map.insert(name, image);
            }
            _ => (),
        }
    }
    Ok(())
}

pub struct ObjectData {
    pub triangles: Vec<Triangle>,
    pub textures: HashMap<String, Texture>,
}

pub fn load_from_obj_file(path: impl AsRef<Path>) -> Result<ObjectData> {
    let path = path.as_ref();

    // Initialize with a dummy value to offset one-based indexing
    let mut vertices = vec![Vec3A::default()];
    let mut texture_coordinates = vec![Vec3A::default()];
    let mut normals = vec![Vec3A::default()];

    let mut triangles = vec![];

    let mut material_library = HashMap::new();

    // Some default materials for debugging
    material_library.insert("cyan".to_string(), Texture::from_color(0, 255, 255));
    material_library.insert("magenta".to_string(), Texture::from_color(255, 0, 255));
    material_library.insert("yellow".to_string(), Texture::from_color(255, 255, 0));

    let mut current_material = None;

    let reader = BufReader::new(File::open(path)?);
    for (line_number, line) in reader.lines().enumerate() {
        let err = |message: &str| {
            format!(
                "In \"{}\" on line {}: {message}",
                path.display(),
                line_number + 1,
            )
        };

        let line = line?;

        if let Some('#') = line.chars().next() {
            continue; // skip comments
        }

        let mut words = line
            .split_whitespace()
            .map(std::string::ToString::to_string);
        let Some(command) = words.next() else {
            continue;
        };

        match command.as_ref() {
            "mtllib" => {
                let library_string = words.next().with_context(|| err("No path provided"))?;
                let library_path = path.with_file_name(library_string);
                load_mtl_file(library_path, &mut material_library)?;
            }
            "usemtl" => {
                let material_name = words
                    .next()
                    .with_context(|| err("No material name provided"))?;
                current_material = Some(material_name);
            }
            "v" | "vn" | "vt" => {
                let destination = match command.as_ref() {
                    "v" => &mut vertices,
                    "vn" => &mut normals,
                    "vt" => &mut texture_coordinates,
                    _ => unreachable!("Match arms should reflect possible commands"),
                };

                // We want to treat every `vt {u} {v}` as `vt {u} {v} 1.0` for later perspective transforms
                let texture_extension = (command == "vt").then_some("1.0".to_string()).into_iter();

                let points: Vec<f32> = words
                    .take(3)
                    .chain(texture_extension)
                    .map(|w| w.parse())
                    .collect::<Result<_, _>>()?;
                if points.len() != 3 {
                    bail!(err("Incorrect number of vertices"));
                }
                destination.push(Vec3A::from_slice(&points));
            }
            "f" => {
                let mut triangle = [Vertex::default(); 3];
                for vertex in &mut triangle {
                    let vertex_data: Vec<usize> = words
                        .next()
                        .with_context(|| err("Expected another vertex"))?
                        .split('/')
                        .filter(|s| !s.is_empty())
                        .map(str::parse)
                        .collect::<Result<_, _>>()?;

                    let (position, texture, normal) = match *vertex_data.as_slice() {
                        [v, vt, vn] => (vertices[v], texture_coordinates[vt], normals[vn]),
                        [v, vt] => (vertices[v], texture_coordinates[vt], DEFAULT_NORMAL),
                        [v] => (vertices[v], DEFAULT_TEXTURE, DEFAULT_NORMAL),
                        _ => bail!(err("Invalid number of face arguments")),
                    };
                    *vertex = Vertex {
                        position,
                        normal,
                        texture,
                    };
                }
                let texture = current_material.clone().unwrap_or_default();
                triangles.push(Triangle::new(triangle, texture.as_str()));
            }
            _ => (),
        }
    }

    let textures = material_library;

    Ok(ObjectData {
        triangles,
        textures,
    })
}
