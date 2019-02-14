
use curve_tracer::model::diode::diode_model;
use failure::Error;
use curve_tracer::options::Opt;
use structopt::StructOpt;

fn main() -> Result<(), Error> {
    let opt = Opt::from_args();
    let trace = opt.device().trace()?;

    let model = diode_model(&trace);
    println!("Diode model: {:?}", model);
    Ok(())
}
