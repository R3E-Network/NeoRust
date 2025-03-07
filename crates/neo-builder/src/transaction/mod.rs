pub use call_flags::*;
pub use contract_parameters_context::*;
pub use invocation_script::*;
pub use oracle_response_code::*;
pub use signers::*;
pub use transaction::*;
pub use transaction_attribute::*;
pub use transaction_builder::*;
pub use transaction_error::*;
pub use transaction_send_token::*;
pub use verification_script::*;
pub use witness::*;
pub use witness_rule::*;
pub use witness_scope::*;

pub mod call_flags;
mod contract_parameters_context;
mod invocation_script;
mod oracle_response_code;
pub mod signers;
mod transaction;
pub mod transaction_attribute;
mod transaction_builder;
mod transaction_builder_tests;
mod transaction_error;
pub mod transaction_send_token;
mod verification_script;
mod witness;
pub mod witness_rule;
mod witness_scope;

use std::sync::Once;
use tracing_subscriber;

static INIT: Once = Once::new();

pub fn init_logger() {
	INIT.call_once(|| {
		tracing_subscriber::fmt().with_max_level(tracing::Level::TRACE).init();
	});
}
