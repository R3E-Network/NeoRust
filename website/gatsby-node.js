/**
 * Implement Gatsby's Node APIs in this file.
 * See: https://www.gatsbyjs.com/docs/reference/config-files/gatsby-node/
 */

const { createFilePath } = require('gatsby-source-filesystem');
const path = require('path');

exports.createSchemaCustomization = ({ actions }) => {
  const { createTypes } = actions;
  
  // Define the schema for MDX nodes to include frontmatter
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

// Process MDX files to create pages
exports.onCreateNode = ({ node, actions, getNode }) => {
  const { createNodeField } = actions;
  
  if (node.internal.type === 'Mdx') {
    const slug = node.frontmatter?.slug || createFilePath({ node, getNode });
    
    createNodeField({
      node,
      name: 'slug',
      value: slug,
    });
  }
};

// Create pages from MDX content
exports.createPages = async ({ graphql, actions, reporter }) => {
  const { createPage } = actions;
  
  const docTemplate = path.resolve(`./src/templates/DocTemplate.tsx`);

  // Query all MDX files
  const result = await graphql(`
    query {
      allMdx {
        nodes {
          id
          fields {
            slug
          }
          internal {
            contentFilePath
          }
        }
      }
    }
  `);

  if (result.errors) {
    reporter.panicOnBuild('Error loading MDX files', result.errors);
    return;
  }

  // Create pages for each MDX file
  const posts = result.data.allMdx.nodes;
  
  // Filter out files that should be excluded (e.g., from src, book, theme directories)
  const filteredPosts = posts.filter(node => {
    const filePath = node.internal.contentFilePath;
    return !filePath.includes('/src/') && 
           !filePath.includes('/book/') && 
           !filePath.includes('/theme/') &&
           !filePath.includes('/assets/');
  });
  
  filteredPosts.forEach(node => {
    createPage({
      path: `docs${node.fields.slug}`,
      component: `${docTemplate}?__contentFilePath=${node.internal.contentFilePath}`,
      context: {
        id: node.id,
      },
    });
  });
};