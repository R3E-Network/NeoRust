import React, { useEffect, useRef } from 'react';
import { useTheme } from '../context/ThemeContext';

const AnimatedBackground: React.FC = () => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const { theme } = useTheme();
  
  useEffect(() => {
    if (!canvasRef.current) return;
    
    const canvas = canvasRef.current;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;
    
    // Set canvas dimensions
    const resizeCanvas = () => {
      canvas.width = window.innerWidth;
      canvas.height = window.innerHeight;
    };
    
    window.addEventListener('resize', resizeCanvas);
    resizeCanvas();
    
    // Particle configuration
    const particleCount = 50; // Increased particle count
    const maxSize = 3;
    const baseSpeed = 0.3;
    const connectionDistance = 180; // Increased connection distance
    const primaryColor = theme === 'dark' ? '#10b981' : '#059669';
    const secondaryColor = theme === 'dark' ? '#ffffff' : '#1a1a1a';
    
    // Mouse interaction
    let mouseX = 0;
    let mouseY = 0;
    let mouseRadius = 150;
    
    canvas.addEventListener('mousemove', (e) => {
      mouseX = e.clientX;
      mouseY = e.clientY;
    });
    
    // Create particles
    const particles: Array<{
      x: number;
      y: number;
      size: number;
      speedX: number;
      speedY: number;
      opacity: number;
      hue: number;
    }> = [];
    
    for (let i = 0; i < particleCount; i++) {
      particles.push({
        x: Math.random() * canvas.width,
        y: Math.random() * canvas.height,
        size: Math.random() * maxSize + 0.5,
        speedX: (Math.random() - 0.5) * baseSpeed,
        speedY: (Math.random() - 0.5) * baseSpeed,
        opacity: Math.random() * 0.5 + 0.3,
        hue: Math.random() * 30 - 15, // Color variation
      });
    }
    
    // Animation function
    const animate = () => {
      requestAnimationFrame(animate);
      
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      
      // Update and draw particles
      for (let i = 0; i < particles.length; i++) {
        const p = particles[i];
        
        // Mouse interaction - particles move away from cursor
        const dx = mouseX - p.x;
        const dy = mouseY - p.y;
        const distance = Math.sqrt(dx * dx + dy * dy);
        
        if (distance < mouseRadius) {
          const forceFactor = (1 - distance / mouseRadius) * 0.05;
          p.speedX -= dx * forceFactor;
          p.speedY -= dy * forceFactor;
        }
        
        // Apply speed limits
        const speedLimit = 2;
        const currentSpeed = Math.sqrt(p.speedX * p.speedX + p.speedY * p.speedY);
        if (currentSpeed > speedLimit) {
          p.speedX = (p.speedX / currentSpeed) * speedLimit;
          p.speedY = (p.speedY / currentSpeed) * speedLimit;
        }
        
        // Apply friction
        p.speedX *= 0.98;
        p.speedY *= 0.98;
        
        // Update position
        p.x += p.speedX;
        p.y += p.speedY;
        
        // Boundary check with smooth wrapping
        if (p.x < -50) p.x = canvas.width + 50;
        if (p.x > canvas.width + 50) p.x = -50;
        if (p.y < -50) p.y = canvas.height + 50;
        if (p.y > canvas.height + 50) p.y = -50;
        
        // Draw particle with custom color
        ctx.beginPath();
        ctx.arc(p.x, p.y, p.size, 0, Math.PI * 2);
        
        // Create a shimmering effect with varying opacity
        const pulse = Math.sin(Date.now() * 0.003 + i) * 0.1 + 0.9;
        const particleColor = theme === 'dark' ? 
          `hsla(${160 + p.hue}, 70%, 60%, ${p.opacity * pulse})` : 
          `hsla(${160 + p.hue}, 70%, 40%, ${p.opacity * pulse})`;
        
        ctx.fillStyle = particleColor;
        ctx.fill();
        
        // Connect particles with gradient lines
        for (let j = i + 1; j < particles.length; j++) {
          const p2 = particles[j];
          const distance = Math.sqrt(
            Math.pow(p.x - p2.x, 2) + Math.pow(p.y - p2.y, 2)
          );
          
          if (distance < connectionDistance) {
            const opacity = (1 - distance / connectionDistance) * 0.15;
            
            // Create gradient line
            const gradient = ctx.createLinearGradient(p.x, p.y, p2.x, p2.y);
            gradient.addColorStop(0, `${primaryColor}${Math.floor(opacity * 255).toString(16).padStart(2, '0')}`);
            gradient.addColorStop(1, `${secondaryColor}${Math.floor(opacity * 255).toString(16).padStart(2, '0')}`);
            
            ctx.beginPath();
            ctx.moveTo(p.x, p.y);
            ctx.lineTo(p2.x, p2.y);
            ctx.strokeStyle = gradient;
            ctx.lineWidth = (p.size + p2.size) * 0.1;
            ctx.stroke();
          }
        }
      }
    };
    
    // Start animation
    animate();
    
    // Cleanup
    return () => {
      window.removeEventListener('resize', resizeCanvas);
      canvas.removeEventListener('mousemove', () => {});
    };
  }, [theme]);
  
  return (
    <canvas
      ref={canvasRef}
      className="absolute top-0 left-0 w-full h-full -z-10 opacity-30"
    />
  );
};

export default AnimatedBackground;