#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate measure_time;
#[macro_use]
extern crate serde_derive;

use std::sync::Arc;

use structopt;

use crate::backend::RawTrace;
use crate::model::IVModel;
use noisy_float::types::R64;
use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

pub mod backend;
pub mod gui;
pub mod model;
pub mod options;
pub mod trace;
pub mod util;

pub type Result<T> = std::result::Result<T, failure::Error>;

#[derive(Copy, Clone)]
pub struct NullTrace {}

#[derive(Copy, Clone)]
pub enum DeviceType {
    TwoTerminal(TwoTerminalDeviceType),
    ThreeTerminal(ThreeTerminalDeviceType),
}

impl Display for DeviceType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceType::TwoTerminal(device) => device.fmt(f),
            DeviceType::ThreeTerminal(device) => device.fmt(f),
        }
    }
}

impl FromStr for DeviceType {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        TwoTerminalDeviceType::from_str(s)
            .map(DeviceType::TwoTerminal)
            .or_else(|()| ThreeTerminalDeviceType::from_str(s).map(DeviceType::ThreeTerminal))
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TwoTerminalDeviceType {
    Diode,
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

#[derive(Clone)]
pub struct TwoTerminalTrace {
    trace: RawTrace,
    model: Option<Arc<dyn IVModel>>,
}

impl From<RawTrace> for TwoTerminalTrace {
    fn from(trace: RawTrace) -> Self {
        TwoTerminalTrace { trace, model: None }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ThreeTerminalDeviceType {
    NPN,
    PNP,
    NFET,
    PFET,
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

#[derive(Clone)]
pub struct ThreeTerminalTrace {
    traces: BTreeMap<R64, TwoTerminalTrace>,
}

impl<B, T, I> From<I> for ThreeTerminalTrace
where
    B: Into<R64>,
    T: Into<TwoTerminalTrace>,
    I: IntoIterator<Item = (B, T)> + Sized,
{
    fn from(i: I) -> Self {
        ThreeTerminalTrace {
            traces: i
                .into_iter()
                .map(|(bias, traces)| (bias.into(), traces.into()))
                .collect(),
        }
    }
}
