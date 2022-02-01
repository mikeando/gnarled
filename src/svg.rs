use std::fmt::Display;

use crate::{
    n2::{cubic_bezier::CubicBezierPath, polyline::PolyLine, lineset::LineSet},
};

pub trait SVGable {
    fn to_svg<W>(&self, w: &mut W) -> Result<(), std::io::Error>
    where
        W: std::io::Write;
}

impl CubicBezierPath {
    pub fn to_svg_with_properties<W>(
        &self,
        w: &mut W,
        props: PolyLineProperties,
    ) -> Result<(), std::io::Error>
    where
        W: std::io::Write,
    {
        if self.ps.len() <= 1 {
            return Ok(());
        }
        assert!(self.ps.len() % 3 == 1);
        writeln!(
            w,
            r#"<path stroke="{}" fill="transparent" d=""#,
            props.stroke
        )?;
        writeln!(w, "M {},{}", self.ps[0].0, self.ps[0].1)?;
        for n in 0..(self.ps.len() - 1) / 3 {
            if let [_x, c1, c2, y] = self.ps[3 * n..3 * n + 4] {
                writeln!(w, "C {} {}, {} {}, {} {}", c1.0, c1.1, c2.0, c2.1, y.0, y.1)?;
            }
        }
        writeln!(w, r#""/>"#)?;

        Ok(())
    }
}

impl SVGable for CubicBezierPath {
    fn to_svg<W>(&self, w: &mut W) -> Result<(), std::io::Error>
    where
        W: std::io::Write,
    {
        self.to_svg_with_properties(w, PolyLineProperties::default())
    }
}

#[derive(Clone, Copy)]
pub enum PolyLineStroke {
    Black,
    Red,
    Green,
}

impl Default for PolyLineStroke {
    fn default() -> Self {
        PolyLineStroke::Black
    }
}

impl Display for PolyLineStroke {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PolyLineStroke::Black => write!(f, "black"),
            PolyLineStroke::Red => write!(f, "red"),
            PolyLineStroke::Green => write!(f, "green"),
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct PolyLineProperties {
    pub stroke: PolyLineStroke,
}

impl PolyLine {
    pub fn to_svg_with_properties<W>(
        &self,
        w: &mut W,
        props: PolyLineProperties,
    ) -> Result<(), std::io::Error>
    where
        W: std::io::Write,
    {
        if self.ps.len() <= 1 {
            return Ok(());
        }
        writeln!(
            w,
            r#"<path stroke="{}" fill="transparent" d=""#,
            props.stroke
        )?;
        writeln!(w, "M {},{}", self.ps[0].0, self.ps[0].1)?;
        for pp in &self.ps[1..] {
            writeln!(w, "L {},{}", pp.0, pp.1)?;
        }
        writeln!(w, r#""/>"#)?;

        Ok(())
    }
}

impl SVGable for PolyLine {
    fn to_svg<W>(&self, w: &mut W) -> Result<(), std::io::Error>
    where
        W: std::io::Write,
    {
        self.to_svg_with_properties(w, PolyLineProperties::default())
    }
}

impl LineSet {
    pub fn to_svg_with_properties<W>(
        &self,
        w: &mut W,
        props: PolyLineProperties,
    ) -> Result<(), std::io::Error>
    where
        W: std::io::Write,
    {
        for line in &self.lines {
            line.to_svg_with_properties(w, props)?;
        }
        Ok(())
    }
}

impl SVGable for LineSet {
    fn to_svg<W>(&self, w: &mut W) -> Result<(), std::io::Error>
    where
        W: std::io::Write,
    {
        for l in &self.lines {
            l.to_svg(w)?
        }
        Ok(())
    }
}
