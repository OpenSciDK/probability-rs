use std::collections::{HashMap, HashSet};

use crate::{DistributionError, RandomVariable};

#[derive(Clone, Debug)]
pub struct ClusterSwitch<T>
where
    T: RandomVariable,
{
    s: Vec<u32>,
    s_inv: HashMap<u32, HashSet<usize>>,
    theta: HashMap<u32, T>,
    max_k: u32,
}

#[derive(thiserror::Error, Debug)]
pub enum ClusterSwitchError {
    #[error("All elements of `s` must be positive")]
    SMustBePositive,
    #[error("Unknown error")]
    Unknown,
}

impl<T> ClusterSwitch<T>
where
    T: RandomVariable,
{
    pub fn new(s: Vec<u32>, theta: HashMap<u32, T>) -> Result<Self, DistributionError> {
        let mut s_inv = HashMap::new();
        let mut max_k = 0;

        for (i, &si) in s.iter().enumerate() {
            if si == 0 {
                return Err(DistributionError::InvalidParameters(
                    ClusterSwitchError::SMustBePositive.into(),
                ));
            }
            s_inv.entry(si).or_insert(HashSet::new()).insert(i);

            if max_k < si {
                max_k = si
            }
        }

        Ok(Self {
            s,
            s_inv,
            theta,
            max_k,
        })
    }

    pub fn s(&self) -> &Vec<u32> {
        &self.s
    }

    pub fn s_inv(&self) -> &HashMap<u32, HashSet<usize>> {
        &self.s_inv
    }

    pub fn theta(&self) -> &HashMap<u32, T> {
        &self.theta
    }

    pub fn set_s(&mut self, i: usize, si: u32) -> u32 {
        self.s_inv
            .entry(self.s[i])
            .or_insert(HashSet::new())
            .remove(&i);
        if self.s_inv.get(&self.s[i]).unwrap().len() == 0 {
            self.s_inv.remove(&self.s[i]);
            self.theta.remove(&self.s[i]);
        }

        if si == 0 {
            self.max_k += 1;
            self.s[i] = self.max_k;
            self.s_inv
                .entry(self.s[i])
                .or_insert(HashSet::new())
                .insert(i);

            return self.max_k;
        }

        self.s[i] = si;
        self.s_inv
            .entry(self.s[i])
            .or_insert(HashSet::new())
            .insert(i);

        if self.max_k < si {
            self.max_k = si;
        }

        si
    }

    pub fn set_s_with_theta(&mut self, i: usize, si: u32, theta: T) -> u32 {
        let new_k = self.set_s(i, si);
        self.theta.insert(new_k, theta);
        new_k
    }

    pub fn n(&self, k: u32) -> usize {
        match self.s_inv.get(&k) {
            Some(v) => v.len(),
            None => 0,
        }
    }

    pub fn clusters_len(&self) -> usize {
        self.s_inv.len()
    }
}

#[cfg(test)]
mod tests {
    use crate::nonparametric::ClusterSwitch;
    use crate::*;
    use std::collections::HashMap;

    #[test]
    fn it_works() {
        let mu = 5.0;
        let s = vec![1u32, 2u32, 3u32, 4u32];
        let mut theta = HashMap::new();
        theta.insert(1u32, NormalParams::new(1.0, 2.0).unwrap());
        theta.insert(2u32, NormalParams::new(2.0, 2.0).unwrap());
        theta.insert(3u32, NormalParams::new(3.0, 2.0).unwrap());
        theta.insert(4u32, NormalParams::new(4.0, 2.0).unwrap());

        let mut cs = ClusterSwitch::new(s, theta).unwrap();
        let new_k = cs.set_s_with_theta(0, 0u32, NormalParams::new(mu, 2.0).unwrap());
        let new_mu = cs.theta().get(&new_k).unwrap().mu();

        assert_eq!(mu, new_mu);
    }

    #[test]
    fn it_works2() {
        let s = vec![1u32, 2u32, 3u32, 4u32];
        let mut theta = HashMap::new();
        theta.insert(1u32, NormalParams::new(1.0, 2.0).unwrap());
        theta.insert(2u32, NormalParams::new(2.0, 2.0).unwrap());
        theta.insert(3u32, NormalParams::new(3.0, 2.0).unwrap());
        theta.insert(4u32, NormalParams::new(4.0, 2.0).unwrap());

        let mut cs = ClusterSwitch::new(s, theta).unwrap();
        cs.set_s(0, 2u32);
        let new_mu = cs.theta().get(&2u32).unwrap().mu();

        assert_eq!(2.0, new_mu);
    }
}
