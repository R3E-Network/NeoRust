use crate::neo_fs::{
	errors::NeoFsResult,
	types::{ContainerId, ObjectId},
};
use serde::{Deserialize, Serialize};

/// Represents a NeoFS object
#[derive(Debug, Clone)]
pub struct Object {
	/// Object ID
	pub id: ObjectId,
	/// Container ID
	pub container_id: ContainerId,
	/// Object data
	pub data: Vec<u8>,
	/// Object attributes
	pub attributes: Vec<Attribute>,
}

/// Object attribute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attribute {
	/// Attribute key
	pub key: String,
	/// Attribute value
	pub value: String,
}

impl Object {
	/// Create a new object with the given ID and container ID
	pub fn new(id: ObjectId, container_id: ContainerId) -> Self {
		Self { id, container_id, data: Vec::new(), attributes: Vec::new() }
	}

	/// Create a new object with data
	pub fn with_data(id: ObjectId, container_id: ContainerId, data: Vec<u8>) -> Self {
		Self { id, container_id, data, attributes: Vec::new() }
	}

	/// Set object data
	pub fn set_data(&mut self, data: Vec<u8>) {
		self.data = data;
	}

	/// Add or update an attribute
	pub fn set_attribute(&mut self, key: impl Into<String>, value: impl Into<String>) {
		let key = key.into();
		let value = value.into();

		// Check if attribute already exists
		for attr in &mut self.attributes {
			if attr.key == key {
				attr.value = value;
				return;
			}
		}

		// Add new attribute
		self.attributes.push(Attribute { key, value });
	}

	/// Get an attribute by key
	pub fn get_attribute(&self, key: &str) -> Option<&str> {
		self.attributes
			.iter()
			.find(|attr| attr.key == key)
			.map(|attr| attr.value.as_str())
	}

	/// Get the size of the object in bytes
	pub fn size(&self) -> usize {
		self.data.len()
	}
}

/// Builder for creating NeoFS objects
pub struct ObjectBuilder {
	container_id: Option<ContainerId>,
	data: Vec<u8>,
	attributes: Vec<Attribute>,
}

impl ObjectBuilder {
	/// Create a new object builder
	pub fn new() -> Self {
		Self { container_id: None, data: Vec::new(), attributes: Vec::new() }
	}

	/// Set the container ID
	pub fn container_id(mut self, container_id: ContainerId) -> Self {
		self.container_id = Some(container_id);
		self
	}

	/// Set the object data
	pub fn data(mut self, data: impl Into<Vec<u8>>) -> Self {
		self.data = data.into();
		self
	}

	/// Add an attribute
	pub fn attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
		self.attributes.push(Attribute { key: key.into(), value: value.into() });
		self
	}

	/// Get the container ID
	pub fn get_container_id(&self) -> Option<&ContainerId> {
		self.container_id.as_ref()
	}

	/// Get the object data
	pub fn get_data(&self) -> &[u8] {
		&self.data
	}

	/// Get the attributes
	pub fn get_attributes(&self) -> &[Attribute] {
		&self.attributes
	}
}

impl Default for ObjectBuilder {
	fn default() -> Self {
		Self::new()
	}
}

/// Options for downloading objects from NeoFS
#[derive(Debug, Clone)]
pub struct DownloadOptions {
	/// Range start position (optional)
	pub range_start: Option<u64>,
	/// Range end position (optional)
	pub range_end: Option<u64>,
	/// Whether to include object attributes
	pub include_attributes: bool,
	/// Raw mode (no transformation of data)
	pub raw: bool,
}

impl Default for DownloadOptions {
	fn default() -> Self {
		Self { range_start: None, range_end: None, include_attributes: true, raw: false }
	}
}

/// Builder for download options
pub struct DownloadOptionsBuilder {
	options: DownloadOptions,
}

impl DownloadOptionsBuilder {
	/// Create a new download options builder
	pub fn new() -> Self {
		Self { options: DownloadOptions::default() }
	}

	/// Set the range start position
	pub fn range_start(mut self, position: u64) -> Self {
		self.options.range_start = Some(position);
		self
	}

	/// Set the range end position
	pub fn range_end(mut self, position: u64) -> Self {
		self.options.range_end = Some(position);
		self
	}

	/// Set whether to include attributes
	pub fn include_attributes(mut self, include: bool) -> Self {
		self.options.include_attributes = include;
		self
	}

	/// Set raw mode
	pub fn raw(mut self, raw: bool) -> Self {
		self.options.raw = raw;
		self
	}

	/// Build the download options
	pub fn build(self) -> DownloadOptions {
		self.options
	}
}

impl Default for DownloadOptionsBuilder {
	fn default() -> Self {
		Self::new()
	}
}
