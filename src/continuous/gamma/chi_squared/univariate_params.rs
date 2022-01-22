use crate::{ChiSquaredError, DistributionError, RandomVariable};

#[derive(Clone, Debug, PartialEq)]
pub struct ChiSquaredParams {
    k: f64,
}

impl ChiSquaredParams {
    pub fn new(k: f64) -> Result<Self, DistributionError> {
        if k <= 0.0 {
            return Err(DistributionError::InvalidParameters(
                ChiSquaredError::KMustBePositive.into(),
            ));
        }

        Ok(Self { k })
    }

    pub fn k(&self) -> f64 {
        self.k
    }
}

impl RandomVariable for ChiSquaredParams {
    type RestoreInfo = ();

    fn transform_vec(self) -> (Vec<f64>, Self::RestoreInfo) {
        todo!()
    }

    fn restore(v: Vec<f64>, info: Self::RestoreInfo) -> Self {
        todo!()
    }
}
