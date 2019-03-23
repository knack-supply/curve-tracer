use std::fmt::Debug;

use nalgebra::allocator::Allocator;
use nalgebra::allocator::Reallocator;
use nalgebra::*;

pub struct GaussNewtonParams<N: Real> {
    pub min_iterations: usize,
    pub max_iterations: usize,
    pub max_absolute_change: N,
    pub max_total_error: N,
    pub max_error_improvement: N,
    pub min_shift_cut: N,
    pub shift_cut_refining_step: N,
    pub shift_cut_speed_up: N,
}

impl<N: Real> Default for GaussNewtonParams<N> {
    fn default() -> Self {
        Self {
            min_iterations: 5,
            max_iterations: 500,
            max_absolute_change: N::from_f64(0.000_000_000_000_000_1).unwrap(),
            max_total_error: N::from_f64(0.000_000_000_000_1).unwrap(),
            max_error_improvement: N::from_f64(0.000_001).unwrap(),
            min_shift_cut: N::from_f64(0.000_000_000_001).unwrap(),
            shift_cut_refining_step: N::from_f64(0.2).unwrap(),
            shift_cut_speed_up: N::from_f64(2.0).unwrap(),
        }
    }
}

pub trait DiffFn<N: Real, D: Dim + DimName> {
    fn params(&self) -> &RowVectorN<N, D>
    where
        DefaultAllocator: Allocator<N, U1, D>;

    fn mut_params(&mut self) -> &mut RowVectorN<N, D>
    where
        DefaultAllocator: Allocator<N, U1, D>;

    fn sanitize_params(&mut self);

    fn value(&self, x: N) -> N
    where
        DefaultAllocator: Allocator<N, U1, D>;

    fn values(&self, xs: &DVector<N>) -> DVector<N>
    where
        DefaultAllocator: Allocator<N, U1, D>;

    fn grad(&self, x: N) -> RowVectorN<N, D>
    where
        DefaultAllocator: Allocator<N, U1, D>;

    fn deriv(&self, x: N) -> N
    where
        DefaultAllocator: Allocator<N, U1, D>;

    fn jacobian(&self, xs: &DVector<N>) -> MatrixMN<N, Dynamic, D>
    where
        DefaultAllocator: Allocator<N, U1, D> + Allocator<N, Dynamic, D>;
}

pub fn gauss_newton<N: Real, D: Dim + DimName, F>(
    xs: &DVector<N>,
    ys: &DVector<N>,
    model: &mut F,
    params: GaussNewtonParams<N>,
) where
    F: DiffFn<N, D> + Clone + Debug,
    DefaultAllocator: Allocator<N, U1, D>
        + Allocator<N, D>
        + Allocator<N, Dynamic, D>
        + Allocator<N, Dynamic, Buffer = VecStorage<N, Dynamic, U1>>,
    VecStorage<N, Dynamic, U1>: nalgebra::storage::Storage<N, Dynamic>,
{
    debug_time!("gauss_newton");
    let mut shift_cut = N::one();
    let mut old_total_error = N::max_value();

    let mut old_model = model.clone();

    for iteration in 0..params.max_iterations {
        trace_time!("gauss_newton iteration {}", iteration);
        trace!("");
        trace!("model: {:?}", model);
        trace!("shift cut: {}", shift_cut);

        let jacobian: MatrixMN<N, Dynamic, D> = {
            trace_time!("jacobian");
            model.jacobian(&xs)
        };

        let svd: SVD<N, Dynamic, D> = {
            trace_time!("SVD");
            SVD::new(jacobian, true, true)
        };

        let residuals: DVector<N> = {
            trace_time!("residuals");
            let values: DVector<N> = model.values(&xs);
            ys - values
        };

        let total_error: N = {
            trace_time!("total error");
            residuals.norm()
        };
        trace!("total error: {}", total_error);

        let correction: RowVectorN<N, D> = {
            trace_time!("correction");
            let v = svd.v_t.unwrap().transpose();
            let sigma = Matrix::from_diagonal(&svd.singular_values.map(N::recip));
            let u_t = svd.u.unwrap().transpose();

            (v * sigma * (u_t * residuals) * shift_cut).transpose()
        };

        if total_error > old_total_error {
            trace!("rolling back");

            model.mut_params().copy_from(old_model.params());

            shift_cut *= params.shift_cut_refining_step;
            old_total_error = N::max_value();
            continue;
        }
        old_model.mut_params().copy_from(model.params());

        let error_improvement = ((total_error - old_total_error) / old_total_error).abs();
        old_total_error = total_error;

        let absolute_change = correction.abs().sum();

        {
            let params = model.params() + correction;
            model.mut_params().copy_from(&params);
            model.sanitize_params();
        }

        shift_cut = (shift_cut * params.shift_cut_speed_up).min(N::one());

        trace!("absolute change: {}", absolute_change);
        trace!("error improvement: {}", error_improvement);

        if iteration < params.min_iterations {
            continue;
        }

        if total_error < params.max_total_error
            || absolute_change <= (params.max_absolute_change * shift_cut)
            || shift_cut < params.min_shift_cut
            || error_improvement <= (params.max_error_improvement * shift_cut)
        {
            break;
        }
    }
}

pub fn linear_regression<N: Real, D: DimName>(
    x: MatrixMN<N, D, Dynamic>,
    y: MatrixMN<N, Dynamic, U1>,
) -> Option<RowVectorN<N, D>>
where
    DefaultAllocator: Allocator<N, Dynamic, Dynamic>
        + Allocator<N, Dynamic>
        + Allocator<N, U1, Dynamic>
        + Allocator<N, Dynamic, D>
        + Allocator<N, D, Dynamic>
        + Allocator<N, D, U1>
        + Reallocator<N, Dynamic, Dynamic, U1, D>,
{
    debug_time!("linear_regression");
    let x = {
        let s = x.shape();
        x.resize(s.0, s.1, N::zero())
    };
    let y = {
        let s = y.shape();
        y.resize(s.0, s.1, N::zero())
    };

    let x_svd = {
        let x_t = x.transpose();
        if x_t.len() == 0 {
            return None;
        }
        trace_time!("SVD");
        SVD::try_new(x_t, true, true, N::default_epsilon(), 1000)?
    };
    let u = x_svd.u.unwrap();
    let s = x_svd.singular_values;
    let v_t = x_svd.v_t.unwrap();

    let alpha = {
        trace_time!("alpha");
        u.transpose() * y
    };

    let s_shape = s.shape();
    let sinv = {
        trace_time!("S^-1");
        alpha.zip_map(&s.resize(s_shape.0, s_shape.1, N::zero()), |a, s| a / s)
    };

    trace_time!("linear_regression (V' * S^-1)'");
    Some(
        (v_t.transpose() * sinv)
            .transpose()
            .fixed_resize::<U1, D>(N::zero()),
    )
}
