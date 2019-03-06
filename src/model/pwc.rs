use itertools::Itertools;
use num_traits::float::Float;

pub struct PieceWiseConstantFunction {
    min: f64,
    max: f64,
    buckets: Vec<f64>,
}

impl PieceWiseConstantFunction {
    pub fn from_points(
        min: f64,
        max: f64,
        buckets: usize,
        min_bucket_population: usize,
        points: &[(f64, f64)],
    ) -> PieceWiseConstantFunction {
        PieceWiseConstantFunction::new(min, max, buckets, min_bucket_population, |s, e| {
            points.iter().cloned().filter_map(
                move |(x, y)| {
                    if x >= s && x < e {
                        Some(y)
                    } else {
                        None
                    }
                },
            )
        })
    }

    pub fn new<F, I>(
        min: f64,
        max: f64,
        buckets: usize,
        min_bucket_population: usize,
        f: F,
    ) -> PieceWiseConstantFunction
    where
        F: Fn(f64, f64) -> I,
        I: Iterator<Item = f64>,
    {
        let mut pwc = Vec::new();
        pwc.reserve_exact(buckets);
        pwc.resize(buckets, f64::nan());

        let span = max - min;
        #[allow(clippy::needless_range_loop)]
        for b in 0..buckets {
            let start = min + span * (b as f64 / buckets as f64);
            let end = min + span * ((b + 1) as f64 / buckets as f64);

            let vs: Vec<f64> = f(start, end).collect_vec();
            if vs.len() >= min_bucket_population {
                pwc[b] = vs.iter().sum::<f64>() / vs.len() as f64;
            }
        }

        PieceWiseConstantFunction {
            min,
            max,
            buckets: pwc,
        }
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = (f64, f64)> + 'a {
        let span = self.max - self.min;
        let buckets_no = self.buckets.len() as f64;
        let min = self.min;

        self.buckets.iter().enumerate().filter_map(move |(ix, v)| {
            if v.is_nan() {
                None
            } else {
                let x = min + span * ((ix as f64 + 0.5) / buckets_no);
                Some((x, *v))
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::model::pwc::PieceWiseConstantFunction;
    use itertools::Itertools;

    #[test]
    fn values_not_in_domain_are_skipped() {
        let h =
            PieceWiseConstantFunction::from_points(0.0, 1.0, 1, 1, &[(-1.0, 12.0), (2.0, 23.0)]);
        assert_eq!(h.iter().collect_vec(), vec![]);
    }

    #[test]
    fn left_end_is_inclusive() {
        let h = PieceWiseConstantFunction::from_points(0.0, 1.0, 1, 1, &[(0.0, 12.0)]);
        assert_eq!(h.iter().collect_vec(), vec![(0.5, 12.0)]);
    }

    #[test]
    fn right_end_is_exclusive() {
        let h = PieceWiseConstantFunction::from_points(0.0, 1.0, 1, 1, &[(1.0, 23.0)]);
        assert_eq!(h.iter().collect_vec(), vec![]);
    }

    #[test]
    fn bucket_value_is_average() {
        let h = PieceWiseConstantFunction::from_points(0.0, 1.0, 1, 1, &[(0.2, 12.0), (0.0, 23.0)]);
        assert_eq!(h.iter().collect_vec(), vec![(0.5, 17.5)]);
    }
}
