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

    writeln!(
        f,
        r#"<svg viewBox="0 0 800 800" xmlns="http://www.w3.org/2000/svg">"#
    )?;
    

    // Single spiral arm of the swirl
    let p = PolyLine { ps, attributes: () };
    let _bb = p.bounds().unwrap();
    let p = p.shift_by(p2(400., 400.));

    // Copy and rotate to get the full swirl
    let mut spiral_lines = LineSet { lines: vec![] };
    let n_spiral_arms = 16;
    for i in 0..n_spiral_arms {
        let beta = 2.0 * std::f32::consts::PI / (n_spiral_arms as f32);
        let pr = p.rotate_by(beta * (i as f32), p2(400., 400.));
        spiral_lines.lines.push(pr);
    }

    let tile_bounds = (p2(200.0, 200.0), p2(600.0, 600.0));
    let spiral_tile = make_tile(tile_bounds, &spiral_lines);

    let output_width = 200.0;

    for j in 0..=3 {
        let x = j as f32 * output_width;
        for k in 0..=3 {
            let y = k as f32 * output_width;
            let t = match (j % 2 == 0, k % 2 == 0) {
                (true, true) => spiral_tile.clone(),
                (true, false) => spiral_tile.flip_y(),
                (false, true) => spiral_tile.flip_x(),
                (false, false) => spiral_tile.flip_xy(),
            };
            let z = t.place_at(p2(x, y), output_width, output_width);
            z.to_svg(&mut f)?;
        }
    }
    writeln!(f, "</svg>")?;
    Ok(())
}
