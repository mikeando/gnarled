use crate::n2::point::Point;

pub trait Rotatable {
    type Result;
    fn rotate_by(&self, radians: f32, center: Point) -> Self::Result;
}
