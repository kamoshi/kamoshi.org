pub fn generate_footer_dither(
    pixel_size: u32,
    tile_width: u32,
    height_pixels: u32,
    seed: u32,
) -> String {
    use std::collections::HashMap;
    use std::fmt::Write;

    let svg_w = tile_width * pixel_size;
    let svg_h = height_pixels * pixel_size;
    let w = tile_width as usize;
    let h = height_pixels as usize;

    // Build the logical grid of filled pixels
    let mut filled = vec![false; w * h];
    for py in 0..h {
        let t = py as f32 / (h.saturating_sub(1).max(1)) as f32;

        // `t` is the row's position from `0.0` (top) to `1.0` (bottom). The
        // formula `t*t*(3.0 - 2.0*t)` is a smoothstep curve that:
        // - starts at 0, ends at 1
        // - has zero derivative at both ends (eases in and out gracefully)
        // - is always increasing, so density only ever grows downward
        //
        // Visually, sparse at the top, dense at the bottom, with a smooth S-curve
        // transition rather than a sharp one.
        let prob = t * t * (3.0 - 2.0 * t);

        for px in 0..w {
            // Rather than using a random number generator (which has state and
            // can't be reproduced per-pixel independently), this computes a
            // hash from the pixel coordinates and a seed.
            let mut hash = seed;

            hash ^= (px as u32).wrapping_mul(374761393); // mix in the column
            hash ^= (py as u32).wrapping_mul(668265263); // mix in the row
            hash = hash.wrapping_mul(1274126177); // avalanche the bits
            hash ^= hash >> 16; // fold high bits into low bits

            filled[py * w + px] = (hash as f32 / u32::MAX as f32) < prob;
        }
    }

    // Scanline interval merge
    let mut active: HashMap<(usize, usize), usize> = HashMap::new();
    let mut rects: Vec<(usize, usize, usize, usize)> = Vec::new();

    let row_intervals = |py: usize| -> Vec<(usize, usize)> {
        let mut intervals = Vec::new();
        let mut px = 0;
        while px < w {
            if filled[py * w + px] {
                let start = px;
                while px < w && filled[py * w + px] {
                    px += 1;
                }
                intervals.push((start, px));
            } else {
                px += 1;
            }
        }
        intervals
    };

    for py in 0..=h {
        let current: HashMap<(usize, usize), ()> = if py < h {
            row_intervals(py).into_iter().map(|iv| (iv, ())).collect()
        } else {
            HashMap::new()
        };

        let closing: Vec<_> = active
            .iter()
            .filter(|(iv, _)| !current.contains_key(*iv))
            .map(|(&iv, &y0)| (iv, y0))
            .collect();

        for ((x0, x1), y0) in closing {
            active.remove(&(x0, x1));
            rects.push((x0, x1, y0, py));
        }

        for &(x0, x1) in current.keys() {
            active.entry((x0, x1)).or_insert(py);
        }
    }

    // Instead of emitting one <rect> element per rectangle (which has
    // significant per-element overhead in SVG parsing and rendering), all
    // rectangles are encoded as sub-paths inside a single <path d="...">.
    let mut path_d = String::with_capacity(rects.len() * 20);
    for (x0, x1, y0, y1) in rects {
        let rw = x1 - x0;
        let rh = y1 - y0;
        // The path commands are:
        // - M: move to top-left corner
        // - h: draw horizontal line
        // - v: draw vertical line
        // - H: draw horizontal line to close the rectangle
        // - z: close the subpath (draws a straight line back to the start)
        write!(path_d, "M{x0} {y0}h{rw}v{rh}H{x0}z").unwrap();
    }

    // All the coordinates in the path are in pixel units (the logical grid),
    // not screen pixels. Rather than multiplying every coordinate by pixel_size
    // while building the path string, a single scale() transform on the
    // containing <g> element handles it for everything inside. This has two
    // advantages:
    // - The path string is shorter (smaller numbers, fewer characters)
    // - The CPU doesn't do N × 4 multiplications during string building, the SVG
    //   renderer does one matrix transform at render time instead
    //
    // The outer width and height on the <svg> element are still set to the true
    // screen pixel dimensions (tile_width * pixel_size) so the document has the
    // right viewport, even though the contents are drawn in scaled coordinates.
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" \
         width=\"{svg_w}\" height=\"{svg_h}\">\
         <g transform=\"scale({pixel_size})\">\
         <path d=\"{path_d}\" fill=\"black\"/>\
         </g></svg>"
    )
}
