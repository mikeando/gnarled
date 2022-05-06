use crate::nbase::point::Point;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineSegment<const N: usize> {
    pub ps: [Point<N>; 2],
}

impl<const N: usize> LineSegment<N> {
    pub(crate) fn new(p1: Point<N>, p2: Point<N>) -> LineSegment<N> {
        LineSegment { ps: [p1, p2] }
    }

    pub(crate) fn len2(&self) -> f32 {
        let d = self.ps[1] - self.ps[0];
        d.dot(d)
    }

    pub(crate) fn split(&self) -> (LineSegment<N>, LineSegment<N>) {
        let mp = Point::lerp(0.5, self.ps[0], self.ps[1]);
        (
            LineSegment {
                ps: [self.ps[0], mp],
            },
            LineSegment {
                ps: [mp, self.ps[1]],
            },
        )
    }

    pub(crate) fn nsplit(&self, n: usize) -> Vec<LineSegment<N>> {
        let inv_n: f32 = 1.0 / (n as f32);
        (0..n)
            .map(|i| {
                let ii = i as f32;
                LineSegment {
                    ps: [
                        Point::lerp(ii * inv_n, self.ps[0], self.ps[1]),
                        Point::lerp((ii + 1.0) * inv_n, self.ps[0], self.ps[1]),
                    ],
                }
            })
            .collect()
    }

    pub(crate) fn reverse(&self) -> LineSegment<N> {
        LineSegment {
            ps: [self.ps[1], self.ps[0]],
        }
    }

    pub(crate) fn midpoint(&self) -> Point<N> {
        (self.ps[0] + self.ps[1]) * 0.5
    }
}
