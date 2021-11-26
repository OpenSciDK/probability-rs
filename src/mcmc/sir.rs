// Sampling Importance Resampling
use crate::rand::SeedableRng;
use crate::{elliptical::*, Meanable};
use crate::{ContinuousSamplesDistribution, Distribution, DistributionError, RandomVariable};
use rand::rngs::StdRng;
use std::hash::Hash;
use std::iter::Sum;
use std::marker::PhantomData;
use std::ops::Div;
pub struct ParticleFilter<Y, X, D1, D2, PD>
where
    Y: RandomVariable,
    X: RandomVariable + Eq + Hash + Sum + Div + Meanable<X>,
    D1: Distribution<Value = Y, Condition = X>,
    D2: Distribution<Value = X, Condition = X>,
    PD: Distribution<Value = X, Condition = X>,
{
    observable: Vec<Y>,
    distr_y: D1,
    distr_x: D2,
    proposal: PD,
    phantom: PhantomData<X>,
}

impl<Y, X, D1, D2, PD> ParticleFilter<Y, X, D1, D2, PD>
where
    Y: RandomVariable,
    X: RandomVariable + Eq + Hash + Sum + Div + Meanable<X>,
    D1: Distribution<Value = Y, Condition = X>,
    D2: Distribution<Value = X, Condition = X>,
    PD: Distribution<Value = X, Condition = X>,
{
    pub fn new(
        observable: Vec<Y>,
        distr_y: D1,
        distr_x: D2,
        proposal: PD,
    ) -> Result<Self, DistributionError> {
        Ok(Self {
            observable,
            distr_y,
            distr_x,
            proposal,
            phantom: PhantomData,
        })
    }

    pub fn filtering(
        &self,
        particles_initial: Vec<X>,
        thr: f64,
    ) -> Result<ContinuousSamplesDistribution<X>, DistributionError> {
        let mut rng = StdRng::from_seed([1; 32]);

        let mut x_vec = vec![];

        let particles_len = particles_initial.len();

        let mut p_prior = particles_initial;

        let w_initial = vec![1.0 / particles_len as f64; particles_len];

        let mut w_prior = w_initial;

        for t in 0..(self.observable).len() {
            let mut p = (0..particles_len)
                .into_iter()
                .map(|i| -> Result<_, DistributionError> {
                    let pi = &self.proposal.sample(&p_prior[i], &mut rng)?;
                    Ok(pi)
                })
                .collect::<Result<Vec<_>, _>>()?;

            loop {
                let w_orig = (0..particles_len)
                    .into_iter()
                    .map(|i| -> Result<_, DistributionError> {
                        let wi_orig = w_prior[i]
                            * self.distr_y.fk(&self.observable[t], &p[i])?
                            * self.distr_x.fk(&p[i], &p_prior[i])?
                            / self.proposal.fk(&p[i], &p_prior[i])?;
                        Ok(wi_orig)
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                let w = (0..particles_len)
                    .into_iter()
                    .map(|i| -> Result<_, DistributionError> {
                        let wi = w_prior[i] / (w_orig.iter().map(|wi_orig| wi_orig).sum::<f64>());
                        Ok(wi)
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                let eff = 1.0 / (w.iter().map(|wi| wi.powi(2)).sum::<f64>());

                if eff > thr {
                    let x = (0..particles_len)
                        .into_iter()
                        .map(|i| -> Result<_, DistributionError> {
                            let xi = w[i] * p[i];
                            Ok(xi)
                        })
                        .sum::<f64>()?;
                    x_vec.append(x);
                    p_prior = p;
                    w_prior = w;
                    break;
                }

                let mut p_sample = vec![];

                for i in 0..w.len() {
                    let num_w = (particles_len as f64 * 100.0 * w[i]).round() as usize;
                    let mut pi_sample = vec![p[i].clone(); num_w];
                    p_sample.append(&mut pi_sample);
                }

                let weighted_distr = ContinuousSamplesDistribution::new(p_sample);

                p = (0..particles_len)
                    .into_iter()
                    .map(|_i| -> Result<_, DistributionError> {
                        let pi = weighted_distr.sample(&(), &mut rng)?;
                        Ok(pi)
                    })
                    .collect::<Result<Vec<_>, _>>()?;
            }
        }
        Ok(x_vec)
    }
}