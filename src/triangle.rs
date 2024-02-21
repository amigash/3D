use std::ops::{Add, AddAssign, Mul};
use glam::Vec3;

#[derive(Debug)]
pub struct Triangle {
    pub a: Vec3,
    pub b: Vec3,
    pub c: Vec3,
    pub normal: Vec3
}
impl Add<Vec3> for Triangle {
    type Output = Self;

    fn add(self, rhs: Vec3) -> Self::Output {
        Triangle {
            a: self.a + rhs,
            b: self.b + rhs,
            c: self.c + rhs,
            normal: self.normal,
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
            normal: self.normal,

        }
    }
}

impl Triangle {
    pub fn new(a: Vec3, b: Vec3, c: Vec3) -> Self {
       Triangle {
           a,
           b,
           c,
           normal: (b - a).cross(c - a).normalize()
       }
    }

    pub fn centroid(&self) -> Vec3 {
        (self.a + self.b + self.c) / 3.0
    }
}