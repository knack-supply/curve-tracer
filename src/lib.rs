#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate measure_time;
#[macro_use]
extern crate serde_derive;

use std::path::Path;
use std::sync::Arc;

use structopt;

use crate::backend::BiasedTrace;
use crate::backend::RawTrace;
use crate::model::IVModel;

pub mod backend;
pub mod gui;
pub mod model;
pub mod options;
pub mod util;

pub type Result<T> = std::result::Result<T, failure::Error>;

#[derive(Clone)]
pub enum Trace {
    Null,
    Unbiased {
        trace: RawTrace,
        model: Option<Arc<dyn IVModel>>,
    },
    Biased {
        traces: Vec<BiasedTrace>,
    },
}

impl Trace {
    pub fn save_as_csv<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let mut out = ::csv::WriterBuilder::new()
            .delimiter(b'\t')
            .from_path(path)?;

        match self {
            Trace::Unbiased { trace, model: _ } => {
                out.write_record(&["v", "i"])?;
                for (v, i) in trace.iter() {
                    out.write_record(&[v.to_string(), i.to_string()])?;
                }
            }
            Trace::Biased { traces } => {
                out.write_record(&["v", "i", "bias"])?;
                for BiasedTrace { bias, trace } in traces {
                    let bias_str = bias.to_string();
                    for (v, i) in trace.iter() {
                        out.write_record(&[&v.to_string(), &i.to_string(), &bias_str])?;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
}
