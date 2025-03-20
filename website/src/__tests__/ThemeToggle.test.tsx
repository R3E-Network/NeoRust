import React from 'react';
import { screen, fireEvent } from '@testing-library/react';
import { render } from './test-utils';
import ThemeToggle from '../components/ThemeToggle';

describe('ThemeToggle Component', () => {
  it('renders the theme toggle button with correct initial state', () => {
    render(<ThemeToggle />);
    
    const toggleButton = screen.getByRole('button', { name: /Switch to light mode/i });
    expect(toggleButton).toBeInTheDocument();
  });

  it('toggles the theme when clicked', () => {
    render(<ThemeToggle />);
    
    // Initial state (dark theme in our mocked environment)
    let toggleButton = screen.getByRole('button', { name: /Switch to light mode/i });
    expect(toggleButton).toBeInTheDocument();
    
    // Click the button to toggle theme
    fireEvent.click(toggleButton);
    
    // Theme should now be light
    toggleButton = screen.getByRole('button', { name: /Switch to dark mode/i });
    expect(toggleButton).toBeInTheDocument();
    
    // Verify localStorage was called with the new theme
    expect(window.localStorage.getItem('neoRustTheme')).toBe('light');
    
    // Click again to toggle back to dark
    fireEvent.click(toggleButton);
    
    // Theme should be dark again
    toggleButton = screen.getByRole('button', { name: /Switch to light mode/i });
    expect(toggleButton).toBeInTheDocument();
    
    // Verify localStorage was called with the new theme
    expect(window.localStorage.getItem('neoRustTheme')).toBe('dark');
  });

  it('applies custom class names when provided', () => {
    render(<ThemeToggle className="custom-class" />);
    
    const toggleButton = screen.getByRole('button');
    expect(toggleButton).toHaveClass('custom-class');
  });

  it('displays the correct icon based on the current theme', () => {
    render(<ThemeToggle />);
    
    // In dark mode, the moon icon should be visible (opacity-100)
    const moonIcon = screen.getByText((content, element) => {
      return element?.tagName.toLowerCase() === 'svg' && 
             element?.classList.contains('opacity-100') &&
             element?.innerHTML.includes('M20.354 15.354A9');
    });
    expect(moonIcon).toBeInTheDocument();
    
    // The sun icon should be hidden (opacity-0)
    const sunIcon = screen.getByText((content, element) => {
      return element?.tagName.toLowerCase() === 'svg' && 
             element?.classList.contains('opacity-0') &&
             element?.innerHTML.includes('M12 3v1m0 16v1m9-9h-1M4');
    });
    expect(sunIcon).toBeInTheDocument();
    
    // Click to toggle theme
    fireEvent.click(screen.getByRole('button'));
    
    // Now the sun icon should be visible and moon icon hidden
    expect(sunIcon.classList.contains('opacity-100')).toBeTruthy();
    expect(moonIcon.classList.contains('opacity-0')).toBeTruthy();
  });
});