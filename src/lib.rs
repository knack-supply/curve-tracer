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

pub mod backend;
pub mod gui;
pub mod model;
pub mod options;
pub mod trace;
pub mod util;

pub type Result<T> = std::result::Result<T, failure::Error>;

#[derive(Copy, Clone)]
pub struct NullTrace {}

pub enum TwoTerminalDevice {
    Diode,
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

pub enum ThreeTerminalDevice {
    NPN,
    PNP,
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
