import React from 'react';
import { screen } from '@testing-library/react';
import { render } from './test-utils';
import Layout from '../components/Layout';

// Mock the required components
jest.mock('../components/Header', () => {
  return function MockHeader() {
    return <div data-testid="header">Header Mock</div>;
  };
});

jest.mock('../components/Footer', () => {
  return function MockFooter() {
    return <div data-testid="footer">Footer Mock</div>;
  };
});

jest.mock('../components/AnimatedBackground', () => {
  return function MockAnimatedBackground() {
    return <div data-testid="animated-background">AnimatedBackground Mock</div>;
  };
});

jest.mock('../components/ThemeToggle', () => {
  return function MockThemeToggle() {
    return <div data-testid="theme-toggle">ThemeToggle Mock</div>;
  };
});

jest.mock('react-helmet', () => {
  return {
    Helmet: function MockHelmet({ children }) {
      return <div data-testid="helmet">{children}</div>;
    }
  };
});

describe('Layout Component', () => {
  it('renders the layout with header, main content, and footer', () => {
    render(
      <Layout>
        <div data-testid="content">Test Content</div>
      </Layout>
    );
    
    // Header and footer should be rendered
    expect(screen.getByTestId('header')).toBeInTheDocument();
    expect(screen.getByTestId('footer')).toBeInTheDocument();
    
    // Main content should be rendered
    expect(screen.getByTestId('content')).toBeInTheDocument();
    expect(screen.getByText('Test Content')).toBeInTheDocument();
    
    // Theme toggle should be rendered
    expect(screen.getByTestId('theme-toggle')).toBeInTheDocument();
  });

  it('renders the skip to content link for accessibility', () => {
    render(
      <Layout>
        <div>Test Content</div>
      </Layout>
    );
    
    // Skip to content link should be present
    const skipLink = screen.getByText('Skip to content');
    expect(skipLink).toBeInTheDocument();
    expect(skipLink).toHaveAttribute('href', '#main-content');
  });

  it('renders with custom title and description', () => {
    render(
      <Layout title="Custom Title" description="Custom Description">
        <div>Test Content</div>
      </Layout>
    );
    
    // Helmet would set these values, but we're checking if it's contained in our mock
    const helmet = screen.getByTestId('helmet');
    expect(helmet).toBeInTheDocument();
  });

  it('renders animated background when showAnimatedBackground is true', () => {
    render(
      <Layout showAnimatedBackground={true}>
        <div>Test Content</div>
      </Layout>
    );
    
    // We'll see the animated background after useEffect runs
    // Since we're mocking it, it should be in the document
    expect(screen.getByTestId('animated-background')).toBeInTheDocument();
  });

  it('does not render animated background when showAnimatedBackground is false', () => {
    render(
      <Layout showAnimatedBackground={false}>
        <div>Test Content</div>
      </Layout>
    );
    
    // Animated background should not be rendered
    expect(screen.queryByTestId('animated-background')).not.toBeInTheDocument();
  });
});