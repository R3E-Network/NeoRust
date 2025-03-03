// DeFi command argument types
//
// This module defines command-line argument structures for DeFi commands

use clap::{Args, Subcommand};
use std::path::PathBuf;

/// Arguments for token operations
#[derive(Args, Debug, Clone)]
pub struct TokenArgs {
    /// Path to wallet file
    #[arg(short, long)]
    pub wallet: Option<PathBuf>,
    
    /// Wallet password
    #[arg(short, long)]
    pub password: Option<String>,
    
    #[clap(subcommand)]
    pub command: TokenCommands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum TokenCommands {
    /// Get token information
    Info {
        /// Token contract address or symbol
        contract: String,
    },
    
    /// Check token balance for an address
    Balance {
        /// Token contract address or symbol
        contract: String,
        
        /// Optional address to check balance for (defaults to wallet's address)
        address: Option<String>,
    },
    
    /// Transfer tokens to an address
    Transfer {
        /// Token contract address or symbol
        token: String,
        
        /// Destination address
        to: String,
        
        /// Amount to transfer
        amount: String,
        
        /// Optional data to include with the transfer
        data: Option<String>,
    },
}

/// Arguments for swap operations
#[derive(Args, Debug, Clone)]
pub struct SwapArgs {
    /// Path to wallet file
    #[arg(short, long)]
    pub wallet: Option<PathBuf>,
    
    /// Wallet password
    #[arg(short, long)]
    pub password: Option<String>,
    
    #[clap(subcommand)]
    pub command: SwapCommands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum SwapCommands {
    /// Swap tokens on a decentralized exchange
    Swap {
        /// Source token (address, symbol, or hash)
        #[arg(long)]
        from_token: String,
        
        /// Destination token (address, symbol, or hash)
        #[arg(long)]
        to_token: String,
        
        /// Amount to swap
        #[arg(long)]
        amount: String,
        
        /// Maximum slippage in percentage (default: 1.0)
        #[arg(long, default_value = "1.0")]
        slippage: f64,
    },
    
    /// Get swap quote without executing the swap
    Quote {
        /// Source token (address, symbol, or hash)
        #[arg(long)]
        from_token: String,
        
        /// Destination token (address, symbol, or hash)
        #[arg(long)]
        to_token: String,
        
        /// Amount to swap
        #[arg(long)]
        amount: String,
    },
}

/// Arguments for liquidity operations
#[derive(Args, Debug, Clone)]
pub struct LiquidityArgs {
    /// Path to wallet file
    #[arg(short, long)]
    pub wallet: Option<PathBuf>,
    
    /// Wallet password
    #[arg(short, long)]
    pub password: Option<String>,
    
    #[clap(subcommand)]
    pub command: LiquidityCommands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum LiquidityCommands {
    /// Add liquidity to a pool
    Add {
        /// First token (address, symbol, or hash)
        #[arg(long)]
        token_a: String,
        
        /// Second token (address, symbol, or hash)
        #[arg(long)]
        token_b: String,
        
        /// Amount of first token to add
        #[arg(long)]
        amount_a: String,
        
        /// Amount of second token to add
        #[arg(long)]
        amount_b: String,
    },
    
    /// Remove liquidity from a pool
    Remove {
        /// First token (address, symbol, or hash)
        #[arg(long)]
        token_a: String,
        
        /// Second token (address, symbol, or hash)
        #[arg(long)]
        token_b: String,
        
        /// Percentage of liquidity to remove (1-100)
        #[arg(long)]
        percentage: f64,
    },
    
    /// List liquidity positions
    List,
}

/// Arguments for staking operations
#[derive(Args, Debug, Clone)]
pub struct StakingArgs {
    /// Path to wallet file
    #[arg(short, long)]
    pub wallet: Option<PathBuf>,
    
    /// Wallet password
    #[arg(short, long)]
    pub password: Option<String>,
    
    #[clap(subcommand)]
    pub command: StakingCommands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum StakingCommands {
    /// Stake tokens
    Stake {
        /// Token to stake (address, symbol, or hash)
        token: String,
        
        /// Amount to stake
        amount: String,
        
        /// Staking period in days (if applicable)
        #[arg(long)]
        period: Option<u32>,
    },
    
    /// Unstake tokens
    Unstake {
        /// Token to unstake (address, symbol, or hash)
        token: String,
        
        /// Amount to unstake
        amount: String,
    },
    
    /// Claim staking rewards
    Claim {
        /// Token to claim rewards for (address, symbol, or hash)
        token: String,
    },
    
    /// View staking positions
    Info {
        /// Token to get staking info for (address, symbol, or hash)
        token: Option<String>,
    },
}
