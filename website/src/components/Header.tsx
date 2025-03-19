import React, { useState, useEffect } from 'react';
import { Link } from 'gatsby';
import { StaticImage } from 'gatsby-plugin-image';
import { useTheme } from '../context/ThemeContext';
import Search from './Search';
import ThemeToggle from './ThemeToggle';

const Header: React.FC = () => {
  const [isMobileMenuOpen, setIsMobileMenuOpen] = useState(false);
  const [isScrolled, setIsScrolled] = useState(false);
  const [activeItem, setActiveItem] = useState('');
  const { theme } = useTheme();
  
  // Handle scroll effect
  useEffect(() => {
    const handleScroll = () => {
      setIsScrolled(window.scrollY > 10);
    };
    
    window.addEventListener('scroll', handleScroll);
    return () => window.removeEventListener('scroll', handleScroll);
  }, []);

  // Detect current page for navigation highlighting
  useEffect(() => {
    if (typeof window !== 'undefined') {
      const path = window.location.pathname;
      if (path.includes('/docs')) setActiveItem('docs');
      else if (path.includes('/guides')) setActiveItem('guides');
      else if (path.includes('/examples')) setActiveItem('examples');
      else if (path.includes('/playground')) setActiveItem('playground');
      else if (path.includes('/api-reference')) setActiveItem('api');
      else setActiveItem('');
    }
  }, []);

  const navItems = [
    { label: 'Documentation', path: '/docs', id: 'docs' },
    { label: 'Guides', path: '/guides', id: 'guides' },
    { label: 'Examples', path: '/examples', id: 'examples' },
    { label: 'Playground', path: '/playground', id: 'playground' },
    { label: 'API', path: '/api-reference', id: 'api' }
  ];

  const getHeaderClasses = () => {
    const baseClasses = 'fixed w-full z-50 transition-all duration-300';
    if (isScrolled) {
      return `${baseClasses} ${
        theme === 'dark' 
          ? 'bg-slate-900/90 backdrop-blur-md shadow-lg shadow-black/10' 
          : 'bg-white/90 backdrop-blur-md shadow-lg shadow-black/5'
      }`;
    } else {
      return `${baseClasses} ${
        theme === 'dark'
          ? 'bg-transparent'
          : 'bg-transparent'
      }`;
    }
  };

  return (
    <header className={getHeaderClasses()}>
      <div className="container mx-auto px-4 py-3 flex justify-between items-center">
        <Link 
          to="/" 
          className="flex items-center space-x-3 transition-all duration-300 hover:scale-105"
          aria-label="Neo Rust SDK Home"
        >
          <div className="w-10 h-10 relative rounded-full bg-gradient-to-br from-green-400 to-teal-500 p-0.5 flex items-center justify-center">
            <StaticImage 
              src="../images/neo-logo.svg" 
              alt="Neo Logo" 
              width={32} 
              height={32}
              placeholder="blurred"
              className="rounded-full filter hover:brightness-110 transition"
            />
          </div>
          <span className={`text-2xl font-bold ${
            theme === 'dark' 
              ? 'text-white' 
              : 'text-gray-900'
          }`}>
            <span className={theme === 'dark' ? 'text-white' : 'text-gray-900'}>Neo</span>
            <span className={theme === 'dark' ? 'text-green-400' : 'text-green-600'}>Rust</span>
          </span>
        </Link>
        
        <nav className="hidden md:flex items-center space-x-1">
          <div className="flex items-center">
            {navItems.map((item) => (
              <Link 
                key={item.id}
                to={item.path} 
                className={`px-4 py-2 rounded-lg transition-all duration-300 ${
                  activeItem === item.id
                    ? theme === 'dark'
                      ? 'text-green-400 bg-green-900/20'
                      : 'text-green-600 bg-green-100'
                    : theme === 'dark'
                      ? 'text-gray-300 hover:text-white hover:bg-white/5'
                      : 'text-gray-700 hover:text-gray-900 hover:bg-gray-100'
                }`}
              >
                {item.label}
              </Link>
            ))}
          </div>
          <div className="ml-2 flex items-center space-x-3">
            <Search />
            <ThemeToggle />
            <a 
              href="https://github.com/R3E-Network/NeoRust" 
              target="_blank" 
              rel="noopener noreferrer" 
              className={`p-2 rounded-lg transition-all duration-300 hover:bg-white/5 ${
                theme === 'dark' ? 'text-gray-300 hover:text-white' : 'text-gray-600 hover:text-gray-900'
              }`}
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
            className={`p-2 rounded-lg transition-all duration-300 hover:bg-white/5 ${
              theme === 'dark' ? 'text-gray-300 hover:text-white' : 'text-gray-700 hover:text-gray-900'
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
          <div className="container mx-auto px-4 py-4">
            <div className="flex justify-between items-center mb-8">
              <Link 
                to="/" 
                className="flex items-center space-x-3"
                onClick={() => setIsMobileMenuOpen(false)}
              >
                <div className="w-10 h-10 relative rounded-full bg-gradient-to-br from-green-400 to-teal-500 p-0.5 flex items-center justify-center">
                  <StaticImage 
                    src="../images/neo-logo.svg" 
                    alt="Neo Logo" 
                    width={32} 
                    height={32}
                    placeholder="blurred"
                    className="rounded-full"
                  />
                </div>
                <span className={`text-2xl font-bold ${
                  theme === 'dark' 
                    ? 'text-white' 
                    : 'text-gray-900'
                }`}>
                  <span className={theme === 'dark' ? 'text-white' : 'text-gray-900'}>Neo</span>
                  <span className={theme === 'dark' ? 'text-green-400' : 'text-green-600'}>Rust</span>
                </span>
              </Link>
              <button 
                onClick={() => setIsMobileMenuOpen(false)}
                className={`p-2 rounded-lg transition-all duration-300 hover:bg-white/5 ${
                  theme === 'dark' ? 'text-gray-300 hover:text-white' : 'text-gray-700 hover:text-gray-900'
                }`}
                aria-label="Close menu"
              >
                <svg className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M6 18L18 6M6 6l12 12"></path>
                </svg>
              </button>
            </div>
            <div className="px-4 py-2 mb-8">
              <Search placeholder="Search..." />
            </div>
            <nav className="flex flex-col space-y-3 text-xl">
              {navItems.map((item, index) => (
                <Link 
                  key={item.id}
                  to={item.path} 
                  className={`px-4 py-3 rounded-lg transition-all duration-300 animate-fade-in-up ${
                    activeItem === item.id
                      ? theme === 'dark'
                        ? 'text-green-400 bg-green-900/20'
                        : 'text-green-600 bg-green-100'
                      : theme === 'dark'
                        ? 'text-gray-300 hover:text-white hover:bg-white/5'
                        : 'text-gray-700 hover:text-gray-900 hover:bg-gray-100'
                  }`}
                  onClick={() => setIsMobileMenuOpen(false)}
                  style={{ animationDelay: `${index * 50 + 100}ms` }}
                >
                  {item.label}
                </Link>
              ))}
              <a 
                href="https://github.com/R3E-Network/NeoRust" 
                target="_blank" 
                rel="noopener noreferrer" 
                className={`mt-6 flex items-center space-x-3 px-4 py-3 rounded-lg transition-all duration-300 animate-fade-in-up hover:bg-white/5 ${
                  theme === 'dark' ? 'text-gray-300 hover:text-white' : 'text-gray-600 hover:text-gray-900'
                }`}
                style={{ animationDelay: '350ms' }}
              >
                <svg className="h-6 w-6" fill="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                  <path fillRule="evenodd" d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z" clipRule="evenodd"></path>
                </svg>
                <span>GitHub Repository</span>
              </a>
            </nav>
          </div>
        </div>
      )}
    </header>
  );
};

export default Header;