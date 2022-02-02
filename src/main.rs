#![feature(trait_alias)]

pub mod point3d;
pub mod svg;

pub mod n2;
pub mod nbase;

use n2::lineset::LineSet;
use n2::polyline::PolyLine;
use n2::traits::Rotatable;
use nbase::traits::*;

use svg::SVGable;

use crate::n2::point::p2;
use crate::n2::tile::make_tile;
use crate::{
    n2::cubic_bezier::{CubicBezierPath, CubicBezierSegment},
    svg::{PolyLineProperties, PolyLineStroke},
};

fn main() -> Result<(), std::io::Error> {
    test_spiral()?;
    test_clip()?;
    test_cbez()?;
    test_shader()?;
    Ok(())
}

fn test_spiral() -> Result<(), std::io::Error> {
    use std::io::Write;
    let file_name = "test.svg";
    let mut f = std::fs::File::create(file_name).unwrap();
    let ps = (0..100)
        .map(|i| (i as f32) * 4.0f32 * std::f32::consts::PI / 100.0f32)
        .map(|t| p2((t * t.cos()) * 40.0, (t * t.sin()) * 40.0))
        .collect::<Vec<_>>();
    let p = PolyLine { ps };
    let bb = p.bounds().unwrap();
    let p = p.shift_by(p2(400., 400.));
    writeln!(
        f,
        r#"<svg viewBox="0 0 800 800" xmlns="http://www.w3.org/2000/svg">"#
    )?;
    let mut spiral_lines = LineSet { lines: vec![] };
    let n_spiral_arms = 16;
    for i in 0..n_spiral_arms {
        let beta = 2.0 * std::f32::consts::PI / (n_spiral_arms as f32);
        let pr = p.rotate_by(beta * (i as f32), p2(400., 400.));
        spiral_lines.lines.push(pr);
    }
    //spiral_lines.to_svg(&mut f)?;
    let mut horz_lines = LineSet { lines: vec![] };
    let n = 20;
    for i in 0..n {
        let dy = (700.0 - 100.0) / (n as f32);
        let y = 100.0 + dy * (0.5 + i as f32);
        let pr = PolyLine {
            ps: (0..800).step_by(10).map(|x| p2(x as f32, y)).collect(),
        };
        horz_lines.lines.push(pr);
    }
    let mut vert_lines = LineSet { lines: vec![] };
    let n = 20;
    for i in 0..n {
        let dx = (700.0 - 100.0) / (n as f32);
        let x = 100.0 + dx * (0.5 + i as f32);
        let pr = PolyLine {
            ps: (0..800).step_by(10).map(|y| p2(x, y as f32)).collect(),
        };
        vert_lines.lines.push(pr);
    }

    let tile_bounds = (p2(200.0, 200.0), p2(600.0, 600.0));
    let hline_tile = make_tile(tile_bounds, &horz_lines);

    let vline_tile = make_tile(tile_bounds, &vert_lines);
    let spiral_tile = make_tile(tile_bounds, &spiral_lines);

    for (i, &tile) in [&spiral_tile, &vline_tile, &hline_tile].iter().enumerate() {
        let output_width = 200.0;

        for (j, dx) in [0.0, 200.0, 400.0, 600.0].iter().enumerate() {
            for (k, dy) in [0.0, 200.0, 400.0, 600.0].iter().enumerate() {
                if i == 0 {
                    let t = match (j % 2 == 0, k % 2 == 0) {
                        (true, true) => tile.clone(),
                        (true, false) => tile.flip_y(),
                        (false, true) => tile.flip_x(),
                        (false, false) => tile.flip_xy(),
                    };
                    let z = t.place_at(p2(*dx, *dy), output_width, output_width);
                    z.to_svg(&mut f)?;
                }
            }
        }
    }
    writeln!(f, "</svg>")?;
    Ok(())
}

