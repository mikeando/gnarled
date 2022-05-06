use crate::attributes::*;
use crate::nbase::point::Point;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineSegment<const N: usize, A> {
    pub ps: [Point<N>; 2],
    pub attributes: A,
}

impl<const N: usize, A> LineSegment<N, A> {
    pub(crate) fn new(p1: Point<N>, p2: Point<N>) -> LineSegment<N, A>
    where
        A: Default,
    {
        LineSegment {
            ps: [p1, p2],
            attributes: Default::default(),
        }
    }

    pub(crate) fn len2(&self) -> f32 {
        let d = self.ps[1] - self.ps[0];
        d.dot(d)
    }

    pub(crate) fn reverse(&self) -> LineSegment<N, A>
    where
        A: AttributeReverse,
    {
        LineSegment {
            ps: [self.ps[1], self.ps[0]],
            attributes: self.attributes.reverse(),
        }
    }

    pub(crate) fn split(&self) -> (LineSegment<N, A>, LineSegment<N, A>)
    where
        A: AttributeRange,
    {
        let mp = Point::lerp(0.5, self.ps[0], self.ps[1]);
        (
            LineSegment {
                ps: [self.ps[0], mp],
                attributes: self.attributes.range(0.0, 0.5),
            },
            LineSegment {
                ps: [mp, self.ps[1]],
                attributes: self.attributes.range(0.5, 1.0),
            },
        )
    }

    pub(crate) fn nsplit(&self, n: usize) -> Vec<LineSegment<N, A>>
    where
        A: AttributeRange,
    {
        let inv_n: f32 = 1.0 / (n as f32);
        (0..n)
            .map(|i| {
                let ii = i as f32;
                LineSegment {
                    ps: [
                        Point::lerp(ii * inv_n, self.ps[0], self.ps[1]),
                        Point::lerp((ii + 1.0) * inv_n, self.ps[0], self.ps[1]),
                    ],
                    attributes: self.attributes.range(ii * inv_n, (ii + 1.0) * inv_n),
                }
            })
            .collect()
    }

    pub(crate) fn midpoint(&self) -> Point<N> {
        (self.ps[0] + self.ps[1]) * 0.5
    }

    pub(crate) fn map_attribute<F, A2>(&self, mut f: F) -> LineSegment<N, A2>
    where
        F: FnMut(&A) -> A2,
    {
        LineSegment {
            ps: self.ps,
            attributes: f(&self.attributes),
        }
    }
}
