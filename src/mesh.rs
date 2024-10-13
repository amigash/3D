use crate::triangle::Triangle;
use glam::Vec3A;
use std::{
    fs::File,
    io::{self, BufRead, BufReader, Error, ErrorKind::InvalidData},
};

pub fn load_from_obj_file(file: File) -> io::Result<Vec<Triangle>> {
    let reader = BufReader::new(file);
    let mut mesh = vec![];
    let mut vertices = vec![];

    for (line_number, line) in reader.lines().enumerate() {
        let line = line?;
        let mut words = line.split_whitespace();

        match words.next() {
            Some("v") => {
                let mut vertex = Vec3A::ZERO;
                for coordinate in vertex.as_mut() {
                    *coordinate = words
                        .next()
                        .ok_or_else(|| {
                            Error::new(
                                InvalidData,
                                format!("Missing vertex coordinate on line {}", line_number + 1),
                            )
                        })?
                        .parse()
                        .map_err(|_| {
                            Error::new(
                                InvalidData,
                                format!(
                                    "Failed to parse vertex coordinate on line {}",
                                    line_number + 1
                                ),
                            )
                        })?;
                }
                vertices.push(vertex);
            }
            Some("f") => {
                let mut points = [Vec3A::ZERO; 3];
                for point in &mut points {
                    let index = words
                        .next()
                        .ok_or_else(|| {
                            Error::new(
                                InvalidData,
                                format!("Missing face index on line {}", line_number + 1),
                            )
                        })?
                        .split('/')
                        .next()
                        .unwrap()
                        .parse::<usize>()
                        .map_err(|_| {
                            Error::new(
                                InvalidData,
                                format!("Failed to parse face index on line {}", line_number + 1),
                            )
                        })?;
                    *point = *vertices.get(index).ok_or_else(|| {
                        Error::new(
                            InvalidData,
                            format!("Vertex index out of bounds on line {}", line_number + 1),
                        )
                    })?;
                }
                mesh.push(Triangle::from(points));
            }
            _ => (),
        }
    }
    Ok(mesh)
}
