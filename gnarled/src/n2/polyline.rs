use crate::n2::{point::Point, traits::Rotatable};

pub type PolyLine = crate::nbase::polyline::PolyLine<2>;

impl Rotatable for PolyLine {
    type Result = PolyLine;
    fn rotate_by(&self, radians: f32, Point { vs: [cx, cy] }: Point) -> PolyLine {
        PolyLine {
            ps: self
                .ps
                .iter()
                .map(|Point { vs: [x, y] }| {
                    let xx = x - cx;
                    let yy = y - cy;
                    let c = radians.cos();
                    let s = radians.sin();
                    let rx = c * xx + s * yy;
                    let ry = -s * xx + c * yy;
                    Point {
                        vs: [rx + cx, ry + cy],
                    }
                })
                .collect(),
        }
    }
}
