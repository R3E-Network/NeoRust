import React, { useState, useEffect } from 'react';
import { Helmet } from 'react-helmet';
import Header from './Header';
import Footer from './Footer';
import { ThemeProvider, useTheme } from '../context/ThemeContext';
import ThemeToggle from './ThemeToggle';
import AnimatedBackground from './AnimatedBackground';
import '../styles/global.css';

// Skip to content link for accessibility
const SkipToContent = () => (
  <a 
    href="#main-content" 
    className="sr-only focus:not-sr-only focus:absolute focus:top-4 focus:left-4 z-50 bg-neo-green-500 text-white px-4 py-2 rounded-md focus:outline-none"
  >
    Skip to content
  </a>
);

interface LayoutProps {
  children: React.ReactNode;
  title?: string;
  description?: string;
  showAnimatedBackground?: boolean;
}

const LayoutContent: React.FC<LayoutProps> = ({ 
  children, 
  title = 'Neo Rust SDK',
  description = 'A comprehensive Rust library for building applications on the Neo N3 blockchain ecosystem',
  showAnimatedBackground = false,
}) => {
  const { theme } = useTheme();
  const [mounted, setMounted] = useState(false);
  
  // After mounting, we can safely show the UI that depends on the theme
  useEffect(() => {
    setMounted(true);
  }, []);

  return (
    <>
      <Helmet>
        <html lang="en" className={theme} />
        <title>{title}</title>
        <meta name="description" content={description} />
        <meta property="og:title" content={title} />
        <meta property="og:description" content={description} />
        <meta property="og:type" content="website" />
        <meta property="og:image" content="/og-image.jpg" />
        <meta name="twitter:card" content="summary_large_image" />
        <meta name="twitter:title" content={title} />
        <meta name="twitter:description" content={description} />
        {/* Theme color for browser UI */}
        <meta name="theme-color" content={theme === 'dark' ? '#0f172a' : '#ffffff'} />
      </Helmet>
      
      <div className={`relative ${mounted ? 'animate-fade-in' : 'opacity-0'}`}>
        <SkipToContent />
        
        {showAnimatedBackground && mounted && <AnimatedBackground />}
        
        <div className="flex flex-col min-h-screen">
          <Header />
          
          <main id="main-content" className="flex-grow pt-20">
            {children}
          </main>
          
          <Footer />
        </div>
        
        {/* Floating Theme Toggle */}
        <div className="fixed bottom-6 right-6 z-40">
          <ThemeToggle className="shadow-lg" />
        </div>
      </div>
    </>
  );
};

// Wrapper component that provides the ThemeContext
const Layout: React.FC<LayoutProps> = (props) => {
  return (
    <ThemeProvider>
      <LayoutContent {...props} />
    </ThemeProvider>
  );
};

export default Layout;