use crate::n2::polyline::PolyLine;
use crate::n2::point::Point;
use crate::n2::bounds::Bounds;
use crate::n2::traits::*;

#[derive(Clone)]
pub struct LineSet {
    pub lines: Vec<PolyLine>,
}

impl LineSet {
    pub fn clip_by(&self, n: Point, v: f32) -> LineSet {
        LineSet {
            lines: self
                .lines
                .iter()
                .flat_map(|line| line.clip_by(n, v).lines)
                .collect(),
        }
    }
}

impl Boundable for LineSet {
    fn bounds(&self) -> Option<Bounds> {
        self.lines
            .iter()
            .flat_map(|line| line.ps.iter())
            .fold(None, crate::n2::point::point_extrema)
    }
}

impl Shiftable for LineSet {
    type Result = LineSet;
    fn shift_by(&self, d: Point) -> Self::Result {
        LineSet {
            lines: self.lines.iter().map(|line| line.shift_by(d)).collect(),
        }
    }
}

impl Scalable for LineSet {
    type Result = LineSet;
    fn scale(&self, center: Point, scalexy: (f32, f32)) -> Self::Result {
        LineSet {
            lines: self
                .lines
                .iter()
                .map(|line| line.scale(center, scalexy))
                .collect(),
        }
    }
}