use glam::Vec3A;
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use win_loop::anyhow::{anyhow, Result};

fn normal([a, b, c]: [Vec3A; 3]) -> Vec3A {
    -(c - b).cross(c - a).normalize()
}

pub fn load_from_obj_file(file: File) -> Result<Vec<([Vec3A; 3], Vec3A)>> {
    let reader = BufReader::new(file);
    let mut mesh = vec![];
    let mut vertices = vec![];

    for line in reader.lines().map_while(Result::ok) {
        let mut words = line.split_whitespace();
        match words.next() {
            Some("v") => {
                let mut vertex = Vec3A::ZERO;
                for coordinate in vertex.as_mut() {
                    *coordinate = words
                        .next()
                        .ok_or(anyhow!("Expected another vertex"))?
                        .parse()?;
                }
                vertices.push(vertex);
            }
            Some("f") => {
                let mut points = [Vec3A::ZERO; 3];
                for point in &mut points {
                    let index = words
                        .next()
                        .ok_or(anyhow!("Expected another index"))?
                        .parse::<usize>()?;
                    *point = vertices[index - 1];
                }
                mesh.push((points, normal(points)));
            }
            _ => (),
        }
    }
    Ok(mesh)
}
