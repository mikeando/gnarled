use std::collections::HashMap;

use gnarled::n2::lineset::LineSet;

use gnarled::nbase::polyline::{LineSegment, PolyLine};
use gnarled::svg::SVGable;
use tokio::sync::mpsc::{channel, Receiver, Sender};

use gnarled::n2::point::p2;
use gnarled::nbase::line_merger::{BinningLineMerger, BinningPolyLineMerger, LineMerger};
use tokio::sync::mpsc::error::SendError;
use tokio::task::{JoinError, JoinHandle};
use tokio::try_join;

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

pub struct MegaMerger<const N: usize> {
    line_merger: LineMerger<N>,
    binning_merger: BinningLineMerger<N>,
    polyline_merger: BinningPolyLineMerger<N>,
}

impl<const N: usize> MegaMerger<N> {
    pub fn new(output_a: Receiver<LineSegment<N>>, input_d: Sender<PolyLine<N>>) -> MegaMerger<N> {
        let (input_b, output_b) = channel(100);
        let (input_c, output_c) = channel(100);

        let line_merger = LineMerger {
            input: output_a,
            output: input_b,
            current_line: None,
        };

        let binning_merger = BinningLineMerger {
            input: output_b,
            output: input_c,
            entries: vec![],
            nodes: HashMap::new(),
        };

        let polyline_merger = BinningPolyLineMerger {
            input: output_c,
            output: input_d,
            entries: vec![],
            nodes: HashMap::new(),
        };

        MegaMerger {
            line_merger,
            binning_merger,
            polyline_merger,
        }
    }

    async fn run(self) -> Result<(), ()> {
        let lm = self.line_merger.run();
        let bm = self.binning_merger.run();
        let pm = self.polyline_merger.run();
        try_join!(lm, bm, pm)?;
        Ok(())
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

        let (input_a, output_a) = channel(100);
        let (input_d, mut output_d) = channel(100);

        let pusher = tokio::spawn(async move { s.apply_async(&circle, input_a).await });
        let mm = MegaMerger::new(output_a, input_d);

        let mm = tokio::spawn(async move { mm.run().await });

        let writer = tokio::spawn(async move {
            let mut result = vec![];
            while let Some(ls) = output_d.recv().await {
                result.push(ls);
            }
            result
        });
        pusher.await?;
        mm.await?.unwrap();
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
