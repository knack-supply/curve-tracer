use std::fmt::{Display, Formatter};
use std::path::Path;
use std::str::FromStr;

use itertools::Itertools;
use itertools_num::linspace;
use noisy_float::prelude::{r64, R64};

use crate::backend::{Backend, BiasedTrace};
use crate::dut::aoi::AreaOfInterest;
use crate::dut::csv::load3_from_csv;
use crate::dut::trace::{ThreeTerminalTrace, TwoTerminalTrace};
use crate::dut::{BiasDrive, Device};
use crate::Result;

#[derive(Clone, Debug)]
pub struct VoltageBiasedDeviceConfig {
    pub min_bias_voltage: R64,
    pub max_bias_voltage: R64,
    pub device_type: VoltageBiasedDeviceType,
}

impl VoltageBiasedDeviceConfig {
    fn new_with_device_type(device_type: VoltageBiasedDeviceType) -> Self {
        VoltageBiasedDeviceConfig {
            min_bias_voltage: r64(0.0),
            max_bias_voltage: r64(5.0),
            device_type,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum VoltageBiasedDeviceType {
    NEFET,
    PEFET,
    NDFET,
    PDFET,
}

#[derive(Clone, Debug)]
pub struct VoltageBiasedDevice {
    config: VoltageBiasedDeviceConfig,
    device_type: VoltageBiasedDeviceType,
}

impl VoltageBiasedDevice {
    pub fn from_type(device_type: VoltageBiasedDeviceType) -> Self {
        VoltageBiasedDevice {
            config: VoltageBiasedDeviceConfig::new_with_device_type(device_type),
            device_type,
        }
    }

    fn polarity(&self) -> R64 {
        match &self.device_type {
            VoltageBiasedDeviceType::NEFET | VoltageBiasedDeviceType::NDFET => r64(1.0),
            VoltageBiasedDeviceType::PEFET | VoltageBiasedDeviceType::PDFET => r64(-1.0),
        }
    }

    fn bias_polarity(&self) -> R64 {
        match &self.device_type {
            VoltageBiasedDeviceType::NEFET | VoltageBiasedDeviceType::PDFET => r64(1.0),
            VoltageBiasedDeviceType::NDFET | VoltageBiasedDeviceType::PEFET => r64(-1.0),
        }
    }

    pub fn bias_levels(&self) -> Vec<R64> {
        let bias_polarity = self.bias_polarity();

        linspace(
            self.config.min_bias_voltage,
            self.config.max_bias_voltage,
            5,
        )
        .map(|l| l * bias_polarity)
        .collect_vec()
    }
}

impl From<&VoltageBiasedDevice> for VoltageBiasedDeviceType {
    fn from(d: &VoltageBiasedDevice) -> Self {
        d.device_type
    }
}

impl Device for VoltageBiasedDevice {
    type Trace = ThreeTerminalTrace;
    type Config = VoltageBiasedDeviceConfig;

    fn area_of_interest(&self) -> AreaOfInterest {
        match &self.device_type {
            VoltageBiasedDeviceType::NEFET | VoltageBiasedDeviceType::NDFET => {
                AreaOfInterest::new_pos_i_pos_v(0.05, 5.0).extended()
            }
            VoltageBiasedDeviceType::PEFET | VoltageBiasedDeviceType::PDFET => {
                AreaOfInterest::new_pos_i_neg_v(0.05, 5.0).extended()
            }
        }
    }

    fn trace(&self, backend: &dyn Backend) -> Result<Self::Trace> {
        let aoi = self.area_of_interest();
        Ok(ThreeTerminalTrace::new(
            self.bias_polarity() < 0.0,
            backend
                .trace_3(self.polarity(), BiasDrive::Voltage, self.bias_levels())?
                .into_iter()
                .map(|BiasedTrace { bias, trace }| {
                    (bias, TwoTerminalTrace::from_raw_trace(trace, aoi))
                })
                .collect(),
        ))
    }

    fn load_from_csv<P: AsRef<Path>>(&self, path: P) -> Result<Self::Trace> {
        load3_from_csv(path, self.polarity() < 0.0, self.area_of_interest())
    }

    fn config(&self) -> VoltageBiasedDeviceConfig {
        self.config.clone()
    }

    fn set_config(&mut self, config: &VoltageBiasedDeviceConfig) {
        self.config = config.clone();
    }
}

impl Display for VoltageBiasedDeviceType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            VoltageBiasedDeviceType::NEFET => f.write_str("NEFET"),
            VoltageBiasedDeviceType::PEFET => f.write_str("PEFET"),
            VoltageBiasedDeviceType::NDFET => f.write_str("NDFET"),
            VoltageBiasedDeviceType::PDFET => f.write_str("PDFET"),
        }
    }
}

impl FromStr for VoltageBiasedDeviceType {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "NEFET" => Ok(VoltageBiasedDeviceType::NEFET),
            "PEFET" => Ok(VoltageBiasedDeviceType::PEFET),
            "NDFET" => Ok(VoltageBiasedDeviceType::NDFET),
            "PDFET" => Ok(VoltageBiasedDeviceType::PDFET),
            _ => Err(()),
        }
    }
}
