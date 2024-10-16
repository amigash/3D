use glam::Vec3A;

#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: Vec3A,
    pub normal: Option<Vec3A>,
    pub texture: Option<Vec3A>,
}

impl Vertex {
    pub fn new(position: Vec3A, normal: Option<Vec3A>, texture: Option<Vec3A>) -> Self {
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
}

impl Triangle {
    pub fn new(a: Vertex, b: Vertex, c: Vertex) -> Self {
        Self {
            vertices: [a, b, c],
            normal: (b.position - c.position)
                .cross(c.position - a.position)
                .normalize(),
        }
    }
}

impl From<[Vertex; 3]> for Triangle {
    fn from([a, b, c]: [Vertex; 3]) -> Self {
        Self::new(a, b, c)
    }
}
