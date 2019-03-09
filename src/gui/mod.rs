use std::f64::consts::PI;
use std::sync::Arc;

use cairo::Context;
use cairo::Format;
use cairo::ImageSurface;
use cairo::Operator;
use itertools::Itertools;
use itertools_num::linspace;

use crate::model::diode::diode_model;
use crate::trace::file::ExportableTrace;
use crate::DeviceType;
use crate::NullTrace;
use crate::Result;
use crate::ThreeTerminalDeviceType;
use crate::ThreeTerminalTrace;
use crate::TwoTerminalDeviceType;
use crate::TwoTerminalTrace;
use noisy_float::prelude::r64;

const MASK_WIDTH: i32 = 10000;
const MASK_HEIGHT: i32 = 2500;
const SCATTER_PLOT_ALPHA: f64 = 0.05;

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

pub trait TraceWithScatterPlot {
    fn apply_mask(&self, cr: &Context) -> Result<()>;
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
        let w = f64::from(MASK_WIDTH);
        let h = f64::from(MASK_HEIGHT);

        let v_k = w / (self.aoi.max_v - self.aoi.min_v);
        let v_b = self.aoi.min_v * v_k;

        let i_k = h / (self.aoi.max_i - self.aoi.min_i);
        let i_b = self.aoi.min_i * i_k;

        cr.save();
        cr.set_source_rgba(0.0, 0.0, 0.0, 1.0);
        cr.translate(0.0, height - 20.0);
        cr.scale(v_factor / v_k, i_factor / -i_k);
        cr.translate(v_b, i_b);
        self.apply_mask(cr).unwrap();
        cr.fill();
        cr.restore();
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

impl TraceWithScatterPlot for TwoTerminalTrace {
    fn apply_mask(&self, cr: &Context) -> Result<()> {
        let mut scatter_plot_mask = self.scatter_plot_mask.borrow_mut();
        if scatter_plot_mask.is_none() {
            let w = MASK_WIDTH;
            let h = MASK_HEIGHT;
            let surface = ImageSurface::create(Format::A8, w, h)
                .map_err(|_| failure::err_msg("Can't create an off-screen surface"))?;
            let cr = Context::new(&surface);
            cr.save();
            cr.set_source_rgba(0.0, 0.0, 0.0, 0.0);
            cr.set_operator(Operator::Source);
            cr.paint();
            cr.restore();

            cr.set_operator(Operator::Over);
            cr.set_source_rgba(0.0, 0.0, 0.0, SCATTER_PLOT_ALPHA);

            let v_k = f64::from(w) / (self.aoi.max_v - self.aoi.min_v);
            let v_b = -self.aoi.min_v * v_k;

            let i_k = f64::from(h) / (self.aoi.max_i - self.aoi.min_i);
            let i_b = -self.aoi.min_i * i_k;

            for (v, i) in self.trace.iter() {
                cr.arc(v_b + v * v_k, i_b + i * i_k, 1.0, 0.0, PI * 2.0);
                cr.fill();
            }
            drop(cr);
            *scatter_plot_mask = Some(surface);
        }

        if let Some(mask) = &*scatter_plot_mask {
            cr.mask_surface(&mask, 0.0, 0.0);
        }
        Ok(())
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
        // FIXME: trace should know it's very own device type
        let traces = if *self.traces.iter().last().unwrap().0 < r64(0.0) {
            self.traces.iter().rev().collect_vec()
        } else {
            self.traces.iter().collect_vec()
        };
        for ((_, trace), color) in traces.iter().zip(COLORS_F64.iter()) {
            let w = f64::from(MASK_WIDTH);
            let h = f64::from(MASK_HEIGHT);

            let v_k = w / (trace.aoi.max_v - trace.aoi.min_v);
            let v_b = trace.aoi.min_v * v_k;

            let i_k = h / (trace.aoi.max_i - trace.aoi.min_i);
            let i_b = trace.aoi.min_i * i_k;

            cr.save();
            cr.set_source_rgba(color.0, color.1, color.2, 1.0);
            cr.translate(0.0, height - 20.0);
            cr.scale(v_factor / v_k, i_factor / -i_k);
            cr.translate(v_b, i_b);
            trace.apply_mask(cr).unwrap();
            cr.fill();
            cr.restore();
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
