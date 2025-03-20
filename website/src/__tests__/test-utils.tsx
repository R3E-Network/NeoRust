import React, { ReactElement } from 'react';
import { render, RenderOptions } from '@testing-library/react';
import { ThemeProvider } from '../context/ThemeContext';
import { LocationProvider } from '@gatsbyjs/reach-router';

// Mock the matchMedia function
beforeAll(() => {
  // Mock window.matchMedia
  Object.defineProperty(window, 'matchMedia', {
    writable: true,
    value: jest.fn().mockImplementation(query => ({
      matches: false,
      media: query,
      onchange: null,
      addListener: jest.fn(), // deprecated
      removeListener: jest.fn(), // deprecated
      addEventListener: jest.fn(),
      removeEventListener: jest.fn(),
      dispatchEvent: jest.fn(),
    })),
  });
});

// Mock for gatsby's Link component
jest.mock('gatsby', () => ({
  Link: jest.fn().mockImplementation(({ to, children, ...rest }) => {
    return <a href={to} {...rest}>{children}</a>;
  }),
  StaticQuery: jest.fn(),
  graphql: jest.fn(),
  navigate: jest.fn(),
}));

// Mock StaticImage
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

// Mock Helmet
jest.mock('react-helmet', () => {
  const Helmet = ({ children }) => <div data-testid="helmet">{children}</div>;
  Helmet.renderStatic = jest.fn().mockReturnValue({
    title: 'Mocked Title',
    meta: [],
    link: [],
  });
  return { Helmet };
});

// Mock localStorage
const localStorageMock = (() => {
  let store = {};
  return {
    getItem: jest.fn(key => store[key]),
    setItem: jest.fn((key, value) => {
      store[key] = value.toString();
    }),
    clear: jest.fn(() => {
      store = {};
    }),
    removeItem: jest.fn(key => {
      delete store[key];
    }),
  };
})();
Object.defineProperty(window, 'localStorage', { value: localStorageMock });

const AllTheProviders = ({ children }) => {
  return (
    <LocationProvider>
      <ThemeProvider>
        {children}
      </ThemeProvider>
    </LocationProvider>
  );
};

const customRender = (
  ui: ReactElement,
  options?: Omit<RenderOptions, 'wrapper'>,
) => render(ui, { wrapper: AllTheProviders, ...options });

// re-export everything
export * from '@testing-library/react';

// override render method
export { customRender as render };

// Add a dummy test so Jest doesn't complain about no tests in this file
test('Utils file exists', () => {
  expect(true).toBe(true);
});