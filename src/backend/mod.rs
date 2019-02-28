mod ad2;
mod csv;

use failure::Error;

type Result<T> = std::result::Result<T, Error>;

pub struct RawTrace {
    pub current: Vec<f64>,
    pub voltage: Vec<f64>,
}

impl RawTrace {
    pub fn new(current: Vec<f64>, voltage: Vec<f64>) -> Self {
        assert_eq!(current.len(), voltage.len());
        RawTrace { current, voltage }
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item=(f64, f64)> + 'a {
        self.voltage.iter().cloned().zip(self.current.iter().cloned())
    }
}

impl Default for RawTrace {
    fn default() -> Self {
        RawTrace::new(Vec::new(), Vec::new())
    }
}

pub trait Backend {
    fn trace(&self) -> Result<RawTrace>;
}

pub use self::ad2::AD2;
pub use self::csv::Csv;
