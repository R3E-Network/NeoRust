const axios = require('axios');

// Neo N3 TestNet RPC endpoints
const NEO_RPC_URL = 'https://testnet1.neo.coz.io:443';

// Cache time in milliseconds (5 minutes)
const CACHE_TIME = 5 * 60 * 1000;

// In-memory cache
let cache = {
  data: null,
  timestamp: 0
};

async function fetchBlockchainStatus() {
  // Check cache first
  const now = Date.now();
  if (cache.data && now - cache.timestamp < CACHE_TIME) {
    return cache.data;
  }
  
  try {
    // Get block count (height)
    const blockCountResponse = await axios.post(NEO_RPC_URL, {
      jsonrpc: '2.0',
      id: 1,
      method: 'getblockcount',
      params: []
    });
    
    const blockCount = blockCountResponse.data.result;
    
    // Get latest block
    const latestBlockResponse = await axios.post(NEO_RPC_URL, {
      jsonrpc: '2.0',
      id: 2,
      method: 'getblock',
      params: [blockCount - 1, 1]
    });
    
    const latestBlock = latestBlockResponse.data.result;
    
    // Get version
    const versionResponse = await axios.post(NEO_RPC_URL, {
      jsonrpc: '2.0',
      id: 3,
      method: 'getversion',
      params: []
    });
    
    const version = versionResponse.data.result;
    
    // Format the response
    const status = {
      height: blockCount,
      latestBlockHash: latestBlock.hash,
      latestBlockTime: new Date(latestBlock.time * 1000).toISOString(),
      latestBlockTx: latestBlock.tx.length,
      version: version.useragent,
      protocol: {
        network: version.network,
        validatorsCount: version.tcpport,
      },
      timestamp: now
    };
    
    // Update cache
    cache = {
      data: status,
      timestamp: now
    };
    
    return status;
  } catch (error) {
    console.error('Error fetching blockchain status:', error);
    
    // If cache is available, use it even if expired
    if (cache.data) {
      return {
        ...cache.data,
        fromCache: true,
        cacheAge: (now - cache.timestamp) / 1000
      };
    }
    
    // Fallback mock data if nothing is available
    return {
      height: 2845632,
      latestBlockHash: '0x3e85983ec94d6e6b281efe3f8636fce98beaf642f8c2a36ac4b04e1c3e6eb792',
      latestBlockTime: new Date().toISOString(),
      latestBlockTx: 5,
      version: 'NEO:3.5.0',
      protocol: {
        network: 844378958,
        validatorsCount: 4,
      },
      mocked: true,
      timestamp: now
    };
  }
}

exports.handler = async function(event, context) {
  // CORS headers to allow requests from your domain
  const headers = {
    'Access-Control-Allow-Origin': '*', // In production, specify your domain
    'Access-Control-Allow-Headers': 'Content-Type',
    'Access-Control-Allow-Methods': 'GET, OPTIONS',
    'Content-Type': 'application/json',
    'Cache-Control': 'public, max-age=300' // 5 minutes
  };
  
  // Handle OPTIONS request for CORS
  if (event.httpMethod === 'OPTIONS') {
    return {
      statusCode: 204,
      headers,
      body: ''
    };
  }
  
  // Only allow GET requests
  if (event.httpMethod !== 'GET') {
    return {
      statusCode: 405,
      headers,
      body: JSON.stringify({ error: 'Method Not Allowed' })
    };
  }
  
  try {
    const status = await fetchBlockchainStatus();
    
    return {
      statusCode: 200,
      headers,
      body: JSON.stringify({ status })
    };
  } catch (error) {
    console.error('Error handling blockchain status request:', error);
    
    return {
      statusCode: 500,
      headers,
      body: JSON.stringify({ error: 'Internal Server Error' })
    };
  }
};