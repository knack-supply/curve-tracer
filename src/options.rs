use std::path::PathBuf;

use log::LevelFilter;
use simplelog::Config;
use structopt::StructOpt;

use crate::backend::Backend;
use crate::backend::Csv;
use crate::backend::DeviceType;
use crate::backend::AD2;
use crate::Result;

pub trait Opt {
    fn device(&self) -> Result<Box<dyn Backend>>;
    fn initialize_logging(&self) -> Result<()>;
}

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
pub struct CliOpt {
    #[structopt(subcommand)]
    device: Option<BackendOption>,
    #[structopt(name = "type")]
    pub device_type: DeviceType,
}

impl Opt for CliOpt {
    fn device(&self) -> Result<Box<dyn Backend>> {
        Ok(match &self.device.as_ref().unwrap_or(&BackendOption::DWF) {
            BackendOption::DWF => Box::new(AD2::new()?),
            BackendOption::Csv { file } => Box::new(Csv::new(file.to_string_lossy().into_owned())),
        })
    }

    fn initialize_logging(&self) -> Result<()> {
        simplelog::TermLogger::init(LevelFilter::Debug, Config::default())?;
        Ok(())
    }
}

#[derive(StructOpt, Debug)]
#[structopt(name = "curve-tracer")]
pub struct GuiOpt {
    #[structopt(subcommand)]
    device: Option<BackendOption>,
}

impl Opt for GuiOpt {
    fn device(&self) -> Result<Box<dyn Backend>> {
        Ok(match &self.device.as_ref().unwrap_or(&BackendOption::DWF) {
            BackendOption::DWF => Box::new(AD2::new()?),
            BackendOption::Csv { file } => Box::new(Csv::new(file.to_string_lossy().into_owned())),
        })
    }

    fn initialize_logging(&self) -> Result<()> {
        simplelog::TermLogger::init(LevelFilter::Debug, Config::default())?;
        Ok(())
    }
}
