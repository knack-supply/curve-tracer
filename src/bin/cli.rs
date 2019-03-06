#[macro_use]
extern crate log;

use structopt::StructOpt;

use ks_curve_tracer::options::CliOpt;
use ks_curve_tracer::options::Opt;
use ks_curve_tracer::Result;

fn main() -> Result<()> {
    let opt = CliOpt::from_args();
    opt.initialize_logging()?;

    let mut trace = opt.trace()?;

    trace.fill_model();
    info!("Diode model: {:?}", trace.model_report());
    Ok(())
}
