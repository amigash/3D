use glam::Vec3;
use std::ops::{Add, AddAssign, Mul};

#[derive(Debug)]
pub struct Triangle {
    pub points: [Vec3; 3],
    pub normal: Vec3,
}
impl Add<Vec3> for Triangle {
    type Output = Self;

    fn add(self, rhs: Vec3) -> Self::Output {
        Triangle {
            points: self.points.map(|point| point + rhs),
            normal: self.normal,
        }
    }
}

impl AddAssign<Vec3> for Triangle {
    fn add_assign(&mut self, rhs: Vec3) {
        self.points = self.points.map(|point| point + rhs);
    }
}

impl Mul<f32> for Triangle {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Triangle {
            points: self.points.map(|point| point * rhs),
            normal: self.normal,
        }
    }
}

impl Triangle {
    pub fn new(points: [Vec3; 3]) -> Self {
        Triangle {
            points,
            normal: (points[1] - points[0]).cross(points[2] - points[0]).normalize(),
        }
    }

    pub fn centroid(&self) -> Vec3 {
        self.points.iter().sum::<Vec3>() / 3.0
    }
}
