use digilent_waveforms::*;
use itertools::Itertools;
use noisy_float::prelude::*;
use time::Duration;

use crate::backend::Backend;
use crate::backend::BiasedTrace;
use crate::backend::RawTrace;
use crate::dut::BiasDrive;
use crate::util::Try;

pub struct AD2 {
    device: Device,
    diode_current_limit_ma: f64,
    current_shunt_ohms: f64,
    bias_limiter_ohms: f64,
    sampling_time: f64,
    capture_offset_stabilization_time: f64,
    bias_level_sampling_time: f64,
    sampling_frequency: f64,
    cycles_to_sample: u32,
    cycles_to_skip: u32,
    max_v: f64,
    min_v: f64,
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
            sampling_time: 0.5,
            capture_offset_stabilization_time: 0.5,
            bias_level_sampling_time: 0.3,
            sampling_frequency: 500_000.0,
            cycles_to_sample: 5,
            cycles_to_skip: 1,
            max_v: 2.2,
            min_v: -2.2,
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
    fn trace_2(&self) -> crate::Result<RawTrace> {
        self.device.reset()?;
        self.device.set_auto_configure(true)?;
        self.device.set_enabled(true)?;

        self.enable_power()?;

        let hz = f64::from(self.cycles_to_sample + self.cycles_to_skip) / self.sampling_time;
        let current_limit = self.diode_current_limit_ma / 1000.0;
        let max_v = (current_limit * self.current_shunt_ohms + 0.5)
            .min(self.max_v)
            .max(self.min_v);
        let time_slack = 0.05;
        let total_time = self.sampling_time + time_slack;

        let out_vf = self.device.analog_out(0);
        out_vf.set_idle_mode(AnalogOutIdleMode::Initial)?;
        out_vf.set_trigger_source(TriggerSource::AnalogIn)?;
        let out_vf_carrier = out_vf.node(0);

        debug!(
            "Setting up a triangle waveform between A and K: [{}, {}] at {} Hz",
            0.0, max_v, hz
        );
        out_vf_carrier.set_function(AnalogOutFunction::Triangle {
            frequency: hz,
            amplitude: max_v / 2.0,
            offset: max_v / 2.0,
            symmetry: 50.0,
            phase_deg: 270.0,
        })?;
        out_vf_carrier.set_enabled(true)?;
        out_vf.set_duration(Duration::nanoseconds((total_time * 1.0e9) as i64))?;
        out_vf.set_repeat_count(0)?;

        out_vf.start()?;

        let input = self.device.analog_input();
        input.set_frequency(self.sampling_frequency)?;
        input.set_record_mode(self.sampling_time)?;

        let in_v_shunt = input.channel(0);
        in_v_shunt.set_offset(2.0)?;
        in_v_shunt.set_range(10.0)?;

        let in_v = input.channel(1);
        in_v.set_offset(-0.5)?;
        in_v.set_range(1.0)?;

        std::thread::sleep(
            Duration::nanoseconds((self.capture_offset_stabilization_time * 1.0e9) as i64)
                .to_std()?,
        );

        input.start()?;

        let mut vs = Vec::new();
        let mut vss = Vec::new();
        {
            debug_time!("Tracing");
            Self::record_raw(&input, &in_v, &mut vs, &in_v_shunt, &mut vss)?;
        }
        out_vf.stop()?;
        self.disable_power()?;

        let start_ix = (vs.len() as f64 * self.cycles_to_skip as f64
            / (self.cycles_to_skip + self.cycles_to_sample) as f64) as usize;
        let is = vss
            .into_iter()
            .skip(start_ix)
            .map(|v_s| v_s / self.current_shunt_ohms)
            .collect_vec();
        Ok(RawTrace::new(is, vs.split_off(start_ix)))
    }

    fn trace_3(
        &self,
        polarity: R64,
        bias_drive: BiasDrive,
        bias_levels: Vec<R64>,
    ) -> crate::Result<Vec<BiasedTrace>> {
        let bias_factor = match bias_drive {
            BiasDrive::Voltage => 1.0,
            BiasDrive::Current => self.bias_limiter_ohms,
        };
        let bias_levels = bias_levels
            .into_iter()
            .map(|l| (l, l * bias_factor))
            .collect_vec();

        self.device.reset()?;
        self.device.set_auto_configure(true)?;
        self.device.set_enabled(true)?;

        self.enable_power()?;

        let hz =
            f64::from(self.cycles_to_sample + self.cycles_to_skip) / self.bias_level_sampling_time;
        let current_limit = self.diode_current_limit_ma / 1000.0;
        let max_v = (current_limit * self.current_shunt_ohms + 0.5 * polarity.raw())
            .min(self.max_v)
            .max(self.min_v);
        let time_slack = 0.05;
        let total_time = self.bias_level_sampling_time + time_slack;

        let out_vf = self.device.analog_out(0);
        out_vf.set_idle_mode(AnalogOutIdleMode::Initial)?;
        out_vf.set_trigger_source(TriggerSource::AnalogIn)?;
        let out_vf_carrier = out_vf.node(0);

        debug!(
            "Setting up a triangle waveform between C/D and E/S: [{}, {}] at {} Hz",
            0.0, max_v, hz
        );
        out_vf_carrier.set_function(AnalogOutFunction::Triangle {
            frequency: hz,
            amplitude: max_v / 2.0,
            offset: max_v / 2.0,
            symmetry: 50.0,
            phase_deg: 270.0,
        })?;
        out_vf_carrier.set_enabled(true)?;
        out_vf.set_duration(Duration::nanoseconds((total_time * 1.0e9) as i64))?;
        out_vf.set_repeat_count(0)?;

        let out_bias = self.device.analog_out(1);
        out_bias.set_idle_mode(AnalogOutIdleMode::Initial)?;
        out_bias.set_trigger_source(TriggerSource::AnalogIn)?;
        let out_bias_carrier = out_bias.node(0);

        out_bias_carrier.set_function(AnalogOutFunction::Const { offset: 0.0 })?;
        out_bias_carrier.set_enabled(true)?;
        out_bias.set_duration(Duration::nanoseconds((total_time * 1.0e9) as i64))?;
        out_bias.set_repeat_count(0)?;

        let input = self.device.analog_input();
        input.set_frequency(self.sampling_frequency)?;
        input.set_record_mode(self.bias_level_sampling_time)?;

        let in_v_shunt = input.channel(0);
        in_v_shunt.set_offset(2.0)?;
        in_v_shunt.set_range(10.0)?;

        let in_v = input.channel(1);
        in_v.set_offset(-0.5)?;
        in_v.set_range(1.0)?;

        std::thread::sleep(
            Duration::nanoseconds((self.capture_offset_stabilization_time * 1.0e9) as i64)
                .to_std()?,
        );

        let mut traces = vec![];

        for (bias_value, bias_v) in bias_levels {
            debug_time!("Tracing at a single bias level");
            debug!("Setting up bias voltage at B/G: {}", bias_v);

            out_bias_carrier.set_function(AnalogOutFunction::Const {
                offset: bias_v.raw(),
            })?;

            out_vf.start()?;
            std::thread::sleep(Duration::nanoseconds((time_slack * 2.0 * 1.0e9) as i64).to_std()?);

            input.start()?;

            let mut vs = Vec::new();
            let mut vss = Vec::new();
            {
                debug_time!("Recording");
                Self::record_raw(&input, &in_v, &mut vs, &in_v_shunt, &mut vss)?;
            }

            out_vf.stop()?;

            let start_ix = (vs.len() as f64 * self.cycles_to_skip as f64
                / (self.cycles_to_skip + self.cycles_to_sample) as f64)
                as usize;
            let is = vss
                .into_iter()
                .skip(start_ix)
                .map(|v_s| v_s / self.current_shunt_ohms)
                .collect_vec();
            let vs = vs
                .into_iter()
                .skip(start_ix)
                .map(|v| (polarity * v).raw())
                .collect_vec();
            traces.push(BiasedTrace {
                bias: bias_value,
                trace: RawTrace::new(is, vs),
            })
        }

        self.disable_power()?;

        Ok(traces)
    }
}
