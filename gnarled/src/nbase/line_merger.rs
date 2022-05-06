use std::collections::HashMap;

use tokio::{
    sync::mpsc::{channel, Receiver, Sender},
    try_join,
};

use crate::nbase::polyline::PolyLine;

use self::merge_and_deduplicate::BinningLineMergerAndDeduplicator;

use super::line_segment::LineSegment;

pub struct LineMerger<const N: usize> {
    pub input: Receiver<LineSegment<N, ()>>,
    pub output: Sender<LineSegment<N, ()>>,
    pub current_line: Option<LineSegment<N, ()>>,
}

#[allow(non_snake_case)]
fn merge<const N: usize>(
    ls1: &LineSegment<N, ()>,
    ls2: &LineSegment<N, ()>,
) -> Option<LineSegment<N, ()>> {
    let x0 = ls1.ps[0];
    let x1 = ls1.ps[1];
    let z = x1 - ls2.ps[0];
    let z2 = z.dot(z);
    if z2 > 0.01 {
        return None;
    }
    let x2 = ls2.ps[1];

    let u = x1 - x0;
    let d = x2 - x0;
    let d2 = d.dot(d);
    let u2 = u.dot(u);
    if u2 == 0.0 {
        return Some(ls2.clone());
    }
    let du = d.dot(u);
    let e2 = d2 - (du * du) / u2;

    // Make sure we're not going back on ourselves.
    if du < u2 {
        return None;
    }
    if e2 <= 0.01 {
        // Make the last point an extension of
        // original segment. (To avoid the line
        // segment slowly bending away from
        // its original direction)
        // x2 = x0 + L u + e for some L, and <e,u>=0
        // =>  d = L u + e
        // <d,u> = L <u,u>
        let L = du / u2;
        let x2_new = x0 + u * L;
        Some(LineSegment {
            ps: [ls1.ps[0], x2_new],
            attributes: (),
        })
    } else {
        None
    }
}

fn merge_pl<const N: usize>(
    pl1: &PolyLine<N, ()>,
    pl2: &PolyLine<N, ()>,
) -> Option<PolyLine<N, ()>> {
    let z = *pl1.ps.last().unwrap() - *pl2.ps.first().unwrap();
    let z2 = z.dot(z);
    if z2 > 0.01 {
        return None;
    }

    return Some(PolyLine {
        ps: pl1.ps.iter().chain(pl2.ps[1..].iter()).cloned().collect(),
        attributes: (),
    });
}

impl<const N: usize> LineMerger<N> {
    pub fn new(
        input: Receiver<LineSegment<N, ()>>,
        output: Sender<LineSegment<N, ()>>,
    ) -> LineMerger<N> {
        LineMerger {
            input,
            output,
            current_line: None,
        }
    }

    // This is mut self so that self is dropped
    // at the end, closing the output channel.
    pub async fn run(mut self) -> Result<(), ()> {
        while let Some(ls) = self.input.recv().await {
            match self.current_line.take() {
                Some(current_line) => {
                    if let Some(lm) = merge(&current_line, &ls) {
                        self.current_line = Some(lm);
                    } else {
                        self.output.send(current_line).await.unwrap();
                        self.current_line = Some(ls);
                    }
                }
                None => {
                    self.current_line = Some(ls);
                }
            }
        }
        if let Some(current_line) = self.current_line.take() {
            self.output.send(current_line).await.unwrap();
        }
        Ok(())
    }
}

#[derive(PartialEq, Eq)]
pub enum StartOrEnd {
    Start,
    End,
}

mod merge_and_deduplicate {
    use std::{
        future::Future,
        sync::{
            atomic::{AtomicU32, Ordering},
            Arc, Mutex,
        },
    };

    use tokio::io::Interest;

    use crate::nbase::point::Point;

    use super::*;

    #[derive(Hash, PartialEq, Eq, Clone, Copy, Debug)]
    pub struct BinIndex<const N: usize> {
        pub n: [i32; N],
        pub p: [i32; N],
    }

    pub struct AugmentedLineSegment<const N: usize> {
        pub t0: f32,
        pub t1: f32,
        // In theory these are not needed....
        pub x0: Point<N>,
        pub x1: Point<N>,
    }

    pub struct IntervalSet<const N: usize> {
        // TODO: Using a vector is inefficient, we should use some kind of tree.
        // intervals are non-overlapping and sorted by t0
        pub intervals: Vec<AugmentedLineSegment<N>>,
    }

    impl<const N: usize> IntervalSet<N> {
        pub fn add_interval(&mut self, interval: AugmentedLineSegment<N>) {
            // Easy cases first.
            // if there are no intervals, just add the interval.
            if self.intervals.is_empty() {
                self.intervals.push(interval);
                return;
            }
            // if ts is completely before the first interval,
            // just add it to the front.
            if interval.t1 < self.intervals[0].t0 {
                self.intervals.insert(0, interval);
                return;
            }
            // if ts is completely after the last interval,
            // just add it to the end.
            if interval.t0 > self.intervals.last().unwrap().t1 {
                self.intervals.push(interval);
                return;
            }

            let mut result = vec![];
            let mut activeX = Some(interval);

            // take ownership of self.intervals
            let mut intervals = self.intervals.drain(..).collect::<Vec<_>>();

            for current in intervals {
                // TODO: Make this configurable.
                let eps = 0.1;

                match &activeX {
                    Some(active) => {
                        // if the current interval is completely before the active interval,
                        // just add it to the result.
                        if current.t1 < active.t0 - eps {
                            result.push(current);
                            continue;
                        }
                        // if the active interval is completely before the current interval,
                        // just add them both to the result, and deactivate.
                        if current.t0 > active.t1 + eps {
                            let active = activeX.take().unwrap();
                            result.push(active);
                            result.push(current);
                            continue;
                        }

                        // if the active interval completely contains the current interval,
                        // just add the active interval to the result.
                        if active.t0 <= current.t0 && current.t1 <= active.t1 {
                            let active = activeX.take().unwrap();
                            result.push(active);
                            continue;
                        }

                        // if the current interval completely contains the active interval
                        // we can just skip it.
                        if current.t0 <= active.t0 && active.t1 <= current.t1 {
                            result.push(current);
                            activeX = None;
                            continue;
                        }

                        // they must overlap. We must merge them.
                        let (t0, x0) = if active.t0 <= current.t0 {
                            (active.t0, active.x0)
                        } else {
                            (current.t0, current.x0)
                        };

                        // We're still active if the max is from the active interval
                        if active.t1 >= current.t1 {
                            activeX = Some(AugmentedLineSegment {
                                t0,
                                t1: active.t1,
                                x0,
                                x1: active.x1,
                            });
                        } else {
                            activeX = None;
                            result.push(AugmentedLineSegment {
                                t0,
                                t1: current.t1,
                                x0,
                                x1: current.x1,
                            });
                        };
                    }
                    None => {
                        result.push(current);
                    }
                }
            }
            // If we still have an active interval, we need to add it to the result.
            if let Some(active) = activeX {
                result.push(active);
            }

            self.intervals = result;
        }

        pub fn from_interval(interval: AugmentedLineSegment<N>) -> Self {
            IntervalSet {
                intervals: vec![interval],
            }
        }
    }

    // Both n and -n are valid directions for a line
    // segment, so we insert the line segment into one
    // as a Bin value and add the Reverse to the other.
    pub enum BinValue<const N: usize> {
        Reverse,
        Bin(IntervalSet<N>),
    }

    impl<const N: usize> BinValue<N> {
        pub fn as_mut_bin(&mut self) -> Option<&mut IntervalSet<N>> {
            match self {
                BinValue::Bin(b) => Some(b),
                _ => None,
            }
        }
    }

