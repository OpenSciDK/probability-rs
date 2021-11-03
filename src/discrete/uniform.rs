use crate::{DiscreteDistribution, Distribution, DistributionError, RandomVariable};
use rand::prelude::*;
use std::{collections::HashSet, hash::Hash, marker::PhantomData};

#[derive(Clone, Debug)]
pub struct DiscreteUniform<T>
where
    T: RandomVariable + Eq + Hash,
{
    phantom: PhantomData<T>,
}

#[derive(thiserror::Error, Debug)]
pub enum DiscreteUniformError {
    #[error("Range is empty.")]
    RangeIsEmpty,
    #[error("Unknown error")]
    Unknown,
}

impl<T> DiscreteUniform<T>
where
    T: RandomVariable + Eq + Hash,
{
    pub fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl<T> Distribution for DiscreteUniform<T>
where
    T: RandomVariable + Eq + Hash,
{
    type T = T;
    type U = HashSet<T>;

    fn fk(&self, _x: &Self::T, theta: &Self::U) -> Result<f64, DistributionError> {
        Ok(1.0 / theta.len() as f64)
    }

    fn sample(&self, theta: &Self::U, rng: &mut dyn RngCore) -> Result<Self::T, DistributionError> {
        let len = theta.len();
        if len == 0 {
            return Err(DistributionError::InvalidParameters(
                DiscreteUniformError::RangeIsEmpty.into(),
            ));
        }
        let i = rng.gen_range(0..len);

        for (j, x) in theta.iter().enumerate() {
            if i == j {
                return Ok(x.clone());
            }
        }

        Err(DistributionError::InvalidParameters(
            DiscreteUniformError::Unknown.into(),
        ))
    }
}

impl<T> DiscreteDistribution for DiscreteUniform<T> where T: RandomVariable + Eq + Hash {}
