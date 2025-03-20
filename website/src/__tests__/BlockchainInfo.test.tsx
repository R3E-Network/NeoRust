import React from 'react';
import { render, screen, fireEvent, act, waitFor } from '@testing-library/react';
import axios from 'axios';
import IndexPage from '../pages/index';

// Mock axios
jest.mock('axios');
const mockAxios = axios as jest.Mocked<typeof axios>;

// Mock Gatsby's Link component
jest.mock('gatsby', () => ({
  Link: jest.fn().mockImplementation(({ to, children, ...rest }) => {
    return <a href={to} {...rest}>{children}</a>;
  }),
}));

// Mock the StaticImage component from Gatsby
jest.mock('gatsby-plugin-image', () => {
  return {
    StaticImage: jest.fn().mockImplementation(({ src, alt, ...rest }) => {
      return <img src={src} alt={alt} {...rest} data-testid="static-image" />;
    }),
  };
});

// Mock the react-particles component
jest.mock('react-particles', () => {
  return {
    __esModule: true,
    default: jest.fn().mockImplementation(props => (
      <div data-testid="particles-js">{props.children}</div>
    )),
  };
});

// Mock loadFull function
jest.mock('tsparticles', () => {
  return {
    loadFull: jest.fn().mockImplementation(() => Promise.resolve()),
  };
});

// Mock react-intersection-observer
jest.mock('react-intersection-observer', () => ({
  useInView: jest.fn().mockReturnValue([null, true]),
}));

// Mock CodeBlock component that's causing issues
jest.mock('../components/CodeBlock', () => {
  return function MockCodeBlock({ code, language }: { code: string; language: string }) {
    return (
      <div data-testid="code-block" data-language={language}>
        <pre>{code}</pre>
      </div>
    );
  };
});

// Mock data for blockchain info responses
const mockBlockCountResponse = {
  data: {
    jsonrpc: '2.0',
    id: 1,
    result: 12345 // Block count
  }
};

const mockBlockResponse = {
  data: {
    jsonrpc: '2.0',
    id: 1,
    result: {
      hash: '0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef',
      time: 1612345678, // Unix timestamp
      tx: Array(10).fill({ /* mock transaction data */ })
    }
  }
};

const mockVersionResponse = {
  data: {
    jsonrpc: '2.0',
    id: 1,
    result: {
      useragent: '/Neo:3.5.0/',
      tcpport: 10333,
      wsport: 10334,
      nonce: 1234567890
    }
  }
};

// Updated mock responses for testing data changes
const mockUpdatedBlockCountResponse = {
  data: {
    jsonrpc: '2.0',
    id: 1,
    result: 12346 // Incremented block count
  }
};

const mockUpdatedBlockResponse = {
  data: {
    jsonrpc: '2.0',
    id: 1,
    result: {
      hash: '0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890',
      time: 1612345778, // 100 seconds later
      tx: Array(15).fill({ /* mock transaction data */ }) // More transactions
    }
  }
};

