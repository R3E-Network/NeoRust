// This is a simple search implementation for a static site
// In a production environment, you would use a proper search index

// Sample search data - in a real implementation, this would be generated from your content
const searchIndex = [
  {
    id: 'getting-started',
    title: 'Getting Started with Neo Rust SDK',
    section: 'Documentation',
    url: '/docs/getting-started/',
    content: 'Learn how to install and use the Neo Rust SDK for blockchain development',
    tags: ['installation', 'setup', 'introduction', 'beginner']
  },
  {
    id: 'wallets',
    title: 'Wallet Management',
    section: 'Documentation',
    url: '/docs/wallets/',
    content: 'Create, load, and manage Neo wallets with the Neo Rust SDK',
    tags: ['wallet', 'account', 'keys', 'address']
  },
  {
    id: 'transactions',
    title: 'Transaction Building',
    section: 'Documentation',
    url: '/docs/transactions/',
    content: 'Build and sign transactions with the Neo Rust SDK',
    tags: ['transaction', 'signing', 'blockchain', 'witness']
  },
  {
    id: 'contracts',
    title: 'Smart Contract Interaction',
    section: 'Documentation',
    url: '/docs/contracts/',
    content: 'Deploy and invoke smart contracts with the Neo Rust SDK',
    tags: ['contract', 'nep17', 'deployment', 'invocation']
  },
  {
    id: 'crypto',
    title: 'Cryptography',
    section: 'Documentation',
    url: '/docs/crypto/',
    content: 'Cryptographic operations in the Neo Rust SDK',
    tags: ['encryption', 'hash', 'keys', 'signature']
  },
  {
    id: 'wallet-api',
    title: 'Wallet Module API',
    section: 'API Reference',
    url: '/api/wallet/',
    content: 'Comprehensive API documentation for the wallet module',
    tags: ['api', 'wallet', 'reference', 'functions']
  },
  {
    id: 'transaction-api',
    title: 'Transaction Module API',
    section: 'API Reference',
    url: '/api/transaction/',
    content: 'Comprehensive API documentation for the transaction module',
    tags: ['api', 'transaction', 'reference', 'functions']
  },
  {
    id: 'contract-api',
    title: 'Contract Module API',
    section: 'API Reference',
    url: '/api/contract/',
    content: 'Comprehensive API documentation for the contract module',
    tags: ['api', 'contract', 'reference', 'functions']
  },
  {
    id: 'wallet-example',
    title: 'Creating a Wallet',
    section: 'Examples',
    url: '/examples/wallet-creation/',
    content: 'Example code for creating and managing Neo wallets',
    tags: ['example', 'wallet', 'creation', 'code']
  },
  {
    id: 'transaction-example',
    title: 'Building Transactions',
    section: 'Examples',
    url: '/examples/create-transaction/',
    content: 'Example code for building and signing transactions',
    tags: ['example', 'transaction', 'signing', 'code']
  }
];

// Simple search function that looks for matches in title, content, and tags
function searchContent(query) {
  if (!query || query.trim() === '') {
    return [];
  }
  
  const searchTerms = query.toLowerCase().trim().split(/\s+/);
  
  return searchIndex
    .filter(item => {
      const searchableText = `${item.title} ${item.content} ${item.tags.join(' ')}`.toLowerCase();
      return searchTerms.every(term => searchableText.includes(term));
    })
    .map(item => ({
      id: item.id,
      title: item.title,
      section: item.section,
      url: item.url,
      preview: item.content.substring(0, 120) + (item.content.length > 120 ? '...' : '')
    }))
    .slice(0, 10); // Limit to 10 results
}

exports.handler = async function(event, context) {
  // CORS headers to allow requests from your domain
  const headers = {
    'Access-Control-Allow-Origin': '*', // In production, specify your domain
    'Access-Control-Allow-Headers': 'Content-Type',
    'Access-Control-Allow-Methods': 'GET, POST, OPTIONS',
    'Content-Type': 'application/json'
  };
  
  // Handle OPTIONS request for CORS
  if (event.httpMethod === 'OPTIONS') {
    return {
      statusCode: 204,
      headers,
      body: ''
    };
  }
  
  // Only allow GET requests for search
  if (event.httpMethod !== 'GET') {
    return {
      statusCode: 405,
      headers,
      body: JSON.stringify({ error: 'Method Not Allowed' })
    };
  }
  
  try {
    const query = event.queryStringParameters?.query || '';
    const results = searchContent(query);
    
    return {
      statusCode: 200,
      headers,
      body: JSON.stringify({ results })
    };
  } catch (error) {
    console.error('Error processing search:', error);
    
    return {
      statusCode: 500,
      headers,
      body: JSON.stringify({ error: 'Internal Server Error' })
    };
  }
};