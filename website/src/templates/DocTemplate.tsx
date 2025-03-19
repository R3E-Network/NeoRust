import React from 'react';
import { graphql } from 'gatsby';
import { MDXProvider } from '@mdx-js/react';
import Layout from '../components/Layout';
import DocSidebar from '../components/DocSidebar';
import CodeBlock from '../components/CodeBlock';

interface DocTemplateProps {
  data: {
    mdx: {
      frontmatter: {
        title: string;
        description: string;
      };
      body: string;
      // For Gatsby MDX v5+
      internal: {
        contentFilePath: string;
      };
    };
  };
  path: string;
  children: React.ReactNode;
}

const components = {
  pre: (props: any) => {
    const className = props.children.props.className || '';
    const matches = className.match(/language-(?<lang>.*)/);
    const language = matches?.groups?.lang || '';
    const code = props.children.props.children.trim();
    
    return (
      <CodeBlock
        code={code}
        language={language}
        filename={props.children.props['data-filename']}
      />
    );
  },
};

const DocTemplate: React.FC<DocTemplateProps> = ({ data, path, children }) => {
  const { frontmatter } = data.mdx;
  
  return (
    <Layout
      title={`${frontmatter.title} | Neo Rust SDK`}
      description={frontmatter.description}
    >
      <div className="container mx-auto px-4 py-12">
        <div className="flex flex-col md:flex-row">
          <DocSidebar currentPath={path} />
          
          <main className="flex-1 md:pl-8 pt-8 md:pt-0">
            <div className="max-w-3xl">
              <h1 className="text-4xl font-bold mb-6">{frontmatter.title}</h1>
              
              <div className="doc-content">
                <MDXProvider components={components}>
                  {children}
                </MDXProvider>
              </div>
              
              <div className="mt-16 pt-8 border-t border-slate-700">
                <div className="flex justify-between">
                  <a href="/" className="text-green-400 flex items-center hover:underline">
                    <svg className="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M15 19l-7-7 7-7"></path>
                    </svg>
                    Previous
                  </a>
                  <a href="/" className="text-green-400 flex items-center hover:underline">
                    Next
                    <svg className="w-4 h-4 ml-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M9 5l7 7-7 7"></path>
                    </svg>
                  </a>
                </div>
              </div>
            </div>
          </main>
        </div>
      </div>
    </Layout>
  );
};

export const query = graphql`
  query DocQuery($id: String) {
    mdx(id: { eq: $id }) {
      frontmatter {
        title
        description
      }
      internal {
        contentFilePath
      }
    }
  }
`;

export default DocTemplate;