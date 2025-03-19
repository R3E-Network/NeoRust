const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');
const crypto = require('crypto');
const os = require('os');

// Maximum execution time (in milliseconds)
const MAX_EXECUTION_TIME = 5000;

// Sandbox configuration - limits for security
const PLAYGROUND_LIMITS = `
#![allow(unused)]
#![forbid(unsafe_code)]
`;

exports.handler = async function(event, context) {
  // Only allow POST requests
  if (event.httpMethod !== 'POST') {
    return {
      statusCode: 405,
      body: JSON.stringify({ error: 'Method Not Allowed' }),
      headers: { 'Content-Type': 'application/json' }
    };
  }

  try {
    const { code, crate_dependencies = {} } = JSON.parse(event.body);
    
    if (!code || typeof code !== 'string') {
      return {
        statusCode: 400,
        body: JSON.stringify({ error: 'Code is required' }),
        headers: { 'Content-Type': 'application/json' }
      };
    }

    // Validate code for security (naive implementation - would need more robust filtering in production)
    if (code.includes('std::process') || code.includes('std::fs') || code.includes('unsafe')) {
      return {
        statusCode: 400,
        body: JSON.stringify({ error: 'Code contains forbidden elements for security reasons' }),
        headers: { 'Content-Type': 'application/json' }
      };
    }

    // For a real production environment, you would use a secure container or external service
    // This is a simplified example that simulates code execution

    // Simulate Neo Rust SDK output based on code content
    let output = '';
    
    if (code.includes('Wallet::new')) {
      output = 'New wallet created successfully\nAddress: NfgHwwTi3wHAS8aFAN243C5vGbkYDpqLHP';
    } else if (code.includes('invoke_function')) {
      output = 'Contract invocation successful\nTransaction: 0x7e2f9c9932e5d81ab12658af9ec5c0e56c37e489fc77ffe5e390ee8c9f9411cd\nResult: {"state":"HALT","gasconsumed":"2.032","stack":[{"type":"Integer","value":"100000000"}]}';
    } else if (code.includes('TransactionBuilder')) {
      output = 'Transaction created successfully\nHash: 0x9f3034d9b38acb782045821a6e0c35550b3ded53e883478c25db66f7ecab14eb\nSize: 334 bytes\nFees: 0.032 GAS';
    } else if (code.includes('NeoClient')) {
      output = 'Connected to Neo N3 TestNet\nHeight: 2,845,632\nVersion: 3.5.0';
    } else {
      output = 'Compilation successful\nExecution successful';
    }

    // For sandbox demos, add some latency to simulate execution time
    await new Promise(resolve => setTimeout(resolve, 800));

    return {
      statusCode: 200,
      body: JSON.stringify({ output }),
      headers: { 'Content-Type': 'application/json' }
    };
  } catch (error) {
    console.error('Error executing code:', error);
    
    return {
      statusCode: 500,
      body: JSON.stringify({ error: 'Internal Server Error' }),
      headers: { 'Content-Type': 'application/json' }
    };
  }
};