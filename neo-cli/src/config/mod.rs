use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use anyhow::{Result, Context};
use std::fs;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CliConfig {
    pub network: NetworkConfig,
    pub wallet: WalletConfig,
    pub storage: StorageConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkConfig {
    pub rpc_url: String,
    pub network_magic: u32,
    pub address_version: u8,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WalletConfig {
    pub default_path: Option<String>,
    pub auto_unlock: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StorageConfig {
    pub path: PathBuf,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoggingConfig {
    pub level: String,
    pub file: Option<PathBuf>,
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            network: NetworkConfig {
                rpc_url: "http://localhost:10332".to_string(),
                network_magic: 860833102, // Mainnet
                address_version: 53,
            },
            wallet: WalletConfig {
                default_path: None,
                auto_unlock: false,
            },
            storage: StorageConfig {
                path: dirs::data_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join("neo-cli"),
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file: None,
            },
        }
    }
}

impl CliConfig {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;
        
        if !config_path.exists() {
            let default_config = Self::default();
            fs::create_dir_all(config_path.parent().unwrap())?;
            let config_str = serde_json::to_string_pretty(&default_config)?;
            fs::write(&config_path, config_str)?;
            return Ok(default_config);
        }
        
        let config_str = fs::read_to_string(&config_path)
            .context(format!("Failed to read config file: {:?}", config_path))?;
        let config: CliConfig = serde_json::from_str(&config_str)
            .context("Failed to parse config file")?;
        
        Ok(config)
    }
    
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        fs::create_dir_all(config_path.parent().unwrap())?;
        let config_str = serde_json::to_string_pretty(self)?;
        fs::write(&config_path, config_str)?;
        Ok(())
    }
    
    fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("neo-cli");
        
        fs::create_dir_all(&config_dir)?;
        
        Ok(config_dir.join("config.json"))
    }
}
