use crate::nbase::bounds::Bounds;
use crate::nbase::lineset::LineSet;
use crate::nbase::point::Point;
use crate::nbase::traits::*;

pub struct LineSegment<const N: usize> {
    pub ps: [Point<N>; 2],
}

impl<const N: usize> LineSegment<N> {
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
}

#[derive(Clone)]
pub struct PolyLine<const N: usize> {
    pub ps: Vec<Point<N>>,
}

impl<const N: usize> PolyLine<N> {
    pub fn clip_by(&self, n: Point<N>, v: f32) -> LineSet<N> {
        // If f is not linear we cant assume
        // 1. we can calculate the zero using simple interpolation
        // 2. that if both ends are the same sign, then all points between
        //    are the same sign.
        let f = |p: Point<N>| p.dot(n) - v;
        if self.ps.is_empty() {
            return LineSet { lines: vec![] };
        }

        let mut p_prev = self.ps[0];
        let mut a_prev = f(p_prev);
        let mut lines: Vec<PolyLine<N>> = vec![];
        let mut current_points: Option<Vec<Point<N>>> = if a_prev >= 0.0 {
            Some(vec![p_prev])
        } else {
            None
        };

        for p in &self.ps[1..] {
            let a = f(*p);
            if a >= 0.0 {
                if a_prev >= 0.0 {
                    assert!(current_points.is_some());
                    current_points.as_mut().unwrap().push(*p);
                } else {
                    assert!(current_points.is_none());
                    // TODO: Handle the case where f is not linear?
                    let da = a - a_prev;
                    let alpha = a / da;
                    //TODO: I think this is wrong!
                    let pp = Point::lerp(alpha, p_prev, *p);
                    current_points = Some(vec![pp, *p]);
                }
            } else {
                if a_prev >= 0.0 {
                    assert!(current_points.is_some());
                    // TODO: Handle the case where f is not linear?
                    let da = a - a_prev;
                    let alpha = a / da;
                    //TODO: I think this is wrong!
                    let pp = Point::lerp(alpha, p_prev, *p);
                    current_points.as_mut().unwrap().push(pp);
                    lines.push(PolyLine {
                        ps: current_points.take().unwrap(),
                    });
                } else {
                    assert!(current_points.is_none());
                }
            }
            p_prev = *p;
            a_prev = a;
        }
        if let Some(ps) = current_points {
            lines.push(PolyLine { ps })
        }

        LineSet { lines }
    }

    pub fn line_segments(&self) -> Vec<LineSegment<N>> {
        let mut result = vec![];
        if self.ps.len() < 2 {
            return result;
        }
        for i in 0..self.ps.len() - 1 {
            result.push(LineSegment {
                ps: [self.ps[i], self.ps[i + 1]],
            });
        }
        result
    }
}

impl<const N: usize> Boundable<N> for PolyLine<N> {
    fn bounds(&self) -> Option<Bounds<N>> {
        self.ps
            .iter()
            .fold(None, crate::nbase::point::point_extrema)
    }
}

impl<const N: usize> Shiftable<N> for PolyLine<N> {
    type Result = PolyLine<N>;
    fn shift_by(&self, d: Point<N>) -> PolyLine<N> {
        PolyLine {
            ps: self.ps.iter().map(|p| *p + d).collect(),
        }
    }
}

impl<const N: usize> Scalable<N> for PolyLine<N> {
    type Result = PolyLine<N>;

    fn scale(&self, c: Point<N>, s: &[f32; N]) -> Self::Result {
        PolyLine {
            ps: self
                .ps
                .iter()
                .map(|p| (*p - c) * Point::from(*s) + c)
                .collect(),
        }
    }
}
