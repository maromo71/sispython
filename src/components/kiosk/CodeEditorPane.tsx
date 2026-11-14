import React, { useEffect, useRef } from 'react';
import { EditorState, RangeSetBuilder, StateEffect, StateField } from '@codemirror/state';
import {
  Decoration,
  type DecorationSet,
  EditorView,
  drawSelection,
  highlightActiveLine,
  highlightActiveLineGutter,
  keymap,
  lineNumbers,
} from '@codemirror/view';
import {
  bracketMatching,
  defaultHighlightStyle,
  indentOnInput,
  syntaxHighlighting,
} from '@codemirror/language';
import { python } from '@codemirror/lang-python';
import { defaultKeymap, history, historyKeymap, indentWithTab } from '@codemirror/commands';
import { AlertCircle, Loader2, Play, RotateCcw, ShieldAlert } from 'lucide-react';
import { useThemeStore } from '../../store/useThemeStore';

interface CodeEditorPaneProps {
  levelId: string;
  code: string;
  isBoss: boolean;
  isRunning: boolean;
  focusLine: number | null;
  emptyWarning: string | null;
  onChange: (value: string) => void;
  onRun: () => void;
  onReset: () => void;
}

const setFocusLineEffect = StateEffect.define<number | null>();

const focusLineField = StateField.define<DecorationSet>({
  create() {
    return Decoration.none;
  },
  update(decorations, tr) {
    for (const effect of tr.effects) {
      if (effect.is(setFocusLineEffect)) {
        const lineNum = effect.value;
        if (!lineNum || lineNum < 1 || lineNum > tr.state.doc.lines) {
          return Decoration.none;
        }
        const line = tr.state.doc.line(lineNum);
        const builder = new RangeSetBuilder<Decoration>();
        builder.add(
          line.from,
          line.from,
          Decoration.line({ class: 'cm-socratic-focus-line' })
        );
        return builder.finish();
      }
    }
    return decorations.map(tr.changes);
  },
  provide: (f) => EditorView.decorations.from(f),
});

