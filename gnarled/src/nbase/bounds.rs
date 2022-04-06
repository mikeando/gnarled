use crate::nbase::point::Point;

use crate::nbase::point::Float;

#[derive(Clone, Copy, Debug)]
pub struct Bounds<const N: usize, F: Float=f32> {
    pub min: Point<N, F>,
    pub max: Point<N, F>,
}

impl<const N: usize> Bounds<N> {
    pub fn expand_by(&self, d: f32) -> Bounds<N> {
        let unit = Point::from([1.0; N]);
        Bounds {
            min: self.min - unit * d,
            max: self.max + unit * d,
        }
    }

    pub fn contains(&self, p: Point<N>) -> bool {
        if (p - self.min).min() < 0.0 {
            return false;
        }
        if (self.max - p).min() < 0.0 {
            return false;
        }
        true
    }
}