pub fn test_clip() -> Result<(), std::io::Error> {
    use std::io::Write;

    let file_name = "test2.svg";
    let mut f = std::fs::File::create(file_name).unwrap();
    writeln!(
        f,
        r#"<svg viewBox="0 0 800 800" xmlns="http://www.w3.org/2000/svg">"#
    )?;
    PolyLine {
        ps: vec![p2(0.0, 400.0), p2(800.0, 400.0)],
    }
    .to_svg(&mut f)?;
    PolyLine {
        ps: vec![p2(0.0, 0.0), p2(800.0, 0.0)],
    }
    .to_svg(&mut f)?;
    PolyLine {
        ps: vec![p2(0.0, 800.0), p2(800.0, 800.0)],
    }
    .to_svg(&mut f)?;

    let ys = &[0.0f32, 200.0, 600.0, 800.0];
    PolyLine {
        ps: ys.iter().map(|y| p2(300.0, *y)).collect(),
    }
    .clip_by(p2(0.0, 1.0), 400.0)
    .to_svg_with_properties(
        &mut f,
        PolyLineProperties {
            stroke: PolyLineStroke::Red,
        },
    )?;
    PolyLine {
        ps: ys.iter().map(|y| p2(500.0, 800.0 - *y)).collect(),
    }
    .clip_by(p2(0.0, 1.0), 400.0)
    .to_svg_with_properties(
        &mut f,
        PolyLineProperties {
            stroke: PolyLineStroke::Green,
        },
    )?;

    writeln!(f, "</svg>")?;

    Ok(())
}

pub fn test_cbez() -> Result<(), std::io::Error> {
    use std::io::Write;

    let file_name = "test3.svg";
    let mut f = std::fs::File::create(file_name).unwrap();
    writeln!(
        f,
        r#"<svg viewBox="0 0 800 800" xmlns="http://www.w3.org/2000/svg">"#
    )?;

    let b1 = CubicBezierPath {
        ps: vec![
            p2(100.0, 300.0),
            p2(200.0, 300.0),
            p2(300.0, 200.0),
            p2(300.0, 100.0),
        ],
    };

    let s1: CubicBezierSegment = b1.segment(0);
    let dt = 1.0 / (5 - 1) as f32;
    for i in 0..5 {
        PolyLine {
            ps: vec![p2(0.0, 0.0), s1.value(i as f32 * dt)],
        }
        .to_svg(&mut f)?;
    }

    b1.to_svg_with_properties(
        &mut f,
        PolyLineProperties {
            stroke: PolyLineStroke::Red,
        },
    )?;

    let (sa, sb) = s1.split(0.5);
    for i in 1..5 {
        let i = i as f32;
        CubicBezierPath {
            ps: sa.shift_by(p2(3.0 * i, 3.0 * i)).ps.to_vec(),
        }
        .to_svg(&mut f)?;
    }
    for i in 1..5 {
        let i = i as f32;
        CubicBezierPath {
            ps: sb.shift_by(p2(-3.0 * i, -3.0 * i)).ps.to_vec(),
        }
        .to_svg(&mut f)?;
    }

    writeln!(f, "</svg>")?;

    Ok(())
}

pub fn test_shader() -> Result<(), std::io::Error> {
    use crate::n2::hl::*;
    use std::io::Write;

    let file_name = "test_shader.svg";
    let mut f = std::fs::File::create(file_name).unwrap();
    writeln!(
        f,
        r#"<svg viewBox="0 0 800 800" xmlns="http://www.w3.org/2000/svg">"#
    )?;

    let rand = Box::new(DefaultHasherRandField2D {});

    let s = ShadingV0 {
        anchor: p2(0.0, 0.0),
        d: p2(5.0, 5.0),
        rand,
    };

    {
        //let whole_aaquad = AxisAlignedQuad(Point(0.0, 0.0), Point(800.0, 800.0));
        let circle = Circle {
            center: p2(400.0, 400.0),
            radius: 400.0,
            shading: Box::new(|p| p.dot(p).sqrt() * (p.vs[0] + 1.0) / 2.0),
        };
        let mut direct_write_to_file = DirectFileSVGConsumer(&mut f);
        s.apply(&circle, &mut direct_write_to_file);
    }

    let ps = (0..101)
        .map(|i| (i as f32) * 2.0f32 * std::f32::consts::PI / 100.0f32)
        .map(|t| p2((400.0 * t.cos()) + 400.0, (400.0 * t.sin()) + 400.0))
        .collect::<Vec<_>>();
    let p = PolyLine { ps };
    p.to_svg(&mut f)?;

    writeln!(f, "</svg>")?;

    Ok(())
}

#[cfg(test)]
pub mod tests {}
