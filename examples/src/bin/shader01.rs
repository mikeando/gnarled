use std::collections::HashMap;

use gnarled::n2::lineset::LineSet;
use gnarled::n2::polyline::PolyLine;

use gnarled::svg::SVGable;
use tokio::sync::mpsc::channel;

use gnarled::n2::point::p2;
use gnarled::nbase::line_merger::{BinningLineMerger, BinningPolyLineMerger, LineMerger};
use tokio::sync::mpsc::error::SendError;
use tokio::task::{JoinHandle, JoinError};

pub struct StoreLinesConsumer {
    linesets: Vec<LineSet>,
}

impl gnarled::n2::hl::Consumer for StoreLinesConsumer {
    fn add(&mut self, p: LineSet) {
        self.linesets.push(p)
    }
}

fn main() -> Result<(), Error> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            async_main().await
        })?;
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
        let circle = Circle {
            center: p2(400.0, 400.0),
            radius: 400.0,
            shading: Box::new(|p| p.dot(p).sqrt() * (p.vs[0] + 1.0) / 2.0),
        };
        let mut store_lines = StoreLinesConsumer { linesets: vec![] };
        s.apply(&circle, &mut store_lines);

        let (input_a, output_a) = channel(100);
        let (input_b, output_b) = channel(100);
        let (input_c, output_c) = channel(100);
        let (input_d,  mut output_d) = channel(100);

        let pusher: JoinHandle<Result<(), Error>> = tokio::spawn(async move {
            for lineset in store_lines.linesets {
                for polyline in lineset.lines {
                    for ls in polyline.line_segments() {
                        input_a.send(ls).await?;
                    }
                }
            }
            Ok(())
        });
        let mut merger = LineMerger {
            input: output_a,
            output: input_b,
            current_line: None,
        };
        let merger = tokio::spawn(async move { merger.run().await });

        let binning_merger = BinningLineMerger {
            input: output_b,
            output: input_c,
            entries: vec![],
            nodes: HashMap::new(),
        };
        let binmerger = tokio::spawn(async move { binning_merger.run().await });

        let polyline_merge = BinningPolyLineMerger {
            input: output_c,
            output: input_d,
            entries: vec![],
            nodes: HashMap::new(),
        };
        let polymerger = tokio::spawn(async move { polyline_merge.run().await });

        let writer = tokio::spawn(async move {
            let mut result = vec![];
            while let Some(ls) = output_d.recv().await {
                result.push(ls);
            }
            result
        });
        pusher.await??;
        merger.await?.unwrap();
        binmerger.await?.unwrap();
        polymerger.await?.unwrap();
        let result = writer.await?;
        for ls in result {
            ls.to_svg(&mut f)?;
        }
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