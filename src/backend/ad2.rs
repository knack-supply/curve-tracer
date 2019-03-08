use digilent_waveforms::*;
use itertools::Itertools;
use noisy_float::prelude::*;
use time::Duration;

use crate::backend::Backend;
use crate::backend::BiasedTrace;
use crate::backend::RawTrace;
use crate::util::Try;
use crate::{ThreeTerminalDeviceType, TwoTerminalDeviceType};

pub struct AD2 {
    device: Device,
    diode_current_limit_ma: f64,
    current_shunt_ohms: f64,
    bias_limiter_ohms: f64,
    sampling_time: f64,
    bias_level_sampling_time: f64,
    cycles_to_sample: u32,
}

impl AD2 {
    pub fn new() -> crate::Result<Self> {
        Ok(AD2 {
            device: {
                devices()?
                    .devices
                    .first()
                    .into_result()
                    .map_err(|_| failure::err_msg("No devices found"))?
                    .configs
                    .first()
                    .into_result()
                    .map_err(|_| failure::err_msg("No device configs found"))?
                    .open()?
            },
            diode_current_limit_ma: 40.0,
            current_shunt_ohms: 101.0,
            bias_limiter_ohms: 100_000.0,
            sampling_time: 1.0,
            bias_level_sampling_time: 1.0,
            cycles_to_sample: 3,
        })
    }

    fn enable_power(&self) -> crate::Result<()> {
        let ps = self.device.analog_io();
        let v_pos = ps.channel(0);
        v_pos.node(1).set_value(5.0)?;
        v_pos.node(0).set_value(1.0)?;

        let v_neg = ps.channel(1);
        v_neg.node(1).set_value(-5.0)?;
        v_neg.node(0).set_value(1.0)?;

        ps.set_enabled(true)?;

        Ok(())
    }

    fn disable_power(&self) -> crate::Result<()> {
        let ps = self.device.analog_io();
        let v_pos = ps.channel(0);
        v_pos.node(1).set_value(0.0)?;
        v_pos.node(0).set_value(0.0)?;

        let v_neg = ps.channel(1);
        v_neg.node(1).set_value(0.0)?;
        v_neg.node(0).set_value(0.0)?;

        ps.set_enabled(false)?;

        Ok(())
    }

    fn record_raw(
        input: &AnalogIn<'_>,
        in1: &AnalogInChannel<'_>,
        dst1: &mut Vec<f64>,
        in2: &AnalogInChannel<'_>,
        dst2: &mut Vec<f64>,
    ) -> crate::Result<()> {
        let mut total_lost = 0;
        let mut total_corrupted = 0;

        loop {
            let status = input.get_status()?;
            if status == AnalogAcquisitionStatus::Config
                || status == AnalogAcquisitionStatus::Prefill
                || status == AnalogAcquisitionStatus::Armed
            {
                std::thread::yield_now();
                continue;
            }
            if status == AnalogAcquisitionStatus::Done {
                break;
            }

            let left = input.get_samples_left()?;
            if left < 0 {
                break;
            }
            let (available, lost, corrupted) = input.get_record_status()?;
            total_lost += lost;
            total_corrupted += corrupted;

            if lost > 0 {
                dst1.extend(itertools::repeat_n(std::f64::NAN, lost as usize));
                dst2.extend(itertools::repeat_n(std::f64::NAN, lost as usize));
            }
            if available > 0 {
                in1.fetch_samples(dst1, available)?;
                in2.fetch_samples(dst2, available)?;
            }
        }

        if total_lost > 0 || total_corrupted > 0 {
            warn!(
                "Lost {} sample(-s), got {} corrupted sample(-s)",
                total_lost, total_corrupted
            );
        }
        Ok(())
    }
}

impl Backend for AD2 {
    fn trace_2(&self, device_type: TwoTerminalDeviceType) -> crate::Result<RawTrace> {
        if device_type != TwoTerminalDeviceType::Diode {
            return Err(failure::err_msg(format!(
                "Unsupported device type {}",
                device_type
            )));
        }

        self.device.reset()?;
        self.device.set_auto_configure(true)?;
        self.device.set_enabled(true)?;

        self.enable_power()?;

        let hz = f64::from(self.cycles_to_sample) / self.sampling_time;
        let current_limit = self.diode_current_limit_ma / 1000.0;
        let max_v = current_limit * self.current_shunt_ohms + 0.5;
        let total_time = self.sampling_time + 0.05;

        let out_vf = self.device.analog_out(0);
        out_vf.set_idle_mode(AnalogOutIdleMode::Initial)?;
        out_vf.set_trigger_source(TriggerSource::AnalogIn)?;
        let out_vf_carrier = out_vf.node(0);

        out_vf_carrier.set_function(AnalogOutFunction::Triangle {
            frequency: hz,
            amplitude: max_v / 2.0,
            offset: max_v / 2.0,
            symmetry: 50.0,
            phase_deg: 270.0,
        })?;
        out_vf_carrier.set_enabled(true)?;
        out_vf.set_duration(Duration::nanoseconds((total_time / 1.0e9) as i64))?;
        out_vf.set_repeat_count(0)?;

        out_vf.start()?;

        let input = self.device.analog_input();
        input.set_frequency(250_000.0)?;
        input.set_record_mode(self.sampling_time)?;

        let in_v_shunt = input.channel(0);
        in_v_shunt.set_offset(2.0)?;
        in_v_shunt.set_range(10.0)?;

        let in_v = input.channel(1);
        in_v.set_offset(-0.5)?;
        in_v.set_range(1.0)?;

        input.start()?;

        let mut vs = Vec::new();
        let mut vss = Vec::new();
        Self::record_raw(&input, &in_v, &mut vs, &in_v_shunt, &mut vss)?;

        out_vf.stop()?;
        self.disable_power()?;

        let start_ix = vs.len() / self.cycles_to_sample as usize;
        let is = vss
            .into_iter()
            .skip(start_ix)
            .map(|v_s| v_s / self.current_shunt_ohms)
            .collect_vec();
        Ok(RawTrace::new(is, vs.split_off(start_ix)))
    }

    fn trace_3(&self, device_type: ThreeTerminalDeviceType) -> crate::Result<Vec<BiasedTrace>> {
        let (polarity, bias_levels): (f64, Vec<(f64, f64)>) = match device_type {
            ThreeTerminalDeviceType::NPN => {
                let polarity = 1.0;
                (
                    polarity,
                    [10.0, 20.0, 30.0, 40.0, 50.0]
                        .iter()
                        .map(|ua| {
                            let a = polarity * ua / 1_000_000.0;
                            (a, a * self.bias_limiter_ohms)
                        })
                        .collect(),
                )
            }
            ThreeTerminalDeviceType::PNP => {
                let polarity = -1.0;
                (
                    polarity,
                    [10.0, 20.0, 30.0, 40.0, 50.0]
                        .iter()
                        .map(|ua| {
                            let a = polarity * ua / 1_000_000.0;
                            (a, a * self.bias_limiter_ohms)
                        })
                        .collect(),
                )
            }
            ThreeTerminalDeviceType::NFET => {
                let polarity = 1.0;
                (
                    polarity,
                    [1.0, 2.0, 3.0, 4.0, 5.0]
                        .iter()
                        .map(|v| {
                            let v = polarity * v;
                            (v, v)
                        })
                        .collect(),
                )
            }
            ThreeTerminalDeviceType::PFET => {
                let polarity = -1.0;
                (
                    polarity,
                    [1.0, 2.0, 3.0, 4.0, 5.0]
                        .iter()
                        .map(|v| {
                            let v = polarity * v;
                            (v, v)
                        })
                        .collect(),
                )
            }
        };

        self.device.reset()?;
        self.device.set_auto_configure(true)?;
        self.device.set_enabled(true)?;

        self.enable_power()?;

        let hz = f64::from(self.cycles_to_sample) / self.bias_level_sampling_time;
        let current_limit = self.diode_current_limit_ma / 1000.0;
        let max_v = current_limit * self.current_shunt_ohms + 0.5 * polarity;
        let time_slack = 0.05;
        let total_time = self.bias_level_sampling_time + time_slack;

        let out_vf = self.device.analog_out(0);
        out_vf.set_idle_mode(AnalogOutIdleMode::Initial)?;
        out_vf.set_trigger_source(TriggerSource::AnalogIn)?;
        let out_vf_carrier = out_vf.node(0);

        out_vf_carrier.set_function(AnalogOutFunction::Triangle {
            frequency: hz,
            amplitude: max_v / 2.0,
            offset: max_v / 2.0,
            symmetry: 50.0,
            phase_deg: 270.0,
        })?;
        out_vf_carrier.set_enabled(true)?;
        out_vf.set_duration(Duration::nanoseconds((total_time / 1.0e9) as i64))?;
        out_vf.set_repeat_count(0)?;

        let out_bias = self.device.analog_out(1);
        out_bias.set_idle_mode(AnalogOutIdleMode::Initial)?;
        out_bias.set_trigger_source(TriggerSource::AnalogIn)?;
        let out_bias_carrier = out_bias.node(0);

        out_bias_carrier.set_function(AnalogOutFunction::Const { offset: 0.0 })?;
        out_bias_carrier.set_enabled(true)?;
        out_bias.set_duration(Duration::nanoseconds((total_time / 1.0e9) as i64))?;
        out_bias.set_repeat_count(0)?;

        let input = self.device.analog_input();
        input.set_frequency(250_000.0)?;
        input.set_record_mode(self.bias_level_sampling_time)?;

        let in_v_shunt = input.channel(0);
        in_v_shunt.set_offset(2.0)?;
        in_v_shunt.set_range(10.0)?;

        let in_v = input.channel(1);
        in_v.set_offset(-0.5)?;
        in_v.set_range(1.0)?;

        let mut traces = vec![];

        for (bias_value, bias_v) in bias_levels {
            out_bias_carrier.set_function(AnalogOutFunction::Const { offset: bias_v })?;

            out_vf.start()?;
            std::thread::sleep(Duration::nanoseconds((time_slack * 2.0 / 1.0e9) as i64).to_std()?);

            input.start()?;

            let mut vs = Vec::new();
            let mut vss = Vec::new();
            Self::record_raw(&input, &in_v, &mut vs, &in_v_shunt, &mut vss)?;

            out_vf.stop()?;

            let start_ix = vs.len() / self.cycles_to_sample as usize;
            let is = vss
                .into_iter()
                .skip(start_ix)
                .map(|v_s| v_s / self.current_shunt_ohms)
                .collect_vec();
            let vs = vs
                .into_iter()
                .skip(start_ix)
                .map(|v| polarity * v)
                .collect_vec();
            traces.push(BiasedTrace {
                bias: r64(bias_value),
                trace: RawTrace::new(is, vs),
            })
        }

        self.disable_power()?;

        Ok(traces)
    }
}
