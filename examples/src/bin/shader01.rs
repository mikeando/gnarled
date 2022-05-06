use std::collections::HashMap;
use std::ops::DerefMut;
use std::sync::{Arc, Mutex};

use gnarled::nbase::line_segment::LineSegment;
use gnarled::nbase::polyline::PolyLine;

use gnarled::svg::SVGable;
use tokio::sync::mpsc::{channel, Receiver, Sender};

use gnarled::n2::point::p2;
use gnarled::nbase::line_merger::{
    BinningLineMerger, BinningPolyLineMerger, LineMerger, MegaMerger,
};
use tokio::sync::mpsc::error::SendError;
use tokio::task::{JoinError, JoinHandle};
use tokio::try_join;

fn main() -> Result<(), Error> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async { async_main().await })?;
    Ok(())
}

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

pub async fn async_main() -> Result<(), Error> {
    use gnarled::n2::hl::*;
    use std::io::Write;

    let file_name = "shader01.svg";
    let f = std::fs::File::create(file_name).unwrap();
    let ff = Arc::new(Mutex::new(f));

    writeln!(
        ff.lock().unwrap(),
        r#"<svg viewBox="0 0 800 800" xmlns="http://www.w3.org/2000/svg">"#
    )?;

    let rand = Box::new(DefaultHasherRandField2D {});

    let s = ShadingV0 {
        anchor: p2(0.0, 0.0),
        d: p2(5.0, 5.0),
        rand,
    };

    {
        let circle = Circle {
            center: p2(400.0, 400.0),
            radius: 400.0,
            shading: Box::new(|p| p.dot(p).sqrt() * (p.vs[0] + 1.0) / 2.0),
        };

        let (input_a, output_a) = channel(100);
        let (input_d, mut output_d) = channel(100);

        let pusher = tokio::spawn(async move { s.apply_async(&circle, input_a).await });
        let mm = MegaMerger::new(output_a, input_d);

        let mm = tokio::spawn(async move { mm.run().await });

        let ff = ff.clone();
        let writer: JoinHandle<Result<(), Error>> = tokio::spawn(async move {
            while let Some(ls) = output_d.recv().await {
                ls.to_svg(ff.lock().unwrap().deref_mut())?;
            }
            Ok(())
        });
        pusher.await?;
        mm.await?.unwrap();
        writer.await??;
    }

    let ps = (0..101)
        .map(|i| (i as f32) * 2.0f32 * std::f32::consts::PI / 100.0f32)
        .map(|t| p2((400.0 * t.cos()) + 400.0, (400.0 * t.sin()) + 400.0))
        .collect::<Vec<_>>();
    let p = PolyLine { ps, attributes: () };
    p.to_svg(ff.lock().unwrap().deref_mut())?;

    writeln!(ff.lock().unwrap().deref_mut(), "</svg>")?;

    Ok(())
}
