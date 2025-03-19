import React, { useState } from 'react';
import { Link } from 'gatsby';
import Layout from '../../components/Layout';
import CodeBlock from '../../components/CodeBlock';
import Search from '../../components/Search';

interface Example {
  id: string;
  title: string;
  description: string;
  category: string;
  code: string;
  difficulty: 'beginner' | 'intermediate' | 'advanced';
}

const examples: Example[] = [
  {
    id: 'basic-connection',
    title: 'Basic Connection',
    description: 'Connect to a Neo N3 node and get basic blockchain information',
    category: 'Getting Started',
    difficulty: 'beginner',
    code: `use neo3::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to Neo N3 TestNet
    let provider = HttpProvider::new("https://testnet1.neo.org:443")?;
    let client = RpcClient::new(provider);
    
    // Get basic blockchain information
    let block_count = client.get_block_count().await?;
    println!("Current block height: {}", block_count);
    
    let best_block_hash = client.get_best_block_hash().await?;
    println!("Best block hash: {}", best_block_hash);
    
    let version = client.get_version().await?;
    println!("Node version: {}", version.user_agent);
    
    Ok(())
}`
  },
  {
    id: 'create-wallet',
    title: 'Create Wallet',
    description: 'Create a new Neo wallet and get information about it',
    category: 'Wallets',
    difficulty: 'beginner',
    code: `use neo3::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new random key pair
    let key_pair = KeyPair::new_random()?;
    println!("Private key: {}", hex::encode(key_pair.private_key.to_bytes()));
    
    // Get the address from the key pair
    let address = key_pair.get_address();
    println!("Address: {}", address);
    
    // Get the script hash
    let script_hash = key_pair.get_script_hash();
    println!("Script hash: {}", script_hash);
    
    // Create a wallet signer
    let wallet = WalletSigner::new_with_signer(key_pair.clone(), address.clone());
    
    // Sign a message
    let message = b"Hello, Neo!";
    let signature = wallet.sign_message(message).await?;
    println!("Message signed successfully");
    
    Ok(())
}`
  },
  {
    id: 'token-info',
    title: 'Get Token Information',
    description: 'Retrieve information about Neo and GAS tokens',
    category: 'Tokens',
    difficulty: 'beginner',
    code: `use neo3::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to Neo N3 TestNet
    let provider = HttpProvider::new("https://testnet1.neo.org:443")?;
    let client = RpcClient::new(provider);
    
    // Get NEO token information
    let neo_token = NeoToken::new(&client);
    let neo_symbol = neo_token.symbol().await?;
    let neo_decimals = neo_token.decimals().await?;
    let neo_total_supply = neo_token.total_supply().await?;
    
    println!("NEO Token:");
    println!("  Symbol: {}", neo_symbol);
    println!("  Decimals: {}", neo_decimals);
    println!("  Total Supply: {}", neo_total_supply);
    
    // Get GAS token information
    let gas_token = GasToken::new(&client);
    let gas_symbol = gas_token.symbol().await?;
    let gas_decimals = gas_token.decimals().await?;
    let gas_total_supply = gas_token.total_supply().await?;
    
    println!("GAS Token:");
    println!("  Symbol: {}", gas_symbol);
    println!("  Decimals: {}", gas_decimals);
    println!("  Total Supply: {}", gas_total_supply);
    
    Ok(())
}`
  },
  {
    id: 'transfer-gas',
    title: 'Transfer GAS',
    description: 'Create a transaction to transfer GAS tokens to another address',
    category: 'Transactions',
    difficulty: 'intermediate',
    code: `use neo3::prelude::*;
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to Neo N3 TestNet
    let provider = HttpProvider::new("https://testnet1.neo.org:443")?;
    let client = RpcClient::new(provider);
    
    // Load your account (replace with your actual private key)
    let account = Account::from_wif("your-private-key-wif")?;
    println!("Sender address: {}", account.get_address());
    
    // Recipient address (replace with the actual recipient)
    let recipient = ScriptHash::from_address("NXxVXJJpwXADM65gJhYPQ5xHiZr17KQj5T")?;
    
    // Amount to transfer (0.1 GAS)
    let amount = 10_000_000; // 0.1 GAS (with 8 decimals)
    
    // Create GAS token contract instance
    let gas_token = GasToken::new(&client);
    
    // Check sender's balance
    let balance = gas_token.balance_of(&account.get_script_hash()).await?;
    println!("Your GAS balance: {}", balance);
    
    if balance < amount {
        println!("Insufficient balance");
        return Ok(());
    }
    
    // Transfer GAS
    let tx_hash = gas_token.transfer(
        &account,
        &recipient,
        amount,
        None // No data
    ).await?;
    
    println!("Transaction sent: {}", tx_hash);
    
    // Wait for confirmation
    println!("Waiting for confirmation...");
    
    Ok(())
}`
  },
  {
    id: 'deploy-contract',
    title: 'Deploy a Smart Contract',
    description: 'Deploy a smart contract to the Neo N3 blockchain',
    category: 'Smart Contracts',
    difficulty: 'advanced',
    code: `use neo3::prelude::*;
use neo3::neo_contract::ContractManagement;
use neo3::neo_types::{ContractManifest, NefFile};
use std::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to Neo N3 TestNet
    let provider = HttpProvider::new("https://testnet1.neo.org:443")?;
    let client = RpcClient::new(provider);
    
    // Load your account (replace with your actual private key)
    let account = Account::from_wif("your-private-key-wif")?;
    println!("Deployer address: {}", account.get_address());
    
    // Check GAS balance
    let gas_token = GasToken::new(&client);
    let balance = gas_token.balance_of(&account.get_script_hash()).await?;
    println!("Your GAS balance: {}", balance);
    
    // Load contract files
    let nef_bytes = fs::read("path/to/contract.nef")?;
    let manifest_json = fs::read_to_string("path/to/contract.manifest.json")?;
    
    // Parse contract files
    let nef = NefFile::from_bytes(&nef_bytes)?;
    let manifest = ContractManifest::from_json(&manifest_json)?;
    
    // Create contract management instance
    let contract_mgmt = ContractManagement::new(&client);
    
    // Deploy the contract
    println!("Deploying contract...");
    let result = contract_mgmt.deploy(
        &nef,
        &manifest,
        None, // No data
        &account
    ).await?;
    
    // Get the contract hash
    let contract_hash = result.script_hash;
    println!("Contract deployed successfully!");
    println!("Contract hash: {}", contract_hash);
    
    Ok(())
}`
  },
  {
    id: 'neo-x-bridge',
    title: 'Neo X Bridge',
    description: 'Transfer tokens between Neo N3 and Neo X chains',
    category: 'Neo X',
    difficulty: 'advanced',
    code: `use neo3::prelude::*;
use neo3::neo_x::{NeoXBridgeContract, BridgeToken};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to Neo N3
    let neo_provider = HttpProvider::new("https://mainnet1.neo.org:443")?;
    let neo_client = RpcClient::new(neo_provider);
    
    // Connect to Neo X
    let neo_x_provider = NeoXProvider::new("https://rpc.neo-x.org", Some(&neo_client));
    
    // Load your account (replace with your actual private key)
    let account = Account::from_wif("your-private-key-wif")?;
    println!("Account address: {}", account.get_address());
    
    // Create bridge contract instance
    let bridge = NeoXBridgeContract::new(neo_client.clone(), neo_x_provider.clone());
    
    // Neo X address to receive tokens (usually derived from Neo N3 address)
    let neo_x_address = "0x1234567890123456789012345678901234567890";
    
    // Amount to transfer (1 GAS)
    let amount = 1_00000000; // 1 GAS (with 8 decimals)
    
    // Bridge GAS to Neo X
    let tx_hash = bridge.bridge_to_neox(
        &account,
        BridgeToken::Gas,
        amount,
        neo_x_address
    ).await?;
    
    println!("Bridge transaction sent: {}", tx_hash);
    println!("This will take several minutes to process");
    
    Ok(())
}`
  }
];

const ExamplesPage: React.FC = () => {
  const [selectedCategory, setSelectedCategory] = useState<string | null>(null);
  const [selectedDifficulty, setSelectedDifficulty] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  
  // Get unique categories
  const categories = [...new Set(examples.map(example => example.category))];
  
  // Filter examples
  const filteredExamples = examples.filter(example => {
    // Filter by category
    if (selectedCategory && example.category !== selectedCategory) {
      return false;
    }
    
    // Filter by difficulty
    if (selectedDifficulty && example.difficulty !== selectedDifficulty) {
      return false;
    }
    
    // Filter by search query
    if (searchQuery) {
      const query = searchQuery.toLowerCase();
      return (
        example.title.toLowerCase().includes(query) ||
        example.description.toLowerCase().includes(query) ||
        example.category.toLowerCase().includes(query)
      );
    }
    
    return true;
  });

  return (
    <Layout
      title="Examples | Neo Rust SDK"
      description="Example code for the Neo Rust SDK"
    >
      <div className="container mx-auto px-4 py-12">
        <div className="mb-8">
          <h1 className="text-4xl font-bold mb-2">Examples</h1>
          <p className="text-xl text-gray-300">
            Explore code examples for common Neo Rust SDK tasks
          </p>
        </div>
        
        <div className="flex flex-col lg:flex-row gap-8">
          {/* Sidebar */}
          <div className="lg:w-64">
            <div className="sticky top-24">
              <div className="p-5 bg-slate-800 rounded-lg border border-slate-700 shadow-lg">
                <h3 className="text-lg font-semibold mb-4">Filters</h3>
                
                {/* Search */}
                <div className="mb-6">
                  <label className="block text-sm font-medium text-gray-400 mb-1">Search</label>
                  <input
                    type="text"
                    value={searchQuery}
                    onChange={(e) => setSearchQuery(e.target.value)}
                    placeholder="Search examples..."
                    className="w-full px-3 py-2 rounded-lg bg-slate-700 border border-slate-600 focus:border-green-400 focus:outline-none focus:ring-1 focus:ring-green-400 text-gray-200 placeholder-gray-400"
                  />
                </div>
                
                {/* Category filter */}
                <div className="mb-6">
                  <label className="block text-sm font-medium text-gray-400 mb-1">Category</label>
                  <select
                    value={selectedCategory || ''}
                    onChange={(e) => setSelectedCategory(e.target.value || null)}
                    className="w-full px-3 py-2 rounded-lg bg-slate-700 border border-slate-600 focus:border-green-400 focus:outline-none focus:ring-1 focus:ring-green-400 text-gray-200"
                  >
                    <option value="">All Categories</option>
                    {categories.map(category => (
                      <option key={category} value={category}>{category}</option>
                    ))}
                  </select>
                </div>
                
                {/* Difficulty filter */}
                <div className="mb-6">
                  <label className="block text-sm font-medium text-gray-400 mb-1">Difficulty</label>
                  <select
                    value={selectedDifficulty || ''}
                    onChange={(e) => setSelectedDifficulty(e.target.value || null)}
                    className="w-full px-3 py-2 rounded-lg bg-slate-700 border border-slate-600 focus:border-green-400 focus:outline-none focus:ring-1 focus:ring-green-400 text-gray-200"
                  >
                    <option value="">All Difficulties</option>
                    <option value="beginner">Beginner</option>
                    <option value="intermediate">Intermediate</option>
                    <option value="advanced">Advanced</option>
                  </select>
                </div>
                
                {/* Reset filters */}
                <button
                  onClick={() => {
                    setSelectedCategory(null);
                    setSelectedDifficulty(null);
                    setSearchQuery('');
                  }}
                  className="w-full px-4 py-2 bg-slate-700 hover:bg-slate-600 rounded-lg transition text-center"
                >
                  Reset Filters
                </button>
              </div>
            </div>
          </div>
          
          {/* Examples grid */}
          <div className="flex-1">
            {filteredExamples.length === 0 ? (
              <div className="text-center py-12">
                <p className="text-gray-400 text-lg">No examples found matching your filters.</p>
                <button
                  onClick={() => {
                    setSelectedCategory(null);
                    setSelectedDifficulty(null);
                    setSearchQuery('');
                  }}
                  className="mt-4 px-4 py-2 bg-green-600 hover:bg-green-700 rounded-lg transition"
                >
                  Reset Filters
                </button>
              </div>
            ) : (
              <div className="grid gap-8">
                {filteredExamples.map(example => (
                  <div key={example.id} className="bg-slate-800 rounded-lg overflow-hidden border border-slate-700 shadow-lg">
                    <div className="p-5 border-b border-slate-700">
                      <div className="flex justify-between items-start mb-2">
                        <h2 className="text-2xl font-bold text-white">{example.title}</h2>
                        <span className={`inline-block rounded-full px-3 py-1 text-xs font-medium ${
                          example.difficulty === 'beginner' 
                            ? 'bg-green-900/30 text-green-400' 
                            : example.difficulty === 'intermediate'
                              ? 'bg-yellow-900/30 text-yellow-400'
                              : 'bg-red-900/30 text-red-400'
                        }`}>
                          {example.difficulty.charAt(0).toUpperCase() + example.difficulty.slice(1)}
                        </span>
                      </div>
                      <p className="text-gray-300 mb-2">{example.description}</p>
                      <p className="text-sm text-gray-400">Category: {example.category}</p>
                    </div>
                    <div className="border-b border-slate-700">
                      <CodeBlock
                        code={example.code}
                        language="rust"
                        filename="main.rs"
                      />
                    </div>
                    <div className="p-4 flex justify-end">
                      <Link
                        to={`/playground?example=${example.id}`}
                        className="flex items-center text-green-400 hover:text-green-300 transition"
                      >
                        Try in Playground
                        <svg className="ml-1 w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M13 7l5 5m0 0l-5 5m5-5H6"></path>
                        </svg>
                      </Link>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>
      </div>
    </Layout>
  );
};

export default ExamplesPage;