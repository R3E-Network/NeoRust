import React from 'react';
import { Link } from 'gatsby';
import Layout from '../../components/Layout';
import Search from '../../components/Search';

interface Guide {
  title: string;
  description: string;
  category: string;
  difficulty: 'Beginner' | 'Intermediate' | 'Advanced';
  path: string;
  timeToRead: string;
}

const guides: Guide[] = [
  {
    title: 'Getting Started with Neo Rust SDK',
    description: 'Learn how to set up your development environment and start building with Neo Rust SDK',
    category: 'Basics',
    difficulty: 'Beginner',
    path: '/guides/getting-started',
    timeToRead: '10 min'
  },
  {
    title: 'Connecting to Neo N3 Networks',
    description: 'Learn how to connect to different Neo N3 networks (MainNet, TestNet, PrivateNet)',
    category: 'Basics',
    difficulty: 'Beginner',
    path: '/guides/connecting-to-networks',
    timeToRead: '8 min'
  },
  {
    title: 'Working with Neo Wallets',
    description: 'Create, load, and manage Neo wallets for transaction signing and account management',
    category: 'Wallets',
    difficulty: 'Beginner',
    path: '/guides/working-with-wallets',
    timeToRead: '15 min'
  },
  {
    title: 'Transaction Building and Signing',
    description: 'Learn how to build, sign, and send transactions on the Neo N3 blockchain',
    category: 'Transactions',
    difficulty: 'Intermediate',
    path: '/guides/transaction-building',
    timeToRead: '20 min'
  },
  {
    title: 'Using NEP-17 Tokens',
    description: 'Work with NEP-17 fungible tokens, including transfers and queries',
    category: 'Tokens',
    difficulty: 'Intermediate',
    path: '/guides/nep17-tokens',
    timeToRead: '15 min'
  },
  {
    title: 'Smart Contract Deployment',
    description: 'Deploy smart contracts to the Neo N3 blockchain',
    category: 'Smart Contracts',
    difficulty: 'Advanced',
    path: '/guides/contract-deployment',
    timeToRead: '25 min'
  },
  {
    title: 'Smart Contract Invocation',
    description: 'Invoke methods on Neo N3 smart contracts',
    category: 'Smart Contracts',
    difficulty: 'Intermediate',
    path: '/guides/contract-invocation',
    timeToRead: '18 min'
  },
  {
    title: 'Handling Events and Notifications',
    description: 'Subscribe to and process events and notifications from the Neo blockchain',
    category: 'Advanced',
    difficulty: 'Advanced',
    path: '/guides/events-notifications',
    timeToRead: '22 min'
  },
  {
    title: 'Neo X Integration',
    description: 'Learn how to use Neo X for EVM compatibility and cross-chain operations',
    category: 'Neo X',
    difficulty: 'Advanced',
    path: '/guides/neo-x-integration',
    timeToRead: '30 min'
  },
  {
    title: 'Security Best Practices',
    description: 'Best practices for secure Neo blockchain development',
    category: 'Security',
    difficulty: 'Intermediate',
    path: '/guides/security-best-practices',
    timeToRead: '15 min'
  },
  {
    title: 'Testing Neo Applications',
    description: 'Testing strategies and tools for Neo-based applications',
    category: 'Development',
    difficulty: 'Intermediate',
    path: '/guides/testing',
    timeToRead: '20 min'
  },
  {
    title: 'Error Handling',
    description: 'Comprehensive guide to error handling in Neo Rust SDK',
    category: 'Development',
    difficulty: 'Intermediate',
    path: '/guides/error-handling',
    timeToRead: '12 min'
  }
];

const GuidesPage: React.FC = () => {
  // Group guides by category
  const guidesGroupedByCategory = guides.reduce((acc, guide) => {
    if (!acc[guide.category]) {
      acc[guide.category] = [];
    }
    acc[guide.category].push(guide);
    return acc;
  }, {} as Record<string, Guide[]>);

  return (
    <Layout
      title="Guides | Neo Rust SDK"
      description="Comprehensive guides for building with Neo Rust SDK"
    >
      <div className="container mx-auto px-4 py-12">
        <div className="mb-8">
          <h1 className="text-4xl font-bold mb-2">Guides</h1>
          <p className="text-xl text-gray-300">
            Learn how to build applications on Neo N3 with step-by-step guides
          </p>
        </div>
        
        <div className="max-w-xl mb-12">
          <Search placeholder="Search guides..." />
        </div>
        
        <div className="grid grid-cols-1 gap-12">
          {Object.entries(guidesGroupedByCategory).map(([category, categoryGuides]) => (
            <div key={category}>
              <h2 className="text-2xl font-bold mb-6 pb-2 border-b border-slate-700">{category}</h2>
              <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-6">
                {categoryGuides.map(guide => (
                  <Link 
                    key={guide.path} 
                    to={guide.path}
                    className="p-6 rounded-xl bg-slate-800 border border-slate-700 shadow-lg hover:border-green-500/30 transition flex flex-col h-full"
                  >
                    <div className="flex-1">
                      <div className="flex items-center justify-between mb-3">
                        <span className={`text-xs font-medium rounded-full px-2.5 py-0.5 ${
                          guide.difficulty === 'Beginner' 
                            ? 'bg-green-900/30 text-green-400' 
                            : guide.difficulty === 'Intermediate'
                              ? 'bg-yellow-900/30 text-yellow-400'
                              : 'bg-red-900/30 text-red-400'
                        }`}>
                          {guide.difficulty}
                        </span>
                        <span className="text-xs text-gray-400">{guide.timeToRead}</span>
                      </div>
                      <h3 className="text-xl font-semibold mb-2">{guide.title}</h3>
                      <p className="text-gray-300">{guide.description}</p>
                    </div>
                    <div className="mt-4 pt-3 border-t border-slate-700 flex justify-end">
                      <span className="flex items-center text-green-400">
                        Read guide
                        <svg className="w-4 h-4 ml-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M13 7l5 5m0 0l-5 5m5-5H6"></path>
                        </svg>
                      </span>
                    </div>
                  </Link>
                ))}
              </div>
            </div>
          ))}
        </div>
        
        <div className="mt-16 p-8 bg-green-900/20 rounded-2xl border border-green-800/30">
          <div className="text-center">
            <h2 className="text-2xl font-bold mb-4">Can't find what you're looking for?</h2>
            <p className="text-xl text-gray-300 mb-6">
              Check out the API reference or ask the community for help
            </p>
            <div className="flex flex-wrap justify-center gap-4">
              <Link to="/api-reference" className="btn btn-primary">
                API Reference
              </Link>
              <a href="https://discord.gg/neo" className="btn btn-secondary">
                Join Discord
              </a>
            </div>
          </div>
        </div>
      </div>
    </Layout>
  );
};

export default GuidesPage;