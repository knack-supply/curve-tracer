#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate measure_time;
#[macro_use]
extern crate relm;
#[macro_use]
extern crate relm_derive;
#[macro_use]
extern crate serde_derive;
#[cfg(test)]
#[macro_use]
extern crate approx;

pub mod backend;
pub mod dut;
pub mod gui;
pub mod model;
pub mod options;
pub mod util;

pub type Error = failure::Error;
pub type Result<T> = std::result::Result<T, Error>;
