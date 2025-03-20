import React from 'react';
import { screen, fireEvent } from '@testing-library/react';
import { render } from './test-utils';
import Header from '../components/Header';
import { ThemeProvider } from '../context/ThemeContext';

// Mock the StaticImage component from Gatsby
jest.mock('gatsby-plugin-image', () => {
  return {
    StaticImage: jest.fn().mockImplementation(({ alt, ...props }) => {
      return <img alt={alt} {...props} data-testid="static-image" />
    }),
  }
});

// Mock Gatsby's useStaticQuery and Link
jest.mock('gatsby', () => {
  return {
    Link: jest.fn().mockImplementation(({ to, children, ...rest }) => {
      return <a href={to} {...rest}>{children}</a>
    }),
    graphql: jest.fn(),
    useStaticQuery: jest.fn(),
  }
});

// Mock the window location
const mockLocation = (path: string) => {
  Object.defineProperty(window, 'location', {
    value: {
      pathname: path
    },
    writable: true
  });
};

describe('Header Component', () => {
  beforeEach(() => {
    mockLocation('/');
    // Mock window.scrollY
    Object.defineProperty(window, 'scrollY', { value: 0, writable: true });
  });

  it('renders the header with logo and navigation', () => {
    render(
      <ThemeProvider>
        <Header />
      </ThemeProvider>
    );
    
    // Check if logo and site name are visible
    expect(screen.getByTestId('static-image')).toBeInTheDocument();
    expect(screen.getByText('Neo')).toBeInTheDocument();
    expect(screen.getByText('Rust')).toBeInTheDocument();
    
    // Check if navigation links are present
    expect(screen.getByText('Documentation')).toBeInTheDocument();
    expect(screen.getByText('Guides')).toBeInTheDocument();
    expect(screen.getByText('Examples')).toBeInTheDocument();
    expect(screen.getByText('Playground')).toBeInTheDocument();
    expect(screen.getByText('API')).toBeInTheDocument();
  });

  it('changes header style on scroll', () => {
    render(
      <ThemeProvider>
        <Header />
      </ThemeProvider>
    );
    
    const header = screen.getByRole('banner');
    
    // Initial state (not scrolled)
    expect(header).toHaveClass('bg-transparent');
    
    // Simulate scrolling
    Object.defineProperty(window, 'scrollY', { value: 20 });
    fireEvent.scroll(window);
    
    // After scrolling
    expect(header).toHaveClass('backdrop-blur-md');
  });

  it('opens and closes mobile menu when menu button is clicked', () => {
    // Set viewport width to mobile size
    global.innerWidth = 500;
    
    render(<Header />);
    
    // Mobile menu should be closed initially
    expect(screen.queryByText('GitHub Repository', { selector: 'span' })).not.toBeInTheDocument();
    
    // Open mobile menu
    const menuButton = screen.getByLabelText('Open menu');
    fireEvent.click(menuButton);
    
    // Mobile menu should be visible now
    expect(screen.getByText('GitHub Repository', { selector: 'span' })).toBeInTheDocument();
    
    // Close mobile menu
    const closeButton = screen.getByLabelText('Close menu');
    fireEvent.click(closeButton);
    
    // Mobile menu should be closed again
    expect(screen.queryByText('GitHub Repository', { selector: 'span' })).not.toBeInTheDocument();
  });

  it('highlights active navigation item based on current path', () => {
    // Set path to /docs
    mockLocation('/docs');
    
    render(<Header />);
    
    // Documentation link should have the active class (check for the text color class)
    const docsLink = screen.getByText('Documentation');
    expect(docsLink).toHaveClass('text-green-400', { exact: false });
    
    // Other links should not have the active class
    const guidesLink = screen.getByText('Guides');
    expect(guidesLink).not.toHaveClass('text-green-400', { exact: false });
  });
});