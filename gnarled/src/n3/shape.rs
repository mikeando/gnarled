use async_trait::async_trait;
use tokio::sync::mpsc::Sender;

use crate::nbase::bounds::Bounds;
use crate::nbase::point::Point;
use crate::nbase::polyline::PolyLine;
use crate::{n3::p3, nbase::polyline::LineSegment};

use super::{Camera, OcclusionInfo};

pub struct Hit<'a>(&'a dyn Shape, f32);

// TODO: direction should be different class?
// TODO: should live in nbase?
pub struct Ray {
    pub origin: Point<3>,
    pub direction: Point<3>,
}

#[async_trait]
pub trait Shape: Send + Sync + std::fmt::Debug {
    fn paths(&self) -> Vec<PolyLine<3>>;
    fn intersect(&self, ray: &Ray) -> Option<Hit>;
    fn contains(&self, x: Point<3>, tol: f32) -> bool;
    fn bounds(&self) -> Bounds<3>;
    fn compile(&self);

    async fn render(
        &self,
        camera: &Camera,
        occlusion_info: &OcclusionInfo,
        consumer: Sender<LineSegment<2>>,
    ) -> Result<(), std::io::Error>;
}

#[derive(Debug)]
pub struct AABox {
    pub bounds: Bounds<3>,
}

#[async_trait]
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
        // We consider the AABB as the intersection of
        // three slabs, one in each direction.

        // Times when the ray hits the three planes through the min
        let n: Point<3> = (self.bounds.min - ray.origin) / ray.direction;
        // Times when the ray hits the three planes through the max
        let f: Point<3> = (self.bounds.max - ray.origin) / ray.direction;

        // Times when the ray enters the slabs in each direction
        let nn = Point::componentwise_min(f, n);
        // Times when the ray leaves the slabs
        let ff = Point::componentwise_max(f, n);

        // the time we hit the box is after we've hit all the slabs
        let t0 = nn.max();
        // the time we leave the box is after we've left any the slabs
        let t1 = ff.min();

        if t0 < 1e-3 && t1 > 1e-3 {
            return Some(Hit(self, t1));
        }
        if t0 >= 1e-3 && t0 < t1 {
            return Some(Hit(self, t0));
        }
        None
    }

    fn contains(&self, x: Point<3>, tol: f32) -> bool {
        self.bounds.expand_by(tol).contains(x)
    }

    fn bounds(&self) -> Bounds<3> {
        self.bounds
    }

    fn compile(&self) {}

    async fn render(
        &self,
        camera: &Camera,
        occlusion_info: &OcclusionInfo,
        consumer: Sender<LineSegment<2>>,
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
                if o1 && o2 {
                    continue;
                }
                if !o1 && !o2 {
                    consumer.send(ls_screen).await.unwrap();
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

#[derive(Debug)]
pub struct Sphere {
    pub center: Point<3>,
    pub radius: f32,
}

#[async_trait]
impl Shape for Sphere {
    fn paths(&self) -> Vec<PolyLine<3>> {
        let mut result = vec![];
        let nz = 20;
        let ns = 80;
        for iz in 0..nz {
            // [0,1]
            let xi = (iz as f32 + 0.5) / (nz as f32);
            // [-1,1]
            let xi = 2.0 * xi - 1.0;
            let z = self.center.vs[2] + xi * self.radius;
            let r = (1.0 - xi * xi).sqrt() * self.radius;
            let ps = (0..=ns)
                .map(|i| 2.0 * std::f32::consts::PI * (i as f32) / (ns as f32))
                .map(|th| self.center + p3(r * th.cos(), r * th.sin(), z))
                .collect();
            result.push(PolyLine { ps });
        }
        result
    }

    fn intersect(&self, ray: &Ray) -> Option<Hit> {
        // Sphere is ||x-c||^2 = r^2
        // Ray is x = o + t u
        // <o-c + t u, o-c +tu> = r^2
        // d = o-c
        // <d + t u, d + t u> = r^2
        // <d,d> + 2<d,u> t + <u,u> t^2  = r^2
        //
        // quadratic formulae
        //
        // t = {-2 <d,u> +/- sqrt( 4<d,u>^2 - 4 (<d,d>-r^2) <u,u>)} / 2 <u,u>
        // t = {-<d,u> +/- sqrt(<d,u>^2 - (<d,d>-r^2)<u,u>)} / <u,u>

        let u = ray.direction;
        let d = ray.origin - self.center;
        let du = d.dot(u);
        let d2 = d.dot(d);
        let u2 = u.dot(u);
        let r2 = (self.radius * 0.995) * (self.radius * 0.995);
        let discr = du * du - (d2 - r2) * u2;

        if discr < 0.0 {
            return None;
        }

        let tp = (-du + discr.sqrt()) / u2;
        let tn = (-du - discr.sqrt()) / u2;
        if tn > 1e-3 {
            return Some(Hit(self, tn));
        }
        if tp > 1e-3 {
            return Some(Hit(self, tp));
        }
        None
    }

    fn contains(&self, x: Point<3>, tol: f32) -> bool {
        let d = x - self.center;
        let d2 = d.dot(d);
        let r2 = (self.radius - tol) * (self.radius - tol);
        d2 < r2
    }

    fn bounds(&self) -> Bounds<3> {
        let r = self.radius;
        let d = p3(r, r, r);
        Bounds {
            min: self.center - d,
            max: self.center + d,
        }
    }

    fn compile(&self) {}

    async fn render(
        &self,
        camera: &Camera,
        occlusion_info: &OcclusionInfo,
        consumer: Sender<LineSegment<2>>,
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
                if o1 && o2 {
                    continue;
                }
                if !o1 && !o2 {
                    consumer.send(ls_screen).await.unwrap();
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
