#[macro_use]
extern crate log;

use structopt::StructOpt;

use ks_curve_tracer::model::diode::diode_model;
use ks_curve_tracer::options::CliOpt;
use ks_curve_tracer::options::Opt;
use ks_curve_tracer::Result;

fn main() -> Result<()> {
    let opt = CliOpt::from_args();
    opt.initialize_logging()?;

    let trace = opt.device()?.trace_2(opt.device_type)?;

    let model = diode_model(&trace);
    info!("Diode model: {:?}", model);
    Ok(())
}
