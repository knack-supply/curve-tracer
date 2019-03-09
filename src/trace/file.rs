use std::ffi::OsStr;
use std::fs::File;
use std::path::Path;

use crate::backend::RawTrace;
use crate::AreaOfInterest;
use crate::NullTrace;
use crate::Result;
use crate::ThreeTerminalTrace;
use crate::TwoTerminalTrace;
use noisy_float::prelude::r64;
use serde::de::DeserializeOwned;
use std::collections::BTreeMap;

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

pub trait ExportableTrace {
    fn save_as_csv(&self, path: &Path) -> Result<()>;
}

trait CsvWriter {
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

impl ExportableTrace for NullTrace {
    fn save_as_csv(&self, _: &Path) -> Result<()> {
        Err(failure::err_msg("No trace to save"))
    }
}

fn is_gz(path: &Path) -> bool {
    path.extension().and_then(OsStr::to_str) == Some("gz")
}

fn csv_writer_from_path(path: &Path) -> Result<Box<dyn CsvWriter>> {
    let mut out_builder = csv::WriterBuilder::new();
    out_builder.delimiter(b'\t');
    Ok(if is_gz(&path) {
        Box::new(out_builder.from_writer(libflate::gzip::Encoder::new(File::create(path)?)?))
    } else {
        Box::new(out_builder.from_path(path)?)
    })
}

fn csv_reader_from_path<D: DeserializeOwned + 'static>(
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

impl ExportableTrace for TwoTerminalTrace {
    fn save_as_csv(&self, path: &Path) -> Result<()> {
        let mut out = csv_writer_from_path(path)?;

        let header = ["v", "i"];
        out.write_record(&header)?;
        for (v, i) in self.trace.iter() {
            let v_str = v.to_string();
            let i_str = i.to_string();
            let rec = [v_str.as_str(), i_str.as_str()];
            out.write_record(&rec)?;
        }
        out.close()?;
        Ok(())
    }
}

impl ExportableTrace for ThreeTerminalTrace {
    fn save_as_csv(&self, path: &Path) -> Result<()> {
        let mut out = csv_writer_from_path(path)?;

        let header = ["v", "i", "bias"];
        out.write_record(&header)?;
        for (bias, trace) in self.traces.iter() {
            let bias_str = bias.to_string();
            for (v, i) in trace.trace.iter() {
                let v_str = v.to_string();
                let i_str = i.to_string();
                let rec = [v_str.as_str(), i_str.as_str(), bias_str.as_str()];
                out.write_record(&rec)?;
            }
        }
        out.close()?;
        Ok(())
    }
}

pub trait ImportableTrace: Sized {
    fn from_csv<P: AsRef<Path>>(path: P, aoi: AreaOfInterest) -> Result<Self>;
}

impl ImportableTrace for NullTrace {
    fn from_csv<P: AsRef<Path>>(_: P, _: AreaOfInterest) -> Result<Self> {
        unreachable!()
    }
}

impl ImportableTrace for TwoTerminalTrace {
    fn from_csv<P: AsRef<Path>>(path: P, aoi: AreaOfInterest) -> Result<Self> {
        let mut vs = Vec::new();
        let mut is = Vec::new();

        for result in csv_reader_from_path(path.as_ref())? {
            let record: Record2 = result?;
            vs.push(record.v);
            is.push(record.i);
        }

        Ok(TwoTerminalTrace::from_raw_trace(RawTrace::new(is, vs), aoi))
    }
}

impl ImportableTrace for ThreeTerminalTrace {
    fn from_csv<P: AsRef<Path>>(path: P, aoi: AreaOfInterest) -> Result<Self> {
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

        Ok(ThreeTerminalTrace {
            traces: traces
                .into_iter()
                .map(|(bias, (vs, is))| {
                    (
                        bias,
                        TwoTerminalTrace::from_raw_trace(RawTrace::new(is, vs), aoi),
                    )
                })
                .collect(),
        })
    }
}
