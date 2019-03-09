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
use cairo::ImageSurface;
use noisy_float::types::R64;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fmt::Display;
use std::fmt::Formatter;
use std::str::FromStr;

pub mod backend;
pub mod gui;
pub mod model;
pub mod options;
pub mod trace;
pub mod util;

pub type Result<T> = std::result::Result<T, failure::Error>;

#[derive(Copy, Clone, Debug)]
pub struct AreaOfInterest {
    pub min_v: f64,
    pub max_v: f64,
    pub min_i: f64,
    pub max_i: f64,
}

impl AreaOfInterest {
    pub fn new_pos_i_pos_v(i: f64, v: f64) -> Self {
        Self {
            min_v: 0.0,
            max_v: v,
            min_i: 0.0,
            max_i: i,
        }
    }

    pub fn new_pos_i_neg_v(i: f64, v: f64) -> Self {
        Self {
            min_v: -v,
            max_v: 0.0,
            min_i: 0.0,
            max_i: i,
        }
    }

    pub fn extended(&self) -> Self {
        let slack = 0.1;
        let v_range = self.max_v - self.min_v;
        let i_range = self.max_i - self.min_i;
        Self {
            min_v: self.min_v - v_range * slack,
            max_v: self.max_v + v_range * slack,
            min_i: self.min_i - i_range * slack,
            max_i: self.max_i + i_range * slack,
        }
    }
}

impl From<DeviceType> for AreaOfInterest {
    fn from(d: DeviceType) -> Self {
        match d {
            DeviceType::TwoTerminal(TwoTerminalDeviceType::Diode) => {
                Self::new_pos_i_pos_v(0.05, 5.0).extended()
            }
            DeviceType::ThreeTerminal(ThreeTerminalDeviceType::NPN) => {
                Self::new_pos_i_pos_v(0.05, 5.0).extended()
            }
            DeviceType::ThreeTerminal(ThreeTerminalDeviceType::PNP) => {
                Self::new_pos_i_neg_v(0.05, 5.0).extended()
            }
            DeviceType::ThreeTerminal(ThreeTerminalDeviceType::NFET) => {
                Self::new_pos_i_pos_v(0.05, 5.0).extended()
            }
            DeviceType::ThreeTerminal(ThreeTerminalDeviceType::PFET) => {
                Self::new_pos_i_neg_v(0.05, 5.0).extended()
            }
        }
    }
}

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
    aoi: AreaOfInterest,
    model: Option<Arc<dyn IVModel>>,
    scatter_plot_mask: RefCell<Option<ImageSurface>>,
}

impl TwoTerminalTrace {
    pub fn from_raw_trace(trace: RawTrace, aoi: AreaOfInterest) -> Self {
        Self {
            trace,
            aoi,
            model: None,
            scatter_plot_mask: RefCell::new(None),
        }
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
