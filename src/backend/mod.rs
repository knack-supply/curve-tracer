mod ad2;
mod csv;

pub use self::ad2::AD2;
pub use self::csv::Csv;

use failure::Error;
use std::path::Path;

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

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = (f64, f64)> + 'a {
        self.voltage
            .iter()
            .cloned()
            .zip(self.current.iter().cloned())
    }

    pub fn save_as_csv<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let mut out = ::csv::WriterBuilder::new()
            .delimiter(b'\t')
            .from_path(path)?;

        out.write_record(&["v", "i"])?;
        for (v, i) in self.iter() {
            out.write_record(&[v.to_string(), i.to_string()])?;
        }
        Ok(())
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
