use super::VectorSampleable;
use crate::{Distribution, RandomVariable};
use rand::prelude::*;
use rayon::prelude::*;
use std::{error::Error, f64::consts::PI};

/// Sample `b` from posterior p(b|a) with likelihood p(a|b) and prior p(b)
/// `b` must be generated by elliptical distribution
pub struct EllipticalSliceSampler<'a, L, P, A, B>
where
    L: Distribution<T = A, U = B>,
    P: Distribution<T = B, U = ()>,
    A: RandomVariable,
    B: VectorSampleable,
{
    value: &'a A,
    likelihood: &'a L,
    prior: &'a P,
}

impl<'a, L, P, A, B> EllipticalSliceSampler<'a, L, P, A, B>
where
    L: Distribution<T = A, U = B>,
    P: Distribution<T = B, U = ()>,
    A: RandomVariable,
    B: VectorSampleable,
{
    pub fn new(value: &'a A, likelihood: &'a L, prior: &'a P) -> Self {
        Self {
            value,
            likelihood,
            prior,
        }
    }

    fn step(mut v: Vec<f64>, theta: f64, nu: &Vec<f64>) -> Vec<f64> {
        let cos = theta.cos();
        let sin = theta.sin();
        v.par_iter_mut()
            .zip(nu.par_iter())
            .for_each(|(bi, &nui)| *bi = *bi * cos + nui * sin);

        v
    }

    pub fn sample(&self, rng: &mut dyn RngCore) -> Result<B, Box<dyn Error>> {
        let nu = self.prior.sample(&(), rng)?;

        let mut b = self.prior.sample(&(), rng)?;

        let rho = self.likelihood.fk(self.value, &b)? * rng.gen_range(0.0..1.0);
        let mut theta = rng.gen_range(0.0..2.0 * PI);

        let mut start = theta - 2.0 * PI;
        let mut end = theta;

        let nu = nu.transform_vec();

        loop {
            let mut buf = b.transform_vec();
            buf.0 = Self::step(buf.0, theta, &nu.0);

            b = B::restore(buf);
            if rho < self.likelihood.fk(self.value, &b)? {
                break;
            }

            if 0.0 < theta {
                end = 0.0;
            } else {
                start = 0.0;
            }
            theta = rng.gen_range(start..end);
        }

        Ok(b)
    }
}

#[cfg(test)]
mod tests {
    use crate::distribution::Distribution;
    use crate::*;
    use core::f64::consts::PI;
    use rand::prelude::*;

    #[test]
    fn it_works() {
        let distr = InstantDistribution::new(
            &|x: &f64, theta: &f64| {
                let mu = *theta;
                Ok(1.0 / (2.0 * PI * 10.0f64.powi(2)).sqrt()
                    * (-(x - mu).powi(2) / (2.0 * 10.0f64.powi(2))).exp())
            },
            &|theta, rng| Normal.sample(&NormalParams::new(*theta, 10.0).unwrap(), rng),
        );
        let distr_2 = InstantDistribution::new(
            &|x: &f64, _theta: &()| {
                let mu: f64 = 10.0;
                Ok(1.0 / (2.0 * PI * 10.0f64.powi(2)).sqrt()
                    * (-(x - mu).powi(2) / (2.0 * 10.0f64.powi(2))).exp())
            },
            &|_theta, rng| {
                Normal.sample(
                    &NormalParams::new(10.0, (10.0f64 * 0.1).abs()).unwrap(),
                    rng,
                )
            },
        );

        let value = 1.0;
        let likelihood = distr;
        let prior = distr_2;

        let start = EllipticalSliceSampler::new(&value, &likelihood, &prior);
        let mut rng = StdRng::from_seed([1; 32]);
        let x = start.sample(&mut rng).unwrap();

        println!("{}", x);
    }
}
