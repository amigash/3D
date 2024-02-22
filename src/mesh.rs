use glam::vec3a;
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use crate::triangle::Triangle;
use win_loop::anyhow::Result;

pub fn load_from_obj_file(file: File) -> Result<Vec<Triangle>> {
    let reader = BufReader::new(file);
    let mut mesh = vec![];
    let mut vertices = vec![];

    for line in reader.lines() {
        let line = line?;
        let mut words = line.split_whitespace();
        match words.next() {
            Some("v") => {
                let x: f32 = words.next().unwrap().parse().unwrap();
                let y: f32 = words.next().unwrap().parse().unwrap();
                let z: f32 = words.next().unwrap().parse().unwrap();
                vertices.push(vec3a(x, y, z));
            }
            Some("f") => {
                let i: usize = words.next().unwrap().parse().unwrap();
                let j: usize = words.next().unwrap().parse().unwrap();
                let k: usize = words.next().unwrap().parse().unwrap();
                mesh.push(Triangle::new([
                    vertices[i - 1],
                    vertices[j - 1],
                    vertices[k - 1],
                ]));
            }
            _ => {}
        }
    }
    Ok(mesh)
}
