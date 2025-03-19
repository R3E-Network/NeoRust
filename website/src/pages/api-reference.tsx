import React, { useState } from 'react';
import { Link } from 'gatsby';
import Layout from '../components/Layout';
import Search from '../components/Search';

interface ApiModule {
  name: string;
  description: string;
  path: string;
  items: ApiItem[];
}

interface ApiItem {
  name: string;
  type: 'struct' | 'enum' | 'trait' | 'function' | 'module';
  description: string;
  path: string;
}

// This would be generated from the actual code documentation in a full implementation
const apiModules: ApiModule[] = [
  {
    name: 'neo_builder',
    description: 'Transaction and script building utilities',
    path: '/api-reference/neo_builder',
    items: [
      {
        name: 'TransactionBuilder',
        type: 'struct',
        description: 'Builder for Neo N3 transactions',
        path: '/api-reference/neo_builder/TransactionBuilder'
      },
      {
        name: 'ScriptBuilder',
        type: 'struct',
        description: 'Builder for Neo N3 scripts',
        path: '/api-reference/neo_builder/ScriptBuilder'
      },
      {
        name: 'Signer',
        type: 'struct',
        description: 'Transaction signer with scope settings',
        path: '/api-reference/neo_builder/Signer'
      },
      {
        name: 'Transaction',
        type: 'struct',
        description: 'Neo N3 transaction',
        path: '/api-reference/neo_builder/Transaction'
      }
    ]
  },
  {
    name: 'neo_crypto',
    description: 'Cryptographic primitives and utilities',
    path: '/api-reference/neo_crypto',
    items: [
      {
        name: 'KeyPair',
        type: 'struct',
        description: 'ECDSA key pair for Neo N3',
        path: '/api-reference/neo_crypto/KeyPair'
      },
      {
        name: 'Secp256r1PrivateKey',
        type: 'struct',
        description: 'Private key on the secp256r1 curve',
        path: '/api-reference/neo_crypto/Secp256r1PrivateKey'
      },
      {
        name: 'Secp256r1PublicKey',
        type: 'struct',
        description: 'Public key on the secp256r1 curve',
        path: '/api-reference/neo_crypto/Secp256r1PublicKey'
      },
      {
        name: 'Secp256r1Signature',
        type: 'struct',
        description: 'Signature on the secp256r1 curve',
        path: '/api-reference/neo_crypto/Secp256r1Signature'
      },
      {
        name: 'NEP2',
        type: 'struct',
        description: 'Implementation of the NEP-2 standard for encrypted private keys',
        path: '/api-reference/neo_crypto/NEP2'
      }
    ]
  },
  {
    name: 'neo_contract',
    description: 'Smart contract interfaces and utilities',
    path: '/api-reference/neo_contract',
    items: [
      {
        name: 'NeoToken',
        type: 'struct',
        description: 'Interface for the Neo governance token',
        path: '/api-reference/neo_contract/NeoToken'
      },
      {
        name: 'GasToken',
        type: 'struct',
        description: 'Interface for the Gas utility token',
        path: '/api-reference/neo_contract/GasToken'
      },
      {
        name: 'FungibleTokenContract',
        type: 'struct',
        description: 'Interface for NEP-17 fungible tokens',
        path: '/api-reference/neo_contract/FungibleTokenContract'
      },
      {
        name: 'NftContract',
        type: 'struct',
        description: 'Interface for NEP-11 non-fungible tokens',
        path: '/api-reference/neo_contract/NftContract'
      },
      {
        name: 'ContractManagement',
        type: 'struct',
        description: 'Interface for the contract management native contract',
        path: '/api-reference/neo_contract/ContractManagement'
      },
      {
        name: 'NameService',
        type: 'struct',
        description: 'Interface for the Neo Name Service',
        path: '/api-reference/neo_contract/NameService'
      }
    ]
  },
  {
    name: 'neo_wallets',
    description: 'Wallet management and signing',
    path: '/api-reference/neo_wallets',
    items: [
      {
        name: 'WalletSigner',
        type: 'struct',
        description: 'Signer implementation for Neo wallets',
        path: '/api-reference/neo_wallets/WalletSigner'
      },
      {
        name: 'Wallet',
        type: 'struct',
        description: 'Neo wallet implementation',
        path: '/api-reference/neo_wallets/Wallet'
      },
      {
        name: 'WalletTrait',
        type: 'trait',
        description: 'Trait for wallet implementations',
        path: '/api-reference/neo_wallets/WalletTrait'
      },
      {
        name: 'LedgerWallet',
        type: 'struct',
        description: 'Ledger hardware wallet integration',
        path: '/api-reference/neo_wallets/LedgerWallet'
      }
    ]
  },
  {
    name: 'neo_clients',
    description: 'RPC client implementations',
    path: '/api-reference/neo_clients',
    items: [
      {
        name: 'RpcClient',
        type: 'struct',
        description: 'Neo N3 JSON-RPC client',
        path: '/api-reference/neo_clients/RpcClient'
      },
      {
        name: 'HttpProvider',
        type: 'struct',
        description: 'HTTP provider for Neo N3 RPC',
        path: '/api-reference/neo_clients/HttpProvider'
      },
      {
        name: 'WebSocketProvider',
        type: 'struct',
        description: 'WebSocket provider for Neo N3 RPC',
        path: '/api-reference/neo_clients/WebSocketProvider'
      }
    ]
  },
  {
    name: 'neo_x',
    description: 'Neo X EVM compatibility layer',
    path: '/api-reference/neo_x',
    items: [
      {
        name: 'NeoXProvider',
        type: 'struct',
        description: 'Provider for connecting to Neo X',
        path: '/api-reference/neo_x/NeoXProvider'
      },
      {
        name: 'NeoXBridgeContract',
        type: 'struct',
        description: 'Interface for the Neo X bridge contract',
        path: '/api-reference/neo_x/NeoXBridgeContract'
      },
      {
        name: 'NeoXContract',
        type: 'struct',
        description: 'Interface for EVM contracts on Neo X',
        path: '/api-reference/neo_x/NeoXContract'
      }
    ]
  }
];

const ApiReferencePage: React.FC = () => {
  const [searchQuery, setSearchQuery] = useState('');
  const [expandedModules, setExpandedModules] = useState<string[]>([]);

  const toggleModule = (moduleName: string) => {
    setExpandedModules(prevState => 
      prevState.includes(moduleName)
        ? prevState.filter(name => name !== moduleName)
        : [...prevState, moduleName]
    );
  };

  // Filter modules and items based on search query
  const filteredModules = searchQuery
    ? apiModules.map(module => ({
        ...module,
        items: module.items.filter(item => 
          item.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
          item.description.toLowerCase().includes(searchQuery.toLowerCase())
        )
      })).filter(module => module.items.length > 0 || module.name.toLowerCase().includes(searchQuery.toLowerCase()) || module.description.toLowerCase().includes(searchQuery.toLowerCase()))
    : apiModules;

  return (
    <Layout
      title="API Reference | Neo Rust SDK"
      description="Complete API reference for the Neo Rust SDK"
    >
      <div className="container mx-auto px-4 py-12">
        <div className="mb-8">
          <h1 className="text-4xl font-bold mb-2">API Reference</h1>
          <p className="text-xl text-gray-300">
            Complete reference documentation for the Neo Rust SDK
          </p>
        </div>
        
        <div className="mb-8 max-w-xl">
          <div className="relative">
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="Search API..."
              className="w-full px-4 py-3 pr-10 rounded-lg bg-slate-700 border border-slate-600 focus:border-green-400 focus:outline-none focus:ring-1 focus:ring-green-400 text-gray-200 placeholder-gray-400"
            />
            <div className="absolute inset-y-0 right-0 flex items-center pr-3 pointer-events-none">
              <svg
                className="h-5 w-5 text-gray-400"
                xmlns="http://www.w3.org/2000/svg"
                viewBox="0 0 20 20"
                fill="currentColor"
                aria-hidden="true"
              >
                <path
                  fillRule="evenodd"
                  d="M8 4a4 4 0 100 8 4 4 0 000-8zM2 8a6 6 0 1110.89 3.476l4.817 4.817a1 1 0 01-1.414 1.414l-4.816-4.816A6 6 0 012 8z"
                  clipRule="evenodd"
                />
              </svg>
            </div>
          </div>
        </div>
        
        <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-8 mb-12">
          {apiModules.map(module => (
            <div key={module.name} className="p-6 rounded-xl bg-slate-800 border border-slate-700 shadow-lg">
              <h2 className="text-xl font-bold mb-2 text-green-400">{module.name}</h2>
              <p className="text-gray-300 mb-4">{module.description}</p>
              <Link
                to={module.path}
                className="text-green-400 flex items-center hover:underline"
              >
                View Module
                <svg className="w-4 h-4 ml-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M13 7l5 5m0 0l-5 5m5-5H6"></path>
                </svg>
              </Link>
            </div>
          ))}
        </div>
        
        <div className="bg-slate-800 rounded-lg border border-slate-700 overflow-hidden shadow-lg">
          <div className="p-5 border-b border-slate-700">
            <h2 className="text-2xl font-bold">Module Reference</h2>
          </div>
          
          <div className="divide-y divide-slate-700">
            {filteredModules.map(module => (
              <div key={module.name} className="p-0">
                <button
                  onClick={() => toggleModule(module.name)}
                  className="w-full text-left px-5 py-4 flex items-center justify-between hover:bg-slate-700/50 transition"
                >
                  <div>
                    <h3 className="text-lg font-semibold">{module.name}</h3>
                    <p className="text-sm text-gray-400">{module.description}</p>
                  </div>
                  <svg
                    className={`w-5 h-5 transition-transform ${expandedModules.includes(module.name) ? 'transform rotate-180' : ''}`}
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                  >
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M19 9l-7 7-7-7" />
                  </svg>
                </button>
                
                {expandedModules.includes(module.name) && (
                  <div className="px-5 pb-4 pt-2 bg-slate-800">
                    <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                      {module.items.map(item => (
                        <Link
                          key={item.name}
                          to={item.path}
                          className="flex items-start p-3 rounded-lg hover:bg-slate-700 transition group"
                        >
                          <div className={`flex-shrink-0 flex items-center justify-center w-10 h-10 rounded-lg mr-3 ${
                            item.type === 'struct' ? 'bg-blue-900/30 text-blue-400' :
                            item.type === 'enum' ? 'bg-purple-900/30 text-purple-400' :
                            item.type === 'trait' ? 'bg-yellow-900/30 text-yellow-400' :
                            'bg-green-900/30 text-green-400'
                          }`}>
                            {item.type === 'struct' ? 'S' :
                             item.type === 'enum' ? 'E' :
                             item.type === 'trait' ? 'T' :
                             item.type === 'function' ? 'F' : 'M'}
                          </div>
                          <div>
                            <h4 className="font-medium group-hover:text-green-400 transition">{item.name}</h4>
                            <p className="text-sm text-gray-400">{item.description}</p>
                          </div>
                        </Link>
                      ))}
                    </div>
                  </div>
                )}
              </div>
            ))}
            
            {filteredModules.length === 0 && (
              <div className="px-5 py-8 text-center">
                <p className="text-gray-400">No API modules found matching your search.</p>
              </div>
            )}
          </div>
        </div>
        
        <div className="mt-12 text-center">
          <p className="text-gray-400 mb-4">
            This API reference is generated from the source code documentation.
          </p>
          <a
            href="https://github.com/neo-project/neo-rust"
            className="text-green-400 hover:underline inline-flex items-center"
          >
            View Source on GitHub
            <svg className="ml-1 w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14"></path>
            </svg>
          </a>
        </div>
      </div>
    </Layout>
  );
};

export default ApiReferencePage;