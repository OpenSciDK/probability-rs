pub mod dirichlet_process;

use std::collections::HashMap;

pub use dirichlet_process::*;

use crate::{Distribution, RandomVariable};

#[derive(Clone, Debug)]
pub struct BaselineMeasure<D, T>
where
    D: Distribution<T = T, U = ()>,
    T: RandomVariable,
{
    pub distr: D,
}

pub type DiscreteMeasurableSpace = HashMap<usize, ()>;

pub trait DiscreteMeasure {
    fn measure(&self, a: DiscreteMeasurableSpace) -> f64;
}
