use std::ops::{Add, AddAssign, Mul};
use glam::Vec3;

#[derive(Debug)]
pub struct Triangle {
    pub a: Vec3,
    pub b: Vec3,
    pub c: Vec3
}
impl Add<Vec3> for Triangle {
    type Output = Self;

    fn add(self, rhs: Vec3) -> Self::Output {
        Triangle {
            a: self.a + rhs,
            b: self.b + rhs,
            c: self.c + rhs,
        }
    }
}

impl AddAssign<Vec3> for Triangle {
    fn add_assign(&mut self, rhs: Vec3) {
        self.a += rhs;
        self.b += rhs;
        self.c += rhs;
    }
}

impl Mul<f32> for Triangle {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Triangle {
            a: self.a * rhs,
            b: self.b * rhs,
            c: self.c * rhs,
        }
    }
}

pub const fn tri(a: Vec3, b: Vec3, c: Vec3, ) -> Triangle {
    Triangle { a, b, c }
}

impl Triangle {
    pub fn surface_normal(&self) -> Vec3 {
        let a = self.b - self.a;
        let b = self.c - self.a;
        a.cross(b).normalize()
    }

    pub fn centroid(&self) -> Vec3 {
        (self.a + self.b + self.c) / 3.0
    }
}