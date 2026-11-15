import React, { useCallback, useEffect, useRef, useState } from 'react';
import { Bot, Code2, Play, Terminal } from 'lucide-react';
import { useKioskStore } from '../../store/useKioskStore';
import { useProgressionStore } from '../../store/useProgressionStore';
import { useThemeStore } from '../../store/useThemeStore';
import { MissionPanel } from '../kiosk/MissionPanel';
import { CodeEditorPane } from '../kiosk/CodeEditorPane';
import { ExecutionConsole } from '../kiosk/ExecutionConsole';

export const SplitKioskView: React.FC = () => {
  const { curriculum, submitActiveCode, requestHint } = useProgressionStore();
  const { toggleTheme } = useThemeStore();
  const {
    activeLevelId,
    codeByLevel,
    isRunning,
    isRequestingHint,
    lastResultByLevel,
    socraticMessagesByLevel,
    focusLine,
    activeMobileTab,
    emptySubmissionWarning,
    setActiveLevel,
    setCode,
    setFocusLine,
    setActiveMobileTab,
    resetLevelCode,
  } = useKioskStore();

  // Desktop split percentages: Left 28%, Center 42%, Right 30%
  const [leftPct, setLeftPct] = useState(28);
  const [rightPct, setRightPct] = useState(30);
  const containerRef = useRef<HTMLDivElement | null>(null);
  const draggingRef = useRef<'left' | 'right' | null>(null);

  const levels = curriculum?.levels ?? [];
  const activeLevel =
    levels.find((l) => l.id === activeLevelId) ??
    levels.find((l) => l.status === 'UNLOCKED') ??
    levels[0];

  const currentCode = activeLevel
    ? codeByLevel[activeLevel.id] ?? activeLevel.last_submitted_code ?? activeLevel.starter_code
    : '';

  const activeIndex = activeLevel ? levels.findIndex((l) => l.id === activeLevel.id) : -1;
  const nextLevel = activeIndex >= 0 && activeIndex + 1 < levels.length ? levels[activeIndex + 1] : null;
  const lastResult = activeLevel ? lastResultByLevel[activeLevel.id] ?? null : null;
  const messages = activeLevel ? socraticMessagesByLevel[activeLevel.id] ?? [] : [];
  const publicTests = activeLevel ? curriculum?.public_test_cases[activeLevel.id] ?? [] : [];

  const handleRun = useCallback(() => {
    if (!activeLevel || isRunning) return;
    submitActiveCode(activeLevel, currentCode);
  }, [activeLevel, currentCode, isRunning, submitActiveCode]);

  const handleHint = useCallback(() => {
    if (!activeLevel || isRequestingHint || activeLevel.level_type === 'BOSS') return;
    requestHint(activeLevel, currentCode);
  }, [activeLevel, currentCode, isRequestingHint, requestHint]);

  const handleAdvanceNext = useCallback(() => {
    if (!nextLevel || nextLevel.status === 'LOCKED') return;
    setActiveLevel(nextLevel.id, nextLevel.last_submitted_code ?? nextLevel.starter_code);
  }, [nextLevel, setActiveLevel]);

  // Global Keyboard Shortcuts (RNF05)
  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key === 'Enter') {
        e.preventDefault();
        if (lastResult?.verdict === 'PASSED') {
          handleAdvanceNext();
        }
        return;
      }
      if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
        e.preventDefault();
        handleRun();
        return;
      }
      if (e.altKey && e.key.toLowerCase() === 'h') {
        e.preventDefault();
        handleHint();
        return;
      }
      if (e.altKey && e.key.toLowerCase() === 't') {
        e.preventDefault();
        toggleTheme();
      }
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [handleRun, handleHint, handleAdvanceNext, lastResult, toggleTheme]);

  // Resizable panel pointer handling
  useEffect(() => {
    const onMouseMove = (e: MouseEvent) => {
      if (!draggingRef.current || !containerRef.current) return;
      const rect = containerRef.current.getBoundingClientRect();
      const relPct = ((e.clientX - rect.left) / rect.width) * 100;
      if (draggingRef.current === 'left') {
        const clamped = Math.max(20, Math.min(45, relPct));
        setLeftPct(clamped);
      } else if (draggingRef.current === 'right') {
        const fromRight = 100 - relPct;
        const clamped = Math.max(22, Math.min(45, fromRight));
        setRightPct(clamped);
      }
    };
    const onMouseUp = () => {
      draggingRef.current = null;
    };
    window.addEventListener('mousemove', onMouseMove);
    window.addEventListener('mouseup', onMouseUp);
    return () => {
      window.removeEventListener('mousemove', onMouseMove);
      window.removeEventListener('mouseup', onMouseUp);
    };
  }, []);

  if (!activeLevel) {
    return (
      <div className="flex-1 flex items-center justify-center p-8 text-sm" style={{ color: 'var(--text-secondary)' }}>
        Carregando fases do currículo...
      </div>
    );
  }

  const centerPct = Math.max(25, 100 - leftPct - rightPct);

  return (
    <div className="flex-1 min-h-0 flex flex-col">
      {/* Desktop 3-Column Resizable Layout (>= 1024px) */}
      <div
        ref={containerRef}
        className="hidden lg:flex flex-1 min-h-0 w-full overflow-hidden"
        data-testid="desktop-split-kiosk"
      >
        <div style={{ width: `${leftPct}%` }} className="h-full min-w-0">
          <MissionPanel
            level={activeLevel}
            code={currentCode}
            messages={messages}
            isRequestingHint={isRequestingHint}
            onRequestHint={handleHint}
            onJumpToLine={(line) => setFocusLine(line)}
          />
        </div>

        <div
          role="separator"
          aria-orientation="vertical"
          onMouseDown={() => {
            draggingRef.current = 'left';
          }}
          className="w-1.5 cursor-col-resize hover:opacity-100 opacity-60 transition-opacity shrink-0"
          style={{ backgroundColor: 'var(--border-subtle)' }}
        />

        <div style={{ width: `${centerPct}%` }} className="h-full min-w-0">
          <CodeEditorPane
            levelId={activeLevel.id}
            code={currentCode}
            isBoss={activeLevel.level_type === 'BOSS'}
            isRunning={isRunning}
            focusLine={focusLine}
            emptyWarning={emptySubmissionWarning}
            onChange={(val) => setCode(activeLevel.id, val)}
            onRun={handleRun}
            onReset={() => resetLevelCode(activeLevel.id, activeLevel.starter_code)}
          />
        </div>

        <div
          role="separator"
          aria-orientation="vertical"
          onMouseDown={() => {
            draggingRef.current = 'right';
          }}
          className="w-1.5 cursor-col-resize hover:opacity-100 opacity-60 transition-opacity shrink-0"
          style={{ backgroundColor: 'var(--border-subtle)' }}
        />

        <div style={{ width: `${rightPct}%` }} className="h-full min-w-0">
          <ExecutionConsole
            level={activeLevel}
            publicTestCases={publicTests}
            isRunning={isRunning}
            result={lastResult}
            hasNextLevel={Boolean(nextLevel && nextLevel.status !== 'LOCKED')}
            onAdvanceNextLevel={handleAdvanceNext}
          />
        </div>
      </div>

      {/* Compact / Mobile Tabbed Layout (< 1024px) */}
      <div className="flex lg:hidden flex-1 min-h-0 flex-col relative">
        <div className="flex-1 min-h-0 overflow-hidden pb-14">
          {activeMobileTab === 'mission' && (
            <MissionPanel
              level={activeLevel}
              code={currentCode}
              messages={messages}
              isRequestingHint={isRequestingHint}
              onRequestHint={handleHint}
              onJumpToLine={(line) => {
                setFocusLine(line);
                setActiveMobileTab('editor');
              }}
            />
          )}
          {activeMobileTab === 'editor' && (
            <CodeEditorPane
              levelId={activeLevel.id}
              code={currentCode}
              isBoss={activeLevel.level_type === 'BOSS'}
              isRunning={isRunning}
              focusLine={focusLine}
              emptyWarning={emptySubmissionWarning}
              onChange={(val) => setCode(activeLevel.id, val)}
              onRun={handleRun}
              onReset={() => resetLevelCode(activeLevel.id, activeLevel.starter_code)}
            />
          )}
          {activeMobileTab === 'console' && (
            <ExecutionConsole
              level={activeLevel}
              publicTestCases={publicTests}
              isRunning={isRunning}
              result={lastResult}
              hasNextLevel={Boolean(nextLevel && nextLevel.status !== 'LOCKED')}
              onAdvanceNextLevel={handleAdvanceNext}
            />
          )}
        </div>

        {/* Floating Execute FAB */}
        <button
          type="button"
          onClick={handleRun}
          disabled={isRunning}
          aria-label="Executar código"
          className="fixed right-4 bottom-16 z-30 flex items-center gap-1.5 px-4 py-2.5 rounded-full shadow-lg text-xs font-bold cursor-pointer"
          style={{
            backgroundColor: 'var(--accent-primary)',
            color: '#0B0F19',
          }}
        >
          <Play className="w-3.5 h-3.5 fill-current" />
          Executar
        </button>

        {/* Bottom Navigation Tabs */}
        <div
          className="fixed bottom-0 inset-x-0 z-20 grid grid-cols-3 border-t h-14"
          style={{
            backgroundColor: 'var(--bg-surface-panel)',
            borderColor: 'var(--border-subtle)',
          }}
        >
          <button
            type="button"
            onClick={() => setActiveMobileTab('mission')}
            className="flex flex-col items-center justify-center gap-0.5 text-[11px] font-semibold cursor-pointer"
            style={{
              color:
                activeMobileTab === 'mission'
                  ? 'var(--accent-primary)'
                  : 'var(--text-secondary)',
            }}
          >
            <Bot className="w-4 h-4" />
            1. Missão & IA
          </button>
          <button
            type="button"
            onClick={() => setActiveMobileTab('editor')}
            className="flex flex-col items-center justify-center gap-0.5 text-[11px] font-semibold cursor-pointer"
            style={{
              color:
                activeMobileTab === 'editor'
                  ? 'var(--accent-primary)'
                  : 'var(--text-secondary)',
            }}
          >
            <Code2 className="w-4 h-4" />
            2. Editor
          </button>
          <button
            type="button"
            onClick={() => setActiveMobileTab('console')}
            className="flex flex-col items-center justify-center gap-0.5 text-[11px] font-semibold cursor-pointer"
            style={{
              color:
                activeMobileTab === 'console'
                  ? 'var(--accent-primary)'
                  : 'var(--text-secondary)',
            }}
          >
            <Terminal className="w-4 h-4" />
            3. Console
          </button>
        </div>
      </div>
    </div>
  );
};
