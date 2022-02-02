use crate::n2::point::p2;
use crate::n2::{lineset::LineSet, point::Point};
use crate::nbase::traits::*;

#[derive(Clone)]
pub struct Tile {
    lines: LineSet,
    bounds: (Point, Point),
}

pub fn make_tile(bounds: (Point, Point), lines: &LineSet) -> Tile {
    let (min, max) = bounds;
    Tile {
        lines: lines
            .clip_by(p2(1.0, 0.0), min.vs[0])
            .clip_by(p2(-1.0, 0.0), -max.vs[0])
            .clip_by(p2(0.0, 1.0), min.vs[1])
            .clip_by(p2(0.0, -1.0), -max.vs[1]),
        bounds,
    }
}

impl Tile {
    pub fn place_at(&self, p: Point, dx: f32, dy: f32) -> LineSet {
        let tile_bounds = self.bounds;
        let z = self.lines.shift_by(tile_bounds.0.neg());
        let w = tile_bounds.1 - tile_bounds.0;
        let z = z.scale(Point::default(), &[dx / w.vs[0], dy / w.vs[1]]);
        z.shift_by(p)
    }
    pub fn flip_x(&self) -> Tile {
        let mid = Point::lerp(0.5, self.bounds.0, self.bounds.1);
        Tile {
            lines: self.lines.scale(mid, &[-1.0, 1.0]),
            bounds: self.bounds,
        }
    }
    pub fn flip_y(&self) -> Tile {
        let mid = Point::lerp(0.5, self.bounds.0, self.bounds.1);
        Tile {
            lines: self.lines.scale(mid, &[1.0, -1.0]),
            bounds: self.bounds,
        }
    }
    pub fn flip_xy(&self) -> Tile {
        let mid = Point::lerp(0.5, self.bounds.0, self.bounds.1);
        Tile {
            lines: self.lines.scale(mid, &[-1.0, -1.0]),
            bounds: self.bounds,
        }
    }
}
