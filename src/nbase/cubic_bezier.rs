use crate::nbase::point::Point;

pub struct CubicBezierSegment<const N: usize> {
    pub ps: [Point<N>; 4],
}

impl<const N: usize> CubicBezierSegment<N> {
    pub fn split(&self, t: f32) -> (CubicBezierSegment<N>, CubicBezierSegment<N>) {
        let s = 1.0 - t;
        let p1 = self.ps[0];
        let p2 = Point::scaled_sum(&[s, t], &[self.ps[0], self.ps[1]]);
        let p3 = Point::scaled_sum(
            &[s * s, 2.0 * s * t, t * t],
            &[self.ps[0], self.ps[1], self.ps[2]],
        );
        let p4 = Point::scaled_sum(
            &[s * s * s, 3.0 * s * s * t, 3.0 * s * t * t, t * t * t],
            &[self.ps[0], self.ps[1], self.ps[2], self.ps[3]],
        );
        let q1 = p4;
        let q2 = Point::scaled_sum(
            &[s * s, 2.0 * s * t, t * t],
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

    pub fn value(&self, t: f32) -> Point<N> {
        let t2 = t * t;
        let t3 = t2 * t;
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let mt3 = mt2 * mt;
        self.ps[0] * mt3 + self.ps[1] * 3.0 * mt2 * t + self.ps[2] * 3.0 * mt * t2 + self.ps[3] * t3
    }
}

// Number of points should be 3*n+1 for some n
#[derive(Clone)]
pub struct CubicBezierPath<const N: usize> {
    pub ps: Vec<Point<N>>,
}

impl<const N: usize> CubicBezierPath<N> {
    pub fn segment(&self, n: usize) -> CubicBezierSegment<N> {
        assert!(n < (self.ps.len() - 1) / 3);
        let ps = &self.ps[3 * n..3 * n + 4];
        CubicBezierSegment {
            ps: [ps[0], ps[1], ps[2], ps[3]],
        }
    }
}
