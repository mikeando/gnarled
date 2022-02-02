use crate::n2::bounds::Bounds;
use crate::n2::lineset::LineSet;
use crate::n2::point::Point;
use crate::n2::traits::*;

#[derive(Clone)]
pub struct PolyLine {
    pub ps: Vec<Point>,
}

impl PolyLine {
    pub fn clip_by(&self, n: Point, v: f32) -> LineSet {
        // If f is not linear we cant assume
        // 1. we can calculate the zero using simple interpolation
        // 2. that if both ends are the same sign, then all points between
        //    are the same sign.
        let f = |p: Point| p.dot(n) - v;
        if self.ps.is_empty() {
            return LineSet { lines: vec![] };
        }

        let mut p_prev = self.ps[0];
        let mut a_prev = f(p_prev);
        let mut lines: Vec<PolyLine> = vec![];
        let mut current_points: Option<Vec<Point>> = if a_prev >= 0.0 {
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
}

impl Boundable for PolyLine {
    fn bounds(&self) -> Option<Bounds> {
        self.ps.iter().fold(None, crate::n2::point::point_extrema)
    }
}

impl Shiftable for PolyLine {
    type Result = PolyLine;
    fn shift_by(&self, d: Point) -> PolyLine {
        PolyLine {
            ps: self.ps.iter().map(|p| *p + d).collect(),
        }
    }
}

impl Rotatable for PolyLine {
    type Result = PolyLine;
    fn rotate_by(&self, radians: f32, Point { vs: [cx, cy] }: Point) -> PolyLine {
        PolyLine {
            ps: self
                .ps
                .iter()
                .map(|Point { vs: [x, y] }| {
                    let xx = x - cx;
                    let yy = y - cy;
                    let c = radians.cos();
                    let s = radians.sin();
                    let rx = c * xx + s * yy;
                    let ry = -s * xx + c * yy;
                    Point {
                        vs: [rx + cx, ry + cy],
                    }
                })
                .collect(),
        }
    }
}

impl Scalable for PolyLine {
    type Result = PolyLine;

    fn scale(&self, Point { vs: [cx, cy] }: Point, (sx, sy): (f32, f32)) -> Self::Result {
        PolyLine {
            ps: self
                .ps
                .iter()
                .map(|Point { vs: [x, y] }| Point {
                    vs: [sx * (x - cx) + cx, sy * (y - cy) + cy],
                })
                .collect(),
        }
    }
}
