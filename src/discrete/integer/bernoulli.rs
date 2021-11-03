use crate::*;
use rand::Rng;
use std::ops::{BitAnd, Mul};

#[derive(Clone, Debug)]
pub struct Bernoulli;

#[derive(thiserror::Error, Debug)]
pub enum BernoulliError {
    #[error("'p' must be probability.")]
    PMustBeProbability,
}

impl Distribution for Bernoulli {
    type T = bool;
    type U = BernoulliParams;

    fn fk(&self, _x: &Self::T, theta: &Self::U) -> Result<f64, DistributionError> {
        Ok(theta.p())
    }

    fn sample(
        &self,
        theta: &Self::U,
        rng: &mut dyn rand::RngCore,
    ) -> Result<Self::T, DistributionError> {
        let u = rng.gen_range(0.0..=1.0);
        Ok(u <= theta.p())
    }
}

impl DiscreteDistribution for Bernoulli {}

#[derive(Clone, Debug)]
pub struct BernoulliParams {
    p: f64,
}

impl BernoulliParams {
    pub fn new(p: f64) -> Result<Self, DistributionError> {
        if p < 0.0 || 1.0 < p {
            return Err(DistributionError::InvalidParameters(
                BernoulliError::PMustBeProbability.into(),
            ));
        }

        Ok(Self { p })
    }

    pub fn p(&self) -> f64 {
        self.p
    }
}

impl<Rhs, TRhs> Mul<Rhs> for Bernoulli
where
    Rhs: Distribution<T = TRhs, U = BernoulliParams>,
    TRhs: RandomVariable,
{
    type Output = IndependentJoint<Self, Rhs, bool, TRhs, BernoulliParams>;

    fn mul(self, rhs: Rhs) -> Self::Output {
        IndependentJoint::new(self, rhs)
    }
}

impl<Rhs, URhs> BitAnd<Rhs> for Bernoulli
where
    Rhs: Distribution<T = BernoulliParams, U = URhs>,
    URhs: RandomVariable,
{
    type Output = DependentJoint<Self, Rhs, bool, BernoulliParams, URhs>;

    fn bitand(self, rhs: Rhs) -> Self::Output {
        DependentJoint::new(self, rhs)
    }
}
