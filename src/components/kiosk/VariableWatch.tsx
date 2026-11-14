import React, { useState } from 'react';
import { Eye, SkipBack, SkipForward } from 'lucide-react';
import type { VariableTraceEntry } from '../../types/ipc';

interface VariableWatchProps {
  trace: VariableTraceEntry[];
}

export const VariableWatch: React.FC<VariableWatchProps> = ({ trace }) => {
  const [selectedIdx, setSelectedIdx] = useState<number | null>(null);

  if (trace.length === 0) {
    return (
      <div
        className="p-3.5 rounded-xl border text-xs"
        style={{
          backgroundColor: 'var(--bg-canvas)',
          borderColor: 'var(--border-subtle)',
          color: 'var(--text-secondary)',
        }}
      >
        <div className="flex items-center gap-1.5 font-semibold mb-1" style={{ color: 'var(--text-primary)' }}>
          <Eye className="w-3.5 h-3.5" style={{ color: 'var(--accent-primary)' }} />
          Inspetor de Variáveis (Step Watch)
        </div>
        Execute seu código para visualizar a evolução das variáveis passo a passo (até 50 iterações).
      </div>
    );
  }

  const activeIdx = selectedIdx !== null && selectedIdx < trace.length ? selectedIdx : trace.length - 1;
  const activeEntry = trace[activeIdx];
  const allVarNames = Array.from(
    new Set(trace.flatMap((entry) => Object.keys(entry.variables)))
  );

  return (
    <div
      className="rounded-xl border overflow-hidden"
      style={{
        backgroundColor: 'var(--bg-surface-elevated)',
        borderColor: 'var(--border-subtle)',
      }}
    >
      <div
        className="px-3.5 py-2 border-b flex items-center justify-between gap-2"
        style={{ borderColor: 'var(--border-subtle)' }}
      >
        <div className="flex items-center gap-1.5 text-xs font-bold">
          <Eye className="w-3.5 h-3.5" style={{ color: 'var(--accent-primary)' }} />
          <span>Inspetor de Variáveis</span>
          <span className="font-mono-code text-[11px]" style={{ color: 'var(--text-secondary)' }}>
            ({trace.length} passos)
          </span>
        </div>

        <div className="flex items-center gap-1">
          <button
            type="button"
            aria-label="Passo anterior"
            disabled={activeIdx <= 0}
            onClick={() => setSelectedIdx(Math.max(0, activeIdx - 1))}
            className="p-1 rounded border disabled:opacity-40 cursor-pointer"
            style={{
              backgroundColor: 'var(--bg-canvas)',
              borderColor: 'var(--border-subtle)',
            }}
          >
            <SkipBack className="w-3 h-3" />
          </button>
          <span className="text-[11px] font-mono-code px-1.5">
            #{activeEntry.step} (L{activeEntry.line})
          </span>
          <button
            type="button"
            aria-label="Próximo passo"
            disabled={activeIdx >= trace.length - 1}
            onClick={() => setSelectedIdx(Math.min(trace.length - 1, activeIdx + 1))}
            className="p-1 rounded border disabled:opacity-40 cursor-pointer"
            style={{
              backgroundColor: 'var(--bg-canvas)',
              borderColor: 'var(--border-subtle)',
            }}
          >
            <SkipForward className="w-3 h-3" />
          </button>
        </div>
      </div>

      {/* Snapshot Table */}
      <div className="max-h-48 overflow-auto">
        <table className="w-full text-left border-collapse font-mono-code text-[11px]">
          <thead>
            <tr style={{ backgroundColor: 'var(--bg-canvas)', color: 'var(--text-secondary)' }}>
              <th className="px-2.5 py-1.5 border-b" style={{ borderColor: 'var(--border-subtle)' }}>
                Passo
              </th>
              <th className="px-2.5 py-1.5 border-b" style={{ borderColor: 'var(--border-subtle)' }}>
                Linha
              </th>
              {allVarNames.map((v) => (
                <th
                  key={v}
                  className="px-2.5 py-1.5 border-b"
                  style={{ borderColor: 'var(--border-subtle)' }}
                >
                  {v}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {trace.map((entry, idx) => {
              const isSelected = idx === activeIdx;
              return (
                <tr
                  key={`${entry.step}-${idx}`}
                  onClick={() => setSelectedIdx(idx)}
                  className="cursor-pointer transition-colors"
                  style={{
                    backgroundColor: isSelected
                      ? 'rgba(6, 182, 212, 0.14)'
                      : 'transparent',
                  }}
                >
                  <td
                    className="px-2.5 py-1 border-b"
                    style={{ borderColor: 'var(--border-subtle)', color: 'var(--text-secondary)' }}
                  >
                    {entry.step}
                  </td>
                  <td
                    className="px-2.5 py-1 border-b"
                    style={{ borderColor: 'var(--border-subtle)', color: 'var(--accent-primary)' }}
                  >
                    L{entry.line}
                  </td>
                  {allVarNames.map((v) => (
                    <td
                      key={v}
                      className="px-2.5 py-1 border-b whitespace-nowrap"
                      style={{ borderColor: 'var(--border-subtle)' }}
                    >
                      {entry.variables[v] ?? '—'}
                    </td>
                  ))}
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
    </div>
  );
};
