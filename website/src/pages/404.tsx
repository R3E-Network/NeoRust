import React from 'react';
import { Link } from 'gatsby';
import Layout from '../components/Layout';

const NotFoundPage: React.FC = () => {
  return (
    <Layout title="Page Not Found | Neo Rust SDK" description="The page you're looking for doesn't exist.">
      <div className="container mx-auto px-4 py-32 flex flex-col items-center justify-center text-center">
        <h1 className="text-6xl md:text-8xl font-bold text-green-400">404</h1>
        <h2 className="text-2xl md:text-3xl font-bold mt-4 mb-6">Page Not Found</h2>
        <p className="text-xl text-gray-300 max-w-lg mb-8">
          The page you're looking for doesn't exist or has been moved.
        </p>
        <Link to="/" className="btn btn-primary">
          Go to Homepage
        </Link>
      </div>
    </Layout>
  );
};

export default NotFoundPage;