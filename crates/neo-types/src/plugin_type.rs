use strum_macros::{Display, EnumString};

#[derive(Display, EnumString, Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum NodePluginType {
	#[strum(serialize = "ApplicationLogs")]
	ApplicationLogs,
	#[strum(serialize = "CoreMetrics")]
	CoreMetrics,
	#[strum(serialize = "ImportBlocks")]
	ImportBlocks,
	#[strum(serialize = "LevelDBStore")]
	LevelDbStore,
	#[strum(serialize = "RocksDBStore")]
	RocksDbStore,
	#[strum(serialize = "RpcNep17Tracker")]
	RpcNep17Tracker,
	#[strum(serialize = "RpcSecurity")]
	RpcSecurity,
	#[strum(serialize = "RpcServerPlugin")]
	RpcServerPlugin,
	#[strum(serialize = "RpcSystemAssetTrackerPlugin")]
	RpcSystemAssetTracker,
	#[strum(serialize = "SimplePolicyPlugin")]
	SimplePolicy,
	#[strum(serialize = "StatesDumper")]
	StatesDumper,
	#[strum(serialize = "SystemLog")]
	SystemLog,
}

impl NodePluginType {
	pub fn value_of_name(name: &str) -> Result<Self, &'static str> {
		match name.parse::<NodePluginType>() {
			Ok(plugin_type) => Ok(plugin_type),
			Err(_) => Err("Invalid plugin type"),
		}
	}
}
