use std::collections::HashMap;

use tokio::sync::mpsc::{Receiver, Sender};

use crate::nbase::polyline::PolyLine;

use super::polyline::LineSegment;

pub struct LineMerger<const N: usize> {
    pub input: Receiver<LineSegment<N>>,
    pub output: Sender<LineSegment<N>>,
    pub current_line: Option<LineSegment<N>>,
}

#[allow(non_snake_case)]
fn merge<const N: usize>(ls1: &LineSegment<N>, ls2: &LineSegment<N>) -> Option<LineSegment<N>> {
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
        })
    } else {
        None
    }
}

fn merge_pl<const N: usize>(pl1: &PolyLine<N>, pl2: &PolyLine<N>) -> Option<PolyLine<N>> {
    let z = *pl1.ps.last().unwrap() - *pl2.ps.first().unwrap();
    let z2 = z.dot(z);
    if z2 > 0.01 {
        return None;
    }

    return Some(PolyLine {
        ps: pl1.ps.iter().chain(pl2.ps[1..].iter()).cloned().collect(),
    });
}

impl<const N: usize> LineMerger<N> {
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

pub struct BinningLineMerger<const N: usize> {
    pub input: Receiver<LineSegment<N>>,
    pub output: Sender<LineSegment<N>>,
    pub entries: Vec<Option<LineSegment<N>>>,
    pub nodes: HashMap<[usize; N], Vec<Option<(StartOrEnd, usize)>>>,
}

impl<const N: usize> BinningLineMerger<N> {
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
    pub input: Receiver<LineSegment<N>>,
    pub output: Sender<PolyLine<N>>,
    pub entries: Vec<Option<PolyLine<N>>>,
    pub nodes: HashMap<[usize; N], Vec<Option<(StartOrEnd, usize)>>>,
}

impl<const N: usize> BinningPolyLineMerger<N> {
    pub async fn run(mut self) -> Result<(), ()> {
        while let Some(ls) = self.input.recv().await {
            let mut pl = PolyLine {
                ps: vec![ls.ps[0], ls.ps[1]],
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
