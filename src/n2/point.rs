use std::ops::{Add, Mul, Sub};
use crate::n2::bounds::Bounds;

#[derive(Clone, Copy, Debug)]
pub struct Point(pub f32, pub f32);

impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}

impl Eq for Point {}

impl Point {
    pub fn neg(&self) -> Self {
        Point(-self.0, -self.1)
    }

    pub fn dot(&self, n: Point) -> f32 {
        self.0 * n.0 + self.1 * n.1
    }

    pub fn lerp(alpha: f32, a: Point, b: Point) -> Point {
        Point::axby(1.0-alpha, a, alpha, b)
    }

    pub fn axby(a: f32, x: Point, b: f32, y: Point) -> Point {
        Point(a * x.0 + b * y.0, a * x.1 + b * y.1)
    }

    pub fn abs(&self) -> Point {
        Point(self.0.abs(), self.1.abs())
    }

    pub fn min(&self) -> f32 {
        self.0.min(self.1)
    }

    pub fn max(&self) -> f32 {
        self.0.max(self.1)
    }

    pub fn scaled_sum(ws: &[f32], ps: &[Point]) -> Point {
        assert!(ws.len() == ps.len());
        let mut sum = Point(0.0, 0.0);
        for i in 0..ws.len() {
            sum.0 += ws[i] * ps[i].0;
            sum.1 += ws[i] * ps[i].1;
        }
        sum
    }
}

impl Mul<(f32, f32)> for Point {
    type Output = Point;
    fn mul(self, (a, b): (f32, f32)) -> Self::Output {
        Point(a * self.0, b * self.1)
    }
}

impl Mul<f32> for Point {
    type Output = Point;
    fn mul(self, s: f32) -> Self::Output {
        Point(self.0 * s, self.1 * s)
    }
}

impl Add<Point> for Point {
    type Output = Point;

    fn add(self, rhs: Point) -> Self::Output {
        Point(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl Sub<Point> for Point {
    type Output = Point;

    fn sub(self, rhs: Point) -> Self::Output {
        Point(self.0 - rhs.0, self.1 - rhs.1)
    }
}

pub fn point_extrema(v: Option<Bounds>, s: &Point) -> Option<Bounds> {
    if let Some(Bounds{mut min,mut max}) = v {
        if s.0 < min.0 {
            min.0 = s.0;
        } else if s.0 > max.0 {
            max.0 = s.0;
        }
        if s.1 < min.1 {
            min.1 = s.1;
        } else if s.1 > max.1 {
            max.1 = s.1;
        }
        Some(Bounds{min, max})
    } else {
        Some(Bounds{min:*s, max:*s})
    }
}