    // This implementation is based on representing a line segment as
    // a section of an infinite line, where infinite lines are defined
    // by their direction and nearest point to the origin.
    // Then we bin based on the line, and store the time segments.
    pub struct DeduplicatorCore<const N: usize> {
        pub entries: HashMap<BinIndex<N>, BinValue<N>>,
        pub n_scale: f32,
        pub p_scale: f32,
    }

    impl<const N: usize> DeduplicatorCore<N> {
        pub fn new(n_scale: f32, p_scale: f32) -> Self {
            DeduplicatorCore {
                entries: HashMap::new(),
                n_scale,
                p_scale,
            }
        }

        pub fn linesegment_to_npt_form(ls: LineSegment<N, ()>) -> (Point<N>, Point<N>, (f32, f32)) {
            let n = (ls.ps[1] - ls.ps[0]).normalize();
            // p is the closest point on the line to the origin.
            let p = ls.ps[0] - n * n.dot(ls.ps[0]);

            let t0 = n.dot(ls.ps[0]);
            let t1 = n.dot(ls.ps[1]);
            (n, p, (t0, t1))
        }

        pub fn npt_to_linesegment(n: Point<N>, p: Point<N>, ts: (f32, f32)) -> LineSegment<N, ()> {
            let ls = LineSegment {
                ps: [p + n * ts.0, p + n * ts.1],
                attributes: (),
            };
            ls
        }

        pub fn np_to_bin_index(&self, n: Point<N>, p: Point<N>) -> BinIndex<N> {
            // We have to scale n and p then quantize them to integers.
            let ns = n.vs.map(|x| (x * self.n_scale).round() as i32);
            let ps = p.vs.map(|x| (x * self.p_scale).round() as i32);
            BinIndex { n: ns, p: ps }
        }

        pub fn bin_index_to_np(&self, bin_index: BinIndex<N>) -> (Point<N>, Point<N>) {
            let inv_n_scale = 1.0 / self.n_scale;
            let inv_p_scale = 1.0 / self.p_scale;
            let n = Point::from(bin_index.n.map(|x| x as f32 * inv_n_scale));
            let p = Point::from(bin_index.p.map(|x| x as f32 * inv_p_scale));
            (n, p)
        }

        pub fn add_linesegment(&mut self, ls: LineSegment<N, ()>) {
            // Find the bin for the linesegment.
            let (n, p, ts) = Self::linesegment_to_npt_form(ls.clone());
            let bin_index = self.np_to_bin_index(n, p);
            // get the bin and check if it is reversed.
            let bin = self.entries.get_mut(&bin_index);
            let bin = match bin {
                Some(BinValue::Reverse) => {
                    // Get the same bin with the normal reversed
                    drop(bin);
                    let rev_bin_index = self.np_to_bin_index(-n, p);
                    // And adjust (t0,t1)
                    // We have to reverse the line segment.
                    let ts = (-ts.1, -ts.0);
                    let interval = AugmentedLineSegment {
                        t0: ts.0,
                        t1: ts.1,
                        x0: ls.ps[1],
                        x1: ls.ps[0],
                    };
                    self.entries
                        .get_mut(&rev_bin_index)
                        .unwrap()
                        .as_mut_bin()
                        .unwrap()
                        .add_interval(interval);
                }
                Some(BinValue::Bin(bin)) => {
                    let interval = AugmentedLineSegment {
                        t0: ts.0,
                        t1: ts.1,
                        x0: ls.ps[0],
                        x1: ls.ps[1],
                    };

                    bin.add_interval(interval);
                }
                None => {
                    let interval = AugmentedLineSegment {
                        t0: ts.0,
                        t1: ts.1,
                        x0: ls.ps[0],
                        x1: ls.ps[1],
                    };
                    // We have to create a new bin and the reverse entry
                    self.entries.insert(
                        bin_index,
                        BinValue::Bin(IntervalSet::from_interval(interval)),
                    );
                    let rev_bin_index = self.np_to_bin_index(-n, p);
                    self.entries.insert(rev_bin_index, BinValue::Reverse);
                }
            };
        }

        pub async fn on_all_segments_async<F, R, E>(&self, mut f: F) -> Result<(), E>
        where
            F: FnMut(LineSegment<N, ()>) -> R,
            R: Future<Output = Result<(), E>>,
        {
            for (bin_index, v) in &self.entries {
                let bin = match v {
                    BinValue::Reverse => continue,
                    BinValue::Bin(bin) => bin,
                };
                let (n, p) = self.bin_index_to_np(*bin_index);

                for interval in &bin.intervals {
                    let ls = LineSegment {
                        ps: [interval.x0, interval.x1],
                        attributes: (),
                    };
                    f(ls).await?;
                }
            }
            Ok(())
        }

        pub fn on_all_segments<F, E>(&self, mut f: F) -> Result<(), E>
        where
            F: FnMut(LineSegment<N, ()>) -> Result<(), E>,
        {
            for (bin_index, v) in &self.entries {
                let bin = match v {
                    BinValue::Reverse => continue,
                    BinValue::Bin(bin) => bin,
                };
                let (n, p) = self.bin_index_to_np(*bin_index);

                for interval in &bin.intervals {
                    let ls = LineSegment {
                        ps: [interval.x0, interval.x1],
                        attributes: (),
                    };
                    f(ls)?;
                }
            }
            Ok(())
        }
    }

    pub struct BinningLineMergerAndDeduplicator<const N: usize> {
        pub input: Receiver<LineSegment<N, ()>>,
        pub output: Sender<LineSegment<N, ()>>,
        pub core: DeduplicatorCore<N>,
    }

    impl<const N: usize> BinningLineMergerAndDeduplicator<N> {
        pub fn new(
            input: Receiver<LineSegment<N, ()>>,
            output: Sender<LineSegment<N, ()>>,
            n_scale: f32,
            p_scale: f32,
        ) -> Self {
            BinningLineMergerAndDeduplicator {
                input,
                output,
                core: DeduplicatorCore::new(n_scale, p_scale),
            }
        }

        pub async fn run(mut self) -> Result<(), ()> {
            while let Some(mut ls) = self.input.recv().await {
                self.core.add_linesegment(ls);
            }
            self.core
                .on_all_segments_async(|ls| async { self.output.send(ls).await })
                .await
                .unwrap();
            Ok(())
        }
    }
}

pub struct BinningLineMerger<const N: usize> {
    pub input: Receiver<LineSegment<N, ()>>,
    pub output: Sender<LineSegment<N, ()>>,
    pub entries: Vec<Option<LineSegment<N, ()>>>,
    pub nodes: HashMap<[usize; N], Vec<Option<(StartOrEnd, usize)>>>,
}

impl<const N: usize> BinningLineMerger<N> {
    pub fn new(
        input: Receiver<LineSegment<N, ()>>,
        output: Sender<LineSegment<N, ()>>,
    ) -> BinningLineMerger<N> {
        BinningLineMerger {
            input,
            output,
            entries: vec![],
            nodes: HashMap::new(),
        }
    }

    pub async fn run(mut self) -> Result<(), ()> {
        while let Some(mut ls) = self.input.recv().await {
            // Find the bin for the start vertex
            // Remove matching line-segment
            // join

            let mut start_idx = ls.ps[0].vs.map(|i| (i * 100.0) as usize);
            if let Some(bin) = self.nodes.get_mut(&start_idx) {
                for (i, e) in bin.iter().enumerate() {
                    if let Some((start_or_end, ls_index)) = e {
                        if let Some(prefix) = &self.entries[*ls_index] {
                            let prefix = if *start_or_end == StartOrEnd::Start {
                                prefix.reverse()
                            } else {
                                prefix.clone()
                            };
                            if let Some(lm) = merge(&prefix, &ls) {
                                // If we've found a prefix we need to remove it from the lists.
                                // And update the current segment.
                                ls = lm;
                                start_idx = ls.ps[0].vs.map(|i| (i * 100.0) as usize);
                                self.entries[*ls_index] = None;
                                bin[i] = None;
                                break;
                            }
                        }
                    }
                }
            }

            // Find the bin for the end vertex
            // Remove matching line-segment
            // join
            let mut end_idx = ls.ps[1].vs.map(|i| (i * 100.0) as usize);
            if let Some(bin) = self.nodes.get_mut(&start_idx) {
                for (i, e) in bin.iter().enumerate() {
                    if let Some((start_or_end, ls_index)) = e {
                        if let Some(suffix) = &self.entries[*ls_index] {
                            let suffix = if *start_or_end == StartOrEnd::End {
                                suffix.reverse()
                            } else {
                                suffix.clone()
                            };
                            if let Some(lm) = merge(&ls, &suffix) {
                                // If we've found a prefix we need to remove it from the lists.
                                // And update the current segment.
                                ls = lm;
                                end_idx = ls.ps[1].vs.map(|i| (i * 100.0) as usize);
                                self.entries[*ls_index] = None;
                                bin[i] = None;
                                break;
                            }
                        }
                    }
                }
            }

            let idx = self.entries.len();
            self.entries.push(Some(ls));
            self.nodes
                .entry(start_idx)
                .or_default()
                .push(Some((StartOrEnd::Start, idx)));
            self.nodes
                .entry(end_idx)
                .or_default()
                .push(Some((StartOrEnd::End, idx)));
        }

        // dump all the lines back out.
        for e in self.entries {
            if let Some(e) = e {
                self.output.send(e).await.unwrap();
            }
        }
        Ok(())
    }
}

pub struct BinningPolyLineMerger<const N: usize> {
    pub input: Receiver<LineSegment<N, ()>>,
    pub output: Sender<PolyLine<N, ()>>,
    pub entries: Vec<Option<PolyLine<N, ()>>>,
    pub nodes: HashMap<[usize; N], Vec<Option<(StartOrEnd, usize)>>>,
}

impl<const N: usize> BinningPolyLineMerger<N> {
    pub fn new(
        input: Receiver<LineSegment<N, ()>>,
        output: Sender<PolyLine<N, ()>>,
    ) -> BinningPolyLineMerger<N> {
        BinningPolyLineMerger {
            input,
            output,
            entries: vec![],
            nodes: HashMap::new(),
        }
    }

    pub async fn run(mut self) -> Result<(), ()> {
        while let Some(ls) = self.input.recv().await {
            let mut pl = PolyLine {
                ps: vec![ls.ps[0], ls.ps[1]],
                attributes: (),
            };

            // Find the bin for the start vertex
            // Remove matching line-segment
            // join

            let mut start_idx = pl.ps.first().unwrap().vs.map(|i| (i * 100.0) as usize);
            if let Some(bin) = self.nodes.get_mut(&start_idx) {
                for (i, e) in bin.iter().enumerate() {
                    if let Some((start_or_end, pl_index)) = e {
                        if let Some(prefix) = &self.entries[*pl_index] {
                            let prefix = if *start_or_end == StartOrEnd::Start {
                                prefix.reverse()
                            } else {
                                prefix.clone()
                            };
                            if let Some(plm) = merge_pl(&prefix, &pl) {
                                // If we've found a prefix we need to remove it from the lists.
                                // And update the current segment.
                                pl = plm;
                                start_idx = pl.ps.first().unwrap().vs.map(|i| (i * 100.0) as usize);
                                self.entries[*pl_index] = None;
                                bin[i] = None;
                                break;
                            }
                        }
                    }
                }
            }

            // Find the bin for the end vertex
            // Remove matching line-segment
            // join
            let mut end_idx = pl.ps.last().unwrap().vs.map(|i| (i * 100.0) as usize);
            if let Some(bin) = self.nodes.get_mut(&start_idx) {
                for (i, e) in bin.iter().enumerate() {
                    if let Some((start_or_end, pl_index)) = e {
                        if let Some(suffix) = &self.entries[*pl_index] {
                            let suffix = if *start_or_end == StartOrEnd::End {
                                suffix.reverse()
                            } else {
                                suffix.clone()
                            };
                            if let Some(plm) = merge_pl(&pl, &suffix) {
                                // If we've found a prefix we need to remove it from the lists.
                                // And update the current segment.
                                pl = plm;
                                end_idx = pl.ps.last().unwrap().vs.map(|i| (i * 100.0) as usize);
                                self.entries[*pl_index] = None;
                                bin[i] = None;
                                break;
                            }
                        }
                    }
                }
            }

            let idx = self.entries.len();
            self.entries.push(Some(pl));
            self.nodes
                .entry(start_idx)
                .or_default()
                .push(Some((StartOrEnd::Start, idx)));
            self.nodes
                .entry(end_idx)
                .or_default()
                .push(Some((StartOrEnd::End, idx)));
        }

        // dump all the lines back out.
        for e in self.entries {
            if let Some(e) = e {
                self.output.send(e).await.unwrap();
            }
        }

        Ok(())
    }
}

pub struct MegaMerger<const N: usize> {
    line_merger: LineMerger<N>,
    // binning_merger: BinningLineMerger<N>,
    binning_merger: BinningLineMergerAndDeduplicator<N>,
    polyline_merger: BinningPolyLineMerger<N>,
}

impl<const N: usize> MegaMerger<N> {
    pub fn new(
        output_a: Receiver<LineSegment<N, ()>>,
        input_d: Sender<PolyLine<N, ()>>,
    ) -> MegaMerger<N> {
        let (input_b, output_b) = channel(100);
        let (input_c, output_c) = channel(100);

        let line_merger = LineMerger::new(output_a, input_b);
        // let binning_merger = BinningLineMerger::new(output_b, input_c);
        let binning_merger = BinningLineMergerAndDeduplicator::new(output_b, input_c, 600.0, 20.0);
        let polyline_merger = BinningPolyLineMerger::new(output_c, input_d);

        MegaMerger {
            line_merger,
            binning_merger,
            polyline_merger,
        }
    }

    pub async fn run(self) -> Result<(), ()> {
        let lm = self.line_merger.run();
        let bm = self.binning_merger.run();
        let pm = self.polyline_merger.run();
        try_join!(lm, bm, pm)?;
        Ok(())
    }
}

#[cfg(test)]
pub mod tests {

    pub mod deduplication_tests {
        use crate::{
            n2::point::p2,
            nbase::{
                line_merger::merge_and_deduplicate::{
                    BinningLineMergerAndDeduplicator, DeduplicatorCore,
                },
                line_segment::LineSegment,
            },
        };

        #[test]
        pub fn decimate_linesegment_horiz() {
            let mut deduplicator: DeduplicatorCore<2> = DeduplicatorCore::new(600.0, 20.0);
            let linesegment = LineSegment {
                ps: [p2(50.0, 50.0), p2(51.0, 50.0)],
                attributes: (),
            };
            for l in linesegment.nsplit(10) {
                deduplicator.add_linesegment(l);
            }
            let mut linesegments = vec![];
            deduplicator
                .on_all_segments(|l| -> Result<(), ()> {
                    linesegments.push(l);
                    Ok(())
                })
                .unwrap();
            assert_eq!(linesegments.len(), 1);
            assert_eq!(linesegments, vec![linesegment]);
        }

        #[test]
        pub fn decimate_linesegment_vert() {
            let mut deduplicator: DeduplicatorCore<2> = DeduplicatorCore::new(600.0, 20.0);
            let linesegment = LineSegment {
                ps: [p2(50.0, 50.0), p2(50.0, 51.0)],
                attributes: (),
            };
            for l in linesegment.nsplit(10) {
                deduplicator.add_linesegment(l);
            }
            let mut linesegments = vec![];
            deduplicator
                .on_all_segments(|l| -> Result<(), ()> {
                    linesegments.push(l);
                    Ok(())
                })
                .unwrap();
            assert_eq!(linesegments.len(), 1);
            assert_eq!(linesegments, vec![linesegment]);
        }
    }
}
