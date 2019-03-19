use std::path::PathBuf;

use log::LevelFilter;
use simplelog::Config;
use structopt::StructOpt;

use crate::backend::Backend;
use crate::backend::AD2;
use crate::dut::trace::{GuiTrace, TwoTerminalGuiTrace};
use crate::dut::Device;
use crate::dut::TwoTerminalDevice;
use crate::Result;

pub trait Opt {
    fn initialize_logging(&self) -> Result<()>;
}

#[derive(StructOpt, Debug)]
enum CliBackendOption {
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
    device: Option<CliBackendOption>,
}

impl Opt for CliOpt {
    fn initialize_logging(&self) -> Result<()> {
        simplelog::TermLogger::init(LevelFilter::Debug, Config::default())?;
        Ok(())
    }
}

impl CliOpt {
    pub fn trace(&self) -> Result<Box<dyn GuiTrace>> {
        Ok(
            match &self.device.as_ref().unwrap_or(&CliBackendOption::DWF) {
                CliBackendOption::DWF => Box::new(TwoTerminalGuiTrace::from(
                    TwoTerminalDevice::Diode.trace(&AD2::new()?)?,
                )),
                CliBackendOption::Csv { file } => Box::new(TwoTerminalGuiTrace::from(
                    TwoTerminalDevice::Diode.load_from_csv(file.as_path())?,
                )),
            },
        )
    }
}

#[derive(StructOpt, Debug)]
enum GuiBackendOption {
    #[structopt(name = "dwf")]
    DWF,
}

#[derive(StructOpt, Debug)]
#[structopt(name = "curve-tracer")]
pub struct GuiOpt {
    #[structopt(subcommand)]
    device: Option<GuiBackendOption>,
}

impl Opt for GuiOpt {
    fn initialize_logging(&self) -> Result<()> {
        simplelog::TermLogger::init(LevelFilter::Debug, Config::default())?;
        Ok(())
    }
}

impl GuiOpt {
    pub fn device(&self) -> Result<Box<dyn Backend>> {
        match &self.device.as_ref().unwrap_or(&GuiBackendOption::DWF) {
            GuiBackendOption::DWF => Ok(Box::new(AD2::new()?)),
        }
    }
}
