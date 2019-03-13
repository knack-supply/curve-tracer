mod aoi;
mod csv;
mod three;
pub mod trace;
mod two;

pub use self::three::*;
pub use self::two::*;
use crate::backend::Backend;
use crate::dut::aoi::AreaOfInterest;
use crate::dut::trace::GuiTrace;
use crate::Result;
use std::fmt::{Display, Formatter};
use std::path::Path;
use std::str::FromStr;

#[derive(Clone)]
pub enum SomeDeviceType {
    TwoTerminal(TwoTerminalDeviceType),
    ThreeTerminal(ThreeTerminalDeviceType),
}

pub enum BiasDrive {
    Voltage,
    Current,
}

pub trait DeviceType {
    type Trace: GuiTrace;
    fn area_of_interest(&self) -> AreaOfInterest;
    fn trace(&self, backend: &dyn Backend) -> Result<Self::Trace>;
    fn load_from_csv<P: AsRef<Path>>(&self, path: P) -> Result<Self::Trace>;
    fn polarity(&self) -> f64;
    fn bias_levels(&self) -> Vec<f64>;
    fn bias_drive(&self) -> BiasDrive;
}

impl DeviceType for SomeDeviceType {
    type Trace = Box<dyn GuiTrace>;

    fn area_of_interest(&self) -> AreaOfInterest {
        match self {
            SomeDeviceType::TwoTerminal(device_type) => device_type.area_of_interest(),
            SomeDeviceType::ThreeTerminal(device_type) => device_type.area_of_interest(),
        }
    }

    fn trace(&self, backend: &dyn Backend) -> Result<Self::Trace> {
        Ok(match self {
            SomeDeviceType::TwoTerminal(device_type) => Box::new(device_type.trace(backend)?),
            SomeDeviceType::ThreeTerminal(device_type) => Box::new(device_type.trace(backend)?),
        })
    }

    fn load_from_csv<P: AsRef<Path>>(&self, path: P) -> Result<Self::Trace> {
        Ok(match self {
            SomeDeviceType::TwoTerminal(device_type) => Box::new(device_type.load_from_csv(path)?),
            SomeDeviceType::ThreeTerminal(device_type) => {
                Box::new(device_type.load_from_csv(path)?)
            }
        })
    }

    fn polarity(&self) -> f64 {
        match self {
            SomeDeviceType::TwoTerminal(device_type) => device_type.polarity(),
            SomeDeviceType::ThreeTerminal(device_type) => device_type.polarity(),
        }
    }

    fn bias_levels(&self) -> Vec<f64> {
        match self {
            SomeDeviceType::TwoTerminal(device_type) => device_type.bias_levels(),
            SomeDeviceType::ThreeTerminal(device_type) => device_type.bias_levels(),
        }
    }

    fn bias_drive(&self) -> BiasDrive {
        match self {
            SomeDeviceType::TwoTerminal(device_type) => device_type.bias_drive(),
            SomeDeviceType::ThreeTerminal(device_type) => device_type.bias_drive(),
        }
    }
}

impl Display for SomeDeviceType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SomeDeviceType::TwoTerminal(device) => device.fmt(f),
            SomeDeviceType::ThreeTerminal(device) => device.fmt(f),
        }
    }
}

impl FromStr for SomeDeviceType {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        TwoTerminalDeviceType::from_str(s)
            .map(SomeDeviceType::TwoTerminal)
            .or_else(|()| ThreeTerminalDeviceType::from_str(s).map(SomeDeviceType::ThreeTerminal))
    }
}
