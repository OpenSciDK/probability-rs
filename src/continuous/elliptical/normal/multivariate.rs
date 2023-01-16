use crate::{
    ConditionDifferentiableDistribution, DependentJoint, Distribution, ExactEllipticalParams,
    IndependentJoint, RandomVariable, SampleableDistribution, ValueDifferentiableDistribution,
};
use crate::{DistributionError, EllipticalParams};
use opensrdk_linear_algebra::{DiagonalMatrix, Vector};
use rand::prelude::*;
use rand_distr::StandardNormal;
use std::marker::PhantomData;
use std::{ops::BitAnd, ops::Mul};

/// Multivariate normal distribution
#[derive(Clone, Debug)]
pub struct MultivariateNormal<T = ExactEllipticalParams>
where
    T: EllipticalParams,
{
    phantom: PhantomData<T>,
}

impl<T> MultivariateNormal<T>
where
    T: EllipticalParams,
{
    pub fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum MultivariateNormalError {}

impl<T> Distribution for MultivariateNormal<T>
where
    T: EllipticalParams,
{
    type Value = Vec<f64>;
    type Condition = T;

    fn p_kernel(&self, x: &Self::Value, theta: &Self::Condition) -> Result<f64, DistributionError> {
        let x_mu = theta.x_mu(x)?.col_mat();
        let n = x.len();

        // For preventing the result from being zero, dividing e^n
        Ok((-1.0 / 2.0 * (x_mu.t() * theta.sigma_inv_mul(x_mu)?)[(0, 0)] / (n as f64).exp()).exp())
    }
}

impl<T, Rhs, TRhs> Mul<Rhs> for MultivariateNormal<T>
where
    T: EllipticalParams,
    Rhs: Distribution<Value = TRhs, Condition = T>,
    TRhs: RandomVariable,
{
    type Output = IndependentJoint<Self, Rhs, Vec<f64>, TRhs, T>;

    fn mul(self, rhs: Rhs) -> Self::Output {
        IndependentJoint::new(self, rhs)
    }
}

impl<T, Rhs, URhs> BitAnd<Rhs> for MultivariateNormal<T>
where
    T: EllipticalParams,
    Rhs: Distribution<Value = T, Condition = URhs>,
    URhs: RandomVariable,
{
    type Output = DependentJoint<Self, Rhs, Vec<f64>, T, URhs>;

    fn bitand(self, rhs: Rhs) -> Self::Output {
        DependentJoint::new(self, rhs)
    }
}

impl SampleableDistribution for MultivariateNormal {
    fn sample(
        &self,
        theta: &Self::Condition,
        rng: &mut dyn RngCore,
    ) -> Result<Self::Value, DistributionError> {
        let z = (0..theta.lsigma_cols())
            .into_iter()
            .map(|_| rng.sample(StandardNormal))
            .collect::<Vec<f64>>();

        Ok(theta.sample(z)?)
    }
}

impl ValueDifferentiableDistribution for MultivariateNormal {
    fn ln_diff_value(
        &self,
        x: &Self::Value,
        theta: &Self::Condition,
    ) -> Result<Vec<f64>, DistributionError> {
        let sigma_inv = theta.lsigma().clone().pptri()?.to_mat();
        let mu_mat = theta.x_mu(x)?.row_mat();
        let x_mat = x.clone().row_mat();
        let x_mu_mat = x_mat - mu_mat;
        let f_x = &-x_mu_mat * &sigma_inv;
        Ok(f_x.vec())
    }
}

impl<T> ConditionDifferentiableDistribution for MultivariateNormal<T>
where
    T: EllipticalParams,
{
    fn ln_diff_condition(
        &self,
        x: &Self::Value,
        theta: &Self::Condition,
    ) -> Result<Vec<f64>, DistributionError> {
        // let lsigma = theta.lsigma().0.to_mat();
        // let _sigma = &lsigma * &lsigma.t();
        // let sigma_inv = theta.lsigma().clone().pptri()?.to_mat();
        // let sigma_inv = DiagonalMatrix::new((&lsigma_mat * lsigma_mat.t()).vec())
        //     .powf(-1.0)
        //     .mat();
        let mu_mat = theta.x_mu(x)?.row_mat();
        let x_mat = x.clone().row_mat();
        let x_mu_mat = x_mat - mu_mat;

        let f_mu = theta.sigma_inv_mul(x_mu_mat.clone()).unwrap();

        let n = theta.lsigma_cols();
        let identity = DiagonalMatrix::<f64>::identity(n).mat();
        let lsigma_inv_t = theta.sigma_inv_mul(identity).unwrap();
        let x_mu_t = x_mu_mat.t();
        let lsigma_inv_mul_x_mu = theta.sigma_inv_mul(x_mu_mat).unwrap();
        let lsigma_inv_mul_lsigma_inv_mul_x_mu = theta.sigma_inv_mul(lsigma_inv_mul_x_mu).unwrap();

        println!("{:#?}", &lsigma_inv_t);
        println!("{:#?}", &x_mu_t);
        //println!("{:#?}", &lsigma_inv_mul_x_mu);
        println!("{:#?}", &lsigma_inv_mul_lsigma_inv_mul_x_mu);

        let f_lsigma = (&x_mu_t * lsigma_inv_mul_lsigma_inv_mul_x_mu - &lsigma_inv_t) * 0.5;
        println!("{:#?}", f_mu.clone().vec());
        println!("{:#?}", f_lsigma.clone().vec());
        let result = [f_mu.vec(), f_lsigma.vec()].concat();
        println!("{:#?}", &result);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ConditionDifferentiableDistribution, ExactMultivariateNormalParams, MultivariateNormal,
        SampleableDistribution, ValueDifferentiableDistribution,
    };
    use opensrdk_linear_algebra::{pp::trf::PPTRF, *};
    use rand::prelude::*;
    #[test]
    fn it_works() {
        let normal = MultivariateNormal::new();
        let mut rng = StdRng::from_seed([1; 32]);

        let mu = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0];
        let lsigma = SymmetricPackedMatrix::from_mat(&mat!(
           1.0,  0.0,  0.0,  0.0,  0.0,  0.0;
           2.0,  3.0,  0.0,  0.0,  0.0,  0.0;
           4.0,  5.0,  6.0,  0.0,  0.0,  0.0;
           7.0,  8.0,  9.0, 10.0,  0.0,  0.0;
          11.0, 12.0, 13.0, 14.0, 15.0,  0.0;
          16.0, 17.0, 18.0, 19.0, 20.0, 21.0
        ))
        .unwrap();
        println!("{:#?}", lsigma);

        let x = normal
            .sample(
                &ExactMultivariateNormalParams::new(mu, PPTRF(lsigma)).unwrap(),
                &mut rng,
            )
            .unwrap();

        println!("{:#?}", x);
    }

    #[test]
    fn it_works2() {
        let normal = MultivariateNormal::new();
        let mut _rng = StdRng::from_seed([1; 32]);

        let mu = vec![0.0, 1.0];
        let lsigma = SymmetricPackedMatrix::from_mat(&mat!(
           1.0,  0.0;
           2.0,  1.0
        ))
        .unwrap();

        let x = vec![0.0, 1.0];

        let f = normal
            .ln_diff_value(
                &x,
                &ExactMultivariateNormalParams::new(mu, PPTRF(lsigma)).unwrap(),
            )
            .unwrap();
        println!("{:#?}", f);
    }

    #[test]
    fn it_works_3() {
        let normal = MultivariateNormal::new();
        let mut _rng = StdRng::from_seed([1; 32]);

        let mu = vec![0.0, 1.0];
        let lsigma = SymmetricPackedMatrix::from_mat(&mat!(
           1.0,  0.0;
           2.0,  1.0
        ))
        .unwrap();

        let x = vec![0.0, 1.0];

        let f = normal
            .ln_diff_condition(
                &x,
                &ExactMultivariateNormalParams::new(mu, PPTRF(lsigma)).unwrap(),
            )
            .unwrap();
        println!("{:#?}", f);
    }
}
