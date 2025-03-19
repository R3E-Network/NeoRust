import React, { useState, useEffect, useRef } from 'react';
import { Link, navigate } from 'gatsby';

interface SearchResult {
  title: string;
  path: string;
  excerpt: string;
  category: string;
}

// This would normally be populated by a search index from a plugin like elasticlunr
// For demo purposes, we'll use a small static set of results
const mockResults: SearchResult[] = [
  {
    title: 'Getting Started',
    path: '/docs/getting-started',
    excerpt: 'Learn how to install and start using the Neo Rust SDK.',
    category: 'Documentation'
  },
  {
    title: 'Wallet Management',
    path: '/docs/wallets',
    excerpt: 'Create and manage Neo wallets securely with the SDK.',
    category: 'Documentation'
  },
  {
    title: 'Smart Contracts',
    path: '/docs/contracts',
    excerpt: 'Deploy and interact with smart contracts on the Neo blockchain.',
    category: 'Documentation'
  },
  {
    title: 'Message Signing',
    path: '/docs/wallets/message-signing',
    excerpt: 'Sign messages using Neo wallets to prove identity.',
    category: 'Documentation'
  },
  {
    title: 'NEP-17 Tokens',
    path: '/docs/contracts/token-standards',
    excerpt: 'Learn about NEP-17 fungible token standard implementation.',
    category: 'Documentation'
  },
  {
    title: 'Bridge Operations',
    path: '/docs/neo-x/bridge',
    excerpt: 'Transfer assets between Neo N3 and Neo X chains.',
    category: 'Documentation'
  },
  {
    title: 'API Reference',
    path: '/api-reference',
    excerpt: 'Complete API reference for the Neo Rust SDK.',
    category: 'Reference'
  }
];

interface SearchProps {
  placeholder?: string;
}

const Search: React.FC<SearchProps> = ({ placeholder = 'Search documentation...' }) => {
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<SearchResult[]>([]);
  const [isOpen, setIsOpen] = useState(false);
  const searchRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (searchRef.current && !searchRef.current.contains(event.target as Node)) {
        setIsOpen(false);
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, []);

  useEffect(() => {
    if (query.length > 1) {
      // In a real app, this would call a search API or use a search index
      const filtered = mockResults.filter(result => 
        result.title.toLowerCase().includes(query.toLowerCase()) || 
        result.excerpt.toLowerCase().includes(query.toLowerCase())
      );
      setResults(filtered);
      setIsOpen(true);
    } else {
      setResults([]);
      setIsOpen(false);
    }
  }, [query]);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && results.length > 0) {
      navigate(results[0].path);
      setIsOpen(false);
      setQuery('');
    }
  };

  return (
    <div className="relative" ref={searchRef}>
      <div className="relative">
        <input
          type="text"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder={placeholder}
          className="w-full md:w-64 px-4 py-2 pr-10 rounded-lg bg-slate-700 border border-slate-600 focus:border-green-400 focus:outline-none focus:ring-1 focus:ring-green-400 text-gray-200 placeholder-gray-400"
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

      {isOpen && results.length > 0 && (
        <div className="absolute z-10 w-full mt-2 bg-slate-800 rounded-lg shadow-lg border border-slate-700 max-h-96 overflow-y-auto">
          <ul className="py-2">
            {results.map((result, index) => (
              <li key={index}>
                <Link
                  to={result.path}
                  className="block px-4 py-3 hover:bg-slate-700"
                  onClick={() => {
                    setIsOpen(false);
                    setQuery('');
                  }}
                >
                  <div className="flex items-center justify-between">
                    <span className="text-green-400 font-medium">{result.title}</span>
                    <span className="text-xs text-gray-400">{result.category}</span>
                  </div>
                  <p className="text-sm text-gray-300 mt-1">{result.excerpt}</p>
                </Link>
              </li>
            ))}
          </ul>
        </div>
      )}
    </div>
  );
};

export default Search;