use digilent_waveforms::*;
use crate::backend::Backend;
use crate::backend::RawTrace;
use itertools::Itertools;
use time::Duration;
use crate::util::Try;

pub struct AD2 {
    skip: f64,
}

impl AD2 {
    pub fn new() -> Self {
        AD2 { skip: 0.5 }
    }
}

impl Backend for AD2 {
    fn trace(&self) -> crate::backend::Result<RawTrace> {
        let device = devices()?
            .devices.first().into_result().map_err(|_| failure::err_msg("No devices found"))?
            .configs.first().into_result().map_err(|_| failure::err_msg("No device configs found"))?
            .open()?;

        device.reset()?;
        device.set_auto_configure(true)?;
        device.set_enabled(true)?;

        let diode_current_limit_ma = 40.0;
        let resistor = 101.0;

        let total_time = 3.0; // decrease to 2-3 if temperature of the device is well controlled
        let sampling_time = 0.5;
        let cycles_to_sample = 2;

        let hz = cycles_to_sample as f64 / sampling_time;

        let current_limit = diode_current_limit_ma / 1000.0;
        let max_v = current_limit * resistor + 0.5;

        let out_vf = device.analog_out(0);
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

        let ps = device.analog_io();
        let v_pos = ps.channel(0);
        v_pos.node(1).set_value(5.0)?;
        v_pos.node(0).set_value(1.0)?;

        let v_neg = ps.channel(1);
        v_neg.node(1).set_value(-5.0)?;
        v_neg.node(0).set_value(1.0)?;

        ps.set_enabled(true)?;

        out_vf.start()?;

        let input = device.analog_input();
        input.set_frequency(200_000.0)?;
//        input.set_buffer_size();
        input.set_record_mode(sampling_time)?;

        let in_v_shunt = input.channel(0);
        in_v_shunt.set_offset(2.0)?;
        in_v_shunt.set_range(10.0)?;

        let in_v = input.channel(1);
        in_v.set_offset(-0.5)?;
        in_v.set_range(1.0)?;

        input.start()?;

        let mut vs = Vec::new();
        let mut vss = Vec::new();
        loop {
            let status = input.get_status().unwrap();
            eprintln!("Status: {:?}", status);
            if status == AnalogAcquisitionStatus::Config
                || status == AnalogAcquisitionStatus::Prefill
                || status == AnalogAcquisitionStatus::Armed {
                std::thread::yield_now();
                continue;
            }
            if status == AnalogAcquisitionStatus::Done {
                break;
            }

            let left = input.get_samples_left().unwrap();
            if left < 0 {
                break;
            }
            let (available, lost, _corrupted) = input.get_record_status().unwrap();

            if lost > 0 {
                vs.extend(itertools::repeat_n(std::f64::NAN, lost as usize));
                vss.extend(itertools::repeat_n(std::f64::NAN, lost as usize));
            }
            if available > 0 {
                in_v.fetch_samples(&mut vs, available)?;
                in_v_shunt.fetch_samples(&mut vss, available)?;
            }
        }

        let start_ix = (vs.len() as f64 * self.skip) as usize;
        let is = vss.into_iter().skip(start_ix).map(|v_s| v_s / 101.0).collect_vec();
        Ok(RawTrace::new(is, vs.split_off(start_ix)))
    }
}
