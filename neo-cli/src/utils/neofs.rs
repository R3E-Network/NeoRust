use crate::errors::CliError;
use std::path::Path;

/// Validates a container ID format
/// Returns Ok if valid, Error with message otherwise
pub fn validate_container_id(container_id: &str) -> Result<(), CliError> {
	// In reality, this would implement proper validation logic
	// For now, just checking minimum length
	if container_id.len() < 8 {
		return Err(CliError::InvalidInput(
			"Container ID must be at least 8 characters".to_string(),
		));
	}
	Ok(())
}

/// Validates a file path exists
pub fn validate_file_path(path: &Path) -> Result<(), CliError> {
	if !path.exists() {
		return Err(CliError::FileSystem(format!("File not found: {}", path.display())));
	}
	if !path.is_file() {
		return Err(CliError::FileSystem(format!("Path is not a file: {}", path.display())));
	}
	Ok(())
}

/// Validates a directory path exists
pub fn validate_directory_path(path: &Path) -> Result<(), CliError> {
	if !path.exists() {
		return Err(CliError::FileSystem(format!("Directory not found: {}", path.display())));
	}
	if !path.is_dir() {
		return Err(CliError::FileSystem(format!("Path is not a directory: {}", path.display())));
	}
	Ok(())
}

/// Formats file size in human-readable format
pub fn format_size(size: u64) -> String {
	const KB: u64 = 1024;
	const MB: u64 = KB * 1024;
	const GB: u64 = MB * 1024;

	if size < KB {
		format!("{} B", size)
	} else if size < MB {
		format!("{:.2} KB", size as f64 / KB as f64)
	} else if size < GB {
		format!("{:.2} MB", size as f64 / MB as f64)
	} else {
		format!("{:.2} GB", size as f64 / GB as f64)
	}
}

/// Validates an endpoint URL
pub fn validate_endpoint(endpoint: &str) -> Result<(), CliError> {
	// Basic validation for now
	if !endpoint.starts_with("http://") && !endpoint.starts_with("https://") {
		return Err(CliError::InvalidInput(
			"Endpoint must start with http:// or https://".to_string(),
		));
	}
	Ok(())
}

/// Extracts storage node info from an endpoint
pub fn get_node_info(endpoint: &str) -> Result<String, CliError> {
	// This would actually connect to the node and get info
	// For now, just returning a placeholder
	Ok(format!("Node: {}", endpoint))
}

/// Checks if an endpoint is available
pub fn check_endpoint_availability(endpoint: &str) -> Result<bool, CliError> {
	// This would actually ping the endpoint to check availability
	// For now, just returning true for demonstration
	validate_endpoint(endpoint)?;
	Ok(true)
}

/// Formats container/object permissions
pub fn format_permissions(is_public_read: bool, is_public_write: bool) -> String {
	match (is_public_read, is_public_write) {
		(true, true) => "Public read/write".to_string(),
		(true, false) => "Public read only".to_string(),
		(false, true) => "Public write only".to_string(),
		(false, false) => "Private".to_string(),
	}
}
