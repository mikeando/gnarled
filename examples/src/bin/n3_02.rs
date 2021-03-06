use std::sync::{Arc, Mutex};

use gnarled::n2::polyline::PolyLine;
use gnarled::nbase::line_merger::MegaMerger;
use gnarled::nbase::line_segment::LineSegment;

use gnarled::svg::SVGable;

use gnarled::n3::Camera;
use gnarled::svg::{PolyLineProperties, PolyLineStroke};
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

    let file_name = "n3_02.svg";
    let f = std::fs::File::create(file_name).unwrap();
    let ff = Arc::new(Mutex::new(f));
    writeln!(
        ff.lock().unwrap().deref_mut(),
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
