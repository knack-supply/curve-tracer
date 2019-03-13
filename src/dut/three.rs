use crate::backend::{Backend, BiasedTrace, RawTrace};
use crate::dut::aoi::AreaOfInterest;
use crate::dut::csv::csv_reader_from_path;
use crate::dut::trace::{ThreeTerminalTrace, TwoTerminalTrace};
use crate::dut::{BiasDrive, DeviceType};
use crate::Result;
use itertools::Itertools;
use noisy_float::prelude::r64;
use std::collections::btree_map::BTreeMap;
use std::fmt::{Display, Formatter};
use std::path::Path;
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ThreeTerminalDeviceType {
    NPN,
    PNP,
    NFET,
    PFET,
}

impl DeviceType for ThreeTerminalDeviceType {
    type Trace = ThreeTerminalTrace;

    fn area_of_interest(&self) -> AreaOfInterest {
        match &self {
            ThreeTerminalDeviceType::NPN | ThreeTerminalDeviceType::NFET => {
                AreaOfInterest::new_pos_i_pos_v(0.05, 5.0).extended()
            }
            ThreeTerminalDeviceType::PNP | ThreeTerminalDeviceType::PFET => {
                AreaOfInterest::new_pos_i_neg_v(0.05, 5.0).extended()
            }
        }
    }

    fn trace(&self, backend: &dyn Backend) -> Result<Self::Trace> {
        let aoi = self.area_of_interest();
        Ok(ThreeTerminalTrace::new(
            self.clone(),
            backend
                .trace_3(self)?
                .into_iter()
                .map(|BiasedTrace { bias, trace }| {
                    (bias, TwoTerminalTrace::from_raw_trace(trace, aoi))
                })
                .collect(),
        ))
    }

    fn load_from_csv<P: AsRef<Path>>(&self, path: P) -> Result<Self::Trace> {
        let mut traces = BTreeMap::new();

        for result in csv_reader_from_path(path.as_ref())? {
            let record: Record3 = result?;
            let bias = r64(record.bias);

            let (vs, is) = traces
                .entry(bias)
                .or_insert_with(|| (Vec::new(), Vec::new()));

            vs.push(record.v);
            is.push(record.i);
        }

        let aoi = self.area_of_interest();

        Ok(ThreeTerminalTrace::new(
            self.clone(),
            traces
                .into_iter()
                .map(|(bias, (vs, is))| {
                    (
                        bias,
                        TwoTerminalTrace::from_raw_trace(RawTrace::new(is, vs), aoi),
                    )
                })
                .collect(),
        ))
    }

    fn polarity(&self) -> f64 {
        match &self {
            ThreeTerminalDeviceType::NPN | ThreeTerminalDeviceType::NFET => 1.0,
            ThreeTerminalDeviceType::PNP | ThreeTerminalDeviceType::PFET => -1.0,
        }
    }

    fn bias_levels(&self) -> Vec<f64> {
        let polarity = self.polarity();
        match &self {
            ThreeTerminalDeviceType::NPN | ThreeTerminalDeviceType::PNP => {
                [10.0, 20.0, 30.0, 40.0, 50.0]
                    .iter()
                    .map(|l| l * polarity / 1_000_000.0)
                    .collect_vec()
            }
            ThreeTerminalDeviceType::NFET | ThreeTerminalDeviceType::PFET => {
                [1.0, 2.0, 3.0, 4.0, 5.0]
                    .iter()
                    .map(|l| l * polarity)
                    .collect_vec()
            }
        }
    }

    fn bias_drive(&self) -> BiasDrive {
        match &self {
            ThreeTerminalDeviceType::NPN | ThreeTerminalDeviceType::PNP => BiasDrive::Current,
            ThreeTerminalDeviceType::NFET | ThreeTerminalDeviceType::PFET => BiasDrive::Voltage,
        }
    }
}

#[derive(Deserialize)]
struct Record3 {
    i: f64,
    v: f64,
    bias: f64,
}

impl Display for ThreeTerminalDeviceType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ThreeTerminalDeviceType::NPN => f.write_str("NPN"),
            ThreeTerminalDeviceType::PNP => f.write_str("PNP"),
            ThreeTerminalDeviceType::NFET => f.write_str("NFET"),
            ThreeTerminalDeviceType::PFET => f.write_str("PFET"),
        }
    }
}

impl FromStr for ThreeTerminalDeviceType {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "NPN" => Ok(ThreeTerminalDeviceType::NPN),
            "PNP" => Ok(ThreeTerminalDeviceType::PNP),
            "NFET" => Ok(ThreeTerminalDeviceType::NFET),
            "PFET" => Ok(ThreeTerminalDeviceType::PFET),
            _ => Err(()),
        }
    }
}
