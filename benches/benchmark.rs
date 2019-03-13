#[macro_use]
extern crate criterion;

use std::time::Duration;

use criterion::black_box;
use criterion::BatchSize;
use criterion::Criterion;

use ks_curve_tracer::dut::trace::TraceWithModel;
use ks_curve_tracer::dut::DeviceType;
use ks_curve_tracer::dut::TwoTerminalDeviceType;

fn criterion_config() -> Criterion {
    Criterion::default()
        .measurement_time(Duration::from_secs(1))
        .sample_size(10)
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function_over_inputs(
        "Shockley model",
        move |b, trace_name| {
            let diode_device_type = TwoTerminalDeviceType::Diode;
            let trace = diode_device_type
                .load_from_csv(format!("res/{}.csv", trace_name))
                .expect("Can't read the test trace");
            b.iter_batched_ref(
                || trace.clone(),
                |trace| {
                    trace.fill_model();
                    black_box(trace.model_report());
                },
                BatchSize::LargeInput,
            )
        },
        &[
            "1N3064", "1N4148", "1N4728A", "1N5817", "1N5711", "1N914B-1", "1N914B-2", "1N914B-3",
            "1N914B-4", "1N914B-5", "BA479G", "BAT41",
        ],
    );
}

criterion_group!(
  name = benches;
  config = criterion_config();
  targets = criterion_benchmark
);
criterion_main!(benches);