describe('Blockchain Info Component', () => {
  beforeEach(() => {
    // Reset mocks
    mockAxios.post.mockReset();
    
    // Mock the axios responses
    mockAxios.post.mockImplementation((url, data) => {
      const { method } = data as any;
      
      if (method === 'getblockcount') {
        return Promise.resolve(mockBlockCountResponse);
      } else if (method === 'getblock') {
        return Promise.resolve(mockBlockResponse);
      } else if (method === 'getversion') {
        return Promise.resolve(mockVersionResponse);
      }
      
      return Promise.reject(new Error('Unexpected method call'));
    });
    
    // Use a fixed date for consistent testing
    jest.spyOn(Date, 'now').mockImplementation(() => 1612345678000);
    jest.spyOn(Date.prototype, 'toLocaleTimeString').mockReturnValue('10:00:00 AM');
    jest.spyOn(Date.prototype, 'toLocaleString').mockReturnValue('1/1/2023, 10:00:00 AM');
    
    // Mock console methods to prevent noise in test output
    jest.spyOn(console, 'log').mockImplementation(() => {});
    jest.spyOn(console, 'error').mockImplementation(() => {});
  });

  afterEach(() => {
    jest.clearAllMocks();
  });

  it('renders the blockchain info section with loading state initially', () => {
    // Prevent the useEffect from running by returning a resolved promise
    mockAxios.post.mockImplementation(() => new Promise(() => {}));
    
    render(<IndexPage />);
    
    // Check for loading state
    expect(screen.getByText('Loading blockchain data...')).toBeInTheDocument();
  });

  it('fetches and displays blockchain information', async () => {
    render(<IndexPage />);
    
    // Wait for data to be loaded and displayed
    await waitFor(() => {
      expect(screen.getByText('12,344')).toBeInTheDocument(); // Block height (blockCount - 1)
    });
    
    // Verify all blockchain info is displayed
    expect(screen.getByText('12,344')).toBeInTheDocument(); // Block height
    
    // Check transactions count
    expect(screen.getByText('10')).toBeInTheDocument(); // Number of transactions
    
    // Check version
    expect(screen.getByText('/Neo:3.5.0/')).toBeInTheDocument(); // Version
    
    // Verify the API calls were made
    expect(mockAxios.post).toHaveBeenCalledTimes(3);
  });

  it('handles refresh button click', async () => {
    render(<IndexPage />);
    
    // Wait for initial load
    await waitFor(() => {
      expect(screen.getByText('12,344')).toBeInTheDocument();
    });
    
    // Clear the mock to track new calls
    mockAxios.post.mockClear();
    
    // Find and click the refresh button by looking for a button with an SVG inside
    const refreshButton = screen.getByRole('button', { name: /refresh/i });
    fireEvent.click(refreshButton);
    
    // Verify the loading state is shown
    expect(screen.getByText('Updating...')).toBeInTheDocument();
    
    // Wait for data to be reloaded
    await waitFor(() => {
      expect(screen.getByText('12,344')).toBeInTheDocument();
    });
    
    // Verify API calls were made again
    expect(mockAxios.post).toHaveBeenCalledTimes(3);
  });

  it('shows last updated time', async () => {
    render(<IndexPage />);
    
    // Wait for data to be loaded
    await waitFor(() => {
      expect(screen.getByText('12,344')).toBeInTheDocument();
    });
    
    // Check that the last updated time is shown
    expect(screen.getByText('Last updated: 10:00:00 AM')).toBeInTheDocument();
  });

  it('handles API errors gracefully', async () => {
    // Mock an error response
    mockAxios.post.mockRejectedValue(new Error('Network error'));
    
    render(<IndexPage />);
    
    // Wait for the loading state to finish
    // The component should not crash and should exit loading state
    await act(async () => {
      // Give time for promises to resolve
      await new Promise(resolve => setTimeout(resolve, 0));
    });
    
    // Check that the error was logged
    expect(console.error).toHaveBeenCalledWith(
      'Error fetching blockchain info:', 
      expect.any(Error)
    );
    
    // Verify we're not in loading state anymore
    expect(screen.queryByText('Loading blockchain data...')).not.toBeInTheDocument();
  });

  it('updates blockchain info periodically', async () => {
    // Mock implementation to track calls
    jest.useFakeTimers();
    
    render(<IndexPage />);
    
    // Initial fetch - let's wait for it to complete
    await waitFor(() => {
      expect(mockAxios.post).toHaveBeenCalled();
    });
    mockAxios.post.mockClear();
    
    // Fast-forward to trigger the interval
    act(() => {
      jest.advanceTimersByTime(15000); // 15 seconds
    });
    
    // Verify the fetches were called again
    expect(mockAxios.post).toHaveBeenCalled();
    
    // Clean up
    jest.useRealTimers();
  });

  it('clears update interval when unmounted', () => {
    const clearIntervalSpy = jest.spyOn(window, 'clearInterval');
    
    const { unmount } = render(<IndexPage />);
    
    // Unmount the component
    unmount();
    
    // Verify clearInterval was called
    expect(clearIntervalSpy).toHaveBeenCalled();
    expect(console.log).toHaveBeenCalledWith('Blockchain data interval cleared');
  });

  it('displays external link for blockchain data', async () => {
    render(<IndexPage />);
    
    // Wait for data to be loaded
    await waitFor(() => {
      expect(screen.getByText('12,344')).toBeInTheDocument();
    });
    
    // Look for an anchor tag with an href that contains 'neo3.neotube.io/block/'
    const links = screen.getAllByRole('link');
    
    // Find the link that contains the block explorer URL
    const explorerLink = links.find(link => 
      link.getAttribute('href')?.includes('neo3.neotube.io/block/')
    );
    
    // Verify the link exists and has the correct URL
    expect(explorerLink).toBeDefined();
    expect(explorerLink).toHaveAttribute(
      'href', 
      expect.stringContaining('neo3.neotube.io/block/')
    );
  });

  it('updates with new blockchain data', async () => {
    render(<IndexPage />);
    
    // Wait for initial data to load
    await waitFor(() => {
      expect(screen.getByText('12,344')).toBeInTheDocument();
    });
    
    // Now update the mock responses for the next fetch
    mockAxios.post.mockImplementation((url, data) => {
      const { method } = data as any;
      
      if (method === 'getblockcount') {
        return Promise.resolve(mockUpdatedBlockCountResponse);
      } else if (method === 'getblock') {
        return Promise.resolve(mockUpdatedBlockResponse);
      } else if (method === 'getversion') {
        return Promise.resolve(mockVersionResponse);
      }
      
      return Promise.reject(new Error('Unexpected method call'));
    });
    
    // Update the Date.now mock to show a different time
    jest.spyOn(Date, 'now').mockImplementation(() => 1612345778000);
    jest.spyOn(Date.prototype, 'toLocaleTimeString').mockReturnValue('10:01:40 AM');
    
    // Clear previous calls
    mockAxios.post.mockClear();
    
    // Click refresh to trigger update - find by role this time
    const refreshButton = screen.getByRole('button', { name: /refresh/i });
    fireEvent.click(refreshButton);
    
    // Wait for updated data to appear
    await waitFor(() => {
      expect(screen.getByText('12,345')).toBeInTheDocument(); // New block height
    });
    
    // Verify updated blockchain info is displayed
    expect(screen.getByText('12,345')).toBeInTheDocument(); // New block height
    expect(screen.getByText('15')).toBeInTheDocument(); // New transaction count
    
    // Verify the updated last updated time
    expect(screen.getByText('Last updated: 10:01:40 AM')).toBeInTheDocument();
  });

  it('disables refresh button while loading', async () => {
    // Allow initial data load
    render(<IndexPage />);
    
    // Wait for data to be loaded so the refresh button appears
    await waitFor(() => {
      expect(screen.getByText('12,344')).toBeInTheDocument();
    });
    
    // Find the refresh button
    const refreshButton = screen.getByRole('button', { name: /refresh/i });
    
    // Make axios never resolve to keep component in loading state
    mockAxios.post.mockImplementation(() => new Promise(() => {}));
    
    // Click refresh
    fireEvent.click(refreshButton);
    
    // Button should change to "Updating..." and be disabled
    const updatingButton = screen.getByText('Updating...');
    expect(updatingButton).toBeInTheDocument();
    expect(updatingButton.closest('button')).toBeDisabled();
  }, 10000); // Increase timeout for this test to avoid timeout issues
});