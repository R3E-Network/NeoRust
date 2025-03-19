# Neo Rust SDK Website

A modern documentation website for the Neo Rust SDK.

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