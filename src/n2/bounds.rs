use crate::n2::point::Point;

#[derive(Clone,Copy,Debug)]
pub struct Bounds {
    pub min: Point,
    pub max: Point,
}