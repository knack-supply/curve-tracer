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
}

impl Default for VoltageBiasedDeviceConfig {
    fn default() -> Self {
        VoltageBiasedDeviceConfig {
            min_bias_voltage: r64(0.0),
            max_bias_voltage: r64(5.0),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum VoltageBiasedDeviceType {
    NFET,
    PFET,
}

#[derive(Clone, Debug)]
pub struct VoltageBiasedDevice {
    config: VoltageBiasedDeviceConfig,
    device_type: VoltageBiasedDeviceType,
}

impl VoltageBiasedDevice {
    pub fn from_type(device_type: VoltageBiasedDeviceType) -> Self {
        VoltageBiasedDevice {
            config: VoltageBiasedDeviceConfig::default(),
            device_type,
        }
    }

    fn polarity(&self) -> R64 {
        match &self.device_type {
            VoltageBiasedDeviceType::NFET => r64(1.0),
            VoltageBiasedDeviceType::PFET => r64(-1.0),
        }
    }

    pub fn bias_levels(&self) -> Vec<R64> {
        let polarity = self.polarity();

        linspace(
            self.config.min_bias_voltage,
            self.config.max_bias_voltage,
            5,
        )
        .map(|l| l * polarity)
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
            VoltageBiasedDeviceType::NFET => AreaOfInterest::new_pos_i_pos_v(0.05, 5.0).extended(),
            VoltageBiasedDeviceType::PFET => AreaOfInterest::new_pos_i_neg_v(0.05, 5.0).extended(),
        }
    }

    fn trace(&self, backend: &dyn Backend) -> Result<Self::Trace> {
        let aoi = self.area_of_interest();
        Ok(ThreeTerminalTrace::new(
            self.polarity() < 0.0,
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
            VoltageBiasedDeviceType::NFET => f.write_str("NFET"),
            VoltageBiasedDeviceType::PFET => f.write_str("PFET"),
        }
    }
}

impl FromStr for VoltageBiasedDeviceType {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "NFET" => Ok(VoltageBiasedDeviceType::NFET),
            "PFET" => Ok(VoltageBiasedDeviceType::PFET),
            _ => Err(()),
        }
    }
}
