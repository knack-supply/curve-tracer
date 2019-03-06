use std::collections::BTreeMap;
use std::fs::File;
use std::path::PathBuf;

use csv::ReaderBuilder;
use itertools::Itertools;
use noisy_float::prelude::*;

use crate::backend::Backend;
use crate::backend::BiasedTrace;
use crate::backend::DeviceType;
use crate::backend::RawTrace;
use crate::Result;

pub struct Csv {
    filename: PathBuf,
}

#[derive(Deserialize)]
struct Record2 {
    i: f64,
    v: f64,
}

#[derive(Deserialize)]
struct Record3 {
    i: f64,
    v: f64,
    bias: f64,
}

impl Csv {
    pub fn new<P: Into<PathBuf>>(filename: P) -> Self {
        Csv {
            filename: filename.into(),
        }
    }
}

impl Backend for Csv {
    fn trace_2(&self, device_type: DeviceType) -> Result<RawTrace> {
        if device_type != DeviceType::PN {
            return Err(failure::err_msg(format!(
                "Unsupported device type {}",
                device_type
            )));
        }

        let mut rdr = ReaderBuilder::new()
            .delimiter(b'\t')
            .from_reader(File::open(&self.filename)?);

        let mut vs = Vec::new();
        let mut is = Vec::new();

        for result in rdr.deserialize() {
            let record: Record2 = result?;
            vs.push(record.v);
            is.push(record.i);
        }

        Ok(RawTrace::new(is, vs))
    }

    fn trace_3(&self, device_type: DeviceType) -> Result<Vec<BiasedTrace>> {
        match device_type {
            DeviceType::NPN | DeviceType::PNP => {}
            device_type => {
                return Err(failure::err_msg(format!(
                    "Unsupported device type {}",
                    device_type
                )));
            }
        };

        let mut rdr = ReaderBuilder::new()
            .delimiter(b'\t')
            .from_reader(File::open(&self.filename)?);

        let mut traces = BTreeMap::new();

        for result in rdr.deserialize() {
            let record: Record3 = result?;
            let bias = r64(record.bias);

            let (vs, is) = traces
                .entry(bias)
                .or_insert_with(|| (Vec::new(), Vec::new()));

            vs.push(record.v);
            is.push(record.i);
        }

        Ok(traces
            .into_iter()
            .map(|(bias, (vs, is))| BiasedTrace {
                bias,
                trace: RawTrace::new(is, vs),
            })
            .collect_vec())
    }
}
