use crypto::digest::Digest;
use crypto::sha2::Sha256;
use primitive_types::H256;

fn hash256(data: &[u8]) -> Vec<u8> {
	let mut hasher = Sha256::new();
	hasher.input(data);
	let mut res = vec![0u8; 32];
	hasher.result(&mut res);
	res
}

fn main() {
	let message = b"test message";
	let hash = hash256(message);
	let h256 = H256::from_slice(&hash);
	println!("Hash: {:?}", h256);
}
