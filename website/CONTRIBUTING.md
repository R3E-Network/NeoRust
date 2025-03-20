# Contributing to NeoRust Website

Thank you for considering contributing to the NeoRust website! This document provides guidelines and instructions for contributing to the project.

## Development Environment

1. Clone the repository:
   ```bash
   git clone https://github.com/neo-rust/website.git
   cd website
   ```

2. Install dependencies:
   ```bash
   npm install
   ```

3. Start the development server:
   ```bash
   npm run develop
   ```

## Testing

We use Jest and React Testing Library for testing components. All components should have corresponding tests.

### Running Tests

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

### Writing Tests

1. Create test files in the `src/__tests__` directory
2. Use the custom render function from `test-utils.tsx`
3. Follow the existing test patterns
4. Ensure you're testing user interactions rather than implementation details

### Test Coverage Requirements

We aim to maintain:
- Statements: >90%
- Branches: >60%
- Functions: >90%
- Lines: >90%

## Documentation-First Development

We follow a documentation-first approach:

1. Begin by documenting the feature or change before implementing it
2. Update documentation alongside code changes
3. Ensure all features have corresponding documentation

## Pull Request Process

1. Create a feature branch for your changes
2. Add tests for any new functionality
3. Ensure all tests pass
4. Update documentation as needed
5. Submit a pull request

## Code Style

- Follow the existing code style of the project
- Use TypeScript for all new code
- Use functional components with hooks
- Follow accessibility best practices

## Commit Messages

Use conventional commit format:
- `feat:` for new features
- `fix:` for bug fixes
- `docs:` for documentation changes
- `test:` for test-related changes
- `refactor:` for code refactoring
- `style:` for formatting changes
- `chore:` for maintenance tasks

## Questions or Need Help?

Feel free to open an issue with questions or suggestions.