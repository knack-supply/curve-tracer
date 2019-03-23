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
    fn new(current_offset: f64, is: f64, n_vt: f64) -> Self {
        ShockleyModel {
            p: RowVectorN::<f64, U3>::new(current_offset, is, n_vt),
        }
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
