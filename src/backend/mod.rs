mod ad2;

pub use self::ad2::AD2;

use crate::ThreeTerminalDeviceType;
use crate::TwoTerminalDeviceType;
use noisy_float::prelude::*;

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

pub trait Backend {
    fn trace_2(&self, device_type: TwoTerminalDeviceType) -> crate::Result<RawTrace>;
    fn trace_3(&self, device_type: ThreeTerminalDeviceType) -> crate::Result<Vec<BiasedTrace>>;
}
