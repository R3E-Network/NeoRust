# DeFi Commands for Neo CLI

This guide explains how to use the Neo CLI to interact with various DeFi platforms on the Neo N3 blockchain.

## Overview

The Neo blockchain ecosystem has several DeFi protocols that provide various financial services. The `neo-cli` tool provides direct access to these protocols through the `defi` command group, allowing you to:

1. Interact with famous DeFi contracts
2. Get token information
3. Check token balances
4. Transfer tokens
5. Swap tokens
6. Provide/remove liquidity
7. Stake tokens and claim rewards
8. And more

## General DeFi Commands

### Get Token Information

To see detailed information about a token:

```bash
neo-cli defi token NEO
neo-cli defi token GAS
neo-cli defi token 0xf970f4cddcd087ab5d8a5697a32b3cfd32c8b465  # Using script hash
```

### Check Token Balance

To check your balance of a specific token:

```bash
# Check balance for the currently loaded wallet
neo-cli defi balance NEO

# Check balance for a specific address
neo-cli defi balance GAS --address NZKvXidwBhnV8rNXh2eXtpm5bH1rkofaDz
```

### Transfer Tokens

To transfer tokens to another address:

```bash
# Transfer 10 GAS
neo-cli defi transfer GAS NZKvXidwBhnV8rNXh2eXtpm5bH1rkofaDz 10

# Transfer with additional data
neo-cli defi transfer NEO NZKvXidwBhnV8rNXh2eXtpm5bH1rkofaDz 100 --data "Payment for services"
```

## Flamingo Finance Commands

Flamingo Finance is a DeFi platform on Neo N3 offering trading, earning, and borrowing services.

### Swap Tokens

To swap one token for another:

```bash
# Swap 10 GAS for NEO with default slippage (10%)
neo-cli defi flamingo swap GAS NEO 10

# Swap with custom minimum return amount
neo-cli defi flamingo swap FLM GAS 100 --min-return 5
```

### Add Liquidity

To add liquidity to a trading pair:

```bash
neo-cli defi flamingo add-liquidity NEO GAS 10 5
```

### Remove Liquidity

To remove liquidity from a trading pair:

```bash
neo-cli defi flamingo remove-liquidity NEO GAS 10
```

### Stake Tokens

To stake tokens and earn rewards:

```bash
neo-cli defi flamingo stake FLM 100
```

### Claim Rewards

To claim rewards from staking:

```bash
neo-cli defi flamingo claim-rewards
```

## NeoBurger Commands

NeoBurger (bNEO) is a wrapped NEO token that allows users to earn GAS while using their NEO in DeFi.

### Wrap NEO to bNEO

To wrap your NEO to bNEO:

```bash
neo-cli defi neoburger wrap 100
```

### Unwrap bNEO to NEO

To unwrap your bNEO back to NEO:

```bash
neo-cli defi neoburger unwrap 100
```

### Claim GAS

To claim GAS rewards from your bNEO:

```bash
neo-cli defi neoburger claim-gas
```

### Get Exchange Rate

To check the current exchange rate between bNEO and NEO:

```bash
neo-cli defi neoburger get-rate
```

## NeoCompound Commands

NeoCompound is an automated interest compounding service for Neo ecosystem tokens.

### Deposit Tokens

To deposit tokens into NeoCompound:

```bash
neo-cli defi neocompound deposit GAS 50
```

### Withdraw Tokens

To withdraw tokens from NeoCompound:

```bash
neo-cli defi neocompound withdraw GAS 25
```

### Compound Interest

To manually compound your interest:

```bash
neo-cli defi neocompound compound GAS
```

### Get APY

To check the current APY for a token:

```bash
neo-cli defi neocompound get-apy GAS
```

## GrandShare Commands

GrandShare is a governance and funding platform for Neo ecosystem projects.

### Submit a Proposal

To submit a new proposal:

```bash
neo-cli defi grandshare submit-proposal "My Project Title" "Project description and details" 1000
```

### Vote on a Proposal

To vote on an existing proposal:

```bash
# Approve a proposal
neo-cli defi grandshare vote 123 --approve

# Reject a proposal
neo-cli defi grandshare vote 123
```

### Fund a Project

To fund an approved project:

```bash
neo-cli defi grandshare fund-project 456 500
```

### Claim Funds

To claim funds for your project (if you're the project owner):

```bash
neo-cli defi grandshare claim-funds 456
```

## Common Options

Most DeFi commands support the following options:

- `--wallet`: Path to your wallet file
- `--password`: Wallet password
- `--network`: Network to use (mainnet or testnet, defaults to mainnet)

Example:

```bash
neo-cli defi --wallet path/to/wallet.json --password mypassword flamingo swap NEO GAS 10
```

## Notes

- All amounts are specified in the token's natural units (e.g., 1 GAS = 1 GAS, not 1 * 10^8 GAS)
- Token symbols are case-insensitive
- You can also use script hashes instead of token symbols
- Make sure your wallet has enough GAS to pay for transaction fees 