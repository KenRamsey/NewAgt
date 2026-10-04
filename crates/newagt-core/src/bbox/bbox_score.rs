//! Normative eval geometry from `Score.c` / `Abuse_Tgt.c` (atan projection, integer box).

use std::f64::consts::PI;

/// Degrees → radians (`DTOR` in legacy C).
const DTOR: f64 = PI / 180.0;

/// NATO defaults when `TgtType` is absent (`Score.c`).
pub const NATO_WIDTH_M: f64 = 2.3;
pub const NATO_HEIGHT_M: f64 = 2.3;
pub const NATO_LENGTH_M: f64 = 6.4;

/// Physical target size in meters (tgt.dat columns 2–4 mapped like C: L, W, H).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TargetDimensions {
    pub length_m: f64,
    pub width_m: f64,
    pub height_m: f64,
}

impl TargetDimensions {
    pub const NATO: Self = Self {
        length_m: NATO_LENGTH_M,
        width_m: NATO_WIDTH_M,
        height_m: NATO_HEIGHT_M,
    };
}

/// Compute inclusive pixel width/height using Score/C assignment semantics.
///
/// C expression: `boxw = ((imw * td) / FovX) / DTOR` with `td` in radians after `atan`.
/// The result is truncated toward zero when stored in `IS32`.
pub fn score_box_dimensions(
    dims: TargetDimensions,
    aspect_deg: f64,
    range_m: f64,
    fov_h_deg: f64,
    fov_v_deg: f64,
    image_width: u32,
    image_height: u32,
) -> (i32, i32) {
    let asp = aspect_deg * DTOR;
    let mut td = (dims.width_m * asp.cos()).abs() + (dims.length_m * asp.sin()).abs();
    td = 2.0 * (0.5 * td / range_m).atan();
    let boxw_f = (f64::from(image_width) * td / fov_h_deg) / DTOR;

    td = dims.height_m;
    td = 2.0 * (0.5 * td / range_m).atan();
    let boxh_f = (f64::from(image_height) * td / fov_v_deg) / DTOR;

    (boxw_f as i32, boxh_f as i32)
}

/// Center an inclusive integer box on `PixLoc` (Score integer math).
pub fn center_inclusive_box(boxw: i32, boxh: i32, pix_x: i32, pix_y: i32) -> (i32, i32, i32, i32) {
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
    fn golden_m1_aspect0_range500() {
        let dims = TargetDimensions {
            length_m: 7.72,
            width_m: 3.66,
            height_m: 2.34,
        };
        let (w, h) = score_box_dimensions(dims, 0.0, 500.0, 30.0, 20.0, 640, 480);
        assert_eq!((w, h), (8, 6));
        let (x1, y1, x2, y2) = center_inclusive_box(w, h, 320, 240);
        assert_eq!((x1, y1, x2, y2), (317, 238, 324, 243));
    }
}
