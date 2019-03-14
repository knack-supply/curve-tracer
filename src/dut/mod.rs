use std::path::Path;

use crate::backend::Backend;
use crate::dut::aoi::AreaOfInterest;
use crate::dut::trace::GuiTrace;
use crate::gui::COLORS_HEX;
use crate::Result;

pub use self::device_type::*;
pub use self::i_biased::*;
pub use self::two::*;
pub use self::v_biased::*;
use crate::gui::widgets::DeviceConfig;
use crate::util::Engineering;

mod aoi;
mod csv;
mod device_type;
mod i_biased;
pub mod trace;
mod two;
mod v_biased;

#[derive(Clone)]
pub enum SomeDevice {
    TwoTerminal(TwoTerminalDevice),
    VoltageBiased(VoltageBiasedDevice),
    CurrentBiased(CurrentBiasedDevice),
}

pub enum BiasDrive {
    Voltage,
    Current,
}

pub trait Device: Sized {
    type Trace: GuiTrace;
    type Config;
    fn area_of_interest(&self) -> AreaOfInterest;
    fn trace(&self, backend: &dyn Backend) -> Result<Self::Trace>;
    fn load_from_csv<P: AsRef<Path>>(&self, path: P) -> Result<Self::Trace>;
    fn config(&self) -> Self::Config;
    fn set_config(&mut self, config: &Self::Config);
}

impl SomeDevice {
    pub fn device_type(&self) -> SomeDeviceType {
        match self {
            SomeDevice::TwoTerminal(device) => {
                SomeDeviceType::TwoTerminal(TwoTerminalDeviceType::from(device))
            }
            SomeDevice::VoltageBiased(device) => {
                SomeDeviceType::VoltageBiased(VoltageBiasedDeviceType::from(device))
            }
            SomeDevice::CurrentBiased(device) => {
                SomeDeviceType::CurrentBiased(CurrentBiasedDeviceType::from(device))
            }
        }
    }

    pub fn legend(&self) -> String {
        match self {
            SomeDevice::TwoTerminal(TwoTerminalDevice::Diode) => String::new(),
            SomeDevice::CurrentBiased(device) => {
                let mut legend = String::from("I<sub>BE</sub>:");
                for (bias, color) in device.bias_levels().iter().zip(COLORS_HEX.iter()) {
                    legend.push_str(&format!(
                        r#" <span fgcolor="white" bgcolor="{}">{}A</span>"#,
                        color,
                        Engineering(bias.raw())
                    ));
                }
                legend
            }
            SomeDevice::VoltageBiased(device) => {
                let mut legend = String::from("V<sub>GS</sub>:");
                for (bias, color) in device.bias_levels().iter().zip(COLORS_HEX.iter()) {
                    legend.push_str(&format!(
                        r#" <span fgcolor="white" bgcolor="{}">{}V</span>"#,
                        color,
                        Engineering(bias.raw())
                    ));
                }
                legend
            }
        }
    }
}

impl From<SomeDeviceType> for SomeDevice {
    fn from(t: SomeDeviceType) -> Self {
        match t {
            SomeDeviceType::TwoTerminal(TwoTerminalDeviceType::Diode) => {
                SomeDevice::TwoTerminal(TwoTerminalDevice::Diode)
            }
            SomeDeviceType::VoltageBiased(device_type) => {
                SomeDevice::VoltageBiased(VoltageBiasedDevice::from_type(device_type))
            }
            SomeDeviceType::CurrentBiased(device_type) => {
                SomeDevice::CurrentBiased(CurrentBiasedDevice::from_type(device_type))
            }
        }
    }
}

impl Device for SomeDevice {
    type Trace = Box<dyn GuiTrace>;
    type Config = DeviceConfig;

    fn area_of_interest(&self) -> AreaOfInterest {
        match self {
            SomeDevice::TwoTerminal(device) => device.area_of_interest(),
            SomeDevice::VoltageBiased(device) => device.area_of_interest(),
            SomeDevice::CurrentBiased(device) => device.area_of_interest(),
        }
    }

    fn trace(&self, backend: &dyn Backend) -> Result<Self::Trace> {
        Ok(match self {
            SomeDevice::TwoTerminal(device) => Box::new(device.trace(backend)?),
            SomeDevice::VoltageBiased(device) => Box::new(device.trace(backend)?),
            SomeDevice::CurrentBiased(device) => Box::new(device.trace(backend)?),
        })
    }

    fn load_from_csv<P: AsRef<Path>>(&self, path: P) -> Result<Self::Trace> {
        Ok(match self {
            SomeDevice::TwoTerminal(device) => Box::new(device.load_from_csv(path)?),
            SomeDevice::VoltageBiased(device) => Box::new(device.load_from_csv(path)?),
            SomeDevice::CurrentBiased(device) => Box::new(device.load_from_csv(path)?),
        })
    }

    fn config(&self) -> Self::Config {
        match self {
            SomeDevice::TwoTerminal(_) => DeviceConfig::None,
            SomeDevice::VoltageBiased(device) => DeviceConfig::FET(device.config()),
            SomeDevice::CurrentBiased(device) => DeviceConfig::BJT(device.config()),
        }
    }

    fn set_config(&mut self, config: &Self::Config) {
        match self {
            SomeDevice::TwoTerminal(device) => {
                for c in config.downcast::<TwoTerminalDeviceConfig>() {
                    device.set_config(&c)
                }
            }
            SomeDevice::VoltageBiased(device) => {
                for c in config.downcast::<VoltageBiasedDeviceConfig>() {
                    device.set_config(&c)
                }
            }
            SomeDevice::CurrentBiased(device) => {
                for c in config.downcast::<CurrentBiasedDeviceConfig>() {
                    device.set_config(&c)
                }
            }
        }
    }
}
