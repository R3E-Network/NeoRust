import React, { useState, useEffect } from 'react';
import { Link } from 'gatsby';
import { StaticImage } from 'gatsby-plugin-image';
import { useTheme } from '../context/ThemeContext';
import Search from './Search';
import ThemeToggle from './ThemeToggle';

const Header: React.FC = () => {
  const [isMobileMenuOpen, setIsMobileMenuOpen] = useState(false);
  const [isScrolled, setIsScrolled] = useState(false);
  const { theme } = useTheme();
  
  // Handle scroll effect
  useEffect(() => {
    const handleScroll = () => {
      setIsScrolled(window.scrollY > 10);
    };
    
    window.addEventListener('scroll', handleScroll);
    return () => window.removeEventListener('scroll', handleScroll);
  }, []);

  return (
    <header className={`fixed w-full z-50 transition-all duration-300 
      ${isScrolled 
        ? theme === 'dark' 
          ? 'bg-slate-900/90 backdrop-blur-md shadow-lg shadow-neo-green-900/5' 
          : 'bg-white/90 backdrop-blur-md shadow-lg'
        : theme === 'dark'
          ? 'bg-slate-900/80 backdrop-blur-sm'
          : 'bg-white/80 backdrop-blur-sm'
      }`}
    >
      <div className="container mx-auto px-4 py-3 flex justify-between items-center">
        <Link to="/" className="flex items-center space-x-3 transition-transform hover:scale-105">
          <StaticImage 
            src="../images/neo-logo.svg" 
            alt="Neo Logo" 
            width={40} 
            height={40}
            placeholder="blurred"
            className="transition-transform hover:rotate-12"
          />
          <span className={`text-2xl font-bold ${theme === 'dark' ? 'text-neo-green-400' : 'text-neo-green-600'}`}>
            Rust SDK
          </span>
        </Link>
        
        <nav className="hidden md:flex items-center space-x-6">
          <div className="flex items-center space-x-4">
            <Link to="/docs" className="nav-link">Documentation</Link>
            <Link to="/guides" className="nav-link">Guides</Link>
            <Link to="/examples" className="nav-link">Examples</Link>
            <Link to="/playground" className="nav-link">Playground</Link>
            <Link to="/api-reference" className="nav-link">API</Link>
          </div>
          <div className="ml-2 flex items-center space-x-3">
            <Search />
            <ThemeToggle />
            <a 
              href="https://github.com/R3E-Network/NeoRust" 
              target="_blank" 
              rel="noopener noreferrer" 
              className={`transition-transform hover:scale-110 ${theme === 'dark' ? 'text-gray-300 hover:text-white' : 'text-gray-600 hover:text-gray-900'}`}
              aria-label="GitHub Repository"
            >
              <svg className="h-6 w-6" fill="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                <path fillRule="evenodd" d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z" clipRule="evenodd"></path>
              </svg>
            </a>
          </div>
        </nav>
        
        <div className="flex items-center space-x-4 md:hidden">
          <ThemeToggle />
          <button 
            className={`focus:outline-none focus:ring-2 focus:ring-neo-green-400 rounded-md p-1 ${
              theme === 'dark' ? '' : 'text-gray-700'
            }`}
            onClick={() => setIsMobileMenuOpen(true)}
            aria-label="Open menu"
            aria-expanded={isMobileMenuOpen}
          >
            <svg className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M4 6h16M4 12h16M4 18h16"></path>
            </svg>
          </button>
        </div>
      </div>

      {/* Mobile menu */}
      {isMobileMenuOpen && (
        <div className={`fixed inset-0 z-50 animate-fade-in ${theme === 'dark' ? 'bg-slate-900' : 'bg-white'}`}>
          <div className="p-4 flex justify-end">
            <button 
              onClick={() => setIsMobileMenuOpen(false)}
              className={`p-1 focus:outline-none focus:ring-2 focus:ring-neo-green-400 rounded-md ${
                theme === 'dark' ? '' : 'text-gray-700'
              }`}
              aria-label="Close menu"
            >
              <svg className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M6 18L18 6M6 6l12 12"></path>
              </svg>
            </button>
          </div>
          <div className="px-4 py-2">
            <Search placeholder="Search..." />
          </div>
          <nav className="flex flex-col items-center space-y-6 text-xl mt-6">
            <Link 
              to="/docs" 
              className="nav-link animate-fade-in-up" 
              onClick={() => setIsMobileMenuOpen(false)}
              style={{ animationDelay: '100ms' }}
            >
              Documentation
            </Link>
            <Link 
              to="/guides" 
              className="nav-link animate-fade-in-up" 
              onClick={() => setIsMobileMenuOpen(false)}
              style={{ animationDelay: '150ms' }}
            >
              Guides
            </Link>
            <Link 
              to="/examples" 
              className="nav-link animate-fade-in-up" 
              onClick={() => setIsMobileMenuOpen(false)}
              style={{ animationDelay: '200ms' }}
            >
              Examples
            </Link>
            <Link 
              to="/playground" 
              className="nav-link animate-fade-in-up" 
              onClick={() => setIsMobileMenuOpen(false)}
              style={{ animationDelay: '250ms' }}
            >
              Playground
            </Link>
            <Link 
              to="/api-reference" 
              className="nav-link animate-fade-in-up" 
              onClick={() => setIsMobileMenuOpen(false)}
              style={{ animationDelay: '300ms' }}
            >
              API
            </Link>
            <a 
              href="https://github.com/R3E-Network/NeoRust" 
              target="_blank" 
              rel="noopener noreferrer" 
              className={`mt-6 flex items-center animate-fade-in-up ${
                theme === 'dark' ? 'text-gray-300 hover:text-white' : 'text-gray-600 hover:text-gray-900'
              }`}
              style={{ animationDelay: '350ms' }}
            >
              <svg className="h-6 w-6 mr-2" fill="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                <path fillRule="evenodd" d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z" clipRule="evenodd"></path>
              </svg>
              GitHub Repository
            </a>
          </nav>
        </div>
      )}
    </header>
  );
};

export default Header;