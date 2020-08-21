use crate::MultivariateDistribution;
use opensrdk_linear_algebra::{Matrix, Vector};
use rand::prelude::*;
use rand_distr::StandardNormal;

pub struct MultivariateNormal {
    mean: Vec<f64>,
    covariance_decomposed: Matrix,
}

impl MultivariateNormal {
    /// # Multivariate normal
    /// `L` is needed as second argument under decomposition `Sigma = L * L^T`
    pub fn new(mean: Vec<f64>, covariance_decomposed: Matrix) -> Self {
        Self {
            mean,
            covariance_decomposed,
        }
    }

    pub fn from(mean: Vec<f64>, covariance: Matrix) -> Result<Self, String> {
        if mean.len() != covariance.get_rows() {
            return Err("dimension mismatch".to_owned());
        }

        let decomposed = {
            let (u, sigma, _) = covariance.gesvd()?;

            u * sigma.dipowf(0.5)
        };

        Ok(Self::new(mean, decomposed))
    }

    pub fn get_mean(&self) -> &[f64] {
        &self.mean
    }

    pub fn get_covariance(&self) -> Matrix {
        &self.covariance_decomposed * self.covariance_decomposed.t()
    }
}

impl MultivariateDistribution for MultivariateNormal {
    fn sample(&self, thread_rng: &mut ThreadRng) -> Result<Vec<f64>, String> {
        let z = (0..self.covariance_decomposed.get_rows())
            .into_iter()
            .map(|_| thread_rng.sample(StandardNormal))
            .collect::<Vec<_>>();

        let y = self.mean.clone().to_column_vector().gemm(
            &self.covariance_decomposed,
            &z.to_column_vector(),
            1.0,
            1.0,
        )?;

        Ok(y.get_elements().to_vec())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
