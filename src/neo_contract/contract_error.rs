use neo::prelude::ProviderError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
	#[error("Invalid NNS name {0}")]
	InvalidNeoName(String),
	#[error("Invalid NNS root {0}")]
	InvalidNeoNameServiceRoot(String),
	#[error("Unexpected return type {0}")]
	UnexpectedReturnType(String),
	#[error("Unresolvable domain name {0}")]
	UnresolvableDomainName(String),
	#[error("Domain name {0} is not available")]
	DomainNameNotAvailable(String),
	#[error("Domain name {0} is not registered")]
	DomainNameNotRegistered(String),
	#[error("Runtime error: {0}")]
	RuntimeError(String),
	#[error("Invalid state error: {0}")]
	InvalidStateError(String),
	#[error("Invalid argument error: {0}")]
	InvalidArgError(String),
	#[error(transparent)]
	ProviderError(#[from] ProviderError),
}
