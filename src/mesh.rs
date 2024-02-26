<<<<<<< HEAD
use std::fs::File;
use nannou::geom::{pt3, Tri};
use std::io::{self, prelude::*, BufReader};

pub fn mesh_from_obj_file(file: File) -> io::Result<Vec<Tri>> {
=======
use glam::Vec3A;
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use crate::triangle::Triangle;
use win_loop::anyhow::{anyhow, Result};

pub fn load_from_obj_file(file: File) -> Result<Vec<Triangle>> {
>>>>>>> new-repo/main
    let reader = BufReader::new(file);
    let mut mesh = vec![];
    let mut vertices = vec![];

    for line in reader.lines() {
        let line = line?;
        let mut words = line.split_whitespace();
        match words.next() {
            Some("v") => {
<<<<<<< HEAD
                let x: f32 = words.next().unwrap().parse().unwrap();
                let y: f32 = words.next().unwrap().parse().unwrap();
                let z: f32 = words.next().unwrap().parse().unwrap();
                vertices.push(pt3(x, y, z));
            }
            Some("f") => {
                let i: usize = words.next().unwrap().parse().unwrap();
                let j: usize = words.next().unwrap().parse().unwrap();
                let k: usize = words.next().unwrap().parse().unwrap();
                mesh.push(Tri([vertices[i - 1], vertices[j - 1], vertices[k - 1]]));
=======
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
>>>>>>> new-repo/main
            }
            _ => {}
        }
    }
    Ok(mesh)
<<<<<<< HEAD
}
=======
}
>>>>>>> new-repo/main
