import React from 'react';
import { Link } from 'gatsby';
import Layout from '../components/Layout';
import CodeBlock from '../components/CodeBlock';

const IndexPage: React.FC = () => {
  const exampleCode = `use neo3::prelude::*;

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
    
    // Create a wallet signer
    let wallet = WalletSigner::new_with_signer(key_pair.clone(), address.clone());
    
    // Initialize a token contract
    let gas_token = GasToken::new(&client);
    let symbol = gas_token.symbol().await?;
    let decimals = gas_token.decimals().await?;
    
    println!("Token: {} with {} decimals", symbol, decimals);
    
    Ok(())
}`;

  return (
    <Layout>
      {/* Hero Section */}
      <section className="relative pt-32 pb-20 md:pt-40 md:pb-28">
        <div className="absolute inset-0 overflow-hidden">
          <div className="absolute top-0 left-0 w-full h-full bg-gradient-to-br from-green-900/20 to-slate-900/10 z-0"></div>
          <div className="absolute top-40 left-20 w-72 h-72 bg-green-500/10 rounded-full filter blur-3xl"></div>
          <div className="absolute top-20 right-20 w-96 h-96 bg-blue-500/10 rounded-full filter blur-3xl"></div>
          <div className="bg-grid absolute inset-0 opacity-10"></div>
        </div>
        
        <div className="container mx-auto px-4 relative z-10">
          <div className="max-w-3xl mx-auto text-center">
            <h1 className="text-4xl md:text-6xl font-bold mb-6">
              <span className="text-transparent bg-clip-text bg-gradient-to-r from-green-400 to-teal-400">Neo Rust SDK</span>
            </h1>
            <p className="text-xl md:text-2xl text-gray-300 mb-10">A comprehensive Rust library for building applications on the Neo N3 blockchain ecosystem</p>
            
            <div className="flex flex-col sm:flex-row justify-center gap-4">
              <Link to="/docs/getting-started" className="btn btn-primary">
                Get Started
              </Link>
              <a href="https://github.com/neo-project/neo-rust" className="btn btn-secondary">
                View on GitHub
              </a>
            </div>
          </div>
        </div>
      </section>

      {/* Features Section */}
      <section className="py-20 bg-slate-800/50">
        <div className="container mx-auto px-4">
          <h2 className="text-3xl md:text-4xl font-bold text-center mb-16">
            <span className="text-transparent bg-clip-text bg-gradient-to-r from-green-400 to-teal-400">Key Features</span>
          </h2>
          
          <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-8">
            <div className="card">
              <div className="w-12 h-12 bg-green-500/20 rounded-lg flex items-center justify-center mb-4">
                <svg className="w-6 h-6 text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M13 10V3L4 14h7v7l9-11h-7z"></path>
                </svg>
              </div>
              <h3 className="text-xl font-semibold mb-2">Performance Optimized</h3>
              <p className="text-gray-300">Built with Rust's performance and safety guarantees for high-throughput blockchain applications.</p>
            </div>
            
            <div className="card">
              <div className="w-12 h-12 bg-green-500/20 rounded-lg flex items-center justify-center mb-4">
                <svg className="w-6 h-6 text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z"></path>
                </svg>
              </div>
              <h3 className="text-xl font-semibold mb-2">Comprehensive Security</h3>
              <p className="text-gray-300">State-of-the-art cryptographic implementations with thorough security considerations.</p>
            </div>
            
            <div className="card">
              <div className="w-12 h-12 bg-green-500/20 rounded-lg flex items-center justify-center mb-4">
                <svg className="w-6 h-6 text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z"></path>
                </svg>
              </div>
              <h3 className="text-xl font-semibold mb-2">Smart Contract Support</h3>
              <p className="text-gray-300">Intuitive interfaces for deploying and interacting with Neo N3 smart contracts.</p>
            </div>
            
            <div className="card">
              <div className="w-12 h-12 bg-green-500/20 rounded-lg flex items-center justify-center mb-4">
                <svg className="w-6 h-6 text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4"></path>
                </svg>
              </div>
              <h3 className="text-xl font-semibold mb-2">Wallet Management</h3>
              <p className="text-gray-300">Complete wallet functionality with NEP-6 standard support and hardware wallet integration.</p>
            </div>
            
            <div className="card">
              <div className="w-12 h-12 bg-green-500/20 rounded-lg flex items-center justify-center mb-4">
                <svg className="w-6 h-6 text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10"></path>
                </svg>
              </div>
              <h3 className="text-xl font-semibold mb-2">Neo X Integration</h3>
              <p className="text-gray-300">Seamless integration with Neo X for EVM compatibility and cross-chain operations.</p>
            </div>
            
            <div className="card">
              <div className="w-12 h-12 bg-green-500/20 rounded-lg flex items-center justify-center mb-4">
                <svg className="w-6 h-6 text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z"></path>
                </svg>
              </div>
              <h3 className="text-xl font-semibold mb-2">Developer Friendly</h3>
              <p className="text-gray-300">Intuitive, well-documented API with type safety and comprehensive examples.</p>
            </div>
          </div>
        </div>
      </section>

      {/* Code Example Section */}
      <section className="py-20">
        <div className="container mx-auto px-4">
          <h2 className="text-3xl md:text-4xl font-bold text-center mb-16">
            <span className="text-transparent bg-clip-text bg-gradient-to-r from-green-400 to-teal-400">Simple to Use</span>
          </h2>
          
          <div className="grid md:grid-cols-2 gap-8 items-center">
            <div>
              <h3 className="text-2xl font-semibold mb-4">Get Started in Minutes</h3>
              <p className="text-gray-300 mb-6">Neo Rust SDK provides a clean, intuitive API for blockchain development. Connect to the network, manage wallets, and interact with smart contracts with just a few lines of code.</p>
              
              <ul className="space-y-3">
                <li className="flex items-start">
                  <svg className="w-6 h-6 text-green-400 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M5 13l4 4L19 7"></path>
                  </svg>
                  <span>Type-safe blockchain interactions</span>
                </li>
                <li className="flex items-start">
                  <svg className="w-6 h-6 text-green-400 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M5 13l4 4L19 7"></path>
                  </svg>
                  <span>Async/await support for modern codebases</span>
                </li>
                <li className="flex items-start">
                  <svg className="w-6 h-6 text-green-400 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M5 13l4 4L19 7"></path>
                  </svg>
                  <span>Comprehensive error handling</span>
                </li>
              </ul>
            </div>
            
            <div className="relative">
              <div className="rounded-xl overflow-hidden shadow-2xl bg-slate-900 border border-slate-700">
                <CodeBlock
                  code={exampleCode}
                  language="rust"
                  filename="main.rs"
                />
              </div>
              <div className="absolute -bottom-6 -right-6 w-32 h-32 bg-green-500/10 rounded-full filter blur-2xl"></div>
            </div>
          </div>
        </div>
      </section>

      {/* Documentation Section */}
      <section className="py-20 bg-slate-800/50">
        <div className="container mx-auto px-4">
          <h2 className="text-3xl md:text-4xl font-bold text-center mb-16">
            <span className="text-transparent bg-clip-text bg-gradient-to-r from-green-400 to-teal-400">Comprehensive Documentation</span>
          </h2>
          
          <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-8">
            <Link to="/docs/wallets" className="card group">
              <h3 className="text-xl font-semibold mb-3 group-hover:text-green-400 transition">Wallet Management</h3>
              <p className="text-gray-300 mb-4">Learn how to create, load, and manage Neo wallets, including key management and transaction signing.</p>
              <span className="text-green-400 flex items-center">Read more <svg className="w-4 h-4 ml-1 group-hover:translate-x-1 transition-transform" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M9 5l7 7-7 7"></path>
              </svg></span>
            </Link>
            
            <Link to="/docs/contracts" className="card group">
              <h3 className="text-xl font-semibold mb-3 group-hover:text-green-400 transition">Smart Contracts</h3>
              <p className="text-gray-300 mb-4">Discover how to deploy and interact with smart contracts on the Neo blockchain.</p>
              <span className="text-green-400 flex items-center">Read more <svg className="w-4 h-4 ml-1 group-hover:translate-x-1 transition-transform" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M9 5l7 7-7 7"></path>
              </svg></span>
            </Link>
            
            <Link to="/docs/neo-x" className="card group">
              <h3 className="text-xl font-semibold mb-3 group-hover:text-green-400 transition">Neo X Integration</h3>
              <p className="text-gray-300 mb-4">Explore the EVM-compatible chain and cross-chain bridge functionality in Neo X.</p>
              <span className="text-green-400 flex items-center">Read more <svg className="w-4 h-4 ml-1 group-hover:translate-x-1 transition-transform" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M9 5l7 7-7 7"></path>
              </svg></span>
            </Link>
            
            <Link to="/docs/crypto" className="card group">
              <h3 className="text-xl font-semibold mb-3 group-hover:text-green-400 transition">Cryptography</h3>
              <p className="text-gray-300 mb-4">Master the cryptographic primitives and utilities available in the Neo Rust SDK.</p>
              <span className="text-green-400 flex items-center">Read more <svg className="w-4 h-4 ml-1 group-hover:translate-x-1 transition-transform" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M9 5l7 7-7 7"></path>
              </svg></span>
            </Link>
            
            <Link to="/docs/contracts/token-standards" className="card group">
              <h3 className="text-xl font-semibold mb-3 group-hover:text-green-400 transition">Token Standards</h3>
              <p className="text-gray-300 mb-4">Learn about NEP-17 fungible tokens and NEP-11 non-fungible tokens implementation.</p>
              <span className="text-green-400 flex items-center">Read more <svg className="w-4 h-4 ml-1 group-hover:translate-x-1 transition-transform" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M9 5l7 7-7 7"></path>
              </svg></span>
            </Link>
            
            <Link to="/api-reference" className="card group">
              <h3 className="text-xl font-semibold mb-3 group-hover:text-green-400 transition">API Reference</h3>
              <p className="text-gray-300 mb-4">Access the complete API reference documentation for the Neo Rust SDK.</p>
              <span className="text-green-400 flex items-center">Read more <svg className="w-4 h-4 ml-1 group-hover:translate-x-1 transition-transform" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M9 5l7 7-7 7"></path>
              </svg></span>
            </Link>
          </div>
        </div>
      </section>

      {/* Playground Call-to-Action */}
      <section className="py-20">
        <div className="container mx-auto px-4">
          <div className="rounded-2xl overflow-hidden bg-gradient-to-br from-slate-700 via-slate-700 to-slate-800 border border-slate-600">
            <div className="p-8 md:p-12 text-center">
              <h2 className="text-3xl md:text-4xl font-bold mb-6">
                <span className="text-transparent bg-clip-text bg-gradient-to-r from-green-400 to-teal-400">Try It In Your Browser</span>
              </h2>
              <p className="text-xl text-gray-300 mb-8 max-w-3xl mx-auto">Experiment with Neo Rust SDK code examples directly in your browser with our interactive playground. No installation required.</p>
              <Link to="/playground" className="btn btn-primary">
                Launch Playground
              </Link>
            </div>
          </div>
        </div>
      </section>

      {/* Community Section */}
      <section className="py-20 bg-slate-800/50">
        <div className="container mx-auto px-4">
          <h2 className="text-3xl md:text-4xl font-bold text-center mb-16">
            <span className="text-transparent bg-clip-text bg-gradient-to-r from-green-400 to-teal-400">Join the Community</span>
          </h2>
          
          <div className="grid md:grid-cols-3 gap-8">
            <a href="https://github.com/neo-project/neo-rust" className="card group flex flex-col items-center text-center">
              <svg className="w-12 h-12 text-gray-300 group-hover:text-green-400 transition mb-4" fill="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                <path fillRule="evenodd" d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z" clipRule="evenodd"></path>
              </svg>
              <h3 className="text-xl font-semibold mb-2 group-hover:text-green-400 transition">GitHub</h3>
              <p className="text-gray-300">Star the repository, report issues, and contribute to the SDK development.</p>
            </a>
            
            <a href="https://discord.gg/neo" className="card group flex flex-col items-center text-center">
              <svg className="w-12 h-12 text-gray-300 group-hover:text-green-400 transition mb-4" fill="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                <path d="M20.317 4.3698a19.7913 19.7913 0 00-4.8851-1.5152.0741.0741 0 00-.0785.0371c-.211.3753-.4447.8648-.6083 1.2495-1.8447-.2762-3.68-.2762-5.4868 0-.1636-.3933-.4058-.8742-.6177-1.2495a.077.077 0 00-.0785-.037 19.7363 19.7363 0 00-4.8852 1.515.0699.0699 0 00-.0321.0277C.5334 9.0458-.319 13.5799.0992 18.0578a.0824.0824 0 00.0312.0561c2.0528 1.5076 4.0413 2.4228 5.9929 3.0294a.0777.0777 0 00.0842-.0276c.4616-.6304.8731-1.2952 1.226-1.9942a.076.076 0 00-.0416-.1057c-.6528-.2476-1.2743-.5495-1.8722-.8923a.077.077 0 01-.0076-.1277c.1258-.0943.2517-.1923.3718-.2914a.0743.0743 0 01.0776-.0105c3.9278 1.7933 8.18 1.7933 12.0614 0a.0739.0739 0 01.0785.0095c.1202.099.246.1981.3728.2924a.077.077 0 01-.0066.1276 12.2986 12.2986 0 01-1.873.8914.0766.0766 0 00-.0407.1067c.3604.698.7719 1.3628 1.225 1.9932a.076.076 0 00.0842.0286c1.961-.6067 3.9495-1.5219 6.0023-3.0294a.077.077 0 00.0313-.0552c.5004-5.177-.8382-9.6739-3.5485-13.6604a.061.061 0 00-.0312-.0286zM8.02 15.3312c-1.1825 0-2.1569-1.0857-2.1569-2.419 0-1.3332.9555-2.4189 2.157-2.4189 1.2108 0 2.1757 1.0952 2.1568 2.419 0 1.3332-.9555 2.4189-2.1569 2.4189zm7.9748 0c-1.1825 0-2.1569-1.0857-2.1569-2.419 0-1.3332.9554-2.4189 2.1569-2.4189 1.2108 0 2.1757 1.0952 2.1568 2.419 0 1.3332-.946 2.4189-2.1568 2.4189Z"></path>
              </svg>
              <h3 className="text-xl font-semibold mb-2 group-hover:text-green-400 transition">Discord</h3>
              <p className="text-gray-300">Join the Neo community Discord for real-time support and discussions.</p>
            </a>
            
            <a href="https://neo.org/dev" className="card group flex flex-col items-center text-center">
              <svg className="w-12 h-12 text-gray-300 group-hover:text-green-400 transition mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9"></path>
              </svg>
              <h3 className="text-xl font-semibold mb-2 group-hover:text-green-400 transition">Developer Portal</h3>
              <p className="text-gray-300">Access the Neo developer portal for additional resources and documentation.</p>
            </a>
          </div>
        </div>
      </section>
    </Layout>
  );
};

export default IndexPage;