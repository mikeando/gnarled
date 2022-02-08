use crate::n3::p3;
use crate::nbase::point::Point;
use crate::nbase::polyline::PolyLine;
use crate::nbase::{bounds::Bounds, polyline::LineSegment};

use super::{Camera, OcclusionInfo};

pub struct Hit<'a>(&'a dyn Shape, f32);

// TODO: direction should be different class?
// TODO: should live in nbase?
pub struct Ray {
    pub origin: Point<3>,
    pub direction: Point<3>,
}

pub trait Shape {
    fn paths(&self) -> Vec<PolyLine<3>>;
    fn intersect(&self, ray: &Ray) -> Option<Hit>;
    fn contains(&self, x: Point<3>, tol: f32) -> bool;
    fn bounds(&self) -> Bounds<3>;
    fn compile(&self);

    fn render(
        &self,
        camera: &Camera,
        occlusion_info: &OcclusionInfo,
        consumer: &mut dyn Consumer,
    ) -> Result<(), std::io::Error>;
}

pub struct AABox {
    pub bounds: Bounds<3>,
}

pub trait Consumer {
    fn add(&mut self, ls: &LineSegment<2>);
}

impl Shape for AABox {
    fn paths(&self) -> Vec<PolyLine<3>> {
        let Point { vs: [x1, y1, z1] } = self.bounds.min;
        let Point { vs: [x2, y2, z2] } = self.bounds.max;
        vec![
            PolyLine {
                ps: vec![p3(x1, y1, z1), p3(x1, y1, z2)],
            },
            PolyLine {
                ps: vec![p3(x1, y1, z1), p3(x1, y2, z1)],
            },
            PolyLine {
                ps: vec![p3(x1, y1, z1), p3(x2, y1, z1)],
            },
            PolyLine {
                ps: vec![p3(x1, y1, z2), p3(x1, y2, z2)],
            },
            PolyLine {
                ps: vec![p3(x1, y1, z2), p3(x2, y1, z2)],
            },
            PolyLine {
                ps: vec![p3(x1, y2, z1), p3(x1, y2, z2)],
            },
            PolyLine {
                ps: vec![p3(x1, y2, z1), p3(x2, y2, z1)],
            },
            PolyLine {
                ps: vec![p3(x1, y2, z2), p3(x2, y2, z2)],
            },
            PolyLine {
                ps: vec![p3(x2, y1, z1), p3(x2, y1, z2)],
            },
            PolyLine {
                ps: vec![p3(x2, y1, z1), p3(x2, y2, z1)],
            },
            PolyLine {
                ps: vec![p3(x2, y1, z2), p3(x2, y2, z2)],
            },
            PolyLine {
                ps: vec![p3(x2, y2, z1), p3(x2, y2, z2)],
            },
        ]
    }

    fn intersect(&self, ray: &Ray) -> Option<Hit> {
        let n: Point<3> = (self.bounds.min - ray.origin) / ray.direction;
        let f: Point<3> = (self.bounds.max - ray.origin) / ray.direction;
        let n = Point::componentwise_min(f, n);
        let f = Point::componentwise_max(f, n);
        let t0 = n.max();
        let t1 = f.min();
        if t0 < 1e-3 && t1 > 1e-3 {
            return Some(Hit(self, t1));
        }
        if t0 >= 1e-3 && t0 < t1 {
            return Some(Hit(self, t0));
        }
        return None;
    }

    fn contains(&self, x: Point<3>, tol: f32) -> bool {
        self.bounds.expand_by(tol).contains(x)
    }

    fn bounds(&self) -> Bounds<3> {
        self.bounds
    }

    fn compile(&self) {}

    fn render(
        &self,
        camera: &Camera,
        occlusion_info: &OcclusionInfo,
        consumer: &mut dyn Consumer,
    ) -> Result<(), std::io::Error> {
        let paths = self.paths();
        for p in paths {
            let mut line_segments = p.line_segments();

            while let Some(ls) = line_segments.pop() {
                // Is the segment on-screen?
                if !camera.is_segment_visible(&ls) {
                    continue;
                }

                // How big is the line segment in screen space
                let ls_screen = camera.project(&ls);
                if ls_screen.len2() > camera.lmax {
                    let (ls1, ls2) = ls.split();
                    line_segments.push(ls2);
                    line_segments.push(ls1);
                    continue;
                }

                // Check if it's occluded.
                let o1 = occlusion_info.is_occluded(&ls.ps[0]);
                let o2 = occlusion_info.is_occluded(&ls.ps[1]);
                if (o1 && o2) {
                    continue;
                }
                if (!o1 && !o2) {
                    consumer.add(&ls_screen);
                    continue;
                }
                if ls_screen.len2() > camera.lmin {
                    let (ls1, ls2) = ls.split();
                    line_segments.push(ls2);
                    line_segments.push(ls1);
                }
            }
        }
        Ok(())
    }
}
