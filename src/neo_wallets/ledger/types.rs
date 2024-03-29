#![allow(clippy::upper_case_acronyms)]

use std::fmt;

use thiserror::Error;

#[derive(Clone, Debug)]
/// Ledger wallet type
pub enum DerivationType {
	/// Ledger Live-generated HD path
	LedgerLive(usize),
	/// Legacy generated HD Path
	Legacy(usize),
	/// Any other path
	Other(String),
}

impl fmt::Display for DerivationType {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
		write!(
			f,
			"{}",
			match self {
				DerivationType::Legacy(index) => format!("m/44'/60'/0'/{index}"),
				DerivationType::LedgerLive(index) => format!("m/44'/60'/{index}'/0/0"),
				DerivationType::Other(inner) => inner.to_owned(),
			}
		)
	}
}

#[derive(Error, Debug)]
/// Error when using the Ledger transport
pub enum LedgerError {
	/// Underlying ledger transport error
	#[error(transparent)]
	LedgerError(#[from] coins_ledger::errors::LedgerError),
	/// Device response was unexpectedly none
	#[error("Received unexpected response from device. Expected data in response, found none.")]
	UnexpectedNullResponse,
	#[error(transparent)]
	/// Error when converting from a hex string
	HexError(#[from] hex::FromHexError),
	#[error(transparent)]
	/// Error when converting a semver requirement
	SemVerError(#[from] semver::Error),
	/// Error when signing EIP712 struct with not compatible Ledger ETH app
	#[error("Ledger neo app requires at least version: {0:?}")]
	UnsupportedAppVersion(String),
	/// Got a response, but it didn't contain as much data as expected
	#[error("Cannot deserialize ledger response, insufficient bytes. Got {got} expected at least {at_least}")]
	ShortResponse { got: usize, at_least: usize },
	/// Payload is empty
	#[error("Payload must not be empty")]
	EmptyPayload,
}

pub const P1_FIRST: u8 = 0x00;

#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[allow(non_camel_case_types)]
pub enum INS {
	GET_PUBLIC_KEY = 0x02,
	SIGN = 0x04,
	GET_APP_CONFIGURATION = 0x06,
	SIGN_PERSONAL_MESSAGE = 0x08,
}

impl std::fmt::Display for INS {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			INS::GET_PUBLIC_KEY => write!(f, "GET_PUBLIC_KEY"),
			INS::SIGN => write!(f, "SIGN"),
			INS::GET_APP_CONFIGURATION => write!(f, "GET_APP_CONFIGURATION"),
			INS::SIGN_PERSONAL_MESSAGE => write!(f, "SIGN_PERSONAL_MESSAGE"),
		}
	}
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[allow(non_camel_case_types)]
pub enum P1 {
	NON_CONFIRM = 0x00,
	MORE = 0x80,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[allow(non_camel_case_types)]
pub enum P2 {
	NO_CHAINCODE = 0x00,
}
