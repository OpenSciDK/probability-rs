pub mod exact_gp;
pub mod ey;
pub mod kernel_matrix;
pub mod kiss_love_gp;
pub mod student_tp;

use crate::{MultivariateNormalParams, NormalParams};
pub use exact_gp::*;
pub use ey::*;
pub use kernel_matrix::*;
pub use kiss_love_gp::*;
use opensrdk_kernel_method::Kernel;
use std::{error::Error, fmt::Debug};
pub use student_tp::*;

#[derive(thiserror::Error, Debug)]
pub enum GaussianProcessError {
    #[error("Data is empty.")]
    Empty,
    #[error("Dimension mismatch.")]
    DimensionMismatch,
    #[error("NaN contaminated.")]
    NaNContamination,
    #[error("Not prepared.")]
    NotPrepared,
}

///
/// ![tex](https://latex.codecogs.com/svg.latex?y_n%3Df%28\mathbf{x}_n%29+\varepsilon_n)
///
/// ![tex](https://latex.codecogs.com/svg.latex?\mathbf{f}%7CX\sim\mathcal{GP}%280,K_{XX}%29)
///
/// ![tex](https://latex.codecogs.com/svg.latex?\mathbf{y}-\bar{\mathbf{y}}%7C\mathbf{f}\sim\mathcal{N}%28\mathbf{f},\sigma^2I%29)
///
/// ![tex](https://latex.codecogs.com/svg.latex?\mathbf{y}-\bar{\mathbf{y}}%7CX\sim\mathcal{N}%280,K_{XX}+\sigma^2I%29)
pub trait GaussianProcess<K, T>
where
    K: Kernel<T>,
    T: Clone + Debug,
{
    fn new(kernel: K) -> Self;

    fn set_x(&mut self, x: Vec<T>) -> Result<&mut Self, Box<dyn Error>>;
    fn set_theta(&mut self, theta: Vec<f64>) -> Result<&mut Self, Box<dyn Error>>;

    fn kernel(&self) -> &K;
    fn theta(&self) -> &[f64];

    fn n(&self) -> usize;
    fn ey(&self) -> f64;
    fn prepare_predict(&mut self, y: &[f64]) -> Result<(), Box<dyn Error>>;

    fn predict(&self, xs: T) -> Result<NormalParams, Box<dyn Error>> {
        let mul_n = self.predict_multivariate(&[xs])?;

        NormalParams::new(mul_n.mu()[0], mul_n.lsigma()[0][0])
    }

    fn predict_multivariate(&self, xs: &[T]) -> Result<MultivariateNormalParams, Box<dyn Error>>;

    fn kxx_inv_vec(
        &self,
        vec: Vec<f64>,
        params: &GaussianProcessParams<T>,
        with_det_lkxx: bool,
    ) -> Result<(Vec<f64>, Option<f64>), Box<dyn Error>>;

    fn lkxx_vec(
        &self,
        vec: Vec<f64>,
        params: &GaussianProcessParams<T>,
    ) -> Result<Vec<f64>, Box<dyn Error>>;
}

#[derive(Clone, Debug, PartialEq)]
pub struct GaussianProcessParams<T>
where
    T: Clone + Debug,
{
    pub x: Option<Vec<T>>,
    pub theta: Option<Vec<f64>>,
}