import React, { useState, useCallback } from 'react';
import Layout from '../components/Layout';
import Editor from '../components/playground/Editor';
import Console from '../components/playground/Console';

// Initial sample code
const DEFAULT_CODE = `use neo3::prelude::*;

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

const EXAMPLES = [
  {
    name: 'Basic Connection',
    description: 'Connect to Neo N3 and get basic blockchain info',
    code: DEFAULT_CODE,
  },
  {
    name: 'Wallet Creation',
    description: 'Create a Neo wallet and get the address',
    code: `use neo3::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new random key pair
    let key_pair = KeyPair::new_random()?;
    
    // Get the address from the key pair
    let address = key_pair.get_address();
    println!("Created new address: {}", address);
    
    // Create a wallet signer
    let wallet = WalletSigner::new_with_signer(key_pair.clone(), address.clone());
    
    // Sign a message
    let message = b"Hello, Neo!";
    let signature = wallet.sign_message(message).await?;
    println!("Message signed successfully");
    println!("Signature: {:?}", signature);
    
    Ok(())
}`,
  },
  {
    name: 'GAS Token Info',
    description: 'Get information about the GAS token',
    code: `use neo3::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to Neo N3 TestNet
    let provider = HttpProvider::new("https://testnet1.neo.org:443")?;
    let client = RpcClient::new(provider);
    
    // Create an instance of the GAS token contract
    let gas_token = GasToken::new(&client);
    
    // Get token information
    let symbol = gas_token.symbol().await?;
    let decimals = gas_token.decimals().await?;
    let total_supply = gas_token.total_supply().await?;
    
    println!("Token: {} (Decimals: {})", symbol, decimals);
    println!("Total Supply: {}", total_supply);
    
    Ok(())
}`,
  },
  {
    name: 'NEO Token Info',
    description: 'Get information about the NEO token and governance',
    code: `use neo3::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to Neo N3 TestNet
    let provider = HttpProvider::new("https://testnet1.neo.org:443")?;
    let client = RpcClient::new(provider);
    
    // Create an instance of the NEO token contract
    let neo_token = NeoToken::new(&client);
    
    // Get token information
    let symbol = neo_token.symbol().await?;
    let decimals = neo_token.decimals().await?;
    let total_supply = neo_token.total_supply().await?;
    
    println!("Token: {} (Decimals: {})", symbol, decimals);
    println!("Total Supply: {}", total_supply);
    
    // Get committee members
    let committee = neo_token.get_committee().await?;
    println!("Committee Members: {}", committee.len());
    
    Ok(())
}`,
  },
];

const PlaygroundPage: React.FC = () => {
  const [code, setCode] = useState(DEFAULT_CODE);
  const [output, setOutput] = useState<string[]>([]);
  const [isRunning, setIsRunning] = useState(false);
  const [selectedExample, setSelectedExample] = useState(0);

  const runCode = useCallback(async () => {
    setIsRunning(true);
    setOutput(['Running code...']);

    try {
      // In a real app, this would call the Netlify function
      // For demo purposes, we'll simulate a successful run
      
      // Simulate API call delay
      await new Promise(resolve => setTimeout(resolve, 2000));
      
      // Simulate successful output
      setOutput([
        'Compiling playground v0.1.0',
        'Finished dev [unoptimized + debuginfo] target(s) in 1.54s',
        'Running `target/debug/playground`',
        'Current block height: 1234567',
        'New address: NXxVXJJpwXADM65gJhYPQ5xHiZr17KQj5T',
      ]);
    } catch (error) {
      setOutput([
        'error: Failed to run code',
        `error: ${error instanceof Error ? error.message : String(error)}`
      ]);
    } finally {
      setIsRunning(false);
    }
  }, [code]);

  const handleExampleChange = (index: number) => {
    setSelectedExample(index);
    setCode(EXAMPLES[index].code);
    setOutput([]);
  };

  return (
    <Layout
      title="Playground | Neo Rust SDK"
      description="Interactive playground for experimenting with Neo Rust SDK code"
    >
      <div className="container mx-auto px-4 py-12">
        <h1 className="text-4xl font-bold mb-2">Neo Rust SDK Playground</h1>
        <p className="text-xl text-gray-300 mb-8">Experiment with Neo Rust SDK code in your browser</p>
        
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-8">
          <div className="md:col-span-2">
            {/* Control bar */}
            <div className="flex flex-wrap justify-between items-center mb-4 gap-4">
              <div className="flex flex-wrap items-center gap-4">
                <button
                  onClick={runCode}
                  disabled={isRunning}
                  className="btn btn-primary flex items-center"
                >
                  {isRunning ? (
                    <>
                      <svg className="animate-spin -ml-1 mr-2 h-4 w-4" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                        <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                        <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                      </svg>
                      Running...
                    </>
                  ) : (
                    <>
                      <svg className="mr-2 h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                      </svg>
                      Run Code
                    </>
                  )}
                </button>
                
                <button
                  onClick={() => setOutput([])}
                  className="btn btn-secondary"
                  disabled={output.length === 0 || isRunning}
                >
                  Clear Console
                </button>
              </div>
              
              <div className="flex items-center">
                <span className="text-gray-400 mr-2">Example:</span>
                <select
                  value={selectedExample}
                  onChange={(e) => handleExampleChange(Number(e.target.value))}
                  className="bg-slate-700 border border-slate-600 text-white rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-green-500"
                >
                  {EXAMPLES.map((example, index) => (
                    <option key={index} value={index}>
                      {example.name}
                    </option>
                  ))}
                </select>
              </div>
            </div>
            
            {/* Editor */}
            <Editor
              initialCode={code}
              onChange={setCode}
              height="400px"
            />

            {/* Console output */}
            <div className="mt-6">
              <Console output={output} isLoading={isRunning} />
            </div>
          </div>
          
          <div className="md:col-span-1">
            <div className="sticky top-24">
              <div className="p-6 rounded-xl bg-slate-800 border border-slate-700 shadow-lg">
                <h2 className="text-xl font-bold mb-4">Example Details</h2>
                <h3 className="text-green-400 font-medium mb-2">{EXAMPLES[selectedExample].name}</h3>
                <p className="text-gray-300 mb-4">{EXAMPLES[selectedExample].description}</p>
                
                <h4 className="font-medium mb-2">Examples</h4>
                <ul className="space-y-2">
                  {EXAMPLES.map((example, index) => (
                    <li key={index}>
                      <button
                        onClick={() => handleExampleChange(index)}
                        className={`block w-full text-left px-3 py-2 rounded-lg transition ${
                          selectedExample === index
                            ? 'bg-green-900/30 text-green-400'
                            : 'hover:bg-slate-700 text-gray-300'
                        }`}
                      >
                        {example.name}
                      </button>
                    </li>
                  ))}
                </ul>
                
                <div className="mt-6">
                  <h4 className="font-medium mb-2">Notes</h4>
                  <p className="text-gray-400 text-sm">
                    This playground simulates execution of Neo Rust SDK code. In a real 
                    deployment, code would run in a secure sandboxed environment.
                  </p>
                  <p className="text-gray-400 text-sm mt-2">
                    For more complex examples, please clone the repository and run locally.
                  </p>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </Layout>
  );
};

export default PlaygroundPage;