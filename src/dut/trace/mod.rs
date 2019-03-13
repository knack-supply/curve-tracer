mod null;
mod three;
mod two;

pub use self::null::*;
pub use self::three::*;
pub use self::two::*;

use std::path::Path;

use crate::Result;
use cairo::Context;

pub trait ExportableTrace {
    fn save_as_csv(&self, path: &Path) -> Result<()>;
}

pub trait TraceWithModel {
    fn fill_model(&mut self);
    fn model_report(&self) -> String;
}

pub trait GuiTrace: DrawableTrace + ExportableTrace {}

pub trait TraceWithScatterPlot {
    fn apply_mask(&self, cr: &Context) -> Result<()>;
}

pub trait DrawableTrace: TraceWithModel {
    fn draw(&self, cr: &Context, v_factor: f64, i_factor: f64, height: f64);
    fn draw_model(&self, cr: &Context, v_factor: f64, i_factor: f64, height: f64);
}

impl GuiTrace for Box<dyn GuiTrace> {}

impl GuiTrace for NullTrace {}

impl GuiTrace for TwoTerminalTrace {}

impl GuiTrace for ThreeTerminalTrace {}

impl DrawableTrace for Box<dyn GuiTrace> {
    fn draw(&self, cr: &Context, v_factor: f64, i_factor: f64, height: f64) {
        GuiTrace::draw(&*self, cr, v_factor, i_factor, height)
    }

    fn draw_model(&self, cr: &Context, v_factor: f64, i_factor: f64, height: f64) {
        GuiTrace::draw_model(&*self, cr, v_factor, i_factor, height)
    }
}

impl TraceWithModel for Box<dyn GuiTrace> {
    fn fill_model(&mut self) {
        GuiTrace::fill_model(&mut *self)
    }

    fn model_report(&self) -> String {
        GuiTrace::model_report(&*self)
    }
}

impl ExportableTrace for Box<dyn GuiTrace> {
    fn save_as_csv(&self, path: &Path) -> Result<()> {
        GuiTrace::save_as_csv(&*self, path)
    }
}
