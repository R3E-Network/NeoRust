# Testing Documentation

This document outlines the testing strategy and testing practices for the Neo Rust Website.

## Testing Strategy

The website uses Jest and React Testing Library for testing components and functionality. Our testing approach follows these principles:

1. **Component-focused testing**: Each UI component should have its own test file.
2. **Behavior-driven**: Tests focus on behavior rather than implementation details.
3. **User-centric**: Tests should simulate user interactions and verify expected outcomes.
4. **Mocking external dependencies**: External services and APIs are mocked to ensure tests are reliable and fast.

## Test Files Structure

Tests are located in the `src/__tests__` directory and follow a naming convention:
- `ComponentName.test.tsx` - for component tests

## Types of Tests

### Unit Tests
Test individual components in isolation, mocking any dependencies.

### Integration Tests
Test interactions between multiple components or functionalities.

### Blockchain Data Tests
Special tests for blockchain data components like `BlockchainInfo.test.tsx` that:
- Mock API responses
- Test loading states
- Test error handling
- Verify periodic updates

## Running Tests

```bash
# Run all tests
npm test

# Run tests with coverage reporting
npm run test:coverage

# Run tests in watch mode
npm test:watch

# Run a specific test file
npm run test:file src/__tests__/ComponentName.test.tsx
```

## Mocking

The tests use Jest's mocking capabilities to mock external dependencies:
- Axios for API requests
- Gatsby components (Link, StaticImage)
- Third-party components (Particles, etc.)

## Testing the Blockchain Info Component

The blockchain info component on the homepage displays live data from the Neo blockchain. The `BlockchainInfo.test.tsx` file provides comprehensive tests for this component, including:

1. **Initial loading state**: Tests that loading indicators are shown while data is being fetched.
2. **Data display**: Tests that blockchain data is correctly displayed after fetching.
3. **Refresh functionality**: Tests that clicking the refresh button triggers a new data fetch.
4. **Error handling**: Tests the component's behavior when API requests fail.
5. **Automatic updates**: Tests that the component updates data automatically on the specified interval.
6. **Cleanup**: Tests that intervals are properly cleaned up when the component unmounts.

### Example Blockchain Info Test

```typescript
it('fetches and displays blockchain information', async () => {
  render(<IndexPage />);
  
  // Wait for data to be loaded and displayed
  await screen.findByText('12,344'); // Block height
  
  // Verify all blockchain info is displayed
  expect(screen.getByText('12,344')).toBeInTheDocument();
  expect(screen.getByText('123456...abcdef')).toBeInTheDocument();
  expect(screen.getByText('10')).toBeInTheDocument();
  expect(screen.getByText('/Neo:3.5.0/')).toBeInTheDocument();
  
  // Verify the API calls were made
  expect(mockAxios.post).toHaveBeenCalledTimes(3);
});
```

## Writing New Tests

When writing new tests, follow these guidelines:

1. Create a test file in the `src/__tests__` directory
2. Import necessary testing utilities and the component to test
3. Mock any external dependencies
4. Structure your tests with `describe` and `it` blocks
5. Use `render` to mount your component
6. Query elements using `screen` methods
7. Use `fireEvent` to simulate user interactions
8. Use `act` for asynchronous operations
9. Make assertions with `expect`

## Continuous Integration

All tests run automatically in the CI pipeline. A test failure will prevent deployment.