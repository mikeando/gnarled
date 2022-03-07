use crate::nbase::bounds::Bounds;
use crate::nbase::point::Point;
pub trait Boundable<const N: usize> {
    fn bounds(&self) -> Option<Bounds<N>>;
}

pub trait Shiftable<const N: usize> {
    type Result;
    fn shift_by(&self, d: Point<N>) -> Self::Result;
}

pub trait Scalable<const N: usize> {
    type Result;
    fn scale(&self, center: Point<N>, scale: &[f32; N]) -> Self::Result;
}
