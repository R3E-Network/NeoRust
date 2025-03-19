import React from 'react';
import { Link } from 'gatsby';
import Layout from '../../components/Layout';
import DocSidebar from '../../components/DocSidebar';
import Search from '../../components/Search';
import CodeBlock from '../../components/CodeBlock';

const DocsIndexPage: React.FC = () => {
  const basicUsageCode = `use neo3::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to Neo N3 TestNet
    let provider = HttpProvider::new("https://testnet1.neo.org:443")?;
    let client = RpcClient::new(provider);
    
    // Get basic blockchain information
    let block_count = client.get_block_count().await?;
    println!("Current block height: {}", block_count);
    
    // Create a key pair
    let key_pair = KeyPair::new_random()?;
    let address = key_pair.get_address();
    println!("New address: {}", address);
    
    Ok(())
}`;

  return (
    <Layout
      title="Documentation | Neo Rust SDK"
      description="Comprehensive documentation for the Neo Rust SDK"
    >
      <div className="container mx-auto px-4 py-12">
        <div className="flex flex-col md:flex-row">
          <DocSidebar currentPath="/docs" />
          
          <main className="flex-1 md:pl-8 pt-8 md:pt-0">
            <div className="max-w-3xl">
              <div className="mb-8">
                <h1 className="text-4xl font-bold mb-2">Documentation</h1>
                <p className="text-xl text-gray-300">
                  Learn how to build applications on Neo N3 with the Neo Rust SDK
                </p>
              </div>
              
              <div className="mt-8 mb-12">
                <Search placeholder="Search documentation..." />
              </div>
              
              <div className="prose prose-invert prose-lg max-w-none">
                <p>
                  Welcome to the Neo Rust SDK documentation. This library provides a comprehensive 
                  set of tools for building applications on the Neo N3 blockchain.
                </p>
                
                <h2>Getting Started</h2>
                
                <h3>Installation</h3>
                
                <p>
                  Add Neo Rust SDK to your Cargo.toml:
                </p>
                
                <CodeBlock 
                  language="toml" 
                  code={`[dependencies]
neo3 = "0.1.9"`} 
                  filename="Cargo.toml"
                />
                
                <p>
                  If you want to use specific features, you can enable them:
                </p>
                
                <CodeBlock 
                  language="toml" 
                  code={`[dependencies]
neo3 = { version = "0.1.9", features = ["futures", "ledger"] }`} 
                  filename="Cargo.toml"
                />
                
                <h3>Basic Usage</h3>
                
                <p>
                  Here's a simple example that connects to the Neo TestNet, retrieves the current 
                  block height, and creates a new address:
                </p>
                
                <CodeBlock 
                  language="rust" 
                  code={basicUsageCode}
                  filename="src/main.rs"
                />
                
                <h2>Key Features</h2>
                
                <ul>
                  <li>
                    <strong>Wallet Management</strong>: Create, load, and manage Neo wallets, generate keys, and sign transactions
                  </li>
                  <li>
                    <strong>Smart Contract Integration</strong>: Deploy and interact with Neo N3 smart contracts
                  </li>
                  <li>
                    <strong>Token Standards</strong>: Built-in support for NEP-17 (fungible) and NEP-11 (non-fungible) tokens
                  </li>
                  <li>
                    <strong>Neo X Support</strong>: Seamless integration with Neo X for EVM compatibility
                  </li>
                  <li>
                    <strong>Cryptography</strong>: Comprehensive cryptographic utilities for blockchain operations
                  </li>
                </ul>
                
                <h2>Explore Documentation</h2>
                
                <div className="grid md:grid-cols-2 gap-6 not-prose mt-6">
                  <Link to="/docs/getting-started" className="card group">
                    <h3 className="text-xl font-semibold mb-3 group-hover:text-green-400 transition">Getting Started</h3>
                    <p className="text-gray-300 mb-4">Installation, basic concepts, and your first Neo Rust application</p>
                  </Link>
                  
                  <Link to="/docs/wallets" className="card group">
                    <h3 className="text-xl font-semibold mb-3 group-hover:text-green-400 transition">Wallet Management</h3>
                    <p className="text-gray-300 mb-4">Create and manage Neo wallets, handle keys, and sign transactions</p>
                  </Link>
                  
                  <Link to="/docs/contracts" className="card group">
                    <h3 className="text-xl font-semibold mb-3 group-hover:text-green-400 transition">Smart Contracts</h3>
                    <p className="text-gray-300 mb-4">Deploy and interact with smart contracts on the Neo blockchain</p>
                  </Link>
                  
                  <Link to="/docs/neo-x" className="card group">
                    <h3 className="text-xl font-semibold mb-3 group-hover:text-green-400 transition">Neo X</h3>
                    <p className="text-gray-300 mb-4">Work with Neo X's EVM compatibility and cross-chain functionality</p>
                  </Link>
                </div>
                
                <h2 className="mt-10">Need Help?</h2>
                
                <p>
                  If you have questions or need assistance, you can:
                </p>
                
                <ul>
                  <li>Check out the <Link to="/examples" className="text-green-400 hover:underline">examples</Link> for practical use cases</li>
                  <li>Visit the <a href="https://github.com/neo-project/neo-rust/issues" className="text-green-400 hover:underline">GitHub issues</a> to report bugs or request features</li>
                  <li>Join the <a href="https://discord.gg/neo" className="text-green-400 hover:underline">Neo Discord</a> for community support</li>
                </ul>
              </div>
            </div>
          </main>
        </div>
      </div>
    </Layout>
  );
};

export default DocsIndexPage;