use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub};

use crate::nbase::bounds::Bounds;
pub trait Float:
    Sized
    + Copy
    + std::ops::Add<Output = Self>
    + std::ops::Sub<Output = Self>
    + std::ops::AddAssign
    + std::cmp::PartialEq
    + std::ops::Neg<Output = Self>
    + std::ops::Mul<Output = Self>
    + std::iter::Sum
    + Default
    + std::ops::Div<Output = Self>
    + std::fmt::Debug
    + std::fmt::Display
    + std::cmp::PartialOrd
    + std::ops::MulAssign
{
    fn zero() -> Self;
    fn one() -> Self;
    fn powi(self, i: i32) -> Self;
    fn sqrt(self) -> Self;
    fn abs(self) -> Self;
    fn nan() -> Self;
    fn min(a: Self, b: Self) -> Self;
    fn max(a: Self, b: Self) -> Self;
    fn ln(self) -> Self;
    fn ceil(self) -> Self;
    fn from_f64(f: f64) -> Self;
    fn clamp(self, a: Self, b: Self) -> Self;
    fn as_f64(self) -> f64;
}

impl Float for f32 {
    fn zero() -> Self {
        0.0
    }

    fn one() -> Self {
        1.0
    }

    fn powi(self, i: i32) -> Self {
        f32::powi(self, i)
    }

    fn sqrt(self) -> Self {
        f32::sqrt(self)
    }

    fn abs(self) -> Self {
        f32::abs(self)
    }

    fn nan() -> Self {
        f32::NAN
    }

    fn min(a: Self, b: Self) -> Self {
        f32::min(a, b)
    }

    fn max(a: Self, b: Self) -> Self {
        f32::max(a, b)
    }

    fn ln(self) -> Self {
        f32::ln(self)
    }

    fn ceil(self) -> Self {
        f32::ceil(self)
    }

    fn from_f64(f: f64) -> Self {
        f as f32
    }

    fn clamp(self, a: Self, b: Self) -> Self {
        f32::clamp(self, a, b)
    }

    fn as_f64(self) -> f64 {
        self as f64
    }
}

impl Float for f64 {
    fn zero() -> Self {
        0.0
    }

    fn one() -> Self {
        1.0
    }

    fn powi(self, i: i32) -> Self {
        f64::powi(self, i)
    }

    fn sqrt(self) -> Self {
        f64::sqrt(self)
    }

    fn abs(self) -> Self {
        f64::abs(self)
    }

    fn nan() -> Self {
        f64::NAN
    }

    fn min(a: Self, b: Self) -> Self {
        f64::min(a, b)
    }

    fn max(a: Self, b: Self) -> Self {
        f64::max(a, b)
    }

    fn ln(self) -> Self {
        f64::ln(self)
    }

    fn ceil(self) -> Self {
        f64::ceil(self)
    }

    fn from_f64(f: f64) -> Self {
        f
    }

    fn clamp(self, a: Self, b: Self) -> Self {
        f64::clamp(self, a, b)
    }

    fn as_f64(self) -> f64 {
        self
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Point<const N: usize, F = f32> {
    pub vs: [F; N],
}

// calculate the distance between points
pub fn distance<const N: usize, F>(a: Point<N, F>, b: Point<N, F>) -> F
where
    F: Float,
{
    let mut d = F::zero();
    for i in 0..N {
        d += (a.vs[i] - b.vs[i]).powi(2);
    }
    d.sqrt()
}

impl<const N: usize, F> Default for Point<N, F>
where
    F: Float,
{
    fn default() -> Self {
        Self { vs: [F::zero(); N] }
    }
}

impl<const N: usize, F> PartialEq for Point<N, F>
where
    F: Float,
{
    fn eq(&self, other: &Self) -> bool {
        for i in 0..N {
            if self.vs[i] != other.vs[i] {
                return false;
            }
        }
        true
    }
}

impl<const N: usize, F> Eq for Point<N, F> where F: Float {}

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

impl<const N: usize, F> Point<N, F>
where
    F: Float,
{
    pub fn zero() -> Self {
        Self { vs: [F::zero(); N] }
    }

    pub fn neg(&self) -> Self {
        Point {
            vs: self.vs.map(|v| -v),
        }
    }

    pub fn dot(&self, n: Point<N, F>) -> F {
        IntoIterator::into_iter(self.vs)
            .zip(IntoIterator::into_iter(n.vs))
            .map(|(a, b)| a * b)
            .sum()
    }

    pub fn norm_squared(&self) -> F {
        self.vs.iter().fold(F::zero(), |acc, &v| acc + v * v)
    }

    pub fn normalize(&self) -> Self {
        let norm = self.norm_squared().sqrt();
        Point {
            vs: self.vs.map(|v| v / norm),
        }
    }

    pub fn lerp(alpha: F, a: Point<N, F>, b: Point<N, F>) -> Point<N, F> {
        Point::axby(F::one() - alpha, a, alpha, b)
    }

    pub fn axby(a: F, x: Point<N, F>, b: F, y: Point<N, F>) -> Point<N, F> {
        Point::bimap(x, y, |x, y| a * x + b * y)
    }

    pub fn abs(&self) -> Point<N, F> {
        Point {
            vs: self.vs.map(|a| a.abs()),
        }
    }

    pub fn min(&self) -> F {
        self.vs.iter().copied().fold(F::nan(), F::min)
    }

    pub fn max(&self) -> F {
        self.vs.iter().copied().fold(F::nan(), F::max)
    }

    pub fn scaled_sum(ws: &[F], ps: &[Point<N, F>]) -> Point<N, F> {
        assert!(ws.len() == ps.len());
        let mut sum = Point::default();
        for i in 0..ws.len() {
            sum = sum + ps[i] * ws[i];
        }
        sum
    }

    pub fn map<FF, G>(self, f: FF) -> Point<N, G>
    where
        FF: Fn(F) -> G,
        G: Float,
    {
        Point { vs: self.vs.map(f) }
    }

    pub fn bimap<FF>(a: Point<N, F>, b: Point<N, F>, f: FF) -> Point<N, F>
    where
        FF: Fn(F, F) -> F,
    {
        Point {
            vs: bimap(&a.vs, &b.vs, f),
        }
    }

    pub fn from(vs: [F; N]) -> Point<N, F> {
        Point { vs }
    }

    pub fn componentwise_min(a: Point<N, F>, b: Point<N, F>) -> Point<N, F> {
        Point::bimap(a, b, F::min)
    }

    pub fn componentwise_max(a: Point<N, F>, b: Point<N, F>) -> Point<N, F> {
        Point::bimap(a, b, F::max)
    }
}

impl<const N: usize, F> Mul<F> for Point<N, F>
where
    F: Float,
{
    type Output = Point<N, F>;
    fn mul(self, s: F) -> Self::Output {
        self.map(|x| x * s)
    }
}

impl<const N: usize, F> Mul<Point<N, F>> for Point<N, F>
where
    F: Float,
{
    type Output = Point<N, F>;
    fn mul(self, rhs: Point<N, F>) -> Self::Output {
        Point::bimap(self, rhs, |a, b| a * b)
    }
}

impl<const N: usize, F: Float> Div<Point<N, F>> for Point<N, F> {
    type Output = Point<N, F>;
    fn div(self, rhs: Point<N, F>) -> Self::Output {
        Point::bimap(self, rhs, |a, b| a / b)
    }
}

impl<const N: usize, F: Float> Add<Point<N, F>> for Point<N, F> {
    type Output = Point<N, F>;

    fn add(self, rhs: Point<N, F>) -> Self::Output {
        Point::bimap(self, rhs, |x, y| x + y)
    }
}

impl<const N: usize, F: Float> Sub<Point<N, F>> for Point<N, F> {
    type Output = Point<N, F>;

    fn sub(self, rhs: Point<N, F>) -> Self::Output {
        Point::bimap(self, rhs, |x, y| x - y)
    }
}

impl<const N: usize, F: Float> AddAssign<Point<N, F>> for Point<N, F> {
    fn add_assign(&mut self, rhs: Point<N, F>) {
        for i in 0..N {
            self.vs[i] += rhs.vs[i];
        }
    }
}

impl<const N: usize, F: Float> Neg for Point<N, F> {
    type Output = Point<N, F>;

    fn neg(self) -> Self::Output {
        self.map(|x| -x)
    }
}

pub fn point_extrema<const N: usize, F: Float>(
    v: Option<Bounds<N, F>>,
    s: &Point<N, F>,
) -> Option<Bounds<N, F>> {
    if let Some(Bounds { mut min, mut max }) = v {
        min = Point::componentwise_min(min, *s);
        max = Point::componentwise_max(max, *s);
        Some(Bounds { min, max })
    } else {
        Some(Bounds { min: *s, max: *s })
    }
}

impl<F: Float> Point<3, F> {
    pub fn cross(self, rhs: Point<3, F>) -> Point<3, F> {
        Point {
            vs: [
                self.vs[1] * rhs.vs[2] - self.vs[2] * rhs.vs[1],
                self.vs[2] * rhs.vs[0] - self.vs[0] * rhs.vs[2],
                self.vs[0] * rhs.vs[1] - self.vs[1] * rhs.vs[0],
            ],
        }
    }
}
