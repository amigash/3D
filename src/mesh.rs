use glam::Vec3A;
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use crate::triangle::Triangle;
use win_loop::anyhow::{anyhow, Result};

pub fn load_from_obj_file(file: File) -> Result<Vec<Triangle>> {
    let reader = BufReader::new(file);
    let mut mesh = vec![];
    let mut vertices = vec![];

    for line in reader.lines() {
        let line = line?;
        let mut words = line.split_whitespace();
        match words.next() {
            Some("v") => {
                let mut vertex = Vec3A::ZERO;
                for i in 0..3 {
                    vertex[i] = words.next().ok_or(anyhow!("Expected another vertex"))?.parse()?;
                }
                vertices.push(vertex);
            }
            Some("f") => {
                let mut points = [Vec3A::ZERO; 3];
                for point in &mut points {
                    let index: usize = words.next().ok_or(anyhow!("Expected another index"))?.parse()?;
                    *point = vertices[index - 1];
                }
                mesh.push(Triangle::new(points));
            }
            _ => {}
        }
    }
    Ok(mesh)
}
