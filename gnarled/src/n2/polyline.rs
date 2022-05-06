use crate::n2::{point::Point, traits::Rotatable};

pub type PolyLine<A> = crate::nbase::polyline::PolyLine<2, A>;

impl<A> Rotatable for PolyLine<A>
where
    A: Clone,
{
    type Result = PolyLine<A>;
    fn rotate_by(&self, radians: f32, Point { vs: [cx, cy] }: Point) -> PolyLine<A> {
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
            attributes: self.attributes.clone(),
        }
    }
}
