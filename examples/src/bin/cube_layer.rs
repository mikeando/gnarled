use std::sync::{Arc, Mutex};

use gnarled::n2::polyline::PolyLine;
use gnarled::nbase::bounds::Bounds;
use gnarled::nbase::line_merger::MegaMerger;
use gnarled::nbase::line_segment::LineSegment;

use gnarled::svg::SVGable;

use gnarled::n3::Camera;
use gnarled::svg::{PolyLineProperties, PolyLineStroke};
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64Mcg;
use tokio::sync::mpsc::channel;
use tokio::sync::mpsc::error::SendError;
use tokio::task::{JoinError, JoinHandle};

#[derive(Debug)]
pub enum Error {
    JoinError(JoinError),
    SendError,
    IOError(std::io::Error),
}

impl<T> From<SendError<T>> for Error {
    fn from(_: SendError<T>) -> Self {
        Error::SendError
    }
}

impl From<JoinError> for Error {
    fn from(e: JoinError) -> Self {
        Error::JoinError(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IOError(e)
    }
}

fn main() -> Result<(), Error> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async { async_main().await })?;
    Ok(())
}

pub async fn async_main() -> Result<(), Error> {
    use gnarled::n3::p3;
    use gnarled::nbase::point::Point;
    use std::io::Write;
    use std::ops::DerefMut;

    let mut rng: Pcg64Mcg = Pcg64Mcg::seed_from_u64(11);

    let file_name = "cube_layer.svg";
    let f = std::fs::File::create(file_name).unwrap();
    let ff = Arc::new(Mutex::new(f));
    writeln!(
        ff.lock().unwrap().deref_mut(),
        r#"<svg viewBox="0 0 800 800" xmlns="http://www.w3.org/2000/svg">"#
    )?;

    let mut scene = gnarled::n3::Scene::new();

    let eye = p3(15.0, 8.0, 8.0);
    let target = p3(0.0, 0.0, 0.0);
    let fwd = (target - eye).normalize();
    let right = fwd.cross(p3(0.0, 0.0, 1.0)).normalize() * 0.5;
    let up = right.cross(fwd).normalize() * 0.5;

    let camera: Camera = gnarled::n3::CameraBuilder::builder()
        .canvas(0.0, 0.0, 800.0, 800.0)
        .eye(eye.vs[0], eye.vs[1], eye.vs[2])
        .fwd(fwd.vs[0], fwd.vs[1], fwd.vs[2])
        .right(right.vs[0], right.vs[1], right.vs[2])
        .up(up.vs[0], up.vs[1], up.vs[2])
        .create()
        .unwrap();

    for i in -10..=10 {
        for j in -10..=10 {
            let z =
                (rng.gen_range(-0.5..0.5) + rng.gen_range(-0.5..0.5) + rng.gen_range(-0.5..0.5))
                    / 2.0;
            //let h = 1.0 + (10.0 + j as f32)/20.0;
            let h = 1.0;
            let cube = gnarled::n3::shape::AABox {
                bounds: Bounds {
                    min: p3(i as f32 - 0.5, j as f32 - 0.5, z - 0.5 * h),
                    max: p3(i as f32 + 0.5, j as f32 + 0.5, z + 0.5 * h),
                },
            };
            scene.add_primitive(cube);
        }
    }

    {
        let (sender, recver) = channel(100);
        let (merged_sender, merged_recver) = channel(100);

        let renderer = tokio::spawn(async move { scene.render(&camera, sender).await });

        let mm = MegaMerger::new(recver, merged_sender);
        let mm = tokio::spawn(async move { mm.run().await });

        let ff = ff.clone();
        let writer: JoinHandle<Result<(), Error>> = tokio::spawn(async move {
            let mut recver = merged_recver;
            while let Some(ls) = recver.recv().await {
                ls.to_svg(ff.lock().unwrap().deref_mut())?;
            }
            Ok(())
        });
        eprintln!("Awaiting writer...");
        writer.await??;
        mm.await?.unwrap();
        renderer.await??;
    }

    PolyLine {
        ps: vec![
            Point::from([0.0, 0.0]),
            Point::from([800.0, 0.0]),
            Point::from([800.0, 800.0]),
            Point::from([0.0, 800.0]),
            Point::from([0.0, 0.0]),
        ],
        attributes: PolyLineProperties {
            stroke: PolyLineStroke::Red,
        },
    }
    .to_svg(ff.lock().unwrap().deref_mut())
    .unwrap();

    writeln!(ff.lock().unwrap().deref_mut(), "</svg>")?;

    Ok(())
}
