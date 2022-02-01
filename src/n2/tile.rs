use crate::n2::{lineset::LineSet, point::Point};
use super::traits::*;


#[derive(Clone)]
pub struct Tile {
    lines: LineSet,
    bounds: (Point, Point),
}

pub fn make_tile(bounds: (Point, Point), lines: &LineSet) -> Tile {
    let (min, max) = bounds;
    Tile {
        lines: lines
            .clip_by(Point(1.0, 0.0), min.0)
            .clip_by(Point(-1.0, 0.0), -max.0)
            .clip_by(Point(0.0, 1.0), min.1)
            .clip_by(Point(0.0, -1.0), -max.1),
        bounds,
    }
}


impl Tile {
    pub fn place_at(&self, p: Point, dx: f32, dy: f32) -> LineSet {
        let tile_bounds = self.bounds;
        let z = self.lines.shift_by(tile_bounds.0.neg());
        let z = z.scale(
            Point(0.0, 0.0),
            (
                dx / ((tile_bounds.1).0 - (tile_bounds.0).0),
                dy / ((tile_bounds.1).1 - (tile_bounds.0).1),
            ),
        );
        z.shift_by(p)
    }
    pub fn flip_x(&self) -> Tile {
        let mx = ((self.bounds.0).0 + (self.bounds.1).0) / 2.0;
        let my = ((self.bounds.0).0 + (self.bounds.1).0) / 2.0;
        Tile {
            lines: self.lines.scale(Point(mx, my), (-1.0, 1.0)),
            bounds: self.bounds,
        }
    }
    pub fn flip_y(&self) -> Tile {
        let mx = ((self.bounds.0).0 + (self.bounds.1).0) / 2.0;
        let my = ((self.bounds.0).0 + (self.bounds.1).0) / 2.0;
        Tile {
            lines: self.lines.scale(Point(mx, my), (1.0, -1.0)),
            bounds: self.bounds,
        }
    }
    pub fn flip_xy(&self) -> Tile {
        let mx = ((self.bounds.0).0 + (self.bounds.1).0) / 2.0;
        let my = ((self.bounds.0).0 + (self.bounds.1).0) / 2.0;
        Tile {
            lines: self.lines.scale(Point(mx, my), (-1.0, -1.0)),
            bounds: self.bounds,
        }
    }
}