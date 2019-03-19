use crate::backend::RawTrace;
use crate::dut::aoi::AreaOfInterest;
use crate::dut::csv::csv_writer_from_path;
use crate::dut::trace::{DrawableTrace, Trace, TraceWithModel};
use crate::gui::{MASK_HEIGHT, MASK_WIDTH, SCATTER_PLOT_ALPHA};
use crate::model::diode::diode_model;
use crate::model::IVModel;
use crate::Result;
use cairo::{Context, Format, ImageSurface, Operator};
use itertools_num::linspace;
use std::cell::RefCell;
use std::f64::consts::PI;
use std::path::Path;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct TwoTerminalTrace {
    pub trace: RawTrace,
    pub aoi: AreaOfInterest,
}

#[derive(Clone)]
pub struct TwoTerminalGuiTrace {
    pub trace: TwoTerminalTrace,
    pub model: Option<Arc<dyn IVModel>>,
    scatter_plot_mask: RefCell<Option<ImageSurface>>,
}

impl TwoTerminalTrace {
    pub fn from_raw_trace(trace: RawTrace, aoi: AreaOfInterest) -> Self {
        Self { trace, aoi }
    }
}

impl From<TwoTerminalGuiTrace> for TwoTerminalTrace {
    fn from(trace: TwoTerminalGuiTrace) -> Self {
        trace.trace
    }
}

impl From<TwoTerminalTrace> for TwoTerminalGuiTrace {
    fn from(trace: TwoTerminalTrace) -> Self {
        TwoTerminalGuiTrace {
            trace,
            model: None,
            scatter_plot_mask: RefCell::new(None),
        }
    }
}

impl TraceWithModel for TwoTerminalGuiTrace {
    fn fill_model(&mut self) {
        if self.model.is_none() {
            self.model = Some(Arc::new(diode_model(&self.trace.trace)))
        }
    }

    fn model_report(&self) -> String {
        self.model
            .as_ref()
            .map(std::string::ToString::to_string)
            .unwrap_or_else(String::new)
    }
}

impl Trace for TwoTerminalTrace {
    fn area_of_interest(&self) -> AreaOfInterest {
        self.aoi
    }

    fn save_as_csv(&self, path: &Path) -> Result<()> {
        let mut out = csv_writer_from_path(path)?;

        let header = ["v", "i"];
        out.write_record(&header)?;
        for (v, i) in self.trace.iter() {
            let v_str = v.to_string();
            let i_str = i.to_string();
            let rec = [v_str.as_str(), i_str.as_str()];
            out.write_record(&rec)?;
        }
        out.close()?;
        Ok(())
    }
}

impl TwoTerminalGuiTrace {
    pub fn apply_mask(&self, cr: &Context) -> Result<()> {
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

            let aoi = self.trace.aoi;

            let v_k = f64::from(w) / (aoi.max_v - aoi.min_v);
            let v_b = -aoi.min_v * v_k;

            let i_k = f64::from(h) / (aoi.max_i - aoi.min_i);
            let i_b = -aoi.min_i * i_k;

            for (v, i) in self.trace.trace.iter() {
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

impl Trace for TwoTerminalGuiTrace {
    fn area_of_interest(&self) -> AreaOfInterest {
        self.trace.area_of_interest()
    }

    fn save_as_csv(&self, path: &Path) -> Result<()> {
        self.trace.save_as_csv(path)
    }
}

impl DrawableTrace for TwoTerminalGuiTrace {
    fn draw(&self, cr: &Context, v_factor: f64, i_factor: f64, height: f64) {
        let w = f64::from(MASK_WIDTH);
        let h = f64::from(MASK_HEIGHT);

        let v_k = w / (self.trace.aoi.max_v - self.trace.aoi.min_v);
        let v_b = self.trace.aoi.min_v * v_k;

        let i_k = h / (self.trace.aoi.max_i - self.trace.aoi.min_i);
        let i_b = self.trace.aoi.min_i * i_k;

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
