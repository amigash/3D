use glam::Vec3A;

type Vertex = Vec3A;

pub struct Triangle {
    vertices: [Vertex; 3],
    normal: Vertex,
}

impl Triangle {
    pub fn new(a: Vertex, b: Vertex, c: Vertex) -> Self {
        Self {
            vertices: [a, b, c],
            normal: (b - c).cross(c - a).normalize(),
        }
    }

    pub fn vertices(&self) -> [Vertex; 3] {
        self.vertices
    }

    pub fn normal(&self) -> Vertex {
        self.normal
    }
}

impl From<[Vertex; 3]> for Triangle {
    fn from([a, b, c]: [Vertex; 3]) -> Self {
        Self::new(a, b, c)
    }
}
