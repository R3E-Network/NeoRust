use serde::{Deserialize, Serialize};
use std::ops::{Add, AddAssign};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Debug, Default)]
pub struct UnclaimedGas {
	pub unclaimed: String,
	pub address: String,
}

impl Add for UnclaimedGas {
	type Output = Self;

	fn add(self, other: Self) -> Self {
		// Parse the unclaimed values as f64 for addition
		let self_unclaimed = self.unclaimed.parse::<f64>().unwrap_or(0.0);
		let other_unclaimed = other.unclaimed.parse::<f64>().unwrap_or(0.0);

		UnclaimedGas {
			unclaimed: (self_unclaimed + other_unclaimed).to_string(),
			address: self.address, // Keep the original address
		}
	}
}

impl AddAssign for UnclaimedGas {
	fn add_assign(&mut self, other: Self) {
		*self = self.clone() + other;
	}
}
