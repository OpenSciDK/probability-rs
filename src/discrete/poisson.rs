use crate::{DependentJoint, Distribution, IndependentJoint, RandomVariable};
use crate::{DiscreteDistribution, DistributionError};
use rand::prelude::*;
use rand_distr::Poisson as RandPoisson;
use std::{ops::BitAnd, ops::Mul};

/// Poisson
#[derive(Clone, Debug)]
pub struct Poisson;

#[derive(thiserror::Error, Debug)]
pub enum PoissonError {
    #[error("'λ' must be positive")]
    LambdaMustBePositive,
}

fn factorial(num: u64) -> u64 {
    match num {
        0 | 1 => 1,
        _ => factorial(num - 1) * num,
    }
}

impl Distribution for Poisson {
    type Value = u64;
    type Condition = PoissonParams;

    fn fk(&self, x: &Self::Value, theta: &Self::Condition) -> Result<f64, DistributionError> {
        let lambda = theta.lambda();

        Ok(lambda.powi(*x as i32) / factorial(*x) as f64 * (-lambda).exp())
    }

    fn sample(
        &self,
        theta: &Self::Condition,
        rng: &mut dyn RngCore,
    ) -> Result<Self::Value, DistributionError> {
        let lambda = theta.lambda();

        let poisson = match RandPoisson::new(lambda) {
            Ok(v) => Ok(v),
            Err(e) => Err(DistributionError::Others(e.into())),
        }?;

        Ok(rng.sample(poisson) as u64)
    }
}

impl DiscreteDistribution for Poisson {}

#[derive(Clone, Debug, PartialEq)]
pub struct PoissonParams {
    lambda: f64,
}

impl PoissonParams {
    pub fn new(lambda: f64) -> Result<Self, PoissonError> {
        if lambda <= 0.0 {
            return Err(PoissonError::LambdaMustBePositive.into());
        }

        Ok(Self { lambda })
    }

    pub fn lambda(&self) -> f64 {
        self.lambda
    }
}

impl<Rhs, TRhs> Mul<Rhs> for Poisson
where
    Rhs: Distribution<Value = TRhs, Condition = PoissonParams>,
    TRhs: RandomVariable,
{
    type Output = IndependentJoint<Self, Rhs, u64, TRhs, PoissonParams>;

    fn mul(self, rhs: Rhs) -> Self::Output {
        IndependentJoint::new(self, rhs)
    }
}

impl<Rhs, URhs> BitAnd<Rhs> for Poisson
where
    Rhs: Distribution<Value = PoissonParams, Condition = URhs>,
    URhs: RandomVariable,
{
    type Output = DependentJoint<Self, Rhs, u64, PoissonParams, URhs>;

    fn bitand(self, rhs: Rhs) -> Self::Output {
        DependentJoint::new(self, rhs)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
