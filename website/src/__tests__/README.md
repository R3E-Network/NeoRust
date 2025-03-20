# NeoRust Website Tests

This directory contains all the test files for the NeoRust website components.

## Overview

- Each component has a corresponding test file named `ComponentName.test.tsx`
- The `test-utils.tsx` file provides shared testing utilities and custom renders
- All tests use Jest and React Testing Library

## Test Files

- `Header.test.tsx` - Tests for the site navigation header
- `Footer.test.tsx` - Tests for the site footer
- `Layout.test.tsx` - Tests for the main layout component
- `ThemeContext.test.tsx` - Tests for theme context functionality
- `ThemeToggle.test.tsx` - Tests for theme toggle button
- `Search.test.tsx` - Tests for the search component

## Running Tests

From the project root:

```bash
# Run all tests
npm test

# Run tests with coverage
npm run test:coverage
```

## Test Utilities

The `test-utils.tsx` file provides:

1. A custom render function that wraps components with all required providers
2. Mock implementations for browser APIs like `window.matchMedia`
3. Re-exports of all testing utilities from React Testing Library

## Writing New Tests

When writing new tests:

1. Use the custom render function from `test-utils.tsx`
2. Follow the pattern of existing tests
3. Make use of data-testid attributes when necessary for test selections
4. Keep tests focused on user interactions rather than implementation details

## Documentation

For more detailed information about the testing approach, see the [Testing Documentation](/docs/testing.md).