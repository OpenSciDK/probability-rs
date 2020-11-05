use crate::{ConditionalDistribution, Distribution};
use rand::prelude::StdRng;
use std::error::Error;

#[derive(thiserror::Error, Debug)]
pub enum ConditionalError {
    #[error("not conditioned")]
    NotConditioned,
}

pub struct InstantConditionalDistribution<T, U> {
    p: Box<dyn Fn(&T, &U) -> Result<f64, Box<dyn Error>>>,
    sample: Box<dyn Fn(&U, &mut StdRng) -> Result<T, Box<dyn Error>>>,
    condition: Option<U>,
}

impl<T, U> InstantConditionalDistribution<T, U> {
    pub fn new(
        p: Box<dyn Fn(&T, &U) -> Result<f64, Box<dyn Error>>>,
        sample: Box<dyn Fn(&U, &mut StdRng) -> Result<T, Box<dyn Error>>>,
    ) -> Self {
        Self {
            p,
            sample,
            condition: None,
        }
    }
}

impl<T, U> Distribution<T> for InstantConditionalDistribution<T, U> {
    fn p(&self, x: &T) -> Result<f64, Box<dyn Error>> {
        if self.condition.is_none() {
            return Err(ConditionalError::NotConditioned.into());
        }

        (self.p)(x, self.condition.as_ref().unwrap())
    }

    fn sample(&self, rng: &mut StdRng) -> Result<T, Box<dyn Error>> {
        if self.condition.is_none() {
            return Err(ConditionalError::NotConditioned.into());
        }

        (self.sample)(self.condition.as_ref().unwrap(), rng)
    }
}

impl<T, U> ConditionalDistribution<T, U> for InstantConditionalDistribution<T, U> {
    fn with_condition(&mut self, condition: U) -> Result<&mut Self, Box<dyn Error>> {
        self.condition = Some(condition);

        Ok(self)
    }
}
