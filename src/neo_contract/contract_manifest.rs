use serde::{Serialize, Deserialize};

/// Represents a Neo contract manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractManifest {
    /// The name of the contract.
    pub name: String,
    /// The groups associated with the contract.
    pub groups: Vec<ContractGroup>,
    /// The features of the contract.
    pub features: serde_json::Value,
    /// The standards supported by the contract.
    pub supported_standards: Vec<String>,
    /// The ABI of the contract.
    pub abi: ContractABI,
    /// The permissions of the contract.
    pub permissions: Vec<ContractPermission>,
    /// The contracts trusted by this contract.
    pub trusts: Vec<String>,
    /// Extra data associated with the contract.
    pub extra: Option<serde_json::Value>,
}

/// Represents a group in a contract manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractGroup {
    /// The public key of the group.
    pub pubkey: String,
    /// The signature of the group.
    pub signature: String,
}

/// Represents the ABI of a contract.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractABI {
    /// The methods exposed by the contract.
    pub methods: Vec<ContractMethod>,
    /// The events exposed by the contract.
    pub events: Vec<ContractEvent>,
}

/// Represents a method in a contract ABI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractMethod {
    /// The name of the method.
    pub name: String,
    /// The parameters of the method.
    pub parameters: Vec<ContractParameter>,
    /// The return type of the method.
    pub return_type: String,
    /// Whether the method is safe to call.
    pub safe: bool,
}

/// Represents a parameter in a contract method.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractParameter {
    /// The name of the parameter.
    pub name: String,
    /// The type of the parameter.
    #[serde(rename = "type")]
    pub parameter_type: String,
}

/// Represents an event in a contract ABI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractEvent {
    /// The name of the event.
    pub name: String,
    /// The parameters of the event.
    pub parameters: Vec<ContractParameter>,
}

/// Represents a permission in a contract manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractPermission {
    /// The contract hash.
    pub contract: String,
    /// The methods allowed.
    pub methods: Vec<String>,
}

impl ContractManifest {
    /// Creates a new contract manifest.
    pub fn new(
        name: String,
        groups: Vec<ContractGroup>,
        features: serde_json::Value,
        supported_standards: Vec<String>,
        abi: ContractABI,
        permissions: Vec<ContractPermission>,
        trusts: Vec<String>,
        extra: Option<serde_json::Value>,
    ) -> Self {
        Self {
            name,
            groups,
            features,
            supported_standards,
            abi,
            permissions,
            trusts,
            extra,
        }
    }
} 