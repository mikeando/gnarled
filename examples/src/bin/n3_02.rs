use gnarled::n2::polyline::PolyLine;
use gnarled::n3::Consumer;
use gnarled::nbase::polyline::LineSegment;

use gnarled::svg::SVGable;

use gnarled::n3::Camera;
use gnarled::svg::{PolyLineProperties, PolyLineStroke};

pub struct ConsumeToFile<'a>(&'a mut std::fs::File);

impl<'a> Consumer for ConsumeToFile<'a> {
    fn add_linesegment(&mut self, ls: &LineSegment<2>) {
        ls.to_svg(self.0).unwrap()
    }

    fn add_lineset(&mut self, ls: gnarled::nbase::lineset::LineSet<2>) {
        ls.to_svg(self.0).unwrap()
    }
}

pub fn main() -> Result<(), std::io::Error> {
    use gnarled::n3::p3;
    use gnarled::nbase::point::Point;
    use std::io::Write;

    let file_name = "n3_02.svg";
    let mut f = std::fs::File::create(file_name).unwrap();
    writeln!(
        f,
        r#"<svg viewBox="0 0 800 800" xmlns="http://www.w3.org/2000/svg">"#
    )?;

    let mut scene = gnarled::n3::Scene::new();
    let camera: Camera = gnarled::n3::CameraBuilder::builder()
        .canvas(0.0, 0.0, 800.0, 800.0)
        .eye(15.0, 15.0, 15.0)
        .fwd(-1.0, -1.0, -1.0)
        .right(1.0, 0.0, 0.0)
        .up(0.0, 0.0, 1.0)
        .create()
        .unwrap();

    let sphere = gnarled::n3::shape::Sphere {
        center: p3(0.0, 0.0, 0.0),
        radius: 5.0,
    };
    scene.add_primitive(sphere);

    {
        let mut consumer = ConsumeToFile(&mut f);
        scene.render(&camera, &mut consumer)?;
    }

    PolyLine {
        ps: vec![
            Point::from([0.0, 0.0]),
            Point::from([800.0, 0.0]),
            Point::from([800.0, 800.0]),
            Point::from([0.0, 800.0]),
            Point::from([0.0, 0.0]),
        ],
    }
    .to_svg_with_properties(
        &mut f,
        PolyLineProperties {
            stroke: PolyLineStroke::Red,
        },
    )
    .unwrap();

    writeln!(f, "</svg>")?;

    Ok(())
}
