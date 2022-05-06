use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;

use async_trait::async_trait;
use tokio::sync::mpsc::Sender;

use crate::n2::bounds::Bounds;
use crate::n2::lineset::LineSet;
use crate::n2::point::Point;
use crate::n2::polyline::PolyLine;
use crate::nbase::line_segment::LineSegment;
use crate::svg::SVGable;

use crate::n2::point::p2;

pub trait Mask {
    fn mask(&self, p: Point) -> f32;
}

pub trait Shadable {
    fn mask(&self) -> &dyn Mask;
    fn bounds(&self) -> Bounds;
    fn weight(&self, p: Point) -> f32;
}

pub trait Consumer {
    fn add(&mut self, p: LineSet);
}

#[async_trait]
pub trait Shading {
    async fn apply_async(
        &self,
        obj: &(dyn Shadable + Send + Sync),
        consumer: Sender<LineSegment<2>>,
    );
}

pub trait RandomField2D {
    // Returns a value in 0-1
    fn at(&self, p: Point) -> f32;
}

pub struct ShadingV0 {
    pub anchor: Point,
    pub d: Point,
    pub rand: Box<dyn RandomField2D + Send + Sync>,
}

impl Bounds {
    //TODO: This is not working yet!
    pub fn clip(&self, ls: LineSegment<2>) -> Option<LineSegment<2>> {
        Some(ls)
    }
}

pub fn clip_by_mask(lsx: LineSegment<2>, mask: &dyn Mask) -> LineSet {
    // For now we just do a linear split based on the values of the
    // end points. Later we might want to do something cleverer and
    // subdivide the line-segment if large.
    let m0 = mask.mask(lsx.ps[0]);
    let m1 = mask.mask(lsx.ps[1]);
    if (m0 >= 0.0) && (m1 >= 0.0) {
        LineSet {
            lines: vec![PolyLine {
                ps: vec![lsx.ps[0], lsx.ps[1]],
            }],
        }
    } else if (m0 < 0.0) && (m1 < 0.0) {
        LineSet { lines: vec![] }
    } else if m0 >= 0.0 {
        let z = -m0 / (m1 - m0);
        let p = Point::lerp(z, lsx.ps[0], lsx.ps[1]);
        LineSet {
            lines: vec![PolyLine {
                ps: vec![lsx.ps[0], p],
            }],
        }
    } else {
        let z = -m0 / (m1 - m0);
        let p = Point::lerp(z, lsx.ps[0], lsx.ps[1]);
        LineSet {
            lines: vec![PolyLine {
                ps: vec![p, lsx.ps[1]],
            }],
        }
    }
}

#[async_trait]
impl Shading for ShadingV0 {
    async fn apply_async(
        &self,
        obj: &(dyn Shadable + Send + Sync),
        consumer: Sender<LineSegment<2>>,
    ) {
        let bounds = obj.bounds();
        let s = self.d.map(|x| 1.0f32 / x);
        let i_min = (bounds.min - self.anchor) * s;
        let i_max = (bounds.max - self.anchor) * s;

        let ix_min = i_min.vs[0].floor() as i32;
        let ix_max = i_max.vs[0].ceil() as i32;

        let iy_min = i_min.vs[1].floor() as i32;
        let iy_max = i_max.vs[1].ceil() as i32;

        // Now we generate a lot of line-segments.
        for iy in iy_min..=iy_max {
            for ix in ix_min..=ix_max {
                let p0 = self.anchor + self.d * p2(ix as f32, iy as f32);
                let px = p0 + self.d * p2(1.0, 0.0);
                let py = p0 + self.d * p2(0.0, 1.0);

                let lsx = LineSegment::new(p0, px);
                let lsx = bounds.clip(lsx);
                if let Some(lsx) = lsx {
                    let mid = lsx.midpoint();
                    let p = self.rand.at(mid);
                    let s = obj.weight(mid);

                    if s >= p {
                        let lsx = clip_by_mask(lsx, obj.mask());
                        for pl in lsx.lines {
                            for ls in pl.line_segments() {
                                consumer.send(ls).await.unwrap();
                            }
                        }
                    }
                }

                let lsy = LineSegment::new(p0, py);
                let lsy = bounds.clip(lsy);
                if let Some(lsy) = lsy {
                    let mid = lsy.midpoint();
                    let p = self.rand.at(mid);
                    let s = obj.weight(mid);
                    if s >= p {
                        let lsy = clip_by_mask(lsy, obj.mask());
                        for pl in lsy.lines {
                            for ls in pl.line_segments() {
                                consumer.send(ls).await.unwrap();
                            }
                        }
                    }
                }
            }
        }
    }
}

pub struct DefaultHasherRandField2D {}

impl RandomField2D for DefaultHasherRandField2D {
    fn at(&self, p: Point) -> f32 {
        let mut h = DefaultHasher::new();
        h.write(&p.vs[0].to_be_bytes());
        h.write(&p.vs[1].to_be_bytes());
        let v = h.finish();
        // We use a resolution of 4096 values
        let v = v % 4096;
        v as f32 / 4095.0
    }
}

pub struct AxisAlignedQuad(pub Point, pub Point);

impl Shadable for AxisAlignedQuad {
    fn bounds(&self) -> Bounds {
        Bounds {
            min: self.0,
            max: self.1,
        }
    }

    fn mask(&self) -> &dyn Mask {
        self
    }

    fn weight(&self, p: Point) -> f32 {
        (p.vs[0] - (self.0).vs[0]) / ((self.1).vs[0] - (self.0).vs[0])
    }
}

impl Mask for AxisAlignedQuad {
    fn mask(&self, p: Point) -> f32 {
        let center = (self.0 + self.1) * 0.5;
        let radius = (self.1 - self.0) * 0.5;
        let d = radius - (p - center).abs();
        d.min()
    }
}

pub struct Circle {
    pub center: Point,
    pub radius: f32,
    pub shading: Box<dyn Fn(Point) -> f32 + Send + Sync>,
}

impl Shadable for Circle {
    fn bounds(&self) -> Bounds {
        Bounds {
            min: self.center - Point { vs: [1.0, 1.0] } * self.radius,
            max: self.center + Point { vs: [1.0, 1.0] } * self.radius,
        }
    }

    fn mask(&self) -> &dyn Mask {
        self
    }

    fn weight(&self, p: Point) -> f32 {
        (self.shading)((p - self.center) * (1.0 / self.radius))
    }
}

impl Mask for Circle {
    fn mask(&self, p: Point) -> f32 {
        let center = self.center;
        let radius = self.radius;
        let d = p - center;
        let d2 = d.dot(d);
        radius * radius - d2
    }
}

pub struct DirectFileSVGConsumer<'a>(pub &'a mut std::fs::File);

impl<'a> Consumer for DirectFileSVGConsumer<'a> {
    fn add(&mut self, p: LineSet) {
        p.to_svg(self.0).unwrap()
    }
}
