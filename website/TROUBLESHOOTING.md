# Troubleshooting the Neo Rust SDK Website

The website is currently experiencing build issues related to the MDX plugin and frontmatter handling.

## Error Analysis

The primary error is:
```
Cannot query field "frontmatter" on type "Mdx".
```

Other related errors:
```
Can't resolve '@mdx-js/react' in '/Users/jinghuiliao/git/NeoRust/docs/contracts'
```

## Fix Recommendations

### Approach 1: Update MDX Plugin Configuration

1. Ensure compatible versions of the MDX dependencies:

```bash
npm install @mdx-js/react@2.3.0 @mdx-js/mdx@2.3.0 gatsby-plugin-mdx@5.11.0
```

2. Update gatsby-config.ts with proper MDX options:

```ts
{
  resolve: "gatsby-plugin-mdx",
  options: {
    extensions: [".mdx", ".md"],
    mdxOptions: {
      remarkPlugins: [],
      rehypePlugins: [],
    },
    gatsbyRemarkPlugins: [
      // ...your plugins
    ],
  },
}
```

3. Create a proper schema definition in gatsby-node.js:

```js
exports.createSchemaCustomization = ({ actions }) => {
  const { createTypes } = actions;
  
  const typeDefs = `
    type Mdx implements Node {
      frontmatter: MdxFrontmatter
    }
    
    type MdxFrontmatter {
      title: String
      date: Date @dateformat
      description: String
      slug: String
      category: String
      tags: [String]
      order: Int
    }
  `;
  
  createTypes(typeDefs);
};
```

### Approach 2: Alternative Documentation Setup

If the MDX issues persist, consider using a different documentation framework:

1. **Docusaurus**: Has excellent support for Markdown/MDX
   - https://docusaurus.io/

2. **VitePress**: Lightweight and fast documentation site generator
   - https://vitepress.dev/

3. **Nextra**: Next.js-based documentation framework
   - https://nextra.site/

## Manual Build Workaround

For now, we've provided a static HTML landing page in the public directory. To deploy the site:

```bash
npm run build:static
```

This will copy the static assets to the public directory without running the Gatsby build process.