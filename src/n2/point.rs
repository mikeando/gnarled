use crate::n2::bounds::Bounds;

pub type Point = crate::nbase::point::Point<2>;

pub fn p2(x: f32, y: f32) -> Point {
    Point::from([x, y])
}
