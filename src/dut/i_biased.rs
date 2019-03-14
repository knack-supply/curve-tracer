use crate::backend::{Backend, BiasedTrace};
use crate::dut::aoi::AreaOfInterest;
use crate::dut::trace::{ThreeTerminalTrace, TwoTerminalTrace};
use crate::dut::{BiasDrive, Device};
use crate::Result;
use itertools::Itertools;
use itertools_num::linspace;

use crate::dut::csv::load3_from_csv;
use noisy_float::prelude::{r64, R64};
use std::fmt::{Display, Formatter};
use std::path::Path;
use std::str::FromStr;

#[derive(Clone, Debug)]
pub struct CurrentBiasedDeviceConfig {
    pub min_bias_current: R64,
    pub max_bias_current: R64,
}

impl Default for CurrentBiasedDeviceConfig {
    fn default() -> Self {
        CurrentBiasedDeviceConfig {
            min_bias_current: r64(0.000_010),
            max_bias_current: r64(0.000_050),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CurrentBiasedDeviceType {
    NPN,
    PNP,
}

#[derive(Clone, Debug)]
pub struct CurrentBiasedDevice {
    config: CurrentBiasedDeviceConfig,
    device_type: CurrentBiasedDeviceType,
}

impl CurrentBiasedDevice {
    pub fn from_type(device_type: CurrentBiasedDeviceType) -> Self {
        CurrentBiasedDevice {
            config: CurrentBiasedDeviceConfig::default(),
            device_type,
        }
    }

    fn polarity(&self) -> R64 {
        match &self.device_type {
            CurrentBiasedDeviceType::NPN => r64(1.0),
            CurrentBiasedDeviceType::PNP => r64(-1.0),
        }
    }

    pub fn bias_levels(&self) -> Vec<R64> {
        let polarity = self.polarity();

        linspace(
            self.config.min_bias_current,
            self.config.max_bias_current,
            5,
        )
        .map(|l| l * polarity / r64(1_000_000.0))
        .collect_vec()
    }
}

impl From<&CurrentBiasedDevice> for CurrentBiasedDeviceType {
    fn from(d: &CurrentBiasedDevice) -> Self {
        d.device_type
    }
}

impl Device for CurrentBiasedDevice {
    type Trace = ThreeTerminalTrace;
    type Config = CurrentBiasedDeviceConfig;

    fn area_of_interest(&self) -> AreaOfInterest {
        match &self.device_type {
            CurrentBiasedDeviceType::NPN => AreaOfInterest::new_pos_i_pos_v(0.05, 5.0).extended(),
            CurrentBiasedDeviceType::PNP => AreaOfInterest::new_pos_i_neg_v(0.05, 5.0).extended(),
        }
    }

    fn trace(&self, backend: &dyn Backend) -> Result<Self::Trace> {
        let aoi = self.area_of_interest();
        Ok(ThreeTerminalTrace::new(
            self.polarity() < 0.0,
            backend
                .trace_3(self.polarity(), BiasDrive::Current, self.bias_levels())?
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

    fn config(&self) -> CurrentBiasedDeviceConfig {
        self.config.clone()
    }

    fn set_config(&mut self, config: &CurrentBiasedDeviceConfig) {
        self.config = config.clone();
    }
}

impl Display for CurrentBiasedDeviceType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CurrentBiasedDeviceType::NPN => f.write_str("NPN"),
            CurrentBiasedDeviceType::PNP => f.write_str("PNP"),
        }
    }
}

impl FromStr for CurrentBiasedDeviceType {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "NPN" => Ok(CurrentBiasedDeviceType::NPN),
            "PNP" => Ok(CurrentBiasedDeviceType::PNP),
            _ => Err(()),
        }
    }
}
