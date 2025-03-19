import React, { useState, useEffect, useRef } from 'react';
import MonacoEditor, { EditorDidMount } from '@monaco-editor/react';

interface EditorProps {
  initialCode: string;
  onChange?: (value: string) => void;
  language?: string;
  theme?: string;
  height?: string;
}

const Editor: React.FC<EditorProps> = ({
  initialCode,
  onChange,
  language = 'rust',
  theme = 'vs-dark',
  height = '70vh',
}) => {
  const [code, setCode] = useState(initialCode);
  const editorRef = useRef<any>(null);

  useEffect(() => {
    // Sync with initialCode if it changes externally
    setCode(initialCode);
  }, [initialCode]);

  const handleEditorDidMount: EditorDidMount = (editor, monaco) => {
    editorRef.current = editor;
    
    // Configure editor settings
    editor.updateOptions({
      minimap: { enabled: false },
      scrollBeyondLastLine: false,
      fontSize: 14,
      fontFamily: 'JetBrains Mono, Menlo, Monaco, "Courier New", monospace',
      cursorBlinking: 'smooth',
      lineNumbersMinChars: 3,
      renderLineHighlight: 'all',
      cursorSmoothCaretAnimation: true,
      smoothScrolling: true,
    });
    
    // Setup custom Rust theme
    monaco.editor.defineTheme('neo-dark', {
      base: 'vs-dark',
      inherit: true,
      rules: [
        { token: 'comment', foreground: '6A9955' },
        { token: 'keyword', foreground: '569CD6', fontStyle: 'bold' },
        { token: 'string', foreground: 'CE9178' },
        { token: 'number', foreground: 'B5CEA8' },
        { token: 'type', foreground: '4EC9B0' },
        { token: 'function', foreground: 'DCDCAA' },
        { token: 'macro', foreground: 'C586C0' },
        { token: 'identifier', foreground: '9CDCFE' },
      ],
      colors: {
        'editor.background': '#1E1E2E',
        'editor.foreground': '#D4D4D4',
        'editorCursor.foreground': '#10b981',
        'editor.lineHighlightBackground': '#2A2A40',
        'editorLineNumber.foreground': '#858585',
        'editor.selectionBackground': '#264F78',
        'editor.inactiveSelectionBackground': '#3A3D41',
      },
    });
    
    monaco.editor.setTheme('neo-dark');
    
    // Configure Rust language settings
    monaco.languages.setLanguageConfiguration('rust', {
      wordPattern: /(-?\d*\.\d\w*)|([^\`\~\!\@\#\%\^\&\*\(\)\-\=\+\[\{\]\}\\\|\;\:\'\"\,\.\<\>\/\?\s]+)/g,
      comments: {
        lineComment: '//',
        blockComment: ['/*', '*/'],
      },
      brackets: [
        ['{', '}'],
        ['[', ']'],
        ['(', ')'],
      ],
      autoClosingPairs: [
        { open: '{', close: '}' },
        { open: '[', close: ']' },
        { open: '(', close: ')' },
        { open: '"', close: '"', notIn: ['string'] },
        { open: '\'', close: '\'', notIn: ['string', 'comment'] },
      ],
      surroundingPairs: [
        { open: '{', close: '}' },
        { open: '[', close: ']' },
        { open: '(', close: ')' },
        { open: '"', close: '"' },
        { open: '\'', close: '\'' },
        { open: '<', close: '>' },
      ],
      folding: {
        markers: {
          start: new RegExp("^\\s*// region:\\b"),
          end: new RegExp("^\\s*// endregion\\b"),
        },
      },
    });
    
    // Focus the editor
    editor.focus();
  };

  const handleEditorChange = (value: string | undefined) => {
    if (value !== undefined) {
      setCode(value);
      if (onChange) {
        onChange(value);
      }
    }
  };

  return (
    <div className="rounded-lg overflow-hidden border border-slate-700 shadow-lg">
      <MonacoEditor
        height={height}
        language={language}
        theme={theme}
        value={code}
        onChange={handleEditorChange}
        onMount={handleEditorDidMount}
        options={{
          readOnly: false,
          minimap: { enabled: false },
          scrollBeyondLastLine: false,
          fontSize: 14,
          lineNumbers: 'on',
          renderFinalNewline: true,
          wordWrap: 'on',
          tabSize: 4,
          insertSpaces: true,
          colorDecorators: true,
          links: true,
        }}
        loading={
          <div className="flex items-center justify-center h-full w-full bg-slate-800">
            <div className="text-green-400 text-lg">Loading Editor...</div>
          </div>
        }
      />
    </div>
  );
};

export default Editor;