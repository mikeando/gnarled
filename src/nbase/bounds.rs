use crate::nbase::point::Point;

#[derive(Clone, Copy, Debug)]
pub struct Bounds<const N: usize> {
    pub min: Point<N>,
    pub max: Point<N>,
}
