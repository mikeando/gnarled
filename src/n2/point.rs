use crate::n2::bounds::Bounds;

pub type Point = crate::nbase::point::Point<2>;

pub fn p2(x: f32, y: f32) -> Point {
    Point::from([x, y])
}

pub fn point_extrema(v: Option<Bounds>, s: &Point) -> Option<Bounds> {
    if let Some(Bounds { mut min, mut max }) = v {
        if s.vs[0] < min.vs[0] {
            min.vs[0] = s.vs[0];
        } else if s.vs[0] > max.vs[0] {
            max.vs[0] = s.vs[0];
        }
        if s.vs[1] < min.vs[1] {
            min.vs[1] = s.vs[1];
        } else if s.vs[1] > max.vs[1] {
            max.vs[1] = s.vs[1];
        }
        Some(Bounds { min, max })
    } else {
        Some(Bounds { min: *s, max: *s })
    }
}
