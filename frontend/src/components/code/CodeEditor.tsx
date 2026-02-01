import { useEffect, useRef } from 'react';
import { EditorState } from '@codemirror/state';
import { EditorView, keymap, lineNumbers, highlightActiveLine, highlightActiveLineGutter } from '@codemirror/view';
import { defaultKeymap, history, historyKeymap } from '@codemirror/commands';
import { syntaxHighlighting, defaultHighlightStyle, bracketMatching } from '@codemirror/language';
import { html } from '@codemirror/lang-html';
import { css } from '@codemirror/lang-css';
import { javascript } from '@codemirror/lang-javascript';

export type Language = 'html' | 'css' | 'javascript';

interface CodeEditorProps {
  value: string;
  onChange: (value: string) => void;
  language: Language;
  placeholder?: string;
  readOnly?: boolean;
  className?: string;
}

const darkTheme = EditorView.theme({
  '&': {
    backgroundColor: 'var(--color-surface)',
    color: 'var(--color-foreground)',
    height: '100%',
  },
  '.cm-content': {
    caretColor: 'var(--color-primary-400)',
    fontFamily: 'var(--font-mono)',
    fontSize: '14px',
    lineHeight: '1.6',
    padding: '8px 0',
  },
  '.cm-cursor': {
    borderLeftColor: 'var(--color-primary-400)',
  },
  '&.cm-focused .cm-selectionBackground, .cm-selectionBackground': {
    backgroundColor: 'var(--color-primary-600)',
    opacity: '0.3',
  },
  '.cm-gutters': {
    backgroundColor: 'var(--color-surface)',
    color: 'var(--color-muted-foreground)',
    border: 'none',
    borderRight: '1px solid var(--color-border)',
  },
  '.cm-activeLineGutter': {
    backgroundColor: 'var(--color-surface-hover)',
  },
  '.cm-activeLine': {
    backgroundColor: 'var(--color-surface-hover)',
  },
  '.cm-line': {
    padding: '0 12px',
  },
  '.cm-placeholder': {
    color: 'var(--color-muted-foreground)',
    fontStyle: 'italic',
  },
});

function getLanguageExtension(language: Language) {
  switch (language) {
    case 'html':
      return html();
    case 'css':
      return css();
    case 'javascript':
      return javascript();
  }
}

export function CodeEditor({
  value,
  onChange,
  language,
  placeholder,
  readOnly = false,
  className = '',
}: CodeEditorProps) {
  const editorRef = useRef<HTMLDivElement>(null);
  const viewRef = useRef<EditorView | null>(null);

  useEffect(() => {
    if (!editorRef.current) return;

    const extensions = [
      lineNumbers(),
      highlightActiveLine(),
      highlightActiveLineGutter(),
      history(),
      bracketMatching(),
      syntaxHighlighting(defaultHighlightStyle),
      keymap.of([...defaultKeymap, ...historyKeymap]),
      getLanguageExtension(language),
      darkTheme,
      EditorView.updateListener.of((update) => {
        if (update.docChanged) {
          onChange(update.state.doc.toString());
        }
      }),
    ];

    if (readOnly) {
      extensions.push(EditorState.readOnly.of(true));
    }

    if (placeholder) {
      extensions.push(EditorView.contentAttributes.of({ 'data-placeholder': placeholder }));
    }

    const state = EditorState.create({
      doc: value,
      extensions,
    });

    const view = new EditorView({
      state,
      parent: editorRef.current,
    });

    viewRef.current = view;

    return () => {
      view.destroy();
    };
  }, [language, readOnly, placeholder]); // Don't include value in deps to avoid recreation

  // Update content when value changes externally
  useEffect(() => {
    const view = viewRef.current;
    if (view && value !== view.state.doc.toString()) {
      view.dispatch({
        changes: {
          from: 0,
          to: view.state.doc.length,
          insert: value,
        },
      });
    }
  }, [value]);

  return (
    <div
      ref={editorRef}
      className={`h-full w-full overflow-auto ${className}`}
    />
  );
}
