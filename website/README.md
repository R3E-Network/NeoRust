# Neo Rust SDK Website

This is the official website for the Neo Rust SDK, a comprehensive Rust library for building applications on the Neo N3 blockchain ecosystem.

## Quick Start

To run the website locally:

```bash
# Install dependencies
npm install

# Start the development server
npm start
```

The site will be available at `http://localhost:8000`.

## Building

To build the website for production:

```bash
npm run build
```

This will generate optimized static files in the `public` directory.

## Deployment

This website is configured for automatic deployment with Netlify. Any changes pushed to the main branch will trigger a new build and deployment.

[![Deploy to Netlify](https://www.netlify.com/img/deploy/button.svg)](https://app.netlify.com/start/deploy?repository=https://github.com/neo-project/neo-rust)

## Features

- **Modern Documentation**: Beautiful, responsive documentation with sidebar navigation
- **Interactive Code Playground**: Try Neo Rust SDK code directly in your browser
- **Search Functionality**: Search across documentation, examples, and API reference
- **API Reference**: Complete API documentation automatically generated from source code
- **Examples Gallery**: Browse example code for common Neo Rust SDK tasks
- **Netlify Functions**: Serverless functions for dynamic features

## Structure

The website is built with Gatsby and follows this structure:

- `src/pages/`: Website pages (Home, Documentation, Examples, Playground, API Reference)
- `src/components/`: Reusable React components 
- `src/components/playground/`: Interactive code playground components
- `src/templates/`: Page templates for programmatically generated pages
- `src/styles/`: Global styles and Tailwind configuration
- `src/images/`: Images and assets
- `netlify/functions/`: Serverless functions for API endpoints

## Documentation

The documentation content is sourced from the `/docs` directory at the root of the Neo Rust SDK repository. Updates to the documentation should be made there.

## Playground

The interactive playground allows users to:

1. Experiment with Neo Rust SDK code directly in the browser
2. Run code examples and see their output
3. Choose from pre-defined examples to learn from

The playground code runs via a Netlify serverless function that safely executes the code in a sandbox.

## Search

The site includes a powerful search feature that:

1. Searches across all documentation, examples, and API reference
2. Provides instant search results
3. Helps users navigate to the most relevant content

## Development

### Prerequisites

- Node.js (v16 or newer)
- npm or yarn

### Adding New Pages

1. Create a new `.tsx` file in `src/pages/`
2. Import the `Layout` component
3. Create and export your page component

### Modifying Styles

This project uses Tailwind CSS. Global styles can be found in `src/styles/global.css`.

### Working with Netlify Functions

Serverless functions are located in `netlify/functions/`. To add a new function:

1. Create a new directory in `netlify/functions/`
2. Add an `index.js` file with your function code
3. Access the function at `/.netlify/functions/your-function-name`

### Working with the Playground

The playground functionality is implemented with:

1. Monaco Editor for the code editing experience
2. A Netlify function for code execution
3. React state management for the UI interaction

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the same license as the Neo Rust SDK (MIT).