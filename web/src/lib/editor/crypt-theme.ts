// The Monaco "Crypt" theme, taken from docs/palette.md. Raw hex values are allowed here only,
// because the Monaco API takes literal colours. Bright green is never used: it means pass.
import type * as Monaco from 'monaco-editor';

export const cryptTheme: Monaco.editor.IStandaloneThemeData = {
  base: 'vs-dark',
  inherit: true,
  rules: [
    { token: '', foreground: 'e6e1d3' },
    { token: 'comment', foreground: '7d8d8b', fontStyle: 'italic' },
    { token: 'keyword', foreground: 'd9a25b' },
    { token: 'string', foreground: '9db3a2' },
    { token: 'number', foreground: 'e8b978' },
    { token: 'type', foreground: '8fb3c0' },
    { token: 'type.identifier', foreground: '8fb3c0' },
    { token: 'function', foreground: 'e6e1d3', fontStyle: 'bold' },
    { token: 'variable', foreground: 'e6e1d3' },
    { token: 'constant', foreground: 'e8b978' },
    { token: 'operator', foreground: '9aa9a6' },
    { token: 'delimiter', foreground: '9aa9a6' },
    { token: 'tag', foreground: '8a9e8f' },
    { token: 'regexp', foreground: '9db3a2' },
    { token: 'invalid', foreground: 'e0675a' }
  ],
  colors: {
    'editor.background': '#0f1719',
    'editor.foreground': '#e6e1d3',
    'editorLineNumber.foreground': '#5d7265',
    'editorLineNumber.activeForeground': '#9aa9a6',
    'editor.lineHighlightBackground': '#162226',
    'editor.selectionBackground': '#26332c',
    'editor.inactiveSelectionBackground': '#1b2a2e',
    'editorCursor.foreground': '#d9a25b',
    'editorIndentGuide.background1': '#23353a',
    'editorIndentGuide.activeBackground1': '#33494f',
    'editorBracketMatch.border': '#8a9e8f',
    'editorBracketMatch.background': '#26332c',
    'editorError.foreground': '#e0675a',
    'editorWarning.foreground': '#d9a25b',
    'editorInfo.foreground': '#8fb3c0',
    'editorGutter.background': '#0f1719',
    'editorWidget.background': '#162226',
    'editorWidget.border': '#23353a',
    'editorSuggestWidget.selectedBackground': '#26332c',
    'scrollbarSlider.background': '#23353a99',
    'scrollbarSlider.hoverBackground': '#33494fcc',
    'minimap.background': '#0f1719'
  }
};
