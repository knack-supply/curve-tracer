use crate::dut::{SomeDeviceType, ThreeTerminalDeviceType, TwoTerminalDeviceType};
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

pub trait DevicePlot {
    fn connection_hint(&self) -> &'static str;
    fn legend(&self) -> String;
}

impl DevicePlot for SomeDeviceType {
    fn connection_hint(&self) -> &'static str {
        match self {
            SomeDeviceType::TwoTerminal(TwoTerminalDeviceType::Diode) => "Top row: AKKKKKK",
            SomeDeviceType::ThreeTerminal(ThreeTerminalDeviceType::NPN) => "Bottom row: CBECBEC",
            SomeDeviceType::ThreeTerminal(ThreeTerminalDeviceType::PNP) => {
                "Bottom row: EBCEBCE (reversed E/C)"
            }
            SomeDeviceType::ThreeTerminal(ThreeTerminalDeviceType::NFET) => "Bottom row: DGSDGSD",
            SomeDeviceType::ThreeTerminal(ThreeTerminalDeviceType::PFET) => {
                "Bottom row: SGDSGDS (reversed S/D)"
            }
        }
    }
    fn legend(&self) -> String {
        match self {
            SomeDeviceType::TwoTerminal(TwoTerminalDeviceType::Diode) => String::new(),
            SomeDeviceType::ThreeTerminal(ThreeTerminalDeviceType::NPN) =>
                format!(r###"I<sub>BE</sub>: <span fgcolor="white" bgcolor="{}">10µA</span> <span fgcolor="white" bgcolor="{}">20µA</span> <span fgcolor="white" bgcolor="{}">30µA</span> <span fgcolor="white" bgcolor="{}">40µA</span> <span fgcolor="white" bgcolor="{}">50µA</span>"###,
                        COLORS_HEX[0], COLORS_HEX[1], COLORS_HEX[2], COLORS_HEX[3], COLORS_HEX[4]),
            SomeDeviceType::ThreeTerminal(ThreeTerminalDeviceType::PNP) =>
                format!(r###"I<sub>BE</sub>: <span fgcolor="white" bgcolor="{}">-10µA</span> <span fgcolor="white" bgcolor="{}">-20µA</span> <span fgcolor="white" bgcolor="{}">-30µA</span> <span fgcolor="white" bgcolor="{}">-40µA</span> <span fgcolor="white" bgcolor="{}">-50µA</span>"###,
                        COLORS_HEX[0], COLORS_HEX[1], COLORS_HEX[2], COLORS_HEX[3], COLORS_HEX[4]),
            SomeDeviceType::ThreeTerminal(ThreeTerminalDeviceType::NFET) =>
                format!(r###"V<sub>GS</sub>: <span fgcolor="white" bgcolor="{}">1V</span> <span fgcolor="white" bgcolor="{}">2V</span> <span fgcolor="white" bgcolor="{}">3V</span> <span fgcolor="white" bgcolor="{}">4V</span> <span fgcolor="white" bgcolor="{}">5V</span>"###,
                        COLORS_HEX[0], COLORS_HEX[1], COLORS_HEX[2], COLORS_HEX[3], COLORS_HEX[4]),
            SomeDeviceType::ThreeTerminal(ThreeTerminalDeviceType::PFET) =>
                format!(r###"V<sub>GS</sub>: <span fgcolor="white" bgcolor="{}">-1V</span> <span fgcolor="white" bgcolor="{}">-2V</span> <span fgcolor="white" bgcolor="{}">-3V</span> <span fgcolor="white" bgcolor="{}">-4V</span> <span fgcolor="white" bgcolor="{}">-5V</span>"###,
                        COLORS_HEX[0], COLORS_HEX[1], COLORS_HEX[2], COLORS_HEX[3], COLORS_HEX[4]),
        }
    }
}
