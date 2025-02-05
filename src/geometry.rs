use glam::{Mat4, Vec2, Vec3A, Vec4};

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

    pub fn project(&self, view_projection_matrix: Mat4) -> ProjectedTriangle {
        ProjectedTriangle {
            vertices: self.vertices.map(|vertex| ProjectedVertex {
                position: view_projection_matrix * vertex.position.extend(1.0),
                normal: vertex.normal,
                texture: vertex.texture,
            }),
            normal: self.normal,
            texture_name: self.texture_name.clone(),
            centroid: self.centroid,
        }
    }

    pub fn is_facing_viewer(&self, viewer_position: Vec3A) -> bool {
        self.normal
            .dot(viewer_position - self.centroid)
            .is_sign_negative()
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

impl ProjectedTriangle {
    pub fn divide_and_scale(&self, size: Vec2) -> Triangle {
        Triangle {
            vertices: self.vertices.map(|vertex| {
                let perspective_divided = Vec3A::from_vec4(vertex.position / vertex.position.w);
                let flipped = perspective_divided.with_y(-perspective_divided.y);
                let centered = flipped + Vec3A::new(1.0, 1.0, 0.0);
                let position = centered * Vec3A::from((0.5 * size).extend(1.0));
                let texture =
                    Vec3A::new(vertex.texture.x, vertex.texture.y, 1.0) / vertex.position.w;
                let normal = Vec3A::new(vertex.normal.x, vertex.normal.y, 1.0) / vertex.position.w;
                Vertex {
                    position,
                    normal,
                    texture,
                }
            }),
            normal: self.normal,
            texture_name: self.texture_name.clone(),
            centroid: self.centroid,
        }
    }
}
