import React, { useEffect } from 'react';
import { act, screen, render } from './test-utils';
import { useTheme } from '../context/ThemeContext';

// Test component that uses the theme context
const TestComponent = () => {
  const { theme, toggleTheme } = useTheme();
  
  return (
    <div data-testid="test-component">
      <div data-testid="theme-value">{theme}</div>
      <button onClick={toggleTheme} data-testid="toggle-button">
        Toggle Theme
      </button>
    </div>
  );
};

describe('ThemeContext', () => {
  beforeEach(() => {
    // Clear localStorage between tests
    window.localStorage.clear();
  });

  it('provides dark theme as default', () => {
    render(<TestComponent />);
    
    expect(screen.getByTestId('theme-value')).toHaveTextContent('dark');
  });

  it('toggles theme when toggleTheme is called', () => {
    render(<TestComponent />);
    
    // Initial theme is dark
    expect(screen.getByTestId('theme-value')).toHaveTextContent('dark');
    
    // Toggle theme to light
    act(() => {
      screen.getByTestId('toggle-button').click();
    });
    
    // Theme should now be light
    expect(screen.getByTestId('theme-value')).toHaveTextContent('light');
    
    // Verify localStorage was updated
    expect(window.localStorage.getItem('neoRustTheme')).toBe('light');
    
    // Toggle back to dark
    act(() => {
      screen.getByTestId('toggle-button').click();
    });
    
    // Theme should be dark again
    expect(screen.getByTestId('theme-value')).toHaveTextContent('dark');
    
    // Verify localStorage was updated
    expect(window.localStorage.getItem('neoRustTheme')).toBe('dark');
  });

  it('initializes from localStorage when a theme is stored', () => {
    // Set initial theme in localStorage
    window.localStorage.setItem('neoRustTheme', 'light');

    render(<TestComponent />);
    
    // Theme should be initialized from localStorage
    expect(screen.getByTestId('theme-value')).toHaveTextContent('light');
  });
});