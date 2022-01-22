use crate::{
    DependentJoint, Distribution, ExactEllipticalParams, IndependentJoint, RandomVariable,
};
use crate::{DistributionError, EllipticalParams};
use opensrdk_linear_algebra::*;
use rand::prelude::*;
use rand_distr::StudentT as RandStudentT;
use std::marker::PhantomData;
use std::{ops::BitAnd, ops::Mul};

/// Multivariate Student-t distribution
#[derive(Clone, Debug)]
pub struct MultivariateStudentT<T = ExactMultivariateStudentTParams, U = ExactEllipticalParams>
where
    T: MultivariateStudentTParams<U>,
    U: EllipticalParams,
{
    phantom: PhantomData<(T, U)>,
}

impl<T, U> MultivariateStudentT<T, U>
where
    T: MultivariateStudentTParams<U>,
    U: EllipticalParams,
{
    pub fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum MultivariateStudentTError {
    #[error("dimension mismatch")]
    DimensionMismatch,
}

impl<T, U> Distribution for MultivariateStudentT<T, U>
where
    T: MultivariateStudentTParams<U>,
    U: EllipticalParams,
{
    type Value = Vec<f64>;
    type Condition = T;

    fn fk(&self, x: &Self::Value, theta: &Self::Condition) -> Result<f64, DistributionError> {
        let elliptical = theta.elliptical();
        let x_mu = elliptical.x_mu(x)?.col_mat();

        let n = x_mu.rows() as f64;
        let nu = theta.nu();

        Ok((1.0 + (x_mu.t() * elliptical.sigma_inv_mul(x_mu)?)[(0, 0)] / nu).powf(-(nu + n) / 2.0))
    }

    fn sample(
        &self,
        theta: &Self::Condition,
        rng: &mut dyn RngCore,
    ) -> Result<Self::Value, DistributionError> {
        let nu = theta.nu();
        let elliptical = theta.elliptical();

        let student_t = match RandStudentT::new(nu) {
            Ok(v) => Ok(v),
            Err(e) => Err(DistributionError::Others(e.into())),
        }?;

        let z = (0..elliptical.lsigma_cols())
            .into_iter()
            .map(|_| rng.sample(student_t))
            .collect::<Vec<_>>();

        Ok(elliptical.sample(z)?)
    }
}

pub trait MultivariateStudentTParams<T>: RandomVariable
where
    T: EllipticalParams,
{
    fn nu(&self) -> f64;
    fn elliptical(&self) -> &T;
}

#[derive(Clone, Debug)]
pub struct ExactMultivariateStudentTParams {
    nu: f64,
    elliptical: ExactEllipticalParams,
}

impl ExactMultivariateStudentTParams {
    /// # Multivariate student t
    /// `L` is needed as second argument under decomposition `Sigma = L * L^T`
    /// lsigma = sigma.potrf()?;
    pub fn new(nu: f64, mu: Vec<f64>, lsigma: Matrix) -> Result<Self, DistributionError> {
        let elliptical = ExactEllipticalParams::new(mu, lsigma)?;

        Ok(Self { nu, elliptical })
    }

    pub fn mu(&self) -> &Vec<f64> {
        self.elliptical.mu()
    }

    pub fn lsigma(&self) -> &Matrix {
        self.elliptical.lsigma()
    }
}

impl RandomVariable for ExactMultivariateStudentTParams {
    type RestoreInfo = usize;

    fn transform_vec(self) -> (Vec<f64>, Self::RestoreInfo) {
        todo!()
    }

    fn restore(v: Vec<f64>, info: Self::RestoreInfo) -> Self {
        todo!()
    }
}

impl MultivariateStudentTParams<ExactEllipticalParams> for ExactMultivariateStudentTParams {
    fn nu(&self) -> f64 {
        self.nu
    }

    fn elliptical(&self) -> &ExactEllipticalParams {
        &self.elliptical
    }
}

impl<T, U, Rhs, TRhs> Mul<Rhs> for MultivariateStudentT<T, U>
where
    T: MultivariateStudentTParams<U>,
    U: EllipticalParams,
    Rhs: Distribution<Value = TRhs, Condition = T>,
    TRhs: RandomVariable,
{
    type Output = IndependentJoint<Self, Rhs, Vec<f64>, TRhs, T>;

    fn mul(self, rhs: Rhs) -> Self::Output {
        IndependentJoint::new(self, rhs)
    }
}

impl<T, U, Rhs, URhs> BitAnd<Rhs> for MultivariateStudentT<T, U>
where
    T: MultivariateStudentTParams<U>,
    U: EllipticalParams,
    Rhs: Distribution<Value = T, Condition = URhs>,
    URhs: RandomVariable,
{
    type Output = DependentJoint<Self, Rhs, Vec<f64>, T, URhs>;

    fn bitand(self, rhs: Rhs) -> Self::Output {
        DependentJoint::new(self, rhs)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Distribution, ExactMultivariateStudentTParams, MultivariateStudentT};
    use opensrdk_linear_algebra::*;
    use rand::prelude::*;
    #[test]
    fn it_works() {
        let student_t = MultivariateStudentT::new();
        let mut rng = StdRng::from_seed([1; 32]);

        let mu = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0];
        let lsigma = mat!(
           1.0,  0.0,  0.0,  0.0,  0.0,  0.0;
           2.0,  3.0,  0.0,  0.0,  0.0,  0.0;
           4.0,  5.0,  6.0,  0.0,  0.0,  0.0;
           7.0,  8.0,  9.0, 10.0,  0.0,  0.0;
          11.0, 12.0, 13.0, 14.0, 15.0,  0.0;
          16.0, 17.0, 18.0, 19.0, 20.0, 21.0
        );
        println!("{:#?}", lsigma);

        let x = student_t
            .sample(
                &ExactMultivariateStudentTParams::new(1.0, mu, lsigma).unwrap(),
                &mut rng,
            )
            .unwrap();

        println!("{:#?}", x);
    }
}
