pub mod widgets;

use itertools::Itertools;

pub const MASK_WIDTH: i32 = 10000;
pub const MASK_HEIGHT: i32 = 2500;
pub const SCATTER_PLOT_ALPHA: f64 = 0.05;

pub const COLORS: [(u8, u8, u8); 8] = [
    (57, 106, 177),
    (218, 124, 48),
    (62, 150, 81),
    (204, 37, 41),
    (83, 81, 84),
    (107, 76, 154),
    (146, 36, 40),
    (148, 139, 61),
];

lazy_static! {
    pub static ref COLORS_HEX: Vec<String> = COLORS
        .iter()
        .map(|(r, g, b)| format!("#{:02x}{:02x}{:02x}", r, g, b))
        .collect_vec();
    pub static ref COLORS_F64: Vec<(f64, f64, f64)> = COLORS
        .iter()
        .cloned()
        .map(|(r, g, b)| (
            f64::from(r) / 255.0,
            f64::from(g) / 255.0,
            f64::from(b) / 255.0
        ))
        .collect_vec();
}
