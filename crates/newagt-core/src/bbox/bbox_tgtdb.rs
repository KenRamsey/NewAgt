//! Linear small-angle sizing from legacy `tgtdb.py` `Tgt.getRect` (not merged with Score).

use std::f64::consts::PI;

const DTOR: f64 = PI / 180.0;

/// Returns `(pix_width, pix_height)` as Python `getRect` (floats, no centering).
#[allow(clippy::too_many_arguments)]
pub fn tgtdb_get_rect(
    length_m: f64,
    width_m: f64,
    height_m: f64,
    aspect_deg: f64,
    range_m: f64,
    fov_h_deg: f64,
    fov_v_deg: f64,
    image_width: u32,
    image_height: u32,
) -> (f64, f64) {
    let hfov = fov_h_deg * DTOR;
    let asp_ang = aspect_deg * DTOR;
    let tgt_width = (length_m * asp_ang.sin()).abs() + (width_m * asp_ang.cos()).abs();
    let pix_width = tgt_width * f64::from(image_width) / (range_m * hfov);

    let vfov = fov_v_deg * DTOR;
    let pix_height = height_m * f64::from(image_height) / (range_m * vfov);
    (pix_width, pix_height)
}

/// Build an inclusive integer box by truncating linear sizes then centering on `PixLoc`.
#[allow(clippy::too_many_arguments)]
pub fn tgtdb_inclusive_box(
    length_m: f64,
    width_m: f64,
    height_m: f64,
    aspect_deg: f64,
    range_m: f64,
    fov_h_deg: f64,
    fov_v_deg: f64,
    image_width: u32,
    image_height: u32,
    pix_x: i32,
    pix_y: i32,
) -> (i32, i32, i32, i32) {
    let (pw, ph) = tgtdb_get_rect(
        length_m,
        width_m,
        height_m,
        aspect_deg,
        range_m,
        fov_h_deg,
        fov_v_deg,
        image_width,
        image_height,
    );
    let boxw = pw as i32;
    let boxh = ph as i32;
    let x1 = pix_x - (boxw - 1) / 2;
    let x2 = x1 + boxw - 1;
    let y1 = pix_y - (boxh - 1) / 2;
    let y2 = y1 + boxh - 1;
    (x1, y1, x2, y2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn m1_aspect0_range500_linear_floats() {
        let (pw, ph) = tgtdb_get_rect(7.72, 3.66, 2.34, 0.0, 500.0, 30.0, 20.0, 640, 480);
        assert!((pw - 8.947_308_928_762_936).abs() < 1e-12);
        assert!((ph - 6.435_461_954_909_406).abs() < 1e-12);
    }
}
