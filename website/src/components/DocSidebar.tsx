import React, { useState } from 'react';
import { Link } from 'gatsby';

interface DocNavItem {
  title: string;
  path: string;
  items?: DocNavItem[];
}

interface DocSidebarProps {
  currentPath: string;
}

const DocSidebar: React.FC<DocSidebarProps> = ({ currentPath }) => {
  const [expandedSections, setExpandedSections] = useState<string[]>([
    'Getting Started',
    'Wallets',
    'Smart Contracts',
    'Neo X'
  ]);

  const toggleSection = (title: string) => {
    setExpandedSections(prev => 
      prev.includes(title) 
        ? prev.filter(section => section !== title)
        : [...prev, title]
    );
  };

  const isActive = (path: string) => {
    return currentPath === path;
  };

  const navigation: DocNavItem[] = [
    {
      title: 'Getting Started',
      path: '/docs/getting-started',
      items: [
        { title: 'Introduction', path: '/docs/getting-started' },
        { title: 'Installation', path: '/docs/getting-started/installation' },
        { title: 'Basic Concepts', path: '/docs/getting-started/basic-concepts' },
        { title: 'First Application', path: '/docs/getting-started/first-application' },
      ]
    },
    {
      title: 'Wallets',
      path: '/docs/wallets',
      items: [
        { title: 'Overview', path: '/docs/wallets' },
        { title: 'Creating Wallets', path: '/docs/wallets/creating-wallets' },
        { title: 'Loading Wallets', path: '/docs/wallets/loading-wallets' },
        { title: 'Message Signing', path: '/docs/wallets/message-signing' },
        { title: 'Hardware Wallets', path: '/docs/wallets/hardware-wallets' },
      ]
    },
    {
      title: 'Smart Contracts',
      path: '/docs/contracts',
      items: [
        { title: 'Overview', path: '/docs/contracts' },
        { title: 'Token Standards', path: '/docs/contracts/token-standards' },
        { title: 'System Contracts', path: '/docs/contracts/system-contracts' },
        { title: 'Contract Deployment', path: '/docs/contracts/contract-deployment' },
        { title: 'Contract Invocation', path: '/docs/contracts/contract-invocation' },
      ]
    },
    {
      title: 'Neo X',
      path: '/docs/neo-x',
      items: [
        { title: 'Overview', path: '/docs/neo-x' },
        { title: 'Bridge Operations', path: '/docs/neo-x/bridge' },
        { title: 'EVM Contracts', path: '/docs/neo-x/evm-contracts' },
      ]
    },
    {
      title: 'Cryptography',
      path: '/docs/crypto',
      items: [
        { title: 'Overview', path: '/docs/crypto' },
        { title: 'Keys & Key Pairs', path: '/docs/crypto/keys' },
        { title: 'Hashing', path: '/docs/crypto/hashing' },
        { title: 'NEP2 Standard', path: '/docs/crypto/nep2' },
      ]
    },
    {
      title: 'Utilities',
      path: '/docs/utils',
      items: [
        { title: 'Overview', path: '/docs/utils' },
        { title: 'Address Formatting', path: '/docs/utils/addresses' },
        { title: 'Base58 Encoding', path: '/docs/utils/base58' },
        { title: 'Script Builder', path: '/docs/utils/script-builder' },
      ]
    },
  ];

  return (
    <aside className="w-full md:w-64 pb-8 md:pb-0 border-b border-slate-700 md:border-b-0 md:border-r">
      <div className="md:sticky md:top-24 md:max-h-[calc(100vh-6rem)] md:overflow-y-auto p-4">
        <div className="space-y-1">
          {navigation.map((section) => (
            <div key={section.title} className="mb-4">
              <button 
                onClick={() => toggleSection(section.title)}
                className="w-full flex items-center justify-between py-2 px-4 rounded-lg hover:bg-slate-700 transition"
              >
                <span className="font-medium">{section.title}</span>
                <svg 
                  className={`w-4 h-4 transition-transform ${expandedSections.includes(section.title) ? 'transform rotate-180' : ''}`} 
                  fill="none" 
                  viewBox="0 0 24 24" 
                  stroke="currentColor"
                >
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M19 9l-7 7-7-7" />
                </svg>
              </button>
              
              {expandedSections.includes(section.title) && section.items && (
                <div className="mt-1 ml-4 space-y-1">
                  {section.items.map((item) => (
                    <Link
                      key={item.path}
                      to={item.path}
                      className={`doc-sidebar-link ${isActive(item.path) ? 'doc-sidebar-link-active' : ''}`}
                    >
                      {item.title}
                    </Link>
                  ))}
                </div>
              )}
            </div>
          ))}
        </div>
      </div>
    </aside>
  );
};

export default DocSidebar;