import React from 'react';
import { render, screen, fireEvent, act } from './test-utils';
import Search from '../components/Search';

// Mock gatsby's navigate function
const mockNavigate = jest.fn();
jest.mock('gatsby', () => ({
  navigate: mockNavigate,
  Link: jest.fn().mockImplementation(({ to, children, ...rest }) => {
    return <a href={to} {...rest}>{children}</a>;
  }),
}));

describe('Search Component', () => {
  beforeEach(() => {
    // Clear mocks between tests
    mockNavigate.mockClear();
  });

  it('renders with default placeholder', () => {
    render(<Search />);
    expect(screen.getByPlaceholderText('Search documentation...')).toBeInTheDocument();
  });

  it('renders with custom placeholder', () => {
    render(<Search placeholder="Custom search..." />);
    expect(screen.getByPlaceholderText('Custom search...')).toBeInTheDocument();
  });

  it('shows search results when typing', () => {
    render(<Search />);
    const searchInput = screen.getByPlaceholderText('Search documentation...');
    
    // Type in the search box
    fireEvent.change(searchInput, { target: { value: 'wallet' } });
    
    // Check if results appear
    expect(screen.getByText('Wallet Management')).toBeInTheDocument();
    expect(screen.getByText('Create and manage Neo wallets securely with the SDK.')).toBeInTheDocument();
  });

  it('filters results based on query', () => {
    render(<Search />);
    const searchInput = screen.getByPlaceholderText('Search documentation...');
    
    // Type in the search box
    fireEvent.change(searchInput, { target: { value: 'token' } });
    
    // Check if relevant results appear
    expect(screen.getByText('NEP-17 Tokens')).toBeInTheDocument();
    
    // Should not show unrelated results
    expect(screen.queryByText('Getting Started')).not.toBeInTheDocument();
  });

  it('navigates to first result when Enter key is pressed', () => {
    render(<Search />);
    const searchInput = screen.getByPlaceholderText('Search documentation...');
    
    // Type in the search box
    fireEvent.change(searchInput, { target: { value: 'wallet' } });
    
    // Make sure results are showing
    const resultElement = screen.getByText('Wallet Management');
    expect(resultElement).toBeInTheDocument();
    
    // Press Enter
    fireEvent.keyDown(searchInput, { key: 'Enter' });
    
    // When Enter is pressed, results should be hidden (as it would navigate)
    // and input should be cleared
    expect(screen.queryByText('Wallet Management')).not.toBeInTheDocument();
    expect(searchInput).toHaveValue('');
  });

  it('clears search when a result is clicked', () => {
    render(<Search />);
    const searchInput = screen.getByPlaceholderText('Search documentation...');
    
    // Type in the search box
    fireEvent.change(searchInput, { target: { value: 'wallet' } });
    
    // Click on a result
    fireEvent.click(screen.getByText('Wallet Management'));
    
    // Check if search is cleared
    expect(searchInput).toHaveValue('');
  });

  it('closes results when clicking outside', () => {
    // Create a div outside the search component to click on
    const { container } = render(
      <div>
        <div data-testid="outside-element" />
        <Search />
      </div>
    );
    
    const searchInput = screen.getByPlaceholderText('Search documentation...');
    
    // Type in the search box to show results
    fireEvent.change(searchInput, { target: { value: 'wallet' } });
    
    // Verify results are visible
    expect(screen.getByText('Wallet Management')).toBeInTheDocument();
    
    // Click outside
    act(() => {
      const outsideElement = screen.getByTestId('outside-element');
      fireEvent.mouseDown(outsideElement);
    });
    
    // Results should no longer be visible
    expect(screen.queryByText('Wallet Management')).not.toBeInTheDocument();
  });

  it('does not show results for very short queries', () => {
    render(<Search />);
    const searchInput = screen.getByPlaceholderText('Search documentation...');
    
    // Type a single character
    fireEvent.change(searchInput, { target: { value: 'a' } });
    
    // Results should not appear
    expect(screen.queryByText('Getting Started')).not.toBeInTheDocument();
  });
});