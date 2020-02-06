use nalgebra::RealField;
use num_traits::ToPrimitive;
use std::fmt::Display;
use std::fmt::Error;
use std::fmt::Formatter;
use std::fmt::Write;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Engineering<N: RealField>(pub N);

impl<N: RealField + ToPrimitive> Display for Engineering<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        if self.0.is_zero() {
            write!(f, "{}", self.0)
        } else {
            let exp: i32 = (self.0.abs().log10() / N::from_i32(3).unwrap_or_else(N::zero))
                .floor()
                .min(N::from_f64(100.0).unwrap_or_else(N::zero))
                .max(N::from_f64(-100.0).unwrap_or_else(N::zero))
                .to_i32()
                .unwrap()
                * 3;
            let mantissa = self.0 / (N::from_i32(10).unwrap().powi(exp));
            write!(f, "{:.3}", mantissa)?;
            match exp {
                0 => {}
                -3 => f.write_char('m')?,
                -6 => f.write_char('µ')?,
                -9 => f.write_char('n')?,
                -12 => f.write_char('p')?,
                -15 => f.write_char('f')?,
                -18 => f.write_char('a')?,
                -21 => f.write_char('z')?,
                -24 => f.write_char('y')?,
                3 => f.write_char('k')?,
                6 => f.write_char('M')?,
                9 => f.write_char('G')?,
                12 => f.write_char('T')?,
                15 => f.write_char('P')?,
                18 => f.write_char('E')?,
                21 => f.write_char('Z')?,
                24 => f.write_char('Y')?,
                exp => write!(f, "e{}", exp)?,
            }
            Ok(())
        }
    }
}

pub trait Try {
    type Ok;
    type Error;

    fn into_result(self) -> Result<Self::Ok, Self::Error>;
}

pub struct NoneError;

impl<T> Try for Option<T> {
    type Ok = T;
    type Error = NoneError;

    fn into_result(self) -> Result<Self::Ok, Self::Error> {
        match self {
            Some(t) => Ok(t),
            None => Err(NoneError),
        }
    }
}
