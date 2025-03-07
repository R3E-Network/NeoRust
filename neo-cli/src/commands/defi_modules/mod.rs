// Modular DeFi command system for the Neo CLI
//
// This module provides a more structured approach to handling DeFi operations
// on the Neo N3 and Neo X blockchains.
//
// It separates functionality into dedicated modules for better organization,
// maintainability, and easier addition of new features.

use crate::{
    cli::CliState,
    error::CliError,
};

// Re-export all the module components
pub mod tokens;
pub mod tokens_integration;
pub mod types;
pub mod types_neofs;
pub mod utils;
pub mod bridge;
pub mod neo_fs;
pub mod neo_fs_integration;
pub mod integration_example;
pub mod final_validation;

// These modules will be implemented in the future
// pub mod swaps;
// pub mod liquidity;
// pub mod staking;

/// DefiModuleHandler handles all DeFi-related commands in the new modular structure
pub struct DefiModuleHandler;

impl DefiModuleHandler {
    /// Main entry point for handling DeFi commands
    ///
    /// # Arguments
    /// * `cmd` - The DeFi command to execute
    /// * `args` - Command arguments
    /// * `state` - Current CLI state
    ///
    /// # Returns
    /// * `Result<(), CliError>` - Success or error
    pub async fn handle_command(
        cmd: &str, 
        args: &[String], 
        state: &mut CliState
    ) -> Result<(), CliError> {
        match cmd {
            // Token operations
            "token" | "t" => {
                if args.is_empty() {
                    return Err(CliError::MissingArgument("token subcommand".to_string()));
                }
                let subcmd = &args[0];
                let subcmd_args = if args.len() > 1 { &args[1..] } else { &[] };
                
                tokens_integration::handle_token_command(subcmd, subcmd_args, state).await
            },
            
            // Bridge operations between Neo N3 and Neo X
            "bridge" | "b" => {
                if args.is_empty() {
                    return Err(CliError::MissingArgument("bridge subcommand".to_string()));
                }
                let subcmd = &args[0];
                let subcmd_args = if args.len() > 1 { &args[1..] } else { &[] };
                
                bridge::handle_bridge_command(subcmd, subcmd_args, state).await
            },
            
            // Future modules will be added here
            // "swap" | "s" => {...}
            // "liquidity" | "lp" => {...}
            // "stake" => {...}
            
            _ => Err(CliError::InvalidCommand(format!("Unknown DeFi command: {}", cmd))),
        }
    }
    
    /// Helper function to prepare a state from an existing CLI state
    /// This is useful for transitioning from the old to new implementation
    pub fn prepare_state_from_existing(state: &CliState) -> CliState {
        CliState {
            network_type: state.network_type.clone(),
            network_magic: state.network_magic,
            rpc_client: state.rpc_client.clone(),
            rpc_url: state.rpc_url.clone(),
            wallet_path: state.wallet_path.clone(),
            current_account: state.current_account.clone(),
            wallet: state.wallet.clone(),
            offline_mode: state.offline_mode,
            verbose: state.verbose,
            ..Default::default()
        }
    }
}
