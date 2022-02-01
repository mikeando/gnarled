use crate::n2::point::Point;
use crate::n2::bounds::Bounds;

pub trait Boundable {
    fn bounds(&self) -> Option<Bounds>;
}

pub trait Shiftable {
    type Result;
    fn shift_by(&self, d: Point) -> Self::Result;
}

pub trait Rotatable {
    type Result;
    fn rotate_by(&self, radians: f32, center: Point) -> Self::Result;
}

pub trait Scalable {
    type Result;
    fn scale(&self, center: Point, scalexy: (f32, f32)) -> Self::Result;
}
