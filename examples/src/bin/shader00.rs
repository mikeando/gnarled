
use gnarled::n2::polyline::PolyLine;


use gnarled::svg::SVGable;

use gnarled::n2::point::p2;


pub fn main() -> Result<(), std::io::Error> {
    use gnarled::n2::hl::*;
    use std::io::Write;

    let file_name = "shader00.svg";
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