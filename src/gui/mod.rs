use crate::backend::BiasedTrace;
use crate::backend::DeviceType;
use crate::Trace;
use cairo::Context;
use itertools::Itertools;
use itertools_num::linspace;
use std::f64::consts::PI;

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
}

pub trait DrawableTrace {
    fn draw(&self, cr: &Context, v_factor: f64, i_factor: f64, height: f64);
    fn draw_model(&self, cr: &Context, v_factor: f64, i_factor: f64, height: f64);
}

impl DrawableTrace for Trace {
    fn draw(&self, cr: &Context, v_factor: f64, i_factor: f64, height: f64) {
        match self {
            Trace::Null => {}
            Trace::Unbiased { trace, model: _ } => {
                cr.set_source_rgba(0.0, 0.0, 0.0, 0.05);
                for (v, i) in trace.iter() {
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
            Trace::Biased { traces } => {
                let colors = [
                    (57, 106, 177),
                    (218, 124, 48),
                    (62, 150, 81),
                    (204, 37, 41),
                    (83, 81, 84),
                    (107, 76, 154),
                    (146, 36, 40),
                    (148, 139, 61),
                ]
                .iter()
                .cloned()
                .map(|(r, g, b)| (r as f64 / 255.0, g as f64 / 255.0, b as f64 / 255.0));

                for (BiasedTrace { bias: _, trace }, color) in traces.iter().zip(colors) {
                    cr.set_source_rgba(color.0, color.1, color.2, 0.05);
                    for (v, i) in trace.iter() {
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
        }
    }

    fn draw_model(&self, cr: &Context, v_factor: f64, i_factor: f64, height: f64) {
        match self {
            Trace::Unbiased {
                trace: _,
                model: Some(model),
            } => {
                cr.set_source_rgba(1.0, 0.0, 0.0, 0.8);

                for (ix, v) in
                    linspace(model.min_v().max(0.0), model.max_v().min(5.0), 101).enumerate()
                {
                    if ix == 0 {
                        cr.move_to(v * v_factor, height - 20.0 - i_factor * model.evaluate(v));
                    } else {
                        cr.line_to(v * v_factor, height - 20.0 - i_factor * model.evaluate(v));
                    }
                }
                cr.stroke();
            }
            _ => {}
        }
    }
}

pub trait DevicePlot {
    fn polarity(&self) -> f64;
    fn connection_hint(&self) -> &'static str;
    fn legend(&self) -> String;
}

impl DevicePlot for DeviceType {
    fn polarity(&self) -> f64 {
        match &self {
            DeviceType::PN => 1.0,
            DeviceType::NPN => 1.0,
            DeviceType::PNP => -1.0,
        }
    }
    fn connection_hint(&self) -> &'static str {
        match self {
            DeviceType::PN => "Top row: AKKKKKK",
            DeviceType::NPN => "Bottom row: CBECBEC",
            DeviceType::PNP => "Bottom row: EBCEBCE (reversed E/C)",
        }
    }
    fn legend(&self) -> String {
        match self {
            DeviceType::PN => String::new(),
            DeviceType::NPN => format!(r###"I<sub>BE</sub>: <span fgcolor="white" bgcolor="{}">10µA</span> <span fgcolor="white" bgcolor="{}">20µA</span> <span fgcolor="white" bgcolor="{}">30µA</span> <span fgcolor="white" bgcolor="{}">40µA</span> <span fgcolor="white" bgcolor="{}">50µA</span>"###,
                                       COLORS_HEX[0], COLORS_HEX[1], COLORS_HEX[2], COLORS_HEX[3], COLORS_HEX[4]),
            DeviceType::PNP => format!(r###"I<sub>BE</sub>: <span fgcolor="white" bgcolor="{}">-10µA</span> <span fgcolor="white" bgcolor="{}">-20µA</span> <span fgcolor="white" bgcolor="{}">-30µA</span> <span fgcolor="white" bgcolor="{}">-40µA</span> <span fgcolor="white" bgcolor="{}">-50µA</span>"###,
                                       COLORS_HEX[0], COLORS_HEX[1], COLORS_HEX[2], COLORS_HEX[3], COLORS_HEX[4]),
        }
    }
}
