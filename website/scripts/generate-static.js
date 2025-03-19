/**
 * Simple script to generate static content when Gatsby build fails
 */

const fs = require('fs');
const path = require('path');

// Create public directory if it doesn't exist
const publicDir = path.join(__dirname, '..', 'public');
if (!fs.existsSync(publicDir)) {
  fs.mkdirSync(publicDir, { recursive: true });
}

// Generate a robots.txt file
const robotsTxt = `User-agent: *
Allow: /
`;
fs.writeFileSync(path.join(publicDir, 'robots.txt'), robotsTxt);

// Generate a .nojekyll file for GitHub Pages
fs.writeFileSync(path.join(publicDir, '.nojekyll'), '');

// Create a 404 page that redirects to index
const notFoundHtml = `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Page Not Found | Neo Rust SDK</title>
  <meta http-equiv="refresh" content="0;url=/" />
  <style>
    body {
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, 'Open Sans', 'Helvetica Neue', sans-serif;
      line-height: 1.6;
      color: #f8fafc;
      background-color: #0f172a;
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      height: 100vh;
      margin: 0;
      text-align: center;
    }
    h1 {
      color: #4CFFB3;
    }
    p {
      margin: 1rem 0;
    }
    a {
      color: #4CFFB3;
      text-decoration: none;
    }
    a:hover {
      text-decoration: underline;
    }
  </style>
</head>
<body>
  <h1>Page Not Found</h1>
  <p>Redirecting to the homepage...</p>
  <p>If you are not redirected automatically, <a href="/">click here</a>.</p>
</body>
</html>`;

fs.writeFileSync(path.join(publicDir, '404.html'), notFoundHtml);

console.log('Static files generated successfully.');