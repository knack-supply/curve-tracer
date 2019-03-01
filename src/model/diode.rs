use std::fmt::Display;

use itertools::Itertools;
use nalgebra::allocator::Allocator;
use nalgebra::allocator::Reallocator;
use nalgebra::*;
use num_traits::float::Float;

use crate::backend::RawTrace;
use crate::model::pwc::PieceWiseConstantFunction;
use crate::model::IVModel;
use crate::util::Engineering;

pub struct NullModel {}

impl IVModel for NullModel {
    fn min_v(&self) -> f64 {
        0.0
    }

    fn max_v(&self) -> f64 {
        0.0
    }

    fn evaluate(&self, _v: f64) -> f64 {
        0.0
    }
}

impl Display for NullModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("")?;
        Ok(())
    }
}

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
) -> LogLinearShockleyModel {
    let xs = MatrixMN::<f64, U2, Dynamic>::from_rows(&[
        RowDVector::from_iterator(trace.len(), trace.iter().cloned().map(|(v, _)| v)),
        RowDVector::from_element(trace.len(), 1.0),
    ]);
    let mut ys = DVector::from_iterator(trace.len(), trace.iter().cloned().map(|(_, i)| i));
    ys.apply(|i| (i - current_offset.current_offset).max(0.00001).ln());

    let betas = linear_regression(xs, ys);
    let n_vt = 1.0 / betas[(0, 0)];
    let is = betas[(0, 1)].exp();

    LogLinearShockleyModel {
        current_offset,
        is,
        n_vt,
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ShockleyModel {
    pub current_offset: CurrentOffsetModel,
    pub is: f64,
    pub n_vt: f64,
}

impl IVModel for ShockleyModel {
    fn min_v(&self) -> f64 {
        0.0
    }

    fn max_v(&self) -> f64 {
        self.n_vt * ((self.is + 1.0 - self.current_offset.current_offset) / self.is).ln()
    }

    fn evaluate(&self, v: f64) -> f64 {
        self.current_offset.evaluate(v) + self.is * ((v / self.n_vt).exp() + 1.0)
    }
}

impl Display for ShockleyModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.current_offset)?;
        writeln!(f, "I<sub>S</sub>\t{:.3}A", Engineering(self.is))?;
        writeln!(f, "n⋅V<sub>T</sub>\t{:.3}V", Engineering(self.n_vt))?;
        Ok(())
    }
}

fn shockley(trace: &[(f64, f64)], simplified_shockley: LogLinearShockleyModel) -> ShockleyModel {
    let max_absolute_change = 0.000_000_000_000_000_1;
    let max_iterations = 50;
    let max_total_error = 0.000_01;
    let min_error_improvement = 0.001;
    let min_shift_cut = 0.000_1;
    let min_iterations = 5;

    let mut shift_cut = 1.0;
    let mut old_total_error = f64::max_value();

    let mut model = [
        simplified_shockley.current_offset.current_offset,
        simplified_shockley.is,
        simplified_shockley.n_vt,
    ];

    let mut old_model = model.clone();

    debug!("model: {:?}", model);

    let f = |m: &[f64]| (m[0] + m[1] * ((m[3] / m[2]).exp() - 1.0));
    let f0 = |_: &[f64]| 1.0;
    let f1 = |m: &[f64]| ((m[3] / m[2]).exp() - 1.0);
    let f2 = |m: &[f64]| -m[1] * m[3] * (m[3] / m[2]).exp() / (m[2] * m[2]);

    for iteration in 0..max_iterations {
        debug!("");
        debug!("shift cut: {}", shift_cut);
        let jacobian_rows: Vec<RowVector3<f64>> = trace
            .iter()
            .cloned()
            .map(|(v, _)| {
                let params = [model[0], model[1], model[2], v];
                RowVector3::new(f0(&params), f1(&params), f2(&params))
            })
            .collect_vec();
        let jacobian: MatrixMN<f64, Dynamic, U3> = MatrixMN::from_rows(jacobian_rows.as_slice());

        debug!("J: {:?}", jacobian_rows.iter().take(2).collect_vec());

        let svd = SVD::new(jacobian, true, true);

        let residuals = DVector::<f64>::from_iterator(
            trace.len(),
            trace.iter().cloned().map(|(v, i)| {
                let params = [model[0], model[1], model[2], v];
                let i_model = f(&params);
                i - i_model
            }),
        );

        let total_error = residuals.map(|r| r.powi(2)).sum().sqrt();
        debug!("total error: {}", total_error);

        let v = svd.v_t.unwrap().transpose();
        let sigma = Matrix::from_diagonal(&svd.singular_values.map(|e| 1.0 / e));
        let u_t = svd.u.unwrap().transpose();
        let correction = v * sigma * (u_t * residuals) * shift_cut;

        if total_error > old_total_error {
            debug!("rolling back");

            model.clone_from_slice(&old_model);

            debug!("model: {:?}", model);

            shift_cut = shift_cut / 10.0;
            continue;
        }
        old_model.clone_from_slice(&model);

        let error_improvement = ((total_error - old_total_error) / old_total_error).abs();
        old_total_error = total_error;

        let absolute_change = correction.map(f64::abs).sum();

        model[0] += correction[0];
        model[1] = (model[1] + correction[1]).max(0.000_000_000_000_000_1);
        model[2] = (model[2] + correction[2]).max(0.000_1);

        shift_cut = (shift_cut * 2.0).min(1.0);

        debug!("absolute change: {}", absolute_change);
        debug!("error improvement: {}", error_improvement);
        debug!("model: {:?}", model);

        if iteration < min_iterations {
            continue;
        }

        if total_error < max_total_error
            || absolute_change <= max_absolute_change
            || shift_cut < min_shift_cut
            || error_improvement < min_error_improvement
        {
            break;
        }
    }

    ShockleyModel {
        current_offset: CurrentOffsetModel {
            current_offset: model[0],
        },
        is: model[1],
        n_vt: model[2],
    }
}

fn linear_regression<N: Real, D: DimName>(
    x: MatrixMN<N, D, Dynamic>,
    y: MatrixMN<N, Dynamic, U1>,
) -> RowVectorN<N, D>
where
    DefaultAllocator: Allocator<N, Dynamic, Dynamic>
        + Allocator<N, Dynamic>
        + Allocator<N, U1, Dynamic>
        + Allocator<N, Dynamic, D>
        + Allocator<N, D, Dynamic>
        + Allocator<N, D, U1>
        + Reallocator<N, Dynamic, Dynamic, U1, D>,
{
    let x = {
        let s = x.shape();
        x.resize(s.0, s.1, N::zero())
    };
    let y = {
        let s = y.shape();
        y.resize(s.0, s.1, N::zero())
    };

    let x_svd = SVD::new(x.transpose(), true, true);
    let u = x_svd.u.unwrap();
    let s = x_svd.singular_values;
    let v_t = x_svd.v_t.unwrap();

    let alpha = u.transpose() * y;
    let s_shape = s.shape();
    let sinv = alpha.zip_map(&s.resize(s_shape.0, s_shape.1, N::zero()), |a, s| a / s);

    (v_t.transpose() * sinv)
        .transpose()
        .fixed_resize::<U1, D>(N::zero())
}

pub fn diode_model(trace: &RawTrace) -> ShockleyModel {
    let trace =
        PieceWiseConstantFunction::from_points(0.0, 5.0, 5000, 1, &trace.iter().collect_vec())
            .iter()
            .collect_vec();
    shockley(
        &trace,
        log_linear_simplified_shockley(&trace, current_offset(&trace)),
    )
}
