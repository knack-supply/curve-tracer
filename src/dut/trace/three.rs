use std::collections::btree_map::BTreeMap;
use std::path::Path;

use cairo::Context;
use itertools::Itertools;
use noisy_float::prelude::R64;

use crate::dut::csv::csv_writer_from_path;
use crate::dut::trace::{
    DrawableTrace, ExportableTrace, TraceWithModel, TraceWithScatterPlot, TwoTerminalTrace,
};
use crate::gui::COLORS_F64;
use crate::gui::{MASK_HEIGHT, MASK_WIDTH};
use crate::Result;
use crate::dut::aoi::AreaOfInterest;

#[derive(Clone)]
pub struct ThreeTerminalTrace {
    reverse_order: bool,
    pub traces: BTreeMap<R64, TwoTerminalTrace>,
}

impl ThreeTerminalTrace {
    pub fn new(reverse_order: bool, traces: BTreeMap<R64, TwoTerminalTrace>) -> ThreeTerminalTrace {
        ThreeTerminalTrace {
            reverse_order,
            traces,
        }
    }
}

impl ExportableTrace for ThreeTerminalTrace {
    fn save_as_csv(&self, path: &Path) -> Result<()> {
        let mut out = csv_writer_from_path(path)?;

        let header = ["v", "i", "bias"];
        out.write_record(&header)?;
        for (bias, trace) in self.traces.iter() {
            let bias_str = bias.to_string();
            for (v, i) in trace.trace.iter() {
                let v_str = v.to_string();
                let i_str = i.to_string();
                let rec = [v_str.as_str(), i_str.as_str(), bias_str.as_str()];
                out.write_record(&rec)?;
            }
        }
        out.close()?;
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
    fn area_of_interest(&self) -> AreaOfInterest {
        if let Some((_, trace)) = self.traces.iter().next() {
            trace.aoi
        } else {
            AreaOfInterest::default()
        }
    }

    fn draw(&self, cr: &Context, v_factor: f64, i_factor: f64, height: f64) {
        let traces = if self.reverse_order {
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
