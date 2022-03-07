use gnarled::{
    n2::{lineset::LineSet, point::p2, polyline::PolyLine, tile::make_tile, traits::Rotatable},
    nbase::traits::{Boundable, Shiftable},
    svg::SVGable,
};

#[allow(clippy::many_single_char_names)]
fn main() -> Result<(), std::io::Error> {
    use std::io::Write;
    let file_name = "spiral.svg";
    let mut f = std::fs::File::create(file_name).unwrap();
    let ps = (0..100)
        .map(|i| (i as f32) * 4.0f32 * std::f32::consts::PI / 100.0f32)
        .map(|t| p2((t * t.cos()) * 40.0, (t * t.sin()) * 40.0))
        .collect::<Vec<_>>();
    let p = PolyLine { ps };
    let _bb = p.bounds().unwrap();
    let p = p.shift_by(p2(400., 400.));
    writeln!(
        f,
        r#"<svg viewBox="0 0 800 800" xmlns="http://www.w3.org/2000/svg">"#
    )?;
    let mut spiral_lines = LineSet { lines: vec![] };
    let n_spiral_arms = 16;
    for i in 0..n_spiral_arms {
        let beta = 2.0 * std::f32::consts::PI / (n_spiral_arms as f32);
        let pr = p.rotate_by(beta * (i as f32), p2(400., 400.));
        spiral_lines.lines.push(pr);
    }
    //spiral_lines.to_svg(&mut f)?;
    let mut horz_lines = LineSet { lines: vec![] };
    let n = 20;
    for i in 0..n {
        let dy = (700.0 - 100.0) / (n as f32);
        let y = 100.0 + dy * (0.5 + i as f32);
        let pr = PolyLine {
            ps: (0..800).step_by(10).map(|x| p2(x as f32, y)).collect(),
        };
        horz_lines.lines.push(pr);
    }
    let mut vert_lines = LineSet { lines: vec![] };
    let n = 20;
    for i in 0..n {
        let dx = (700.0 - 100.0) / (n as f32);
        let x = 100.0 + dx * (0.5 + i as f32);
        let pr = PolyLine {
            ps: (0..800).step_by(10).map(|y| p2(x, y as f32)).collect(),
        };
        vert_lines.lines.push(pr);
    }

    let tile_bounds = (p2(200.0, 200.0), p2(600.0, 600.0));
    let hline_tile = make_tile(tile_bounds, &horz_lines);

    let vline_tile = make_tile(tile_bounds, &vert_lines);
    let spiral_tile = make_tile(tile_bounds, &spiral_lines);

    for (i, &tile) in [&spiral_tile, &vline_tile, &hline_tile].iter().enumerate() {
        let output_width = 200.0;

        for (j, dx) in [0.0, 200.0, 400.0, 600.0].iter().enumerate() {
            for (k, dy) in [0.0, 200.0, 400.0, 600.0].iter().enumerate() {
                if i == 0 {
                    let t = match (j % 2 == 0, k % 2 == 0) {
                        (true, true) => tile.clone(),
                        (true, false) => tile.flip_y(),
                        (false, true) => tile.flip_x(),
                        (false, false) => tile.flip_xy(),
                    };
                    let z = t.place_at(p2(*dx, *dy), output_width, output_width);
                    z.to_svg(&mut f)?;
                }
            }
        }
    }
    writeln!(f, "</svg>")?;
    Ok(())
}
