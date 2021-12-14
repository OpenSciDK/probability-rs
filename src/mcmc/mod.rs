pub mod elliptical_slice_sampling;
pub mod importance_sampling;
pub mod metropolis;
pub mod metropolis_hastings;
pub mod sir;
pub mod slice_sampling;

use std::{iter::Sum, ops::Div};

pub use elliptical_slice_sampling::*;
pub use importance_sampling::*;
pub use metropolis::*;
pub use metropolis_hastings::*;
pub use sir::*;
pub use slice_sampling::*;

use crate::RandomVariable;
use opensrdk_linear_algebra::Matrix;

pub trait VectorSampleable: RandomVariable {
    type T;
    fn transform_vec(self) -> (Vec<f64>, Self::T);
    fn restore(v: (Vec<f64>, Self::T)) -> Self;
}

impl VectorSampleable for f64 {
    type T = ();

    fn transform_vec(self) -> (Vec<f64>, Self::T) {
        (vec![self], ())
    }

    fn restore(v: (Vec<f64>, Self::T)) -> Self {
        v.0[0]
    }
}

impl VectorSampleable for Vec<f64> {
    type T = ();

    fn transform_vec(self) -> (Vec<f64>, Self::T) {
        (self, ())
    }

    fn restore(v: (Vec<f64>, Self::T)) -> Self {
        v.0
    }
}

impl VectorSampleable for Matrix {
    type T = usize;

    fn transform_vec(self) -> (Vec<f64>, Self::T) {
        let rows = self.rows();
        (self.vec(), rows)
    }

    fn restore(v: (Vec<f64>, Self::T)) -> Self {
        Matrix::from(v.1, v.0).unwrap()
    }
}

pub trait Meanable<T>: Iterator
where
    T: VectorSampleable + Sum + Div<f64, Output = T>,
{
    fn mean(self) -> T;
}

impl<I, T> Meanable<T> for I
where
    I: Iterator<Item = T>,
    T: VectorSampleable + Sum + Div<f64, Output = T>,
{
    fn mean(self) -> T {
        let n = self.size_hint().0 as f64;
        let s = self.sum::<T>();
        let m = s / n;
        m
    }
}
