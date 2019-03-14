#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate measure_time;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate relm;

#[macro_use]
extern crate relm_derive;

use structopt;

pub mod backend;
pub mod dut;
pub mod gui;
pub mod model;
pub mod options;
pub mod util;

pub type Result<T> = std::result::Result<T, failure::Error>;
