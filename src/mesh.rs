use crate::triangle::{Triangle, Vertex};
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
    pub fn get_pixel(&self, x: usize, y: usize) -> [u8; 4] {
        let index = (x.min(self.width - 1) + self.width * y.min(self.height - 1)) * 4;
        [
            self.pixels[index],
            self.pixels[index + 1],
            self.pixels[index + 2],
            self.pixels[index + 3],
        ]
    }
}

pub fn load_mtl_file(path: impl AsRef<Path>) -> Result<HashMap<String, Texture>> {
    let mut map = HashMap::new();
    let file = File::open(path.as_ref())?;
    let reader = BufReader::new(file);
    let mut material_name = None;

    for line in reader.lines() {
        let line = line?;
        let mut words = line.split_whitespace();
        let Some(command) = words.next() else {
            continue;
        };

        match command {
            "newmtl" => {
                if material_name.is_some() {
                    bail!("Material name defined without being used")
                }
                let name = words.next().with_context(|| "No material specified")?.to_string();
                material_name = Some(name);
            }
            "map_Kd" => {
                let name = material_name.take().with_context(|| "No material name specified")?;
                let image_string = words.next().with_context(|| "No path specified")?;
                let partial_image_path: &Path = image_string.as_ref();
                let image_path = if let Some(parent_directory) = path.as_ref().parent() {
                    parent_directory.join(partial_image_path)
                } else {
                    partial_image_path.to_path_buf()
                };
                let image = load_image_into_pixel_buffer(image_path)?;
                map.try_insert(name, image)
                    .expect("Material texture is already defined");
            }
            _ => bail!("Only newmtl and map_Kd are currently supported"),
        }
    }
    Ok(map)
}

pub fn load_image_into_pixel_buffer(path: impl AsRef<Path>) -> Result<Texture> {
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

    let mut material_library = None;
    let mut current_material = None;

    let reader = BufReader::new(File::open(path)?);
    for (line_number, line) in reader.lines().enumerate() {
        let err = |message: &str| {
            format!("In \"{}\" on line {}: {message}", line_number + 1, path.display())
        };
        
        let line = line?;
        let mut words = line
            .split_whitespace()
            .map(std::string::ToString::to_string);
        let Some(command) = words.next() else {
            continue;
        };

        match command.as_ref() {
            "mtllib" => {
                if material_library.as_ref().is_some() {
                    bail!(err("Referencing multiple .mtl files is not currently supported"));
                }
                let library_string = words.next().with_context(|| err("No path provided"))?;
                let partial_library_path: &Path = library_string.as_ref();
                let library_path = if let Some(parent_directory) = path.parent() {
                    parent_directory.join(partial_library_path)
                } else {
                    partial_library_path.to_path_buf()
                };
                material_library = Some(load_mtl_file(library_path)?);
            }
            "usemtl" => {
                if material_library.as_ref().is_none() {
                    bail!(err("mttlib is undefined"));
                }
                let material_name = words.next().with_context(|| err("No path provided"))?;
                current_material = Some(material_name);
            }
            coordinate_type @ ("v" | "vn" | "vt") => {
                let destination = match coordinate_type {
                  "v" => &mut vertices,
                  "vn" => &mut normals,
                  "vt" => &mut texture_coordinates,
                    _ => unreachable!("Match arms should reflect coordinate_type"),
                };
                
                // We want to treat every `vt {u} {v}` as `vt {u} {v} 1.0` for later perspective transforms 
                let texture_extension = (coordinate_type == "vt").then_some("1.0".to_string()).into_iter();
                
                let points: Vec<f32> = words
                    .take(3)
                    .chain(texture_extension)
                    .map(|w| w.parse())
                    .collect::<Result<_, _>>()?;
                if points.len() != 3 {
                    bail!(err("Incorrect number of vertices"));
                }
                destination.push(Vec3A::from_slice(&points));
            },
            "f" => {
                let mut triangle = [Vertex::default(); 3];
                for vertex in &mut triangle {
                    let vertex_data: Vec<usize> = words
                        .next()
                        .with_context(|| err("Expected another vertex"))?
                        .split('/')
                        .map(str::parse)
                        .collect::<Result<_, _>>()?;
                    
                    let (position, texture, normal) = match vertex_data[..] {
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
                let texture = current_material.as_ref().with_context(|| err("No material provided"))?;
                triangles.push(Triangle::new(triangle, texture));
            }
            unknown_element => bail!("{} \"{unknown_element}\"", err("Unknown element")),
        }
    }

    let textures = material_library.with_context(|| format!("{}: mtllib is undefined", path.display()))?;

    Ok(ObjectData {
        triangles,
        textures,
    })
}
