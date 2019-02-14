use nalgebra::*;
use nalgebra::allocator::Allocator;
use nalgebra::allocator::Reallocator;

use crate::backend::RawTrace;
use crate::model::IVModel;
use std::fmt::Display;
use crate::util::Engineering;

pub struct NullModel {}

impl IVModel for NullModel {
    fn evaluate(&self, _v: f64) -> f64 {
        0.0
    }
}

impl Display for NullModel {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("")?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CurrentOffsetModel {
    pub current_offset: f64,
}

impl IVModel for CurrentOffsetModel {
    fn evaluate(&self, _v: f64) -> f64 {
        self.current_offset
    }
}

impl Display for CurrentOffsetModel {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "I<sub>OS</sub>\t{:.3}A", Engineering(self.current_offset))
    }
}

fn current_offset(trace: &RawTrace) -> CurrentOffsetModel {
    let mut i_sum = 0.0;
    let mut samples = 0u32;
    for (&v, &i) in trace.voltage.iter().zip(trace.current.iter()) {
        if 0.0 <= v && v < 0.05 {
            i_sum += i;
            samples += 1;
        }
    }
    CurrentOffsetModel {
        current_offset: if samples > 100 {
            i_sum / (samples as f64)
        } else {
            0.0
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LogLinearShockleyModel {
    pub current_offset: CurrentOffsetModel,
    pub is: f64,
    pub n_vt: f64,
}

impl IVModel for LogLinearShockleyModel {
    fn evaluate(&self, v: f64) -> f64 {
        self.current_offset.evaluate(v) + self.is * (v / self.n_vt).exp()
    }
}

impl Display for LogLinearShockleyModel {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "{}", self.current_offset)?;
        writeln!(f, "I<sub>S</sub>\t{:.3}A", Engineering(self.is))?;
        writeln!(f, "n⋅V<sub>T</sub>\t{:.3}V", Engineering(self.n_vt))?;
        Ok(())
    }
}


fn log_linear_simplified_shockley(trace: &RawTrace, current_offset: CurrentOffsetModel) -> LogLinearShockleyModel {
    let xs = MatrixMN::<f64, U2, Dynamic>::from_rows(&[
        RowDVector::from_row_slice(&trace.voltage),
        RowDVector::from_element(trace.voltage.len(), 1.0)
    ]);
    let mut ys = DVector::from_column_slice(&trace.current);
    ys.apply(|i| (i - current_offset.current_offset).max(0.00001).ln());

    let betas = linear_regression(xs, ys);
    let n_vt = 1.0 / betas[(0, 0)];
    let is = betas[(0, 1)].exp();

    LogLinearShockleyModel { current_offset, is, n_vt }
}


fn linear_regression_naive<N: Real, D: DimName>(x: MatrixMN<N, D, Dynamic>, y: DVector<N>) -> DVector<N>
    where DefaultAllocator: Allocator<N, Dynamic, Dynamic> + Allocator<N, D, Dynamic>
{
    let a = &x.transpose() * &x;
    let i = a.pseudo_inverse(N::from_f64(0.000001).unwrap()).unwrap();
    i * x.transpose() * y
}

fn linear_regression<N: Real, D: DimName>(x: MatrixMN<N, D, Dynamic>, y: MatrixMN<N, Dynamic, U1>) -> RowVectorN<N, D>
    where DefaultAllocator: Allocator<N, Dynamic, Dynamic>
    + Allocator<N, Dynamic>
    + Allocator<N, U1, Dynamic>
    + Allocator<N, Dynamic, D>
    + Allocator<N, D, Dynamic>
    + Allocator<N, D, U1>
    + Reallocator<N, Dynamic, Dynamic, U1, D>
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

    (v_t.transpose() * sinv).transpose().fixed_resize::<U1, D>(N::zero())
}

pub fn diode_model(trace: &RawTrace) -> LogLinearShockleyModel {
    log_linear_simplified_shockley(&trace, current_offset(&trace))
}
