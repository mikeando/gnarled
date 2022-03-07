
use gnarled::n2::polyline::PolyLine;

use gnarled::svg::SVGable;

use gnarled::n2::point::p2;
use gnarled::{
    svg::{PolyLineProperties, PolyLineStroke},
};

pub fn main() -> Result<(), std::io::Error> {
    use std::io::Write;

    let file_name = "clip01.svg";
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