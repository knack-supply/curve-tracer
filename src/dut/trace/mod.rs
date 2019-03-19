mod null;
mod three;
mod two;

pub use self::null::*;
pub use self::three::*;
pub use self::two::*;

use std::fmt::Debug;
use std::path::Path;

use crate::dut::aoi::AreaOfInterest;
use crate::Result;
use cairo::Context;

pub trait Trace {
    fn area_of_interest(&self) -> AreaOfInterest;
    fn save_as_csv(&self, path: &Path) -> Result<()>;
}

pub trait TraceWithModel {
    fn fill_model(&mut self);
    fn model_report(&self) -> String;
}

pub trait ShareableTrace: Trace + Send + Sync + Debug {
    fn as_gui_trace(&self) -> Box<dyn GuiTrace>;
}

pub trait GuiTrace: Trace + DrawableTrace {}

pub trait DrawableTrace: TraceWithModel {
    fn draw(&self, cr: &Context, v_factor: f64, i_factor: f64, height: f64);
    fn draw_model(&self, cr: &Context, v_factor: f64, i_factor: f64, height: f64);
}

impl Trace for Box<dyn ShareableTrace> {
    fn area_of_interest(&self) -> AreaOfInterest {
        ShareableTrace::area_of_interest(&**self)
    }

    fn save_as_csv(&self, path: &Path) -> Result<()> {
        ShareableTrace::save_as_csv(&**self, path)
    }
}

impl GuiTrace for NullTrace {}

impl GuiTrace for TwoTerminalGuiTrace {}

impl GuiTrace for ThreeTerminalGuiTrace {}

impl ShareableTrace for TwoTerminalTrace {
    fn as_gui_trace(&self) -> Box<dyn GuiTrace> {
        Box::new(TwoTerminalGuiTrace::from(self.clone()))
    }
}

impl ShareableTrace for ThreeTerminalTrace {
    fn as_gui_trace(&self) -> Box<dyn GuiTrace> {
        Box::new(ThreeTerminalGuiTrace::from(self.clone()))
    }
}
