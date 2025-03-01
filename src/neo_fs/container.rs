use serde::{Deserialize, Serialize};
use crate::neo_fs::{
    types::{ContainerId, AccessRule, StoragePolicy},
    errors::NeoFsResult,
};

/// Represents a NeoFS container
#[derive(Debug, Clone)]
pub struct Container {
    /// Container ID
    pub id: ContainerId,
    /// Basic ACL bitmask
    pub basic_acl: u32,
    /// Owner's public key
    pub owner: Vec<u8>,
    /// Container attributes
    pub attributes: Vec<Attribute>,
    /// Placement policy
    pub placement_policy: Option<StoragePolicy>,
}

/// Container attribute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attribute {
    /// Attribute key
    pub key: String,
    /// Attribute value
    pub value: String,
}

impl Container {
    /// Create a new container with the given ID
    pub fn new(id: ContainerId) -> Self {
        Self {
            id,
            basic_acl: 0,
            owner: Vec::new(),
            attributes: Vec::new(),
            placement_policy: None,
        }
    }
    
    /// Set a container attribute
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
    
    /// Get a container attribute by key
    pub fn get_attribute(&self, key: &str) -> Option<&str> {
        self.attributes.iter()
            .find(|attr| attr.key == key)
            .map(|attr| attr.value.as_str())
    }
    
    /// Set the container's basic ACL
    pub fn set_basic_acl(&mut self, acl: u32) {
        self.basic_acl = acl;
    }
    
    /// Check if a specific ACL flag is set
    pub fn has_acl_flag(&self, flag: u32) -> bool {
        (self.basic_acl & flag) == flag
    }
    
    /// Set the container's placement policy
    pub fn set_placement_policy(&mut self, policy: StoragePolicy) {
        self.placement_policy = Some(policy);
    }
}

/// Builder for creating NeoFS containers
pub struct ContainerBuilder {
    attributes: Vec<Attribute>,
    basic_acl: u32,
    placement_policy: Option<StoragePolicy>,
    access_rules: Vec<AccessRule>,
}

impl ContainerBuilder {
    /// Create a new container builder
    pub fn new() -> Self {
        Self {
            attributes: Vec::new(),
            basic_acl: 0,
            placement_policy: None,
            access_rules: Vec::new(),
        }
    }
    
    /// Add an attribute to the container
    pub fn attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.push(Attribute {
            key: key.into(),
            value: value.into(),
        });
        self
    }
    
    /// Set the basic ACL value
    pub fn basic_acl(mut self, acl: u32) -> Self {
        self.basic_acl = acl;
        self
    }
    
    /// Set the placement policy
    pub fn placement_policy(mut self, policy: StoragePolicy) -> Self {
        self.placement_policy = Some(policy);
        self
    }
    
    /// Add an access rule
    pub fn access_rule(mut self, rule: AccessRule) -> Self {
        self.access_rules.push(rule);
        self
    }
    
    /// Get the attributes
    pub fn get_attributes(&self) -> &[Attribute] {
        &self.attributes
    }
    
    /// Get the basic ACL
    pub fn get_basic_acl(&self) -> u32 {
        self.basic_acl
    }
    
    /// Get the placement policy
    pub fn get_placement_policy(&self) -> Option<&StoragePolicy> {
        self.placement_policy.as_ref()
    }
    
    /// Get the access rules
    pub fn get_access_rules(&self) -> &[AccessRule] {
        &self.access_rules
    }
}

impl Default for ContainerBuilder {
    fn default() -> Self {
        Self::new()
    }
} 