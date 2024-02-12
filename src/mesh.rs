use std::fs::File;
use nannou::geom::{pt3, Tri};
use std::io::{self, prelude::*, BufReader};

pub fn mesh_from_obj_file(file: File) -> io::Result<Vec<Tri>> {
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
                vertices.push(pt3(x, y, z));
            }
            Some("f") => {
                let i: usize = words.next().unwrap().parse().unwrap();
                let j: usize = words.next().unwrap().parse().unwrap();
                let k: usize = words.next().unwrap().parse().unwrap();
                mesh.push(Tri([vertices[i - 1], vertices[j - 1], vertices[k - 1]]));
            }
            _ => {}
        }
    }
    Ok(mesh)
}