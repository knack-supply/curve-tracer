use crate::dut::two::TwoTerminalDevice;
use crate::dut::CurrentBiasedDevice;
use crate::dut::{
    CurrentBiasedDeviceType, Device, SomeDevice, TwoTerminalDeviceType, VoltageBiasedDevice,
    VoltageBiasedDeviceType,
};
use std::fmt::{Display, Formatter};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SomeDeviceType {
    TwoTerminal(TwoTerminalDeviceType),
    VoltageBiased(VoltageBiasedDeviceType),
    CurrentBiased(CurrentBiasedDeviceType),
}

pub trait DeviceType {
    type Device: Device;

    fn to_device(&self) -> Self::Device;
}

impl SomeDeviceType {
    pub fn connection_hint(&self) -> &'static str {
        match self {
            SomeDeviceType::TwoTerminal(TwoTerminalDeviceType::Diode) => "Top row: AKKKKKK",
            SomeDeviceType::CurrentBiased(CurrentBiasedDeviceType::NPN) => "Bottom row: CBECBEC",
            SomeDeviceType::CurrentBiased(CurrentBiasedDeviceType::PNP) => {
                "Bottom row: EBCEBCE (reversed E/C)"
            }
            SomeDeviceType::VoltageBiased(VoltageBiasedDeviceType::NFET) => "Bottom row: DGSDGSD",
            SomeDeviceType::VoltageBiased(VoltageBiasedDeviceType::PFET) => {
                "Bottom row: SGDSGDS (reversed S/D)"
            }
        }
    }
}

impl DeviceType for SomeDeviceType {
    type Device = SomeDevice;

    fn to_device(&self) -> Self::Device {
        match self {
            SomeDeviceType::TwoTerminal(TwoTerminalDeviceType::Diode) => {
                SomeDevice::TwoTerminal(TwoTerminalDevice::Diode)
            }
            SomeDeviceType::CurrentBiased(CurrentBiasedDeviceType::NPN) => {
                SomeDevice::CurrentBiased(CurrentBiasedDevice::from_type(
                    CurrentBiasedDeviceType::NPN,
                ))
            }
            SomeDeviceType::CurrentBiased(CurrentBiasedDeviceType::PNP) => {
                SomeDevice::CurrentBiased(CurrentBiasedDevice::from_type(
                    CurrentBiasedDeviceType::PNP,
                ))
            }
            SomeDeviceType::VoltageBiased(VoltageBiasedDeviceType::NFET) => {
                SomeDevice::VoltageBiased(VoltageBiasedDevice::from_type(
                    VoltageBiasedDeviceType::NFET,
                ))
            }
            SomeDeviceType::VoltageBiased(VoltageBiasedDeviceType::PFET) => {
                SomeDevice::VoltageBiased(VoltageBiasedDevice::from_type(
                    VoltageBiasedDeviceType::PFET,
                ))
            }
        }
    }
}

impl Display for SomeDeviceType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SomeDeviceType::TwoTerminal(device_type) => device_type.fmt(f),
            SomeDeviceType::VoltageBiased(device_type) => device_type.fmt(f),
            SomeDeviceType::CurrentBiased(device_type) => device_type.fmt(f),
        }
    }
}
