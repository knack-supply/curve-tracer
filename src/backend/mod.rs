mod ad2;

pub use self::ad2::AD2;

use noisy_float::prelude::*;
use std::fmt::Display;
use std::fmt::Formatter;
use std::str::FromStr;

#[derive(Clone)]
pub struct BiasedTrace {
    pub bias: R64,
    pub trace: RawTrace,
}

#[derive(Clone)]
pub struct RawTrace {
    current: Vec<f64>,
    voltage: Vec<f64>,
}

impl RawTrace {
    pub fn new(current: Vec<f64>, voltage: Vec<f64>) -> Self {
        assert_eq!(current.len(), voltage.len());
        RawTrace { current, voltage }
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = (f64, f64)> + 'a {
        self.voltage
            .iter()
            .cloned()
            .zip(self.current.iter().cloned())
    }
}

pub trait Backend {
    fn trace_2(&self, device_type: DeviceType) -> crate::Result<RawTrace>;
    fn trace_3(&self, device_type: DeviceType) -> crate::Result<Vec<BiasedTrace>>;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DeviceType {
    PN,
    NPN,
    PNP,
}

impl Display for DeviceType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceType::PN => f.write_str("PN"),
            DeviceType::NPN => f.write_str("NPN"),
            DeviceType::PNP => f.write_str("PNP"),
        }
    }
}

impl FromStr for DeviceType {
    type Err = failure::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "PN" => Ok(DeviceType::PN),
            "NPN" => Ok(DeviceType::NPN),
            "PNP" => Ok(DeviceType::PNP),
            v => Err(failure::err_msg(format!("Invalid device type: {}", v))),
        }
    }
}
