// SGX EDL Module
//
// This module contains Enclave Definition Language (EDL) related functionality
// for SGX enclaves.

// Re-export the EDL file for use in the build process
pub const NEORUSTL_EDL: &str = include_str!("NeoRust.edl");

#[cfg(test)]
mod tests {
	#[test]
	fn it_works() {
		assert_eq!(2 + 2, 4);
	}
}
