use std::fs::File;

use crate::backend::Backend;
use crate::backend::RawTrace;
use crate::backend::Result;
use csv::ReaderBuilder;
use std::path::PathBuf;

pub struct Csv {
    filename: PathBuf,
    skip: f64,
}

#[derive(Deserialize)]
struct Record {
    i: f64,
    v: f64,
}

impl Csv {
    pub fn new<P: Into<PathBuf>>(filename: P) -> Self {
        Csv {
            filename: filename.into(),
            skip: 0.0,
        }
    }
}

impl Backend for Csv {
    fn trace(&self) -> Result<RawTrace> {
        let mut rdr = ReaderBuilder::new()
            .delimiter(b'\t')
            .from_reader(File::open(&self.filename)?);

        let mut vs = Vec::new();
        let mut is = Vec::new();

        for result in rdr.deserialize() {
            let record: Record = result?;
            vs.push(record.v);
            is.push(record.i);
        }

        let start_ix = (vs.len() as f64 * self.skip) as usize;
        Ok(RawTrace::new(
            is.split_off(start_ix),
            vs.split_off(start_ix),
        ))
    }
}
