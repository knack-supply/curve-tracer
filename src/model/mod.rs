use std::fmt::Display;

pub mod curvefit;
pub mod diode;
pub mod pwc;

pub trait IVModel: Display + Send + Sync {
    fn min_v(&self) -> f64;
    fn max_v(&self) -> f64;
    fn evaluate(&self, v: f64) -> f64;
}
