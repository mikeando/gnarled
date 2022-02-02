use crate::n2::point::Point;
use crate::n2::traits::*;

pub type CubicBezierSegment = crate::nbase::cubic_bezier::CubicBezierSegment<2>;
pub type CubicBezierPath = crate::nbase::cubic_bezier::CubicBezierPath<2>;

impl Shiftable for CubicBezierPath {
    type Result = CubicBezierPath;

    fn shift_by(&self, d: Point) -> Self::Result {
        CubicBezierPath {
            ps: self.ps.iter().map(|p| *p + d).collect(),
        }
    }
}

impl Shiftable for CubicBezierSegment {
    type Result = CubicBezierSegment;

    fn shift_by(&self, d: Point) -> Self::Result {
        CubicBezierSegment {
            ps: self.ps.map(|p| p + d),
        }
    }
}
