import React from 'react';
import { screen } from '@testing-library/react';
import { render } from './test-utils';
import Footer from '../components/Footer';

// Mock the StaticImage component from Gatsby
jest.mock('gatsby-plugin-image', () => {
  return {
    StaticImage: jest.fn().mockImplementation(({ src, alt, ...rest }) => {
      return <img src={src} alt={alt} {...rest} data-testid="static-image" />;
    }),
    GatsbyImage: jest.fn().mockImplementation(({ image, alt, ...rest }) => {
      return <img alt={alt} {...rest} data-testid="gatsby-image" />;
    }),
    getImage: jest.fn().mockImplementation(image => image),
  };
});

// Mock Gatsby's Link component
jest.mock('gatsby', () => {
  return {
    Link: jest.fn().mockImplementation(({ to, children, ...rest }) => {
      return <a href={to} {...rest}>{children}</a>
    }),
    graphql: jest.fn(),
    useStaticQuery: jest.fn(),
  }
});

describe('Footer Component', () => {
  beforeEach(() => {
    // Mock Date to ensure consistent year for copyright
    const mockDate = new Date('2023-01-01T00:00:00Z');
    jest.spyOn(global, 'Date').mockImplementation(() => mockDate);
  });

  afterEach(() => {
    jest.restoreAllMocks();
  });

  it('renders the footer with logo and copyright', () => {
    render(<Footer />);
    
    // Check if logo is visible
    expect(screen.getByTestId('static-image')).toBeInTheDocument();
    
    // Check site name
    expect(screen.getByText('Neo')).toBeInTheDocument();
    expect(screen.getByText('Rust')).toBeInTheDocument();
    
    // Check current year in copyright
    expect(screen.getByText(/© 2023 Neo Rust SDK. All rights reserved./)).toBeInTheDocument();
  });

  it('renders the navigation links', () => {
    render(<Footer />);
    
    // Check documentation section
    expect(screen.getByText('Documentation')).toBeInTheDocument();
    expect(screen.getByText('Getting Started')).toBeInTheDocument();
    expect(screen.getByText('Wallet Management')).toBeInTheDocument();
    
    // Check resources section
    expect(screen.getByText('Resources')).toBeInTheDocument();
    expect(screen.getByText('Examples')).toBeInTheDocument();
    expect(screen.getByText('Playground')).toBeInTheDocument();
    
    // Check community section
    expect(screen.getByText('Community')).toBeInTheDocument();
    expect(screen.getByText('Discord')).toBeInTheDocument();
    expect(screen.getByText('Twitter')).toBeInTheDocument();
  });

  it('renders social media links', () => {
    render(<Footer />);
    
    // Check for GitHub link
    const githubLink = screen.getByLabelText('GitHub Repository');
    expect(githubLink).toBeInTheDocument();
    expect(githubLink).toHaveAttribute('href', 'https://github.com/R3E-Network/NeoRust');
    
    // Check for Twitter link
    const twitterLink = screen.getByLabelText('Neo Twitter');
    expect(twitterLink).toBeInTheDocument();
    expect(twitterLink).toHaveAttribute('href', 'https://twitter.com/neo_blockchain');
    
    // Check for Discord link
    const discordLink = screen.getByLabelText('Neo Discord');
    expect(discordLink).toBeInTheDocument();
    expect(discordLink).toHaveAttribute('href', 'https://discord.gg/neo');
  });

  it('displays the "Built with ♥" message', () => {
    render(<Footer />);
    
    expect(screen.getByText(/Built with/, { exact: false })).toBeInTheDocument();
    expect(screen.getByText(/for the Neo community/, { exact: false })).toBeInTheDocument();
  });
});