use crate::dut::trace::DrawableTrace;
use crate::dut::trace::ExportableTrace;
use crate::dut::trace::TraceWithModel;
use crate::Result;
use cairo::Context;
use std::path::Path;
use crate::dut::aoi::AreaOfInterest;

#[derive(Copy, Clone)]
pub struct NullTrace {}

impl ExportableTrace for NullTrace {
    fn save_as_csv(&self, _: &Path) -> Result<()> {
        Err(failure::err_msg("No trace to save"))
    }
}

impl TraceWithModel for NullTrace {
    fn fill_model(&mut self) {}
    fn model_report(&self) -> String {
        String::new()
    }
}

impl DrawableTrace for NullTrace {
    fn area_of_interest(&self) -> AreaOfInterest {
        AreaOfInterest::default()
    }
    fn draw(&self, _: &Context, _: f64, _: f64, _: f64) {}
    fn draw_model(&self, _: &Context, _: f64, _: f64, _: f64) {}
}
