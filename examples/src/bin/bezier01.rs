use gnarled::n2::polyline::PolyLine;
use gnarled::nbase::traits::*;

use gnarled::svg::SVGable;

use gnarled::n2::point::p2;
use gnarled::{
    n2::cubic_bezier::{CubicBezierPath, CubicBezierSegment},
    svg::{PolyLineProperties, PolyLineStroke},
};

pub fn main() -> Result<(), std::io::Error> {
    use std::io::Write;

    let file_name = "bezier01.svg";
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
            attributes: (),
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
