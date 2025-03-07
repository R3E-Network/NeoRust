// Type definitions for DeFi operations
//
// This module defines the command line argument types and structures
// used throughout the DeFi modules.

use clap::{Args, Subcommand};
use std::fmt;

/// Arguments for the Token command
#[derive(Args)]
pub struct TokenArgs {
    /// Token subcommand to execute
    #[clap(subcommand)]
    pub command: TokenCommands,
}

/// Arguments for the Bridge command
#[derive(Args)]
pub struct BridgeArgs {
    /// Bridge subcommand to execute
    #[clap(subcommand)]
    pub command: BridgeCommands,
}

/// Available Bridge subcommands
#[derive(Subcommand)]
pub enum BridgeCommands {
    /// Deposit tokens from Neo N3 to Neo X
    #[clap(name = "deposit", alias = "d")]
    Deposit {
        /// Token contract address or symbol
        #[clap(value_name = "TOKEN")]
        token: String,
        
        /// Recipient address on the target network
        #[clap(value_name = "RECIPIENT")]
        recipient: String,
        
        /// Amount to deposit
        #[clap(value_name = "AMOUNT")]
        amount: String,
    },
    
    /// Withdraw tokens from Neo X to Neo N3
    #[clap(name = "withdraw", alias = "w")]
    Withdraw {
        /// Token contract address or symbol
        #[clap(value_name = "TOKEN")]
        token: String,
        
        /// Recipient address on the target network
        #[clap(value_name = "RECIPIENT")]
        recipient: String,
        
        /// Amount to withdraw
        #[clap(value_name = "AMOUNT")]
        amount: String,
    },
    
    /// Display information about bridge contracts and supported tokens
    #[clap(name = "info", alias = "i")]
    Info,
}

/// Available Token subcommands
#[derive(Subcommand)]
pub enum TokenCommands {
    /// Get token information (symbol, decimals, etc.)
    #[clap(name = "info", alias = "i")]
    Info {
        /// Token contract address or symbol
        #[clap(value_name = "CONTRACT")]
        contract: String,
    },
    
    /// Get token balance for a specific address
    #[clap(name = "balance", alias = "b")]
    Balance {
        /// Token contract address or symbol
        #[clap(value_name = "CONTRACT")]
        contract: String,
        
        /// Target address to check balance for
        #[clap(value_name = "ADDRESS")]
        address: String,
    },
    
    /// Transfer tokens to another address
    #[clap(name = "transfer", alias = "t")]
    Transfer {
        /// Token contract address or symbol
        #[clap(value_name = "CONTRACT")]
        contract: String,
        
        /// Recipient address
        #[clap(value_name = "ADDRESS")]
        to_address: String,
        
        /// Amount to transfer
        #[clap(value_name = "AMOUNT")]
        amount: String,
    },
    
    /// List available tokens on the current network
    #[clap(name = "list")]
    List,
}

/// Base arguments for all DeFi operations
#[derive(Args)]
pub struct DefiArgs {
    /// DeFi subcommand to execute
    #[clap(subcommand)]
    pub command: DefiCommands,
}

/// Available DeFi subcommands
#[derive(Subcommand)]
pub enum DefiCommands {
    /// Token operations (info, balance, transfer)
    #[clap(name = "token", alias = "t")]
    Token(TokenArgs),
    
    /// Bridge operations between Neo N3 and Neo X
    #[clap(name = "bridge", alias = "b")]
    Bridge(BridgeArgs),
    
    // More commands will be added as implementations progress
    // #[clap(name = "swap", alias = "s")]
    // Swap(SwapArgs),
    
    // #[clap(name = "liquidity", alias = "lp")]
    // Liquidity(LiquidityArgs),
    
    // #[clap(name = "stake")]
    // Stake(StakeArgs),
}

/// Helper for converting enum variants to strings for display
impl fmt::Display for TokenCommands {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenCommands::Info { .. } => write!(f, "info"),
            TokenCommands::Balance { .. } => write!(f, "balance"),
            TokenCommands::Transfer { .. } => write!(f, "transfer"),
            TokenCommands::List => write!(f, "list"),
        }
    }
}

/// Helper for converting enum variants to strings for display
impl fmt::Display for DefiCommands {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DefiCommands::Token(_) => write!(f, "token"),
            DefiCommands::Bridge(_) => write!(f, "bridge"),
            // Add other commands as they are implemented
        }
    }
}

/// Helper for converting bridge command enum variants to strings for display
impl fmt::Display for BridgeCommands {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BridgeCommands::Deposit { .. } => write!(f, "deposit"),
            BridgeCommands::Withdraw { .. } => write!(f, "withdraw"),
            BridgeCommands::Info => write!(f, "info"),
        }
    }
}
