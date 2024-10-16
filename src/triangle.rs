use glam::Vec3A;

#[derive(Copy, Clone, Default, Debug)]
pub struct Vertex {
    pub position: Vec3A,
    pub normal: Vec3A,
    pub texture: Vec3A,
}

impl Vertex {
    pub fn new(position: Vec3A, normal: Vec3A, texture: Vec3A) -> Self {
        Self {
            position,
            normal,
            texture,
        }
    }
}

pub struct Triangle {
    pub vertices: [Vertex; 3],
    pub normal: Vec3A,
    pub texture_name: String,
    pub centroid: Vec3A,
}

impl Triangle {
    pub fn new([a, b, c]: [Vertex; 3], texture_name: &(impl ToString + ?Sized)) -> Self {
        Self {
            vertices: [a, b, c],
            normal: (b.position - c.position)
                .cross(c.position - a.position)
                .normalize(),
            texture_name: texture_name.to_string(),
            centroid: (a.position + b.position + c.position) / 3.0
        }
    }
}
