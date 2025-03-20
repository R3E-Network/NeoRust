# Neo Rust SDK Website

A modern documentation website for the Neo Rust SDK.

[![Netlify Status](https://api.netlify.com/api/v1/badges/99a932d5-49e7-49bc-bd2e-c9eab2a2e24a/deploy-status)](https://app.netlify.com/sites/neorust/deploys)

## Development Status

The website is currently using a static fallback due to MDX plugin compatibility issues. See [TROUBLESHOOTING.md](./TROUBLESHOOTING.md) for more details.

## Quick Start

```bash
# Install dependencies
npm install

# Build the static site
npm run build:static

# Serve the site locally
npx serve public
```

## Testing

The website has comprehensive unit tests for all major components:

```bash
# Run all tests
npm test

# Run tests with coverage reporting
npm run test:coverage

# Run tests in watch mode
npm test:watch

# Run a specific test file
npm run test:file src/__tests__/ComponentName.test.tsx
```

For more details, see the [testing documentation](./docs/testing.md).

## Architecture

The website is built with:

- **Gatsby**: React-based static site generator
- **Tailwind CSS**: Utility-first CSS framework
- **MDX**: Markdown + JSX for content
- **Monaco Editor**: For the code playground

## Directory Structure

- `/src`: React components and pages
- `/docs`: Documentation files (MDX/Markdown)
- `/static`: Static assets that bypass the build process
- `/public`: Generated output files
- `/scripts`: Utility scripts

## Static Fallback

Due to build issues with MDX configuration, we're temporarily using a static HTML fallback. To deploy:

1. The `netlify.toml` file has been configured to use the `build:static` script
2. The static site is a simplified version of the full design

## Future Improvements

Once the MDX configuration issues are resolved:

1. Update the DocTemplate component to work with Gatsby MDX v5+
2. Implement proper schema customization for MDX nodes
3. Re-enable the full site functionality