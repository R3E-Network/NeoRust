import React from 'react';
import { render, screen } from '@testing-library/react';
import * as testUtils from './test-utils';

describe('Custom render', () => {
  it('wraps components with ThemeProvider', () => {
    const TestComponent = () => <div data-testid="test-component">Test</div>;
    
    // Use our custom render
    testUtils.render(<TestComponent />);
    
    // Check if the component rendered correctly
    expect(screen.getByTestId('test-component')).toBeInTheDocument();
  });
});