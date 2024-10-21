use glam::{Vec3A, Vec4};

#[derive(Copy, Clone, Default, Debug)]
pub struct Vertex {
    pub position: Vec3A,
    pub normal: Vec3A,
    pub texture: Vec3A,
}

#[derive(Debug)]
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
            centroid: (a.position + b.position + c.position) / 3.0,
        }
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct ProjectedVertex {
    pub position: Vec4,
    pub normal: Vec3A,
    pub texture: Vec3A,
}

impl ProjectedVertex {
    pub fn lerp(&self, rhs: Self, s: f32) -> Self {
        Self {
            position: self.position.lerp(rhs.position, s),
            normal: self.normal.lerp(rhs.normal, s),
            texture: self.texture.lerp(rhs.texture, s),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ProjectedTriangle {
    pub vertices: [ProjectedVertex; 3],
    pub normal: Vec3A,
    pub texture_name: String,
    pub centroid: Vec3A,
}

impl From<&Triangle> for ProjectedTriangle {
    fn from(triangle: &Triangle) -> Self {
        ProjectedTriangle {
            vertices: triangle.vertices.map(|v| ProjectedVertex {
                position: v.position.extend(1.0),
                normal: v.normal,
                texture: v.texture,
            }),
            normal: triangle.normal,
            texture_name: triangle.texture_name.clone(),
            centroid: triangle.centroid,
        }
    }
}
