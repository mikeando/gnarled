use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;

use crate::n2::point::Point;
use crate::n2::bounds::Bounds;
use crate::n2::lineset::LineSet;
use crate::n2::polyline::PolyLine;
use crate::svg::SVGable;

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

pub trait Shading {
    fn apply(&self, obj: &dyn Shadable, consumer: &mut dyn Consumer);
}

pub trait RandomField2D {
    // Returns a value in 0-1
    fn at(&self, p: Point) -> f32;
}

pub struct ShadingV0 {
    pub anchor: Point,
    pub d: Point,
    pub rand: Box<dyn RandomField2D>,
}

impl Bounds {
    //TODO: This is not working yet!
    pub fn clip(&self, ls: LineSegment) -> Option<LineSegment> {
        Some(ls)
    }
}

pub fn clip_by_mask(lsx: LineSegment, mask: &dyn Mask) -> LineSet {
    // For now we just do a linear split based on the values of the
    // end points. Later we might want to do something cleverer and
    // subdivide the line-segment if large.
    let m0 = mask.mask(lsx.0);
    let m1 = mask.mask(lsx.1);
    if (m0>=0.0) && (m1>=0.0) {
        LineSet {
            lines: vec![PolyLine {
                ps: vec![lsx.0, lsx.1],
            }],
        }
    } else if (m0<0.0) && (m1<0.0) {
        LineSet{ lines:vec![] }
    } else if m0>=0.0 {
        let z = -m0 / (m1-m0);
        let p = Point::lerp(z, lsx.0, lsx.1);
        LineSet{ lines:vec![PolyLine {ps:vec![lsx.0, p]}]}
    } else {
        let z = -m0 / (m1-m0);
        let p = Point::lerp(z, lsx.0, lsx.1);
        LineSet{ lines:vec![PolyLine {ps:vec![p, lsx.1]}]}
    }
}

pub struct LineSegment(Point, Point);

impl LineSegment {
    fn midpoint(&self) -> Point {
        (self.0 + self.1) * 0.5
    }
}

impl Shading for ShadingV0 {
    fn apply(&self, obj: &dyn Shadable, consumer: &mut dyn Consumer) {
        let bounds = obj.bounds();
        let ix_min = ((bounds.min.0 - self.anchor.0) / self.d.0).floor() as i32;
        let ix_max = ((bounds.max.0 - self.anchor.0) / self.d.0).ceil() as i32;

        let iy_min = ((bounds.min.1 - self.anchor.1) / self.d.1).floor() as i32;
        let iy_max = ((bounds.max.1 - self.anchor.1) / self.d.1).ceil() as i32;

        // Now we generate a lot of line-segments.
        for iy in iy_min..=iy_max {
            for ix in ix_min..=ix_max {
                let p0 = self.anchor + self.d * (ix as f32, iy as f32);
                let px = p0 + self.d * (1.0, 0.0);
                let py = p0 + self.d * (0.0, 1.0);

                let lsx = LineSegment(p0, px);
                let lsx = bounds.clip(lsx);
                if let Some(lsx) = lsx {
                    let mid = lsx.midpoint();
                    let p = self.rand.at(mid);
                    let s = obj.weight(mid);

                    if s >= p {
                        let lsx = clip_by_mask(lsx, obj.mask());
                        consumer.add(lsx);
                    }
                }

                let lsy = LineSegment(p0, py);
                let lsy = bounds.clip(lsy);
                if let Some(lsy) = lsy {
                    let mid = lsy.midpoint();
                    let p = self.rand.at(mid);
                    let s = obj.weight(mid);
                    if s >= p {
                        let lsy = clip_by_mask(lsy, obj.mask());
                        consumer.add(lsy);
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
        h.write(&p.0.to_be_bytes());
        h.write(&p.1.to_be_bytes());
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
        (p.0 - (self.0).0) / ((self.1).0 - (self.0).0)
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
    pub shading: Box<dyn Fn(Point)->f32>,
}

impl Shadable for Circle {
    fn bounds(&self) -> Bounds {
        Bounds {
            min: self.center - Point(1.0,1.0) * self.radius,
            max: self.center + Point(1.0,1.0) * self.radius,
        }
    }

    fn mask(&self) -> &dyn Mask {
        self
    }

    fn weight(&self, p: Point) -> f32 {
        (self.shading)((p-self.center)*(1.0/self.radius))
    }
}

impl Mask for Circle {
    fn mask(&self, p: Point) -> f32 {
        let center = self.center;
        let radius = self.radius;
        let d = p - center;
        let d2 = d.dot(d);
        radius*radius - d2
    }
}

pub struct DirectFileSVGConsumer<'a>(pub &'a mut std::fs::File);

impl<'a> Consumer for DirectFileSVGConsumer<'a> {
    fn add(&mut self, p: LineSet) {
        p.to_svg(self.0).unwrap()
    }
}