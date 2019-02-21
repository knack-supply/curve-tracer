use std::path::PathBuf;
use structopt::StructOpt;

use crate::backend::Backend;
use crate::backend::Csv;
use crate::backend::AD2;

#[derive(StructOpt, Debug)]
enum BackendOption {
    #[structopt(name = "dwf")]
    DWF,
    #[structopt(name = "csv")]
    Csv {
        #[structopt(short = "f", long = "file", parse(from_os_str))]
        file: PathBuf,
    },
}

#[derive(StructOpt, Debug)]
#[structopt(name = "curve-tracer-cli")]
pub struct Opt {
    #[structopt(subcommand)]
    device: Option<BackendOption>,
}

impl Opt {
    pub fn device(&self) -> Box<dyn Backend> {
        match &self.device.as_ref().unwrap_or(&BackendOption::DWF) {
            BackendOption::DWF => Box::new(AD2::new()),
            BackendOption::Csv { file } => Box::new(Csv::new(file.to_string_lossy().into_owned())),
        }
    }
}
