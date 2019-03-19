mod ad2;

pub use self::ad2::AD2;

use crate::dut::BiasDrive;
use noisy_float::prelude::*;
use std::fmt::Debug;

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

impl Debug for RawTrace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("RawTrace({})", self.current.len()))
    }
}

#[allow(clippy::len_without_is_empty)]
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

    pub fn len(&self) -> usize {
        self.voltage.len()
    }
}

pub trait Backend: Send + Sync {
    fn trace_2(&self) -> crate::Result<RawTrace>;
    fn trace_3(
        &self,
        polarity: R64,
        bias_drive: BiasDrive,
        bias_levels: Vec<R64>,
    ) -> crate::Result<Vec<BiasedTrace>>;
}
