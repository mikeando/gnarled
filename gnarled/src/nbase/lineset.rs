use crate::nbase::bounds::Bounds;
use crate::nbase::point::Point;
use crate::nbase::polyline::PolyLine;
use crate::nbase::traits::*;

#[derive(Clone)]
pub struct LineSet<const N: usize> {
    pub lines: Vec<PolyLine<N, ()>>,
}

impl<const N: usize> LineSet<N> {
    pub fn clip_by(&self, n: Point<N>, v: f32) -> LineSet<N> {
        LineSet {
            lines: self
                .lines
                .iter()
                .flat_map(|line| line.clip_by(n, v).lines)
                .collect(),
        }
    }
}

impl<const N: usize> Boundable<N> for LineSet<N> {
    fn bounds(&self) -> Option<Bounds<N>> {
        self.lines
            .iter()
            .flat_map(|line| line.ps.iter())
            .fold(None, crate::nbase::point::point_extrema)
    }
}

impl<const N: usize> Shiftable<N> for LineSet<N> {
    type Result = LineSet<N>;
    fn shift_by(&self, d: Point<N>) -> Self::Result {
        LineSet {
            lines: self.lines.iter().map(|line| line.shift_by(d)).collect(),
        }
    }
}

impl<const N: usize> Scalable<N> for LineSet<N> {
    type Result = LineSet<N>;
    fn scale(&self, center: Point<N>, scalexy: &[f32; N]) -> Self::Result {
        LineSet {
            lines: self
                .lines
                .iter()
                .map(|line| line.scale(center, scalexy))
                .collect(),
        }
    }
}
