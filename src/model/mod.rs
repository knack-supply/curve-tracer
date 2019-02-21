use std::fmt::Display;

pub mod diode;

pub trait IVModel: Display {
    fn evaluate(&self, v: f64) -> f64;
}
