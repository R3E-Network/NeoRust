import React, { useEffect } from 'react';
import { Link } from 'gatsby';
import Layout from '../components/Layout';
import CodeBlock from '../components/CodeBlock';
import { useCallback } from 'react';
import Particles from "react-particles";
import { loadFull } from 'tsparticles';
import { useInView } from 'react-intersection-observer';
import type { Engine } from 'tsparticles-engine';

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

  const particlesInit = useCallback(async (engine: Engine) => {
    await loadFull(engine);
  }, []);

  const [featureRef, featureInView] = useInView({
    triggerOnce: true,
    threshold: 0.1,
  });

  const [codeRef, codeInView] = useInView({
    triggerOnce: true,
    threshold: 0.1,
  });

  const [docsRef, docsInView] = useInView({
    triggerOnce: true,
    threshold: 0.1,
  });

  return (
    <Layout showAnimatedBackground={false}>
      {/* Hero Section with Particles */}
      <section className="relative min-h-screen flex items-center">
        <div className="absolute inset-0 overflow-hidden -z-10">
          <Particles
            id="tsparticles"
            init={particlesInit}
            options={{
              fullScreen: { enable: false },
              background: {
                color: {
                  value: "transparent",
                },
              },
              fpsLimit: 60,
              particles: {
                color: {
                  value: "#10b981",
                },
                links: {
                  color: "#10b981",
                  distance: 150,
                  enable: true,
                  opacity: 0.2,
                  width: 1,
                },
                move: {
                  direction: "none",
                  enable: true,
                  outModes: {
                    default: "bounce",
                  },
                  random: false,
                  speed: 1,
                  straight: false,
                },
                number: {
                  density: {
                    enable: true,
                    area: 800,
                  },
                  value: 80,
                },
                opacity: {
                  value: 0.5,
                  animation: {
                    enable: true,
                    speed: 0.5,
                    minimumValue: 0.1,
                  },
                },
                shape: {
                  type: "circle",
                },
                size: {
                  value: { min: 1, max: 3 },
                },
              },
              detectRetina: true,
            }}
            className="absolute inset-0"
          />
          <div className="absolute inset-0 bg-gradient-to-b from-slate-900/20 via-slate-900/80 to-slate-900"></div>
          <div className="absolute top-1/3 left-1/4 w-96 h-96 bg-green-500/10 rounded-full filter blur-3xl"></div>
          <div className="absolute bottom-1/4 right-1/4 w-96 h-96 bg-blue-500/10 rounded-full filter blur-3xl"></div>
        </div>
        
        <div className="container mx-auto px-4 relative z-10 mt-24 md:mt-0">
          <div className="max-w-4xl mx-auto">
            <div className="flex flex-col items-center text-center">
              <div className="w-24 h-24 mb-6 relative">
                <div className="absolute inset-0 bg-green-500/20 rounded-full animate-pulse-slow"></div>
                <div className="absolute inset-2 bg-gradient-to-br from-green-400 to-teal-500 rounded-full"></div>
                <div className="absolute inset-3 flex items-center justify-center">
                  <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" className="w-full h-full text-white">
                    <path d="M11.584 2.376a.75.75 0 01.832 0l9 6a.75.75 0 11-.832 1.248L12 3.901 3.416 9.624a.75.75 0 01-.832-1.248l9-6z" />
                    <path fillRule="evenodd" d="M20.25 10.332v9.918H21a.75.75 0 010 1.5H3a.75.75 0 010-1.5h.75v-9.918a.75.75 0 01.634-.74A49.109 49.109 0 0112 9c2.59 0 5.134.202 7.616.592a.75.75 0 01.634.74zm-7.5 2.418a.75.75 0 00-1.5 0v6.75a.75.75 0 001.5 0v-6.75zm3-.75a.75.75 0 01.75.75v6.75a.75.75 0 01-1.5 0v-6.75a.75.75 0 01.75-.75zM9 12.75a.75.75 0 00-1.5 0v6.75a.75.75 0 001.5 0v-6.75z" clipRule="evenodd" />
                    <path d="M12 7.875a1.125 1.125 0 100-2.25 1.125 1.125 0 000 2.25z" />
                  </svg>
                </div>
              </div>
              <h1 className="text-5xl md:text-7xl font-bold mb-4">
                <span className="text-white">Neo </span>
                <span className="text-transparent bg-clip-text bg-gradient-to-r from-green-400 to-teal-400">Rust SDK</span>
              </h1>
              <p className="text-xl md:text-2xl text-gray-300 mb-10 max-w-3xl">
                A comprehensive Rust library for building high-performance applications on the Neo N3 blockchain ecosystem
              </p>
              
              <div className="flex flex-col sm:flex-row justify-center gap-4 mb-12">
                <Link to="/docs/getting-started" className="btn btn-primary flex items-center justify-center group">
                  Get Started
                  <svg xmlns="http://www.w3.org/2000/svg" className="ml-2 h-5 w-5 group-hover:translate-x-1 transition-transform" viewBox="0 0 20 20" fill="currentColor">
                    <path fillRule="evenodd" d="M12.293 5.293a1 1 0 011.414 0l4 4a1 1 0 010 1.414l-4 4a1 1 0 01-1.414-1.414L14.586 11H3a1 1 0 110-2h11.586l-2.293-2.293a1 1 0 010-1.414z" clipRule="evenodd" />
                  </svg>
                </Link>
                <a href="https://github.com/R3E-Network/NeoRust" className="btn btn-secondary flex items-center justify-center">
                  <svg className="mr-2 h-5 w-5" fill="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                    <path fillRule="evenodd" d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z" clipRule="evenodd"></path>
                  </svg>
                  View on GitHub
                </a>
              </div>
              
              <div className="grid grid-cols-3 gap-6 w-full max-w-2xl">
                <div className="px-4 py-3 bg-slate-800/50 backdrop-blur rounded-lg border border-slate-700 text-center">
                  <div className="text-3xl font-bold text-green-400 mb-1">100%</div>
                  <div className="text-sm text-gray-400">Rust-Native</div>
                </div>
                <div className="px-4 py-3 bg-slate-800/50 backdrop-blur rounded-lg border border-slate-700 text-center">
                  <div className="text-3xl font-bold text-green-400 mb-1">N3</div>
                  <div className="text-sm text-gray-400">Full Support</div>
                </div>
                <div className="px-4 py-3 bg-slate-800/50 backdrop-blur rounded-lg border border-slate-700 text-center">
                  <div className="text-3xl font-bold text-green-400 mb-1">Neo X</div>
                  <div className="text-sm text-gray-400">Integration</div>
                </div>
              </div>
              
              <div className="absolute bottom-10 left-1/2 transform -translate-x-1/2 animate-bounce">
                <svg xmlns="http://www.w3.org/2000/svg" className="h-6 w-6 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 14l-7 7m0 0l-7-7m7 7V3" />
                </svg>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Features Section */}
      <section ref={featureRef} className="py-20 bg-slate-800/50">
        <div className="container mx-auto px-4">
          <h2 className="text-3xl md:text-4xl font-bold text-center mb-6">
            <span className="text-transparent bg-clip-text bg-gradient-to-r from-green-400 to-teal-400">Key Features</span>
          </h2>
          <p className="text-xl text-gray-300 text-center max-w-3xl mx-auto mb-16">
            Built with Rust's performance and safety guarantees for robust blockchain applications
          </p>
          
          <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-8">
            {[
              {
                icon: (
                  <svg className="w-6 h-6 text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M13 10V3L4 14h7v7l9-11h-7z"></path>
                  </svg>
                ),
                title: "Performance Optimized",
                description: "Built with Rust's performance and safety guarantees for high-throughput blockchain applications."
              },
              {
                icon: (
                  <svg className="w-6 h-6 text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z"></path>
                  </svg>
                ),
                title: "Comprehensive Security",
                description: "State-of-the-art cryptographic implementations with thorough security considerations."
              },
              {
                icon: (
                  <svg className="w-6 h-6 text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z"></path>
                  </svg>
                ),
                title: "Smart Contract Support",
                description: "Intuitive interfaces for deploying and interacting with Neo N3 smart contracts."
              },
              {
                icon: (
                  <svg className="w-6 h-6 text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4"></path>
                  </svg>
                ),
                title: "Wallet Management",
                description: "Complete wallet functionality with NEP-6 standard support and hardware wallet integration."
              },
              {
                icon: (
                  <svg className="w-6 h-6 text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10"></path>
                  </svg>
                ),
                title: "Neo X Integration",
                description: "Seamless integration with Neo X for EVM compatibility and cross-chain operations."
              },
              {
                icon: (
                  <svg className="w-6 h-6 text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z"></path>
                  </svg>
                ),
                title: "Developer Friendly",
                description: "Intuitive, well-documented API with type safety and comprehensive examples."
              }
            ].map((feature, index) => (
              <div 
                key={index} 
                className={`card backdrop-blur-sm transition-all transform ${
                  featureInView ? 'opacity-100 translate-y-0' : 'opacity-0 translate-y-10'
                }`}
                style={{ 
                  transitionDelay: `${index * 100}ms`,
                  transitionDuration: '500ms'
                }}
              >
                <div className="w-12 h-12 bg-green-500/20 rounded-lg flex items-center justify-center mb-4 group-hover:bg-green-500/30 transition-colors">
                  {feature.icon}
                </div>
                <h3 className="text-xl font-semibold mb-2 group-hover:text-green-400 transition-colors">{feature.title}</h3>
                <p className="text-gray-300">{feature.description}</p>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Code Example Section */}
      <section ref={codeRef} className="py-20">
        <div className="container mx-auto px-4">
          <h2 className="text-3xl md:text-4xl font-bold text-center mb-6">
            <span className="text-transparent bg-clip-text bg-gradient-to-r from-green-400 to-teal-400">Simple to Use</span>
          </h2>
          <p className="text-xl text-gray-300 text-center max-w-3xl mx-auto mb-16">
            Write clean, type-safe blockchain code with modern Rust features
          </p>
          
          <div className="grid md:grid-cols-2 gap-12 items-center">
            <div className={`transition-all duration-700 ${codeInView ? 'opacity-100 translate-x-0' : 'opacity-0 -translate-x-10'}`}>
              <h3 className="text-2xl font-semibold mb-6">Get Started in Minutes</h3>
              <p className="text-gray-300 mb-8">Neo Rust SDK provides a clean, intuitive API for blockchain development. Connect to the network, manage wallets, and interact with smart contracts with just a few lines of code.</p>
              
              <div className="space-y-5">
                {[
                  { icon: "check", text: "Type-safe blockchain interactions" },
                  { icon: "check", text: "Async/await support for modern codebases" },
                  { icon: "check", text: "Comprehensive error handling" },
                  { icon: "check", text: "Extensive documentation and examples" }
                ].map((item, index) => (
                  <div 
                    key={index} 
                    className="flex items-start"
                    style={{ 
                      transitionDelay: `${index * 150 + 300}ms`,
                      transitionDuration: '500ms'
                    }}
                  >
                    <div className="flex-shrink-0 w-6 h-6 bg-green-500/20 rounded-full flex items-center justify-center mr-3">
                      <svg className="w-4 h-4 text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M5 13l4 4L19 7"></path>
                      </svg>
                    </div>
                    <span className="text-gray-200">{item.text}</span>
                  </div>
                ))}
              </div>
              
              <div className="mt-10">
                <Link to="/docs/getting-started" className="text-green-400 flex items-center group">
                  <span>Explore the Documentation</span>
                  <svg className="ml-2 w-5 h-5 group-hover:translate-x-1 transition-transform" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M13 7l5 5m0 0l-5 5m5-5H6"></path>
                  </svg>
                </Link>
              </div>
            </div>
            
            <div className={`relative transition-all duration-700 ${codeInView ? 'opacity-100 translate-x-0' : 'opacity-0 translate-x-10'}`}>
              <div className="absolute -top-6 -left-6 w-32 h-32 bg-green-500/10 rounded-full filter blur-2xl"></div>
              <div className="absolute -bottom-6 -right-6 w-32 h-32 bg-blue-500/10 rounded-full filter blur-2xl"></div>
              
              <div className="relative rounded-xl overflow-hidden shadow-2xl shadow-green-900/20 bg-slate-900 border border-slate-700">
                <div className="bg-slate-800 border-b border-slate-700 px-4 py-2 flex items-center">
                  <div className="flex space-x-1.5 mr-4">
                    <div className="w-3 h-3 rounded-full bg-red-500"></div>
                    <div className="w-3 h-3 rounded-full bg-yellow-500"></div>
                    <div className="w-3 h-3 rounded-full bg-green-500"></div>
                  </div>
                  <div className="text-sm text-gray-400">main.rs</div>
                </div>
                <CodeBlock
                  code={exampleCode}
                  language="rust"
                  filename="main.rs"
                />
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Documentation Section */}
      <section ref={docsRef} className="py-20 bg-slate-800/50">
        <div className="container mx-auto px-4">
          <h2 className="text-3xl md:text-4xl font-bold text-center mb-6">
            <span className="text-transparent bg-clip-text bg-gradient-to-r from-green-400 to-teal-400">Comprehensive Documentation</span>
          </h2>
          <p className="text-xl text-gray-300 text-center max-w-3xl mx-auto mb-16">
            Everything you need to build powerful Neo N3 applications
          </p>
          
          <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-8">
            {[
              {
                title: "Wallet Management",
                description: "Learn how to create, load, and manage Neo wallets, including key management and transaction signing.",
                icon: (
                  <svg xmlns="http://www.w3.org/2000/svg" className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 10h18M7 15h1m4 0h1m-7 4h12a3 3 0 003-3V8a3 3 0 00-3-3H6a3 3 0 00-3 3v8a3 3 0 003 3z" />
                  </svg>
                ),
                path: "/docs/wallets"
              },
              {
                title: "Smart Contracts",
                description: "Discover how to deploy and interact with smart contracts on the Neo blockchain.",
                icon: (
                  <svg xmlns="http://www.w3.org/2000/svg" className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                  </svg>
                ),
                path: "/docs/contracts"
              },
              {
                title: "Neo X Integration",
                description: "Explore the EVM-compatible chain and cross-chain bridge functionality in Neo X.",
                icon: (
                  <svg xmlns="http://www.w3.org/2000/svg" className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
                  </svg>
                ),
                path: "/docs/neo-x"
              },
              {
                title: "Cryptography",
                description: "Master the cryptographic primitives and utilities available in the Neo Rust SDK.",
                icon: (
                  <svg xmlns="http://www.w3.org/2000/svg" className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
                  </svg>
                ),
                path: "/docs/crypto"
              },
              {
                title: "Token Standards",
                description: "Learn about NEP-17 fungible tokens and NEP-11 non-fungible tokens implementation.",
                icon: (
                  <svg xmlns="http://www.w3.org/2000/svg" className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17 9V7a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2m2 4h10a2 2 0 002-2v-6a2 2 0 00-2-2H9a2 2 0 00-2 2v6a2 2 0 002 2zm7-5a2 2 0 11-4 0 2 2 0 014 0z" />
                  </svg>
                ),
                path: "/docs/contracts/token-standards"
              },
              {
                title: "API Reference",
                description: "Access the complete API reference documentation for the Neo Rust SDK.",
                icon: (
                  <svg xmlns="http://www.w3.org/2000/svg" className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
                  </svg>
                ),
                path: "/api-reference"
              }
            ].map((doc, index) => (
              <Link 
                to={doc.path} 
                key={index} 
                className={`card group hover:border-green-400/50 transition-all duration-500 transform ${
                  docsInView ? 'opacity-100 translate-y-0' : 'opacity-0 translate-y-10'
                }`}
                style={{ 
                  transitionDelay: `${index * 100}ms`,
                  transitionDuration: '500ms'
                }}
              >
                <div className="flex space-x-4">
                  <div className="w-10 h-10 bg-green-500/20 rounded-lg flex items-center justify-center flex-shrink-0 group-hover:bg-green-500/30 transition-colors">
                    <span className="text-green-400">
                      {doc.icon}
                    </span>
                  </div>
                  <div>
                    <h3 className="text-xl font-semibold mb-2 group-hover:text-green-400 transition-colors">{doc.title}</h3>
                    <p className="text-gray-300 mb-4">{doc.description}</p>
                    <span className="text-green-400 flex items-center">
                      Read more 
                      <svg className="w-4 h-4 ml-1 group-hover:translate-x-1 transition-transform" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M9 5l7 7-7 7"></path>
                      </svg>
                    </span>
                  </div>
                </div>
              </Link>
            ))}
          </div>
        </div>
      </section>

      {/* Playground Call-to-Action */}
      <section className="py-20">
        <div className="container mx-auto px-4">
          <div className="rounded-2xl overflow-hidden relative">
            {/* Gradient background */}
            <div className="absolute inset-0 bg-gradient-to-br from-slate-800 via-slate-800 to-slate-900"></div>
            
            {/* Glowy effects */}
            <div className="absolute top-1/4 left-1/4 w-64 h-64 bg-green-500/20 rounded-full filter blur-3xl"></div>
            <div className="absolute bottom-1/4 right-1/4 w-64 h-64 bg-blue-500/20 rounded-full filter blur-3xl"></div>
            
            {/* Grid pattern overlay */}
            <div className="absolute inset-0 bg-grid opacity-10"></div>
            
            {/* Border overlay */}
            <div className="absolute inset-0 border border-green-500/20 rounded-2xl"></div>
            
            {/* Content */}
            <div className="relative p-8 md:p-12 lg:p-16 flex flex-col md:flex-row items-center">
              <div className="md:w-1/2 mb-10 md:mb-0">
                <h2 className="text-3xl md:text-4xl font-bold mb-6">
                  <span className="text-transparent bg-clip-text bg-gradient-to-r from-green-400 to-teal-400">Try It In Your Browser</span>
                </h2>
                <p className="text-xl text-gray-300 mb-8">
                  Experiment with the Neo Rust SDK directly in your browser. No installation required.
                </p>
                <Link 
                  to="/playground" 
                  className="btn btn-primary group flex items-center"
                >
                  <svg className="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                  </svg>
                  Open Playground 
                  <svg className="ml-1 w-4 h-4 group-hover:translate-x-1 transition-transform" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M9 5l7 7-7 7"></path>
                  </svg>
                </Link>
              </div>
              
              <div className="md:w-1/2 flex justify-center">
                <div className="transform transition-all duration-500 hover:scale-105 hover:-rotate-1 w-full max-w-md">
                  <div className="bg-slate-950 rounded-xl overflow-hidden shadow-2xl shadow-green-900/20 border border-green-500/20">
                    <div className="bg-slate-800 border-b border-slate-700 px-4 py-2 flex items-center">
                      <div className="flex space-x-1.5 mr-4">
                        <div className="w-3 h-3 rounded-full bg-red-500"></div>
                        <div className="w-3 h-3 rounded-full bg-yellow-500"></div>
                        <div className="w-3 h-3 rounded-full bg-green-500"></div>
                      </div>
                      <div className="text-sm text-gray-400">Neo Rust SDK Playground</div>
                    </div>
                    <div className="p-4 font-mono text-sm text-green-400 bg-[#1e1e1e] h-48 overflow-hidden">
                      <div className="mb-2 text-gray-400">// Neo N3 Interactive Playground</div>
                      <div><span className="text-pink-400">use</span> <span className="text-blue-400">neo3::prelude::*</span>;</div>
                      <div>&nbsp;</div>
                      <div><span className="text-pink-400">async fn</span> <span className="text-yellow-400">main</span>() -&gt; <span className="text-blue-400">Result</span>&lt;(), <span className="text-blue-400">Box</span>&lt;<span className="text-blue-400">dyn</span> <span className="text-blue-400">std::error::Error</span>&gt;&gt; {"{"}</div>
                      <div>&nbsp;&nbsp;&nbsp;&nbsp;<span className="text-gray-400">// Connect to Neo TestNet</span></div>
                      <div>&nbsp;&nbsp;&nbsp;&nbsp;<span className="text-pink-400">let</span> provider = <span className="text-yellow-400">HttpProvider::new</span>(<span className="text-green-300">"https://testnet.neo.org"</span>)?;</div>
                      <div>&nbsp;&nbsp;&nbsp;&nbsp;<span className="text-pink-400">let</span> client = <span className="text-yellow-400">RpcClient::new</span>(provider);</div>
                      <div className="relative">
                        <span className="absolute left-0 h-full w-2 bg-gray-500/30 animate-pulse"></span>
                        <span className="ml-3">&nbsp;&nbsp;&nbsp;&nbsp;<span className="text-gray-400">// Your code here</span></span>
                      </div>
                      <div>&nbsp;&nbsp;&nbsp;&nbsp;<span className="text-blue-400">Ok</span>(())</div>
                      <div>{"}"}</div>
                    </div>
                  </div>
                </div>
              </div>
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
            <a href="https://github.com/R3E-Network/NeoRust" className="card group flex flex-col items-center text-center">
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