use std::ops::{Add, Mul, Sub};

#[derive(Clone, Copy, Debug)]
pub struct Point(pub f32, pub f32, pub f32);

impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1 && self.2 == other.2
    }
}

impl Eq for Point {}

impl Point {
    pub fn neg(&self) -> Self {
        Point(-self.0, -self.1, -self.2)
    }

    pub fn dot(&self, n: Point) -> f32 {
        self.0 * n.0 + self.1 * n.1 + self.2 * n.2
    }

    pub fn lerp(alpha: f32, a: Point, b: Point) -> Point {
        Point::axby(alpha, a, 1.0 - alpha, b)
    }

    pub fn axby(a: f32, x: Point, b: f32, y: Point) -> Point {
        Point(a * x.0 + b * y.0, a * x.1 + b * y.1, a * x.2 + b * y.2)
    }

    pub fn abs(&self) -> Point {
        Point(self.0.abs(), self.1.abs(), self.2.abs())
    }

    pub fn min(&self) -> f32 {
        self.0.min(self.1).min(self.2)
    }

    pub fn max(&self) -> f32 {
        self.0.max(self.1).max(self.2)
    }

    pub fn scaled_sum(ws: &[f32], ps: &[Point]) -> Point {
        assert!(ws.len() == ps.len());
        let mut sum = Point(0.0, 0.0, 0.0);
        for i in 0..ws.len() {
            sum.0 += ws[i] * ps[i].0;
            sum.1 += ws[i] * ps[i].1;
            sum.2 += ws[i] * ps[i].2;
        }
        sum
    }
}

impl Mul<(f32, f32, f32)> for Point {
    type Output = Point;
    fn mul(self, (a, b, c): (f32, f32, f32)) -> Self::Output {
        Point(a * self.0, b * self.1, c * self.2)
    }
}

impl Mul<f32> for Point {
    type Output = Point;
    fn mul(self, s: f32) -> Self::Output {
        Point(self.0 * s, self.1 * s, self.2 * s)
    }
}

impl Add<Point> for Point {
    type Output = Point;

    fn add(self, rhs: Point) -> Self::Output {
        Point(self.0 + rhs.0, self.1 + rhs.1, self.2 + rhs.2)
    }
}

impl Sub<Point> for Point {
    type Output = Point;

    fn sub(self, rhs: Point) -> Self::Output {
        Point(self.0 - rhs.0, self.1 - rhs.1, self.2 - rhs.2)
    }
}
