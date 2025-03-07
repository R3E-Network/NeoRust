use std::hash::{Hash, Hasher};

use neo_crypto::{PublicKeyExtension, Secp256r1PublicKey};
use serde::{Deserialize, Serialize};

// Custom serialization for Secp256r1PublicKey
mod secp256r1_serde {
    use neo_crypto::Secp256r1PublicKey;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize_public_key_option<S>(
        public_key_opt: &Option<Secp256r1PublicKey>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match public_key_opt {
            Some(public_key) => {
                let encoded = public_key.get_encoded(true);
                let hex_string = hex::encode(encoded);
                serializer.serialize_str(&hex_string)
            }
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize_public_key_option<'de, D>(
        deserializer: D,
    ) -> Result<Option<Secp256r1PublicKey>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt = Option::<String>::deserialize(deserializer)?;
        match opt {
            Some(s) => {
                let bytes = hex::decode(s).map_err(serde::de::Error::custom)?;
                let public_key = Secp256r1PublicKey::from_bytes(&bytes)
                    .map_err(|e| serde::de::Error::custom(format!("Invalid public key: {:?}", e)))?;
                Ok(Some(public_key))
            }
            None => Ok(None),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct AccountState {
	pub balance: i64,
	pub balance_height: Option<i64>,
	#[serde(deserialize_with = "secp256r1_serde::deserialize_public_key_option")]
	#[serde(serialize_with = "secp256r1_serde::serialize_public_key_option")]
	pub public_key: Option<Secp256r1PublicKey>,
}

impl Hash for AccountState {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.balance.hash(state);
		self.balance_height.hash(state);

		// Only hash the public key if it exists
		if let Some(public_key) = &self.public_key {
			public_key.to_vec().hash(state);
		}
	}
}

impl AccountState {
	pub fn with_no_vote(balance: i64, update_height: i64) -> Self {
		Self { balance, balance_height: Some(update_height), public_key: None }
	}

	pub fn with_no_balance() -> Self {
		Self { balance: 0, balance_height: None, public_key: None }
	}
}
