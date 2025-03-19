// A simple Netlify function to get the current block count from a Neo N3 node
const axios = require('axios');

exports.handler = async function (event, context) {
  try {
    // RPC call to a Neo N3 node
    const response = await axios.post(
      'https://mainnet1.neo.org:443',
      {
        jsonrpc: '2.0',
        id: 1,
        method: 'getblockcount',
        params: []
      }
    );

    // Return the block count
    return {
      statusCode: 200,
      body: JSON.stringify({ 
        blockCount: response.data.result,
        timestamp: new Date().toISOString()
      })
    };
  } catch (error) {
    return {
      statusCode: 500,
      body: JSON.stringify({ 
        error: 'Failed to fetch block count', 
        message: error.message 
      })
    };
  }
};