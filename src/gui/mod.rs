use std::f64::consts::PI;
use std::sync::Arc;

use cairo::Context;
use itertools::Itertools;
use itertools_num::linspace;

use crate::model::diode::diode_model;
use crate::trace::file::ExportableTrace;
use crate::ThreeTerminalTrace;
use crate::TwoTerminalTrace;
use crate::{DeviceType, NullTrace, ThreeTerminalDeviceType, TwoTerminalDeviceType};

const COLORS: [(u8, u8, u8); 8] = [
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
    static ref COLORS_HEX: Vec<String> = COLORS
        .iter()
        .map(|(r, g, b)| format!("#{:02x}{:02x}{:02x}", r, g, b))
        .collect_vec();
    static ref COLORS_F64: Vec<(f64, f64, f64)> = COLORS
        .iter()
        .cloned()
        .map(|(r, g, b)| (
            f64::from(r) / 255.0,
            f64::from(g) / 255.0,
            f64::from(b) / 255.0
        ))
        .collect_vec();
}

pub trait GuiTrace: DrawableTrace + ExportableTrace {}

impl GuiTrace for NullTrace {}

impl GuiTrace for TwoTerminalTrace {}

impl GuiTrace for ThreeTerminalTrace {}

pub trait TraceWithModel {
    fn fill_model(&mut self);
    fn model_report(&self) -> String;
}

pub trait DrawableTrace: TraceWithModel {
    fn draw(&self, cr: &Context, v_factor: f64, i_factor: f64, height: f64);
    fn draw_model(&self, cr: &Context, v_factor: f64, i_factor: f64, height: f64);
}

impl TraceWithModel for NullTrace {
    fn fill_model(&mut self) {}
    fn model_report(&self) -> String {
        String::new()
    }
}

impl DrawableTrace for NullTrace {
    fn draw(&self, _: &Context, _: f64, _: f64, _: f64) {}
    fn draw_model(&self, _: &Context, _: f64, _: f64, _: f64) {}
}

impl TraceWithModel for TwoTerminalTrace {
    fn fill_model(&mut self) {
        if self.model.is_none() {
            self.model = Some(Arc::new(diode_model(&self.trace)))
        }
    }

    fn model_report(&self) -> String {
        self.model
            .as_ref()
            .map(std::string::ToString::to_string)
            .unwrap_or_else(String::new)
    }
}

impl DrawableTrace for TwoTerminalTrace {
    fn draw(&self, cr: &Context, v_factor: f64, i_factor: f64, height: f64) {
        cr.set_source_rgba(0.0, 0.0, 0.0, 0.05);
        for (v, i) in self.trace.iter() {
            cr.arc(
                v * v_factor,
                height - 20.0 - i * i_factor,
                1.0,
                0.0,
                PI * 2.0,
            );
            cr.fill();
        }
    }

    fn draw_model(&self, cr: &Context, v_factor: f64, i_factor: f64, height: f64) {
        if let Some(model) = &self.model {
            cr.set_source_rgba(1.0, 0.0, 0.0, 0.8);

            for (ix, v) in linspace(model.min_v().max(0.0), model.max_v().min(5.0), 101).enumerate()
            {
                if ix == 0 {
                    cr.move_to(v * v_factor, height - 20.0 - i_factor * model.evaluate(v));
                } else {
                    cr.line_to(v * v_factor, height - 20.0 - i_factor * model.evaluate(v));
                }
            }
            cr.stroke();
        }
    }
}

impl TraceWithModel for ThreeTerminalTrace {
    fn fill_model(&mut self) {}
    fn model_report(&self) -> String {
        String::new()
    }
}

impl DrawableTrace for ThreeTerminalTrace {
    fn draw(&self, cr: &Context, v_factor: f64, i_factor: f64, height: f64) {
        for ((_, trace), color) in self.traces.iter().zip(COLORS_F64.iter()) {
            cr.set_source_rgba(color.0, color.1, color.2, 0.05);
            for (v, i) in trace.trace.iter() {
                cr.arc(
                    v * v_factor,
                    height - 20.0 - i * i_factor,
                    1.0,
                    0.0,
                    PI * 2.0,
                );
                cr.fill();
            }
        }
    }

    fn draw_model(&self, _: &Context, _: f64, _: f64, _: f64) {}
}

pub trait DevicePlot {
    fn polarity(&self) -> f64;
    fn connection_hint(&self) -> &'static str;
    fn legend(&self) -> String;
}

impl DevicePlot for DeviceType {
    fn polarity(&self) -> f64 {
        match &self {
            DeviceType::TwoTerminal(TwoTerminalDeviceType::Diode) => 1.0,
            DeviceType::ThreeTerminal(ThreeTerminalDeviceType::NPN) => 1.0,
            DeviceType::ThreeTerminal(ThreeTerminalDeviceType::PNP) => -1.0,
            DeviceType::ThreeTerminal(ThreeTerminalDeviceType::NFET) => 1.0,
            DeviceType::ThreeTerminal(ThreeTerminalDeviceType::PFET) => -1.0,
        }
    }
    fn connection_hint(&self) -> &'static str {
        match self {
            DeviceType::TwoTerminal(TwoTerminalDeviceType::Diode) => "Top row: AKKKKKK",
            DeviceType::ThreeTerminal(ThreeTerminalDeviceType::NPN) => "Bottom row: CBECBEC",
            DeviceType::ThreeTerminal(ThreeTerminalDeviceType::PNP) => {
                "Bottom row: EBCEBCE (reversed E/C)"
            }
            DeviceType::ThreeTerminal(ThreeTerminalDeviceType::NFET) => "Bottom row: DGSDGSD",
            DeviceType::ThreeTerminal(ThreeTerminalDeviceType::PFET) => {
                "Bottom row: SGDSGDS (reversed S/D)"
            }
        }
    }
    fn legend(&self) -> String {
        match self {
            DeviceType::TwoTerminal(TwoTerminalDeviceType::Diode) => String::new(),
            DeviceType::ThreeTerminal(ThreeTerminalDeviceType::NPN) =>
                format!(r###"I<sub>BE</sub>: <span fgcolor="white" bgcolor="{}">10µA</span> <span fgcolor="white" bgcolor="{}">20µA</span> <span fgcolor="white" bgcolor="{}">30µA</span> <span fgcolor="white" bgcolor="{}">40µA</span> <span fgcolor="white" bgcolor="{}">50µA</span>"###,
                        COLORS_HEX[0], COLORS_HEX[1], COLORS_HEX[2], COLORS_HEX[3], COLORS_HEX[4]),
            DeviceType::ThreeTerminal(ThreeTerminalDeviceType::PNP) =>
                format!(r###"I<sub>BE</sub>: <span fgcolor="white" bgcolor="{}">-10µA</span> <span fgcolor="white" bgcolor="{}">-20µA</span> <span fgcolor="white" bgcolor="{}">-30µA</span> <span fgcolor="white" bgcolor="{}">-40µA</span> <span fgcolor="white" bgcolor="{}">-50µA</span>"###,
                        COLORS_HEX[0], COLORS_HEX[1], COLORS_HEX[2], COLORS_HEX[3], COLORS_HEX[4]),
            DeviceType::ThreeTerminal(ThreeTerminalDeviceType::NFET) =>
                format!(r###"V<sub>GS</sub>: <span fgcolor="white" bgcolor="{}">1V</span> <span fgcolor="white" bgcolor="{}">2V</span> <span fgcolor="white" bgcolor="{}">3V</span> <span fgcolor="white" bgcolor="{}">4V</span> <span fgcolor="white" bgcolor="{}">5V</span>"###,
                        COLORS_HEX[0], COLORS_HEX[1], COLORS_HEX[2], COLORS_HEX[3], COLORS_HEX[4]),
            DeviceType::ThreeTerminal(ThreeTerminalDeviceType::PFET) =>
                format!(r###"V<sub>GS</sub>: <span fgcolor="white" bgcolor="{}">-1V</span> <span fgcolor="white" bgcolor="{}">-2V</span> <span fgcolor="white" bgcolor="{}">-3V</span> <span fgcolor="white" bgcolor="{}">-4V</span> <span fgcolor="white" bgcolor="{}">-5V</span>"###,
                        COLORS_HEX[0], COLORS_HEX[1], COLORS_HEX[2], COLORS_HEX[3], COLORS_HEX[4]),
        }
    }
}
