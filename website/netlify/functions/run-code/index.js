// This function executes Rust code in a sandbox
// In a production environment, this would use a service like Rust Playground API
// or a containerized execution environment
const axios = require('axios');

exports.handler = async function (event, context) {
  // Only allow POST requests
  if (event.httpMethod !== 'POST') {
    return {
      statusCode: 405,
      body: JSON.stringify({ error: 'Method Not Allowed' }),
    };
  }

  try {
    // Parse request body
    const requestBody = JSON.parse(event.body);
    const { code } = requestBody;

    if (!code) {
      return {
        statusCode: 400,
        body: JSON.stringify({ error: 'No code provided' }),
      };
    }

    // For demonstration purposes, we'll use the Rust Playground API
    // In a real implementation, you might use AWS Lambda with custom runtime or other solutions
    const response = await axios.post('https://play.rust-lang.org/execute', {
      channel: 'stable',
      mode: 'debug',
      edition: '2021',
      crateType: 'bin',
      tests: false,
      code,
      backtrace: false,
    });

    // Process the response
    const result = response.data;
    
    // Return the execution result
    return {
      statusCode: 200,
      body: JSON.stringify({
        success: result.success,
        stdout: result.stdout || '',
        stderr: result.stderr || '',
        error: result.error || null,
      }),
    };
  } catch (error) {
    console.error('Error running code:', error);
    
    // Return a more friendly error message
    return {
      statusCode: 500,
      body: JSON.stringify({ 
        success: false,
        error: 'Failed to execute code', 
        details: error.message,
        stdout: '',
        stderr: 'An error occurred while trying to run your code. Please try again later.',
      }),
    };
  }
};