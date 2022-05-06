use crate::attributes::AttributeReverse;
use crate::nbase::bounds::Bounds;
use crate::nbase::line_segment::LineSegment;
use crate::nbase::lineset::LineSet;
use crate::nbase::point::Point;
use crate::nbase::traits::*;

#[derive(Clone, Debug)]
pub struct PolyLine<const N: usize, A> {
    pub ps: Vec<Point<N>>,
    pub attributes: A,
}

pub trait PolyLineAttribute {
    type LineAttribute;
    fn attribute_for_line_segment(&self, index: usize) -> Self::LineAttribute;
    fn poly_range(&self, start: (usize, f32), end: (usize, f32)) -> Self;
}

impl PolyLineAttribute for () {
    type LineAttribute = ();
    fn attribute_for_line_segment(&self, _index: usize) -> Self::LineAttribute {
        ()
    }
    fn poly_range(&self, start: (usize, f32), end: (usize, f32)) -> Self {
        ()
    }
}

struct OpenSegment<const N: usize> {
    line: Vec<Point<N>>,
    start_iz: (usize, f32),
}

struct SegmentState<const N: usize> {
    last_p: Point<N>,
    last_a: f32,
    open_segment: Option<OpenSegment<N>>,
}

impl<const N: usize, A> PolyLine<N, A> {
    pub fn clip_by(&self, n: Point<N>, v: f32) -> LineSet<N>
    where
        A: PolyLineAttribute,
    {
        // If f is not linear we cant assume
        // 1. we can calculate the zero using simple interpolation
        // 2. that if both ends are the same sign, then all points between
        //    are the same sign.
        let f = |p: Point<N>| p.dot(n) - v;
        if self.ps.is_empty() {
            return LineSet { lines: vec![] };
        }

        let mut lines: Vec<PolyLine<N, A>> = vec![];
        let a = f(self.ps[0]);
        let mut state = SegmentState {
            last_p: self.ps[0],
            last_a: a,
            open_segment: if a >= 0.0 {
                Some(OpenSegment {
                    line: vec![self.ps[0]],
                    start_iz: (0, 0.0),
                })
            } else {
                None
            },
        };

        for i in 1..self.ps.len() {
            let p = &self.ps[i];
            let a = f(*p);
            let new_segment = match state.open_segment.take() {
                Some(mut seg) => {
                    if a >= 0.0 {
                        // Segment remains open
                        seg.line.push(*p);
                        Some(seg)
                    } else {
                        // Segment done.
                        let da = a - state.last_a;
                        let alpha = a / da;
                        let pp = Point::lerp(alpha, state.last_p, *p);
                        seg.line.push(pp);

                        lines.push(PolyLine {
                            ps: seg.line,
                            attributes: self.attributes.poly_range(seg.start_iz, (i - 1, alpha)),
                        });
                        None
                    }
                }
                None => {
                    if a >= 0.0 {
                        // Start a new segment
                        let da = a - state.last_a;
                        let alpha = a / da;
                        let pp = Point::lerp(alpha, state.last_p, *p);
                        let seg = OpenSegment {
                            line: vec![pp],
                            start_iz: (i - 1, alpha),
                        };
                        Some(seg)
                    } else {
                        // Still outside a valid segment
                        None
                    }
                }
            };
            state.last_a = a;
            state.last_p = *p;
            state.open_segment = new_segment;
        }
        if let Some(seg) = state.open_segment {
            lines.push(PolyLine {
                ps: seg.line,
                attributes: self
                    .attributes
                    .poly_range(seg.start_iz, (self.ps.len() - 1, 1.0)),
            });
        }

        LineSet {
            lines: lines.iter().map(|pl| pl.map_attribute(|a| ())).collect(),
        }
    }
}

impl<const N: usize, A> PolyLine<N, A> {
    pub fn line_segments(&self) -> Vec<LineSegment<N, <A as PolyLineAttribute>::LineAttribute>>
    where
        A: PolyLineAttribute,
    {
        let mut result = vec![];
        if self.ps.len() < 2 {
            return result;
        }
        for i in 0..self.ps.len() - 1 {
            result.push(LineSegment {
                ps: [self.ps[i], self.ps[i + 1]],
                attributes: self.attributes.attribute_for_line_segment(i),
            });
        }
        result
    }

    pub(crate) fn reverse(&self) -> PolyLine<N, A>
    where
        A: AttributeReverse,
    {
        let mut ps = self.ps.clone();
        ps.reverse();
        PolyLine {
            ps,
            attributes: self.attributes.reverse(),
        }
    }
}

impl<const N: usize, A> PolyLine<N, A> {
    pub(crate) fn map_attribute<F, A2>(&self, mut f: F) -> PolyLine<N, A2>
    where
        F: FnMut(&A) -> A2,
    {
        PolyLine {
            ps: self.ps.clone(),
            attributes: f(&self.attributes),
        }
    }
}

impl<const N: usize, A> Boundable<N> for PolyLine<N, A> {
    fn bounds(&self) -> Option<Bounds<N>> {
        self.ps
            .iter()
            .fold(None, crate::nbase::point::point_extrema)
    }
}

impl<const N: usize, A> Shiftable<N> for PolyLine<N, A>
where
    A: Clone,
{
    type Result = PolyLine<N, A>;
    fn shift_by(&self, d: Point<N>) -> PolyLine<N, A> {
        PolyLine {
            ps: self.ps.iter().map(|p| *p + d).collect(),
            attributes: self.attributes.clone(),
        }
    }
}

impl<const N: usize, A> Scalable<N> for PolyLine<N, A>
where
    A: Clone,
{
    type Result = PolyLine<N, A>;

    fn scale(&self, c: Point<N>, s: &[f32; N]) -> Self::Result {
        PolyLine {
            ps: self
                .ps
                .iter()
                .map(|p| (*p - c) * Point::from(*s) + c)
                .collect(),
            attributes: self.attributes.clone(),
        }
    }
}
