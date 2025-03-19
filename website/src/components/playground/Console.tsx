import React, { useRef, useEffect } from 'react';

interface ConsoleProps {
  output: string[];
  isLoading?: boolean;
}

const Console: React.FC<ConsoleProps> = ({ output, isLoading = false }) => {
  const consoleEndRef = useRef<HTMLDivElement>(null);

  // Auto-scroll to the bottom when new output is added
  useEffect(() => {
    if (consoleEndRef.current) {
      consoleEndRef.current.scrollIntoView({ behavior: 'smooth' });
    }
  }, [output]);

  return (
    <div className="relative rounded-lg overflow-hidden border border-slate-700 shadow-lg">
      <div className="flex items-center justify-between px-4 py-2 bg-slate-800 border-b border-slate-700">
        <div className="flex items-center space-x-2">
          <div className="w-3 h-3 bg-red-500 rounded-full"></div>
          <div className="w-3 h-3 bg-yellow-500 rounded-full"></div>
          <div className="w-3 h-3 bg-green-500 rounded-full"></div>
        </div>
        <div className="text-xs text-gray-400">Console Output</div>
      </div>

      <div className="bg-slate-900 p-4 h-64 overflow-y-auto text-sm font-mono">
        {output.length === 0 && !isLoading ? (
          <div className="text-gray-400 italic">
            Run code to see output here...
          </div>
        ) : (
          output.map((line, index) => (
            <div key={index} className="mb-1">
              {line.startsWith('error:') ? (
                <span className="text-red-400">{line}</span>
              ) : line.startsWith('warning:') ? (
                <span className="text-yellow-400">{line}</span>
              ) : (
                <span className="text-gray-200">{line}</span>
              )}
            </div>
          ))
        )}

        {isLoading && (
          <div className="flex items-center space-x-2 text-green-400">
            <svg className="animate-spin h-4 w-4" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
              <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
              <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
            </svg>
            <span>Running code...</span>
          </div>
        )}

        <div ref={consoleEndRef}></div>
      </div>
    </div>
  );
};

export default Console;