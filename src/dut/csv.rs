use crate::backend::RawTrace;
use crate::dut::aoi::AreaOfInterest;
use crate::dut::trace::{ThreeTerminalTrace, TwoTerminalTrace};
use crate::Result;
use noisy_float::prelude::r64;
use serde::de::DeserializeOwned;
use std::collections::btree_map::BTreeMap;
use std::ffi::OsStr;
use std::fs::File;
use std::path::Path;

pub trait CsvWriter {
    fn write_record(&mut self, record: &[&str]) -> Result<()>;
    fn close(self: Box<Self>) -> Result<()>;
}

impl<W: std::io::Write> CsvWriter for csv::Writer<libflate::gzip::Encoder<W>> {
    fn write_record(&mut self, record: &[&str]) -> Result<()> {
        csv::Writer::write_record(self, record)?;
        Ok(())
    }

    fn close(self: Box<Self>) -> Result<()> {
        self.into_inner()
            .map_err(|_| failure::err_msg("Error writing the file"))?
            .finish()
            .into_result()?;
        Ok(())
    }
}

impl CsvWriter for csv::Writer<std::fs::File> {
    fn write_record(&mut self, record: &[&str]) -> Result<()> {
        csv::Writer::write_record(self, record)?;
        Ok(())
    }

    fn close(self: Box<Self>) -> Result<()> {
        self.into_inner().map_err(failure::Error::from)?;
        Ok(())
    }
}

fn is_gz(path: &Path) -> bool {
    path.extension().and_then(OsStr::to_str) == Some("gz")
}

pub fn csv_writer_from_path(path: &Path) -> Result<Box<dyn CsvWriter>> {
    let mut out_builder = csv::WriterBuilder::new();
    out_builder.delimiter(b'\t');
    Ok(if is_gz(&path) {
        Box::new(out_builder.from_writer(libflate::gzip::Encoder::new(File::create(path)?)?))
    } else {
        Box::new(out_builder.from_path(path)?)
    })
}

pub fn csv_reader_from_path<D: DeserializeOwned + 'static>(
    path: &Path,
) -> Result<Box<dyn Iterator<Item = csv::Result<D>>>> {
    let mut builder = csv::ReaderBuilder::new();
    builder.has_headers(true);
    builder.delimiter(b'\t');
    Ok(if is_gz(&path) {
        Box::new(
            builder
                .from_reader(libflate::gzip::Decoder::new(File::open(path)?)?)
                .into_deserialize(),
        )
    } else {
        Box::new(builder.from_path(path)?.into_deserialize())
    })
}

#[derive(Deserialize)]
struct Record3 {
    i: f64,
    v: f64,
    bias: f64,
}

pub fn load3_from_csv<P: AsRef<Path>>(
    path: P,
    reverse_order: bool,
    aoi: AreaOfInterest,
) -> Result<ThreeTerminalTrace> {
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

    Ok(ThreeTerminalTrace::new(
        reverse_order,
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
