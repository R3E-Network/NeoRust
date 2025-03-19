/** @type {import('tailwindcss').Config} */
module.exports = {
  darkMode: 'class',
  content: [
    "./src/pages/**/*.{js,jsx,ts,tsx}",
    "./src/components/**/*.{js,jsx,ts,tsx}",
    "./src/templates/**/*.{js,jsx,ts,tsx}",
    "./src/context/**/*.{js,jsx,ts,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        'neo-green': {
          50: '#f0fdf9',
          100: '#ddfcf1',
          200: '#baf8e2',
          300: '#4CFFB3',
          400: '#10b981',
          500: '#00E599',
          600: '#059669',
          700: '#047857',
          800: '#065f46',
          900: '#064e3b',
          950: '#022c22',
        },
        'neo-dark': {
          50: '#f6f7f9',
          100: '#eceef2',
          200: '#d5dae3',
          300: '#b2bbcb',
          400: '#8896ae',
          500: '#6a7994',
          600: '#546079',
          700: '#444e62',
          800: '#1E293B',
          900: '#0f172a',
          950: '#0A0F1A',
        },
      },
      keyframes: {
        'pulse-slow': {
          '0%, 100%': { opacity: 0.8 },
          '50%': { opacity: 0.4 },
        },
        'fade-in': {
          '0%': { opacity: 0 },
          '100%': { opacity: 1 },
        },
        'fade-in-up': {
          '0%': { opacity: 0, transform: 'translateY(10px)' },
          '100%': { opacity: 1, transform: 'translateY(0)' },
        },
        'slide-in-right': {
          '0%': { transform: 'translateX(20px)', opacity: 0 },
          '100%': { transform: 'translateX(0)', opacity: 1 },
        },
        'slide-in-left': {
          '0%': { transform: 'translateX(-20px)', opacity: 0 },
          '100%': { transform: 'translateX(0)', opacity: 1 },
        },
        'grow': {
          '0%': { transform: 'scale(0.95)', opacity: 0 },
          '100%': { transform: 'scale(1)', opacity: 1 },
        },
      },
      animation: {
        'pulse-slow': 'pulse-slow 3s cubic-bezier(0.4, 0, 0.6, 1) infinite',
        'fade-in': 'fade-in 0.5s ease-out',
        'fade-in-up': 'fade-in-up 0.5s ease-out',
        'slide-in-right': 'slide-in-right 0.5s ease-out',
        'slide-in-left': 'slide-in-left 0.5s ease-out',
        'grow': 'grow 0.3s ease-out',
      },
      typography: (theme) => ({
        DEFAULT: {
          css: {
            color: theme('colors.gray.200'),
            a: {
              color: theme('colors.neo-green.400'),
              '&:hover': {
                color: theme('colors.neo-green.300'),
              },
            },
            h1: {
              color: theme('colors.gray.100'),
            },
            h2: {
              color: theme('colors.gray.100'),
            },
            h3: {
              color: theme('colors.gray.100'),
            },
            h4: {
              color: theme('colors.gray.100'),
            },
            strong: {
              color: theme('colors.gray.100'),
            },
            code: {
              color: theme('colors.neo-green.400'),
              backgroundColor: theme('colors.gray.800'),
              padding: theme('spacing.1'),
              borderRadius: theme('borderRadius.md'),
            },
            pre: {
              backgroundColor: theme('colors.gray.900'),
              border: `1px solid ${theme('colors.gray.700')}`,
              borderRadius: theme('borderRadius.xl'),
            },
            blockquote: {
              color: theme('colors.gray.300'),
              borderLeftColor: theme('colors.neo-green.500'),
            },
            hr: {
              borderColor: theme('colors.gray.700'),
            },
          },
        },
        light: {
          css: {
            color: theme('colors.gray.700'),
            a: {
              color: theme('colors.neo-green.600'),
              '&:hover': {
                color: theme('colors.neo-green.700'),
              },
            },
            h1: {
              color: theme('colors.gray.900'),
            },
            h2: {
              color: theme('colors.gray.900'),
            },
            h3: {
              color: theme('colors.gray.900'),
            },
            h4: {
              color: theme('colors.gray.900'),
            },
            strong: {
              color: theme('colors.gray.900'),
            },
            code: {
              color: theme('colors.neo-green.700'),
              backgroundColor: theme('colors.gray.100'),
              padding: theme('spacing.1'),
              borderRadius: theme('borderRadius.md'),
            },
            pre: {
              backgroundColor: theme('colors.gray.100'),
              border: `1px solid ${theme('colors.gray.300')}`,
              borderRadius: theme('borderRadius.xl'),
            },
            blockquote: {
              color: theme('colors.gray.700'),
              borderLeftColor: theme('colors.neo-green.500'),
            },
            hr: {
              borderColor: theme('colors.gray.300'),
            },
          },
        },
      }),
      backgroundImage: {
        'gradient-radial': 'radial-gradient(var(--tw-gradient-stops))',
        'grid-pattern': 'linear-gradient(to right, var(--grid-color) 1px, transparent 1px), linear-gradient(to bottom, var(--grid-color) 1px, transparent 1px)',
      },
    },
  },
  plugins: [
    require('@tailwindcss/typography'),
  ],
}