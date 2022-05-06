use crate::nbase::point::Point;
use crate::nbase::traits::Shiftable;

use super::point::Float;

#[derive(Clone, Debug)]
pub struct CubicBezierSegment<const N: usize, F> {
    pub ps: [Point<N, F>; 4],
}

impl<const N: usize, F> CubicBezierSegment<N, F>
where
    F: Float,
{
    pub fn split(&self, t: F) -> (CubicBezierSegment<N, F>, CubicBezierSegment<N, F>) {
        let c2 = F::from_f64(2.0);
        let c3 = F::from_f64(3.0);
        let s = F::one() - t;
        let p1 = self.ps[0];
        let p2 = Point::scaled_sum(&[s, t], &[self.ps[0], self.ps[1]]);
        let p3 = Point::scaled_sum(
            &[s * s, c2 * s * t, t * t],
            &[self.ps[0], self.ps[1], self.ps[2]],
        );
        let p4 = Point::scaled_sum(
            &[s * s * s, c3 * s * s * t, c3 * s * t * t, t * t * t],
            &[self.ps[0], self.ps[1], self.ps[2], self.ps[3]],
        );
        let q1 = p4;
        let q2 = Point::scaled_sum(
            &[s * s, c2 * s * t, t * t],
            &[self.ps[1], self.ps[2], self.ps[3]],
        );
        let q3 = Point::scaled_sum(&[s, t], &[self.ps[2], self.ps[3]]);
        let q4 = self.ps[3];
        (
            CubicBezierSegment {
                ps: [p1, p2, p3, p4],
            },
            CubicBezierSegment {
                ps: [q1, q2, q3, q4],
            },
        )
    }

    pub fn value(&self, t: F) -> Point<N, F> {
        let b = cubic_basis(t);
        self.ps[0] * b[0] + self.ps[1] * b[1] + self.ps[2] * b[2] + self.ps[3] * b[3]
    }
}

#[inline]
pub fn cubic_basis<F: Float>(t: F) -> [F; 4] {
    let c3 = F::from_f64(3.0);
    let t2 = t * t;
    let t3 = t2 * t;
    let mt = F::one() - t;
    let mt2 = mt * mt;
    let mt3 = mt2 * mt;
    [mt3, c3 * mt2 * t, c3 * mt * t2, t3]
}

// Number of points should be 3*n+1 for some n
#[derive(Clone)]
pub struct CubicBezierPath<const N: usize> {
    pub ps: Vec<Point<N, f32>>,
}

impl<const N: usize> CubicBezierPath<N> {
    pub fn segment(&self, n: usize) -> CubicBezierSegment<N, f32> {
        assert!(n < (self.ps.len() - 1) / 3);
        let ps = &self.ps[3 * n..3 * n + 4];
        CubicBezierSegment {
            ps: [ps[0], ps[1], ps[2], ps[3]],
        }
    }
}

impl<const N: usize> Shiftable<N> for CubicBezierPath<N> {
    type Result = CubicBezierPath<N>;

    fn shift_by(&self, d: Point<N>) -> Self::Result {
        CubicBezierPath {
            ps: self.ps.iter().map(|p| *p + d).collect(),
        }
    }
}

impl<const N: usize> Shiftable<N> for CubicBezierSegment<N, f32> {
    type Result = CubicBezierSegment<N, f32>;

    fn shift_by(&self, d: Point<N>) -> Self::Result {
        CubicBezierSegment {
            ps: self.ps.map(|p| p + d),
        }
    }
}
