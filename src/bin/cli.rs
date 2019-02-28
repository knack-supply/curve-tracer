#[macro_use]
extern crate log;

use failure::Error;
use ks_curve_tracer::model::diode::diode_model;
use ks_curve_tracer::options::Opt;
use structopt::StructOpt;

fn main() -> Result<(), Error> {
    let opt = Opt::from_args();
    opt.initialize_logging()?;

    let trace = opt.device().trace()?;

    let model = diode_model(&trace);
    info!("Diode model: {:?}", model);
    Ok(())
}