export const CodeEditorPane: React.FC<CodeEditorPaneProps> = ({
  levelId,
  code,
  isBoss,
  isRunning,
  focusLine,
  emptyWarning,
  onChange,
  onRun,
  onReset,
}) => {
  const editorContainerRef = useRef<HTMLDivElement | null>(null);
  const viewRef = useRef<EditorView | null>(null);
  const onChangeRef = useRef(onChange);
  const onRunRef = useRef(onRun);
  const { theme } = useThemeStore();

  onChangeRef.current = onChange;
  onRunRef.current = onRun;

  useEffect(() => {
    const parent = editorContainerRef.current;
    if (!parent) return;

    const customTheme = EditorView.theme(
      {
        '&': {
          height: '100%',
          fontSize: '14px',
          backgroundColor: 'var(--bg-canvas)',
          color: 'var(--text-primary)',
        },
        '.cm-scroller': {
          fontFamily: "'JetBrains Mono', 'Fira Code', ui-monospace, monospace",
          fontVariantLigatures: 'contextual',
          overflow: 'auto',
        },
        '.cm-gutters': {
          backgroundColor: 'var(--bg-surface-panel)',
          color: 'var(--text-secondary)',
          borderRight: '1px solid var(--border-subtle)',
        },
        '.cm-activeLine': {
          backgroundColor:
            theme === 'dark-tech'
              ? 'rgba(6, 182, 212, 0.08)'
              : 'rgba(2, 132, 199, 0.08)',
        },
        '.cm-activeLineGutter': {
          backgroundColor:
            theme === 'dark-tech'
              ? 'rgba(6, 182, 212, 0.14)'
              : 'rgba(2, 132, 199, 0.14)',
        },
        '.cm-socratic-focus-line': {
          backgroundColor: 'var(--code-focus-line)',
          borderLeft: '3px solid var(--status-warning)',
        },
        '.cm-cursor': {
          borderLeftColor: 'var(--accent-primary)',
        },
      },
      { dark: theme === 'dark-tech' }
    );

    const state = EditorState.create({
      doc: code,
      extensions: [
        lineNumbers(),
        highlightActiveLineGutter(),
        history(),
        drawSelection(),
        indentOnInput(),
        syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
        bracketMatching(),
        highlightActiveLine(),
        python(),
        customTheme,
        focusLineField,
        keymap.of([
          ...defaultKeymap,
          ...historyKeymap,
          indentWithTab,
          {
            key: 'Mod-Enter',
            run: () => {
              onRunRef.current();
              return true;
            },
          },
        ]),
        EditorView.domEventHandlers({
          paste: (event) => {
            if (isBoss) {
              event.preventDefault();
              return true;
            }
            return false;
          },
        }),
        EditorView.updateListener.of((update) => {
          if (update.docChanged) {
            onChangeRef.current(update.state.doc.toString());
          }
        }),
      ],
    });

    const view = new EditorView({ state, parent });
    viewRef.current = view;

    return () => {
      view.destroy();
      viewRef.current = null;
    };
    // Recria editor ao trocar de fase, tema ou modo Boss
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [levelId, theme, isBoss]);

  // Sincroniza reset externo do código
  useEffect(() => {
    const view = viewRef.current;
    if (!view) return;
    const currentDoc = view.state.doc.toString();
    if (currentDoc !== code) {
      view.dispatch({
        changes: { from: 0, to: currentDoc.length, insert: code },
      });
    }
  }, [code]);

  // Aplica destaque na focus_line indicada pelo Agente Socrático
  useEffect(() => {
    const view = viewRef.current;
    if (!view) return;
    view.dispatch({
      effects: setFocusLineEffect.of(focusLine),
    });
    if (focusLine && focusLine >= 1 && focusLine <= view.state.doc.lines) {
      const line = view.state.doc.line(focusLine);
      view.dispatch({
        effects: EditorView.scrollIntoView(line.from, { y: 'center' }),
      });
    }
  }, [focusLine]);

  return (
    <section
      className="h-full flex flex-col overflow-hidden"
      style={{ backgroundColor: 'var(--bg-canvas)' }}
      aria-label="Editor de Código Python"
    >
      {/* Editor Toolbar */}
      <div
        className="px-3.5 py-2.5 border-b flex items-center justify-between gap-2"
        style={{
          backgroundColor: 'var(--bg-surface-panel)',
          borderColor: 'var(--border-subtle)',
        }}
      >
        <div className="flex items-center gap-2">
          <span className="text-xs font-mono-code font-semibold" style={{ color: 'var(--text-secondary)' }}>
            kiosk_solution.py
          </span>
          {isBoss && (
            <span
              className="inline-flex items-center gap-1 px-2 py-0.5 rounded text-[10px] font-semibold"
              style={{
                backgroundColor: 'rgba(168, 85, 247, 0.18)',
                color: 'var(--accent-gamification)',
              }}
              title="Colagem externa desabilitada em Desafios Chefão"
            >
              <ShieldAlert className="w-3 h-3" />
              Anti-Cola Ativo
            </span>
          )}
        </div>

        <div className="flex items-center gap-2">
          <button
            type="button"
            onClick={onReset}
            className="inline-flex items-center gap-1 px-2.5 py-1.5 rounded-lg border text-xs font-medium cursor-pointer"
            style={{
              backgroundColor: 'var(--bg-surface-elevated)',
              borderColor: 'var(--border-subtle)',
              color: 'var(--text-secondary)',
            }}
            title="Restaurar código inicial da fase"
          >
            <RotateCcw className="w-3.5 h-3.5" />
            <span className="hidden sm:inline">Reiniciar</span>
          </button>

          <button
            type="button"
            disabled={isRunning}
            onClick={onRun}
            className="inline-flex items-center gap-2 px-3.5 py-1.5 rounded-lg text-xs font-bold cursor-pointer disabled:opacity-60 transition-transform active:scale-98"
            style={{
              backgroundColor: 'var(--accent-primary)',
              color: '#0B0F19',
            }}
          >
            {isRunning ? (
              <Loader2 className="w-4 h-4 animate-spin" />
            ) : (
              <Play className="w-3.5 h-3.5 fill-current" />
            )}
            <span>{isRunning ? 'Executando...' : 'Executar (Ctrl+Enter)'}</span>
          </button>
        </div>
      </div>

      {/* Empty/Comments-Only Warning Banner */}
      {emptyWarning && (
        <div
          role="alert"
          className="px-4 py-2 text-xs flex items-center gap-2 border-b"
          style={{
            backgroundColor: 'rgba(245, 158, 11, 0.14)',
            borderColor: 'var(--status-warning)',
            color: 'var(--status-warning)',
          }}
        >
          <AlertCircle className="w-4 h-4 shrink-0" />
          <span>{emptyWarning}</span>
        </div>
      )}

      {/* CodeMirror Mount */}
      <div
        ref={editorContainerRef}
        className="flex-1 min-h-0 overflow-hidden"
        data-testid="code-editor-container"
      />
    </section>
  );
};
