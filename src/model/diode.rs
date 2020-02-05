use std::fmt::Display;

use itertools::Itertools;
use nalgebra::*;
use num_traits::float::Float;

use crate::backend::RawTrace;
use crate::model::curvefit::gauss_newton;
use crate::model::curvefit::linear_regression;
use crate::model::curvefit::DiffFn;
use crate::model::curvefit::GaussNewtonParams;
use crate::model::pwc::PieceWiseConstantFunction;
use crate::model::IVModel;
use crate::util::Engineering;

#[derive(Clone, Copy, Debug)]
pub struct CurrentOffsetModel {
    pub current_offset: f64,
}

impl IVModel for CurrentOffsetModel {
    fn min_v(&self) -> f64 {
        f64::min_value()
    }

    fn max_v(&self) -> f64 {
        f64::max_value()
    }

    fn evaluate(&self, _v: f64) -> f64 {
        self.current_offset
    }
}

impl Display for CurrentOffsetModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "I<sub>OS</sub>\t{:.3}A",
            Engineering(self.current_offset)
        )
    }
}

fn current_offset(trace: &[(f64, f64)]) -> CurrentOffsetModel {
    let mut i_sum = 0.0;
    let mut samples = 0u32;
    for (v, i) in trace.iter().cloned() {
        if 0.0 <= v && v < 0.05 {
            i_sum += i;
            samples += 1;
        }
    }
    CurrentOffsetModel {
        current_offset: if samples > 100 {
            i_sum / (f64::from(samples))
        } else {
            0.0
        },
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LogLinearShockleyModel {
    pub current_offset: CurrentOffsetModel,
    pub is: f64,
    pub n_vt: f64,
}

impl IVModel for LogLinearShockleyModel {
    fn min_v(&self) -> f64 {
        0.0
    }

    fn max_v(&self) -> f64 {
        self.n_vt * ((1.0 - self.current_offset.current_offset) / self.is).ln()
    }

    fn evaluate(&self, v: f64) -> f64 {
        self.current_offset.evaluate(v) + self.is * (v / self.n_vt).exp()
    }
}

impl Display for LogLinearShockleyModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.current_offset)?;
        writeln!(f, "I<sub>S</sub>\t{:.3}A", Engineering(self.is))?;
        writeln!(f, "n⋅V<sub>T</sub>\t{:.3}V", Engineering(self.n_vt))?;
        Ok(())
    }
}

fn log_linear_simplified_shockley(
    trace: &[(f64, f64)],
    current_offset: CurrentOffsetModel,
) -> Option<LogLinearShockleyModel> {
    let xs = MatrixMN::<f64, U2, Dynamic>::from_rows(&[
        RowDVector::from_iterator(trace.len(), trace.iter().cloned().map(|(v, _)| v)),
        RowDVector::from_element(trace.len(), 1.0),
    ]);
    let mut ys = DVector::from_iterator(trace.len(), trace.iter().cloned().map(|(_, i)| i));
    ys.apply(|i| (i - current_offset.current_offset).max(0.00001).ln());

    let betas = linear_regression(xs, ys)?;
    let n_vt = 1.0 / betas[(0, 0)];
    let is = betas[(0, 1)].exp();

    Some(LogLinearShockleyModel {
        current_offset,
        is,
        n_vt,
    })
}

#[derive(Clone, Debug)]
pub struct ShockleyModel {
    p: RowVectorN<f64, U3>,
}

impl ShockleyModel {
    pub fn new(current_offset: f64, is: f64, n_vt: f64) -> Self {
        ShockleyModel {
            p: RowVectorN::<f64, U3>::new(current_offset, is, n_vt),
        }
    }

    pub fn current_offset(&self) -> f64 {
        self.p[0]
    }

    pub fn is(&self) -> f64 {
        self.p[1]
    }

    pub fn n_vt(&self) -> f64 {
        self.p[2]
    }
}

impl IVModel for ShockleyModel {
    fn min_v(&self) -> f64 {
        0.0
    }

    fn max_v(&self) -> f64 {
        self.p[2] * ((self.p[1] + 1.0 - self.p[0]) / self.p[1]).ln()
    }

    fn evaluate(&self, v: f64) -> f64 {
        self.p[0] + self.p[1] * ((v / self.p[2]).exp() - 1.0)
    }
}

impl Display for ShockleyModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "I<sub>OS</sub>\t{}A", Engineering(self.p[0]))?;
        writeln!(f, "I<sub>S</sub>\t{:.3}A", Engineering(self.p[1]))?;
        writeln!(f, "n⋅V<sub>T</sub>\t{:.3}V", Engineering(self.p[2]))?;
        Ok(())
    }
}

impl DiffFn<f64, U3> for ShockleyModel {
    fn params(&self) -> &RowVectorN<f64, U3> {
        &self.p
    }

    fn mut_params(&mut self) -> &mut RowVectorN<f64, U3> {
        &mut self.p
    }

    fn sanitize_params(&mut self) {
        self.p[1] = self.p[1].max(0.000_000_000_000_000_1);
        self.p[2] = self.p[2].max(0.000_000_1);
    }

    #[inline]
    fn value(&self, x: f64) -> f64 {
        self.p[0] + self.p[1] * ((x / self.p[2]).exp() - 1.0)
    }

    #[inline]
    fn values(&self, xs: &DVector<f64>) -> DVector<f64> {
        DVector::from_iterator(xs.nrows(), xs.iter().map(|x| self.value(*x)))
    }

    #[inline]
    fn grad(&self, x: f64) -> RowVectorN<f64, U3> {
        RowVectorN::<f64, U3>::new(
            1.0,
            (x / self.p[2]).exp() - 1.0,
            -self.p[1] * x * (x / self.p[2]).exp() / (self.p[2] * self.p[2]),
        )
    }

    #[inline]
    fn deriv(&self, x: f64) -> f64 {
        self.p[1] * (x / self.p[2]).exp() / self.p[2]
    }

    #[inline]
    fn jacobian(&self, xs: &DVector<f64>) -> MatrixMN<f64, Dynamic, U3> {
        MatrixMN::from_rows(xs.iter().map(|x| self.grad(*x)).collect_vec().as_slice())
    }
}

fn shockley(trace: &[(f64, f64)], simplified_shockley: LogLinearShockleyModel) -> ShockleyModel {
    let xs = DVector::from_iterator(trace.len(), trace.iter().map(|(v, _)| *v));
    let ys = DVector::from_iterator(trace.len(), trace.iter().map(|(_, i)| *i));

    let mut model = ShockleyModel::new(
        simplified_shockley.current_offset.current_offset,
        simplified_shockley.is,
        simplified_shockley.n_vt,
    );

    gauss_newton(&xs, &ys, &mut model, GaussNewtonParams::default());

    model
}

pub fn diode_model(trace: &RawTrace) -> Option<ShockleyModel> {
    let trace =
        PieceWiseConstantFunction::from_points(0.0, 5.0, 5000, 1, &trace.iter().collect_vec())
            .iter()
            .collect_vec();
    Some(shockley(
        &trace,
        log_linear_simplified_shockley(&trace, current_offset(&trace))?,
    ))
}

#[cfg(test)]
mod test {
    use crate::dut::{Device, TwoTerminalDevice};
    use crate::model::diode::diode_model;

    #[test]
    pub fn diode_model_1n914b_1() {
        let device = TwoTerminalDevice::Diode {};

        let trace = device.load_from_csv("res/1N914B-1.csv").unwrap();
        let model = diode_model(&trace.trace).unwrap();

        assert_ulps_eq!(model.current_offset(), -0.00010351606472697958);
        assert_ulps_eq!(model.is(), 0.00000001339246591537158);
        assert_ulps_eq!(model.n_vt(), 0.05362089917627807);
    }

    #[test]
    pub fn diode_model_1n914b_2() {
        let device = TwoTerminalDevice::Diode {};

        let trace = device.load_from_csv("res/1N914B-2.csv").unwrap();
        let model = diode_model(&trace.trace).unwrap();

        assert_ulps_eq!(model.current_offset(), -0.00010891667963445857);
        assert_ulps_eq!(model.is(), 0.00000001575645583320489);
        assert_ulps_eq!(model.n_vt(), 0.05453485353716681);
    }

    #[test]
    pub fn diode_model_1n914b_3() {
        let device = TwoTerminalDevice::Diode {};

        let trace = device.load_from_csv("res/1N914B-3.csv").unwrap();
        let model = diode_model(&trace.trace).unwrap();

        assert_ulps_eq!(model.current_offset(), -0.00010750577383021644);
        assert_ulps_eq!(model.is(), 0.00000001737436843869876);
        assert_ulps_eq!(model.n_vt(), 0.0548066592909008);
    }

    #[test]
    pub fn diode_model_1n914b_4() {
        let device = TwoTerminalDevice::Diode {};

        let trace = device.load_from_csv("res/1N914B-4.csv").unwrap();
        let model = diode_model(&trace.trace).unwrap();

        assert_ulps_eq!(model.current_offset(), -0.00011048096014883068);
        assert_ulps_eq!(model.is(), 0.000000013579226111879658);
        assert_ulps_eq!(model.n_vt(), 0.053818676891366456);
    }

    #[test]
    pub fn diode_model_1n914b_5() {
        let device = TwoTerminalDevice::Diode {};

        let trace = device.load_from_csv("res/1N914B-5.csv").unwrap();
        let model = diode_model(&trace.trace).unwrap();

        assert_ulps_eq!(model.current_offset(), -0.00010362189187519665);
        assert_ulps_eq!(model.is(), 0.000000013350754252951973);
        assert_ulps_eq!(model.n_vt(), 0.05372213169983885);
    }

    #[test]
    pub fn diode_model_1n3064() {
        let device = TwoTerminalDevice::Diode {};

        let trace = device.load_from_csv("res/1N3064.csv").unwrap();
        let model = diode_model(&trace.trace).unwrap();

        assert_ulps_eq!(model.current_offset(), -0.00009030952069581359);
        assert_ulps_eq!(model.is(), 0.000000020598921141859168);
        assert_ulps_eq!(model.n_vt(), 0.056010773687125884);
    }

    #[test]
    pub fn diode_model_1n4148() {
        let device = TwoTerminalDevice::Diode {};

        let trace = device.load_from_csv("res/1N4148.csv").unwrap();
        let model = diode_model(&trace.trace).unwrap();

        assert_ulps_eq!(model.current_offset(), -0.00011126775461178329);
        assert_ulps_eq!(model.is(), 0.00000002550253216264574);
        assert_ulps_eq!(model.n_vt(), 0.05694685448419158);
    }

    #[test]
    pub fn diode_model_1n4728a_1() {
        let device = TwoTerminalDevice::Diode {};

        let trace = device.load_from_csv("res/1N4728A-1.csv").unwrap();
        let model = diode_model(&trace.trace).unwrap();

        assert_ulps_eq!(model.current_offset(), -0.00006731822330829186);
        assert_ulps_eq!(model.is(), 0.00000000000010418575215848056);
        assert_ulps_eq!(model.n_vt(), 0.02972677299183207);
    }

    #[test]
    pub fn diode_model_1n4728a_2() {
        let device = TwoTerminalDevice::Diode {};

        let trace = device.load_from_csv("res/1N4728A-2.csv").unwrap();
        let model = diode_model(&trace.trace).unwrap();

        assert_ulps_eq!(model.current_offset(), -0.00010072469003392257);
        assert_ulps_eq!(model.is(), 0.00000000000008253828486712959);
        assert_ulps_eq!(model.n_vt(), 0.029615439704890802);
    }

    #[test]
    pub fn diode_model_1n5711() {
        let device = TwoTerminalDevice::Diode {};

        let trace = device.load_from_csv("res/1N5711.csv").unwrap();
        let model = diode_model(&trace.trace).unwrap();

        assert_ulps_eq!(model.current_offset(), -0.0009343569366573986);
        assert_ulps_eq!(model.is(), 0.0012411529539231917);
        assert_ulps_eq!(model.n_vt(), 0.31938696433362007);
    }

    #[test]
    pub fn diode_model_1n5817() {
        let device = TwoTerminalDevice::Diode {};

        let trace = device.load_from_csv("res/1N5817.csv").unwrap();
        let model = diode_model(&trace.trace).unwrap();

        assert_ulps_eq!(model.current_offset(), -0.00008409489807032524);
        assert_ulps_eq!(model.is(), 0.0000014443844196456792);
        assert_ulps_eq!(model.n_vt(), 0.027616463683553666);
    }

    #[test]
    pub fn diode_model_ba479g() {
        let device = TwoTerminalDevice::Diode {};

        let trace = device.load_from_csv("res/BA479G.csv").unwrap();
        let model = diode_model(&trace.trace).unwrap();

        assert_ulps_eq!(model.current_offset(), -0.00010205114135516167);
        assert_ulps_eq!(model.is(), 0.00000001488752641197316);
        assert_ulps_eq!(model.n_vt(), 0.062077401968406325);
    }

    #[test]
    pub fn diode_model_bat41() {
        let device = TwoTerminalDevice::Diode {};

        let trace = device.load_from_csv("res/BAT41.csv").unwrap();
        let model = diode_model(&trace.trace).unwrap();

        assert_ulps_eq!(model.current_offset(), -0.00021971907183807187);
        assert_ulps_eq!(model.is(), 0.00012064581788991523);
        assert_ulps_eq!(model.n_vt(), 0.1560197078159157);
    }
}
