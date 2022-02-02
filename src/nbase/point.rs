use std::ops::{Add, Mul, Sub};

use crate::nbase::bounds::Bounds;

#[derive(Clone, Copy, Debug)]
pub struct Point<const N: usize> {
    pub vs: [f32; N],
}

impl<const N: usize> Default for Point<N> {
    fn default() -> Self {
        Self { vs: [0.0f32; N] }
    }
}

impl<const N: usize> PartialEq for Point<N> {
    fn eq(&self, other: &Self) -> bool {
        for i in 0..N {
            if self.vs[i] != other.vs[i] {
                return false;
            }
        }
        true
    }
}

impl<const N: usize> Eq for Point<N> {}

fn bimap<A, B, F, C, const N: usize>(a: &[A; N], b: &[B; N], f: F) -> [C; N]
where
    F: Fn(A, B) -> C,
    A: Copy,
    B: Copy,
    C: Default + Copy,
{
    let mut result = [C::default(); N];
    for i in 0..N {
        result[i] = f(a[i], b[i]);
    }
    result
}

impl<const N: usize> Point<N> {
    pub fn neg(&self) -> Self {
        Point {
            vs: self.vs.map(|v| -v),
        }
    }

    pub fn dot(&self, n: Point<N>) -> f32 {
        self.vs
            .into_iter()
            .zip(n.vs.into_iter())
            .map(|(a, b)| a * b)
            .sum()
    }

    pub fn lerp(alpha: f32, a: Point<N>, b: Point<N>) -> Point<N> {
        Point::axby(1.0 - alpha, a, alpha, b)
    }

    pub fn axby(a: f32, x: Point<N>, b: f32, y: Point<N>) -> Point<N> {
        Point::bimap(x, y, |x, y| a * x + b * y)
    }

    pub fn abs(&self) -> Point<N> {
        Point {
            vs: self.vs.map(|a| a.abs()),
        }
    }

    pub fn min(&self) -> f32 {
        self.vs.iter().copied().fold(f32::NAN, f32::min)
    }

    pub fn max(&self) -> f32 {
        self.vs.iter().copied().fold(f32::NAN, f32::max)
    }

    pub fn scaled_sum(ws: &[f32], ps: &[Point<N>]) -> Point<N> {
        assert!(ws.len() == ps.len());
        let mut sum = Point::default();
        for i in 0..ws.len() {
            sum = sum + ps[i] * ws[i];
        }
        sum
    }

    pub fn map<F>(self, f: F) -> Point<N>
    where
        F: Fn(f32) -> f32,
    {
        Point { vs: self.vs.map(f) }
    }

    pub fn bimap<F>(a: Point<N>, b: Point<N>, f: F) -> Point<N>
    where
        F: Fn(f32, f32) -> f32,
    {
        Point {
            vs: bimap(&a.vs, &b.vs, f),
        }
    }

    pub fn from(vs: [f32; N]) -> Point<N> {
        Point { vs }
    }

    fn componentwise_min(a: Point<N>, b: Point<N>) -> Point<N> {
        Point::bimap(a, b, f32::min)
    }

    fn componentwise_max(a: Point<N>, b: Point<N>) -> Point<N> {
        Point::bimap(a, b, f32::max)
    }
}

impl<const N: usize> Mul<f32> for Point<N> {
    type Output = Point<N>;
    fn mul(self, s: f32) -> Self::Output {
        self.map(|x| x * s)
    }
}

impl<const N: usize> Mul<Point<N>> for Point<N> {
    type Output = Point<N>;
    fn mul(self, rhs: Point<N>) -> Self::Output {
        Point::bimap(self, rhs, |a, b| a * b)
    }
}

impl<const N: usize> Add<Point<N>> for Point<N> {
    type Output = Point<N>;

    fn add(self, rhs: Point<N>) -> Self::Output {
        Point::bimap(self, rhs, |x, y| x + y)
    }
}

impl<const N: usize> Sub<Point<N>> for Point<N> {
    type Output = Point<N>;

    fn sub(self, rhs: Point<N>) -> Self::Output {
        Point::bimap(self, rhs, |x, y| x - y)
    }
}

pub fn point_extrema<const N: usize>(v: Option<Bounds<N>>, s: &Point<N>) -> Option<Bounds<N>> {
    if let Some(Bounds { mut min, mut max }) = v {
        min = Point::componentwise_min(min, *s);
        max = Point::componentwise_max(max, *s);
        Some(Bounds { min, max })
    } else {
        Some(Bounds { min: *s, max: *s })
    }
}
