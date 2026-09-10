//! Nonce Tracker Module
//!
//! Caller-owned cache for application challenge reuse detection. This type is
//! not wired into quote verification. Callers must place a fresh challenge in
//! REPORTDATA, verify that binding through a complete DCAP QVL flow, and only
//! then mark the challenge as used.

use std::time::SystemTime;

/// Simple in-memory application-challenge tracker with TTL.
///
/// This cache alone does not prove freshness or prevent replay. It is not
/// durable and is not shared between processes.
pub struct NonceTracker {
	/// Set of used nonces (with timestamps)
	seen_nonces: Vec<([u8; 16], u64)>,
	/// TTL for nonce entries (seconds before considered stale)
	ttl_seconds: u64,
}

impl Default for NonceTracker {
	fn default() -> Self {
		Self {
			seen_nonces: Vec::new(),
			ttl_seconds: 300, // 5 minutes default
		}
	}
}

impl NonceTracker {
	/// Create a new nonce tracker
	pub fn new() -> Self {
		Self::default()
	}

	/// Set TTL in seconds
	pub fn set_ttl(&mut self, ttl_seconds: u64) {
		self.ttl_seconds = ttl_seconds;
	}

	/// Check if a nonce has been seen recently (not expired)
	pub fn is_seen_recently(&self, nonce: &[u8; 16]) -> bool {
		let now = SystemTime::now()
			.duration_since(SystemTime::UNIX_EPOCH)
			.unwrap_or_default()
			.as_secs();

		for (seen_nonce, timestamp) in &self.seen_nonces {
			if timestamp.saturating_add(self.ttl_seconds) < now {
				// Entry expired, don't count it
				continue;
			}
			if seen_nonce == nonce {
				return true;
			}
		}
		false
	}

	/// Mark a nonce as used
	pub fn mark_used(&mut self, nonce: [u8; 16]) {
		let timestamp = SystemTime::now()
			.duration_since(SystemTime::UNIX_EPOCH)
			.unwrap_or_default()
			.as_secs();

		self.seen_nonces.push((nonce, timestamp));
	}

	/// Clean up expired entries
	pub fn cleanup_expired(&mut self) {
		let now = SystemTime::now()
			.duration_since(SystemTime::UNIX_EPOCH)
			.unwrap_or_default()
			.as_secs();

		self.seen_nonces
			.retain(|(_, timestamp)| timestamp.saturating_add(self.ttl_seconds) >= now);
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn maximum_ttl_does_not_overflow() {
		let nonce = [0xA5; 16];
		let mut tracker = NonceTracker::new();
		tracker.set_ttl(u64::MAX);
		tracker.seen_nonces.push((nonce, u64::MAX));

		assert!(tracker.is_seen_recently(&nonce));
		tracker.cleanup_expired();
		assert_eq!(tracker.seen_nonces.len(), 1);
	}
}
