use crate::backend::{Backend, RawTrace};
use crate::dut::aoi::AreaOfInterest;
use crate::dut::csv::csv_reader_from_path;
use crate::dut::trace::TwoTerminalTrace;
use crate::dut::Device;
use crate::Result;
use std::fmt::{Display, Formatter};
use std::path::Path;
use std::str::FromStr;

#[derive(Clone, Debug)]
pub struct TwoTerminalDeviceConfig {}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TwoTerminalDeviceType {
    Diode,
}

#[derive(Clone, Debug)]
pub enum TwoTerminalDevice {
    Diode,
}

impl From<&TwoTerminalDevice> for TwoTerminalDeviceType {
    fn from(d: &TwoTerminalDevice) -> Self {
        match d {
            TwoTerminalDevice::Diode => TwoTerminalDeviceType::Diode,
        }
    }
}

impl Display for TwoTerminalDeviceType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TwoTerminalDeviceType::Diode => f.write_str("PN"),
        }
    }
}

impl FromStr for TwoTerminalDeviceType {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "PN" => Ok(TwoTerminalDeviceType::Diode),
            _ => Err(()),
        }
    }
}

impl Device for TwoTerminalDevice {
    type Trace = TwoTerminalTrace;
    type Config = TwoTerminalDeviceConfig;

    fn area_of_interest(&self) -> AreaOfInterest {
        AreaOfInterest::new_pos_i_pos_v(0.05, 5.0).extended()
    }

    fn trace(&self, backend: &dyn Backend) -> Result<Self::Trace> {
        Ok(TwoTerminalTrace::from_raw_trace(
            backend.trace_2()?,
            self.area_of_interest(),
        ))
    }

    fn load_from_csv<P: AsRef<Path>>(&self, path: P) -> Result<Self::Trace> {
        let mut vs = Vec::new();
        let mut is = Vec::new();

        for result in csv_reader_from_path(path.as_ref())? {
            let record: Record2 = result?;
            vs.push(record.v);
            is.push(record.i);
        }

        Ok(TwoTerminalTrace::from_raw_trace(
            RawTrace::new(is, vs),
            self.area_of_interest(),
        ))
    }

    fn config(&self) -> TwoTerminalDeviceConfig {
        TwoTerminalDeviceConfig {}
    }

    fn set_config(&mut self, _: &Self::Config) {}
}

#[derive(Deserialize)]
struct Record2 {
    i: f64,
    v: f64,
}
