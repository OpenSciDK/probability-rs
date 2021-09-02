pub mod regressor;

pub use rayon::prelude::*;
pub use regressor::*;

use super::{BaseEllipticalProcessParams, EllipticalProcessParams};
use crate::nonparametric::{ey, kernel_matrix, EllipticalProcessError};
use crate::{opensrdk_linear_algebra::*, RandomVariable};
use crate::{DistributionError, EllipticalParams};
use ey::y_ey;
use opensrdk_kernel_method::*;

/// Elliptical Process without approximation for scalability.
///
///
/// |                 | order                                                   |
/// | --------------- | ------------------------------------------------------- |
/// | pre-computation | $O(n^3)$ |
/// | prediction      | $O(n^2)$ |
///
/// | type args | mathematical expression                                 |
/// | --------- | ------------------------------------------------------- |
/// | `T`       | ![tex](https://latex.codecogs.com/svg.latex?\mathbb{D}) |
///
#[derive(Clone, Debug)]
pub struct ExactEllipticalProcessParams<K, T>
where
    K: Kernel<T>,
    T: RandomVariable,
{
    base: BaseEllipticalProcessParams<K, T>,
    mu: Vec<f64>,
    lsigma: Matrix,
    sigma_inv_y: Matrix,
    mahalanobis_squared: f64,
}

impl<K, T> ExactEllipticalProcessParams<K, T>
where
    K: Kernel<T>,
    T: RandomVariable,
{
    fn new(base: BaseEllipticalProcessParams<K, T>, y: &[f64]) -> Result<Self, DistributionError> {
        let n = y.len();
        if n == 0 {
            return Err(DistributionError::InvalidParameters(
                EllipticalProcessError::Empty.into(),
            ));
        }
        if n != base.x.len() {
            return Err(DistributionError::InvalidParameters(
                EllipticalProcessError::DimensionMismatch.into(),
            ));
        }

        let ey = ey(y);
        let mu = vec![ey; base.x.len()];
        let kxx = kernel_matrix(&base.kernel, &base.theta, &base.x, &base.x)?;
        let sigma = kxx + base.sigma.powi(2) * DiagonalMatrix::identity(n);
        let lsigma = sigma.potrf()?;
        let y_ey = y_ey(y, ey).col_mat();
        let y_ey_t = y_ey.t();
        let sigma_inv_y = lsigma.potrs(y_ey)?;
        let mahalanobis_squared = (y_ey_t * &sigma_inv_y)[0][0];

        Ok(Self {
            base,
            mu,
            lsigma,
            sigma_inv_y,
            mahalanobis_squared,
        })
    }
}

impl<K, T> BaseEllipticalProcessParams<K, T>
where
    K: Kernel<T>,
    T: RandomVariable,
{
    pub fn exact(self, y: &[f64]) -> Result<ExactEllipticalProcessParams<K, T>, DistributionError> {
        ExactEllipticalProcessParams::new(self, y)
    }
}

impl<K, T> EllipticalParams for ExactEllipticalProcessParams<K, T>
where
    K: Kernel<T>,
    T: RandomVariable,
{
    fn mu(&self) -> &Vec<f64> {
        &self.mu
    }

    fn sigma_inv_mul(&self, v: Matrix) -> Result<Matrix, DistributionError> {
        Ok(self.lsigma.potrs(v)?)
    }

    fn sigma_det_sqrt(&self) -> f64 {
        self.lsigma.trdet()
    }

    fn lsigma_cols(&self) -> usize {
        self.lsigma.cols()
    }

    fn sample(&self, z: Vec<f64>) -> Result<Vec<f64>, DistributionError> {
        Ok((self.mu[0] + &self.lsigma * z.col_mat()).vec())
    }
}

impl<K, T> EllipticalProcessParams<K, T> for ExactEllipticalProcessParams<K, T>
where
    K: Kernel<T>,
    T: RandomVariable,
{
    fn mahalanobis_squared(&self) -> f64 {
        self.mahalanobis_squared
    }
}