use gnarled::n2::polyline::PolyLine;
use gnarled::nbase::point::Point;

use gnarled::svg::SVGable;

use gnarled::n2::point::p2;

pub struct Triangle {
    vertices: [Point<2>; 3],
}

impl Triangle {
    pub fn scale(self, s: f32) -> Triangle {
        let mp = (self.vertices[0] + self.vertices[1] + self.vertices[2]) * (1.0 / 3.0);
        Triangle {
            vertices: self.vertices.map(|t| Point::lerp(s, mp, t)),
        }
    }
    pub fn rotate(self, th: f32) -> Triangle {
        let mp = (self.vertices[0] + self.vertices[1] + self.vertices[2]) * (1.0 / 3.0);
        let vertices = self.vertices.map(|x| x - mp).map(|x| {
            p2(
                x.vs[0] * f32::cos(th) - x.vs[1] * f32::sin(th),
                x.vs[0] * f32::sin(th) + x.vs[1] * f32::cos(th),
            ) + mp
        });
        Triangle { vertices }
    }

    #[allow(non_snake_case)]
    pub fn sierpinski_refine(self) -> [Triangle; 3] {
        // let our initial triangle be ABC
        let A = self.vertices[0];
        let B = self.vertices[1];
        let C = self.vertices[2];

        let AB = Point::lerp(0.5, A, B);
        let BC = Point::lerp(0.5, B, C);
        let CA = Point::lerp(0.5, C, A);

        [
            Triangle {
                vertices: [A, AB, CA],
            }
            .scale(0.95)
            .rotate(0.1),
            Triangle {
                vertices: [B, AB, BC],
            }
            .scale(0.95)
            .rotate(0.1),
            Triangle {
                vertices: [C, CA, BC],
            }
            .scale(0.95)
            .rotate(0.1),
        ]
    }
}

pub fn to_polyline(t: Triangle) -> gnarled::nbase::polyline::PolyLine<2> {
    PolyLine {
        ps: vec![t.vertices[0], t.vertices[1], t.vertices[2], t.vertices[0]],
    }
}

pub fn to_lines(ts: Vec<Triangle>) -> gnarled::nbase::lineset::LineSet<2> {
    let mut ls = gnarled::nbase::lineset::LineSet { lines: vec![] };
    for t in ts {
        ls.lines.push(to_polyline(t))
    }
    ls
}

pub fn main() -> Result<(), std::io::Error> {
    use std::io::Write;

    let file_name = "seir01.svg";
    let mut f = std::fs::File::create(file_name).unwrap();
    writeln!(
        f,
        r#"<svg viewBox="0 0 800 800" xmlns="http://www.w3.org/2000/svg">"#
    )?;

    let c = p2(400., 400.);
    let r = 350f32;

    let theta = 2.0 * std::f32::consts::PI / 3.0;
    let t = Triangle {
        vertices: [
            c + p2(f32::cos(0.0), f32::sin(0.0)) * r,
            c + p2(f32::cos(theta), f32::sin(theta)) * r,
            c + p2(f32::cos(-theta), f32::sin(-theta)) * r,
        ],
    };

    let mut ts = vec![t];
    for _ in 0..7 {
        ts = ts.into_iter().flat_map(|t| t.sierpinski_refine()).collect();
    }

    to_lines(ts).to_svg(&mut f)?;

    writeln!(f, "</svg>")?;

    Ok(())
}