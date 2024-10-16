use crate::triangle::{Triangle, Vertex};
use glam::{Vec2, Vec3A};
use image::{ImageReader};
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};
use win_loop::anyhow::Result;


pub fn load_image_into_pixel_buffer(path: impl AsRef<Path>) -> Result<Vec<u8>> {
    Ok(ImageReader::open(path)?
        .with_guessed_format()?
        .decode()?
        .flipv()
        .to_rgba8()
        .into_raw())
}

pub fn load_from_obj_file(path: impl AsRef<Path>) -> Result<Vec<Triangle>> {
    let reader = BufReader::new(File::open(path)?);
    let mut mesh = vec![];

    let mut vertices = vec![];
    let mut texture_coordinates = vec![];
    let mut normals = vec![];

    for line in reader.lines() {
        let line = line?;
        let mut words = line.split_whitespace();
        let Some(first) = words.next() else { continue };

        match first {
            "vn" => {
                let mut vertex = Vec3A::ZERO;
                for coordinate in vertex.as_mut() {
                    *coordinate = words
                        .next()
                        .expect("Expected another vertex")
                        .parse()?;
                }
                normals.push(vertex);
            }
            "vt" => {
                let mut vertex = Vec2::ZERO;
                for coordinate in vertex.as_mut() {
                    *coordinate = words
                        .next()
                        .expect("Expected another vertex")
                        .parse()?;
                }
                texture_coordinates.push(vertex.extend(1.0).into());
            }
            "v" => {
                let mut vertex = Vec3A::ZERO;
                for coordinate in vertex.as_mut() {
                    *coordinate = words
                        .next()
                        .expect("Expected another vertex")
                        .parse()?;
                }
                vertices.push(vertex);
            }
            "f" => {
                let mut triangle = vec![];
                for word in words {
                    let vertex_data = &word
                        .split('/')
                        .map(|s| s.parse::<usize>().unwrap_or(0))
                        .collect::<Vec<_>>()[..];
                    let (position, texture, normal) = match vertex_data {
                        &[v, vt, vn] => (
                            vertices[v - 1],
                            (vt != 0).then(|| texture_coordinates[vt - 1]),
                            Some(normals[vn - 1]),
                        ),
                        [v, vt] => (vertices[v - 1], Some(texture_coordinates[vt - 1]), None),
                        [v] => (vertices[v - 1], None, None),
                        _ => panic!(),
                    };
                    triangle.push(Vertex {
                        position,
                        normal,
                        texture,
                    });
                }
                let [a, b, c] = triangle[0..=2] else { panic!() };

                mesh.push(Triangle::new(a, b, c));
            }
            _ => (),
        }
    }
    Ok(mesh)
}
