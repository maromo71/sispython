import React from 'react';
import {
  AlertTriangle,
  Bot,
  CheckCircle2,
  Circle,
  HelpCircle,
  Lock,
  ShieldAlert,
  Sparkles,
} from 'lucide-react';
import type { LevelProgressRecord } from '../../types/ipc';
import type { SocraticMessage } from '../../store/useKioskStore';

interface MissionPanelProps {
  level: LevelProgressRecord;
  code: string;
  messages: SocraticMessage[];
  isRequestingHint: boolean;
  onRequestHint: () => void;
  onJumpToLine?: (line: number) => void;
}

const NODE_LABELS: Record<string, string> = {
  FunctionDef: 'Definir função (def)',
  Assign: 'Atribuição de variável (=)',
  BinOp: 'Operação matemática (+, -, *, /)',
  Return: 'Retornar resultado (return)',
  If: 'Estrutura condicional (if/elif/else)',
  Compare: 'Comparação lógica (<, <=, >, >=, ==)',
  While: 'Laço de repetição while',
  For: 'Laço de repetição for',
  Break: 'Parada antecipada (break)',
};

function checkAstHintPresent(node: string, code: string): boolean {
  switch (node) {
    case 'FunctionDef':
      return /\bdef\s+[a-zA-Z_]\w*\s*\(/.test(code);
    case 'Assign':
      return /(^|[^=!<>])=([^=]|$)/.test(code);
    case 'BinOp':
      return /[+\-*/%]/.test(code.replace(/#.*/g, ''));
    case 'Return':
      return /\breturn\b/.test(code);
    case 'If':
      return /\bif\b/.test(code);
    case 'Compare':
      return /(==|!=|<=|>=|<|>)/.test(code.replace(/#.*/g, ''));
    case 'While':
      return /\bwhile\b/.test(code);
    case 'For':
      return /\bfor\b/.test(code);
    case 'Break':
      return /\bbreak\b/.test(code);
    default:
      return false;
  }
}

function renderFormattedMarkdown(md: string): React.ReactNode {
  const lines = md.split('\n');
  return lines.map((line, i) => {
    const trimmed = line.trim();
    if (trimmed.startsWith('### ')) {
      return (
        <h3 key={i} className="text-sm font-bold mt-1 mb-1.5" style={{ color: 'var(--accent-primary)' }}>
          {trimmed.slice(4)}
        </h3>
      );
    }
    if (trimmed.startsWith('- ')) {
      return (
        <li key={i} className="ml-4 list-disc text-xs leading-relaxed mb-1">
          {renderInlineCode(trimmed.slice(2))}
        </li>
      );
    }
    if (!trimmed) return <div key={i} className="h-1.5" />;
    return (
      <p key={i} className="text-xs leading-relaxed mb-1.5">
        {renderInlineCode(trimmed)}
      </p>
    );
  });
}

function renderInlineCode(text: string): React.ReactNode {
  const parts = text.split(/(`[^`]+`)/g);
  return parts.map((part, idx) => {
    if (part.startsWith('`') && part.endsWith('`')) {
      return (
        <code
          key={idx}
          className="font-mono-code px-1.5 py-0.5 rounded text-[11px]"
          style={{
            backgroundColor: 'var(--bg-surface-elevated)',
            color: 'var(--accent-primary)',
          }}
        >
          {part.slice(1, -1)}
        </code>
      );
    }
    return <React.Fragment key={idx}>{part}</React.Fragment>;
  });
}

export const MissionPanel: React.FC<MissionPanelProps> = ({
  level,
  code,
  messages,
  isRequestingHint,
  onRequestHint,
  onJumpToLine,
}) => {
  const isBoss = level.level_type === 'BOSS';

  return (
    <section
      className="h-full flex flex-col overflow-hidden"
      style={{ backgroundColor: 'var(--bg-surface-panel)' }}
      aria-label="Painel da Missão e Mentor Socrático"
    >
      {/* Level Header */}
      <div
        className="px-4 py-3 border-b flex items-start justify-between gap-2"
        style={{ borderColor: 'var(--border-subtle)' }}
      >
        <div>
          <div className="flex items-center gap-2">
            {isBoss && (
              <span
                className="inline-flex items-center gap-1 px-2 py-0.5 rounded text-[10px] font-bold uppercase tracking-wider"
                style={{
                  backgroundColor: 'var(--accent-gamification)',
                  color: '#FFFFFF',
                }}
              >
                <ShieldAlert className="w-3 h-3" />
                Boss Level
              </span>
            )}
            <span className="text-[11px] font-mono-code" style={{ color: 'var(--text-secondary)' }}>
              +{level.base_xp_reward} XP base
            </span>
          </div>
          <h2 className="text-sm font-bold mt-1 leading-snug">{level.title}</h2>
        </div>

        {level.status === 'COMPLETED_MASTERY' && (
          <span
            className="inline-flex items-center gap-1 px-2 py-1 rounded-md text-[11px] font-semibold shrink-0"
            style={{
              backgroundColor: 'rgba(16, 185, 129, 0.15)',
              color: 'var(--status-success)',
            }}
          >
            <CheckCircle2 className="w-3.5 h-3.5" />
            Maestria
          </span>
        )}
      </div>

      {/* Scrollable Body */}
      <div className="flex-1 overflow-y-auto p-4 space-y-4">
        {/* Problem Statement */}
        <div
          className="p-3.5 rounded-xl border"
          style={{
            backgroundColor: 'var(--bg-canvas)',
            borderColor: 'var(--border-subtle)',
          }}
        >
          {renderFormattedMarkdown(level.problem_statement_md)}
        </div>

        {/* Live AST Linter Pre-Flight Checklist (Nielsen Heuristic #3) */}
        <div
          className="p-3.5 rounded-xl border"
          style={{
            backgroundColor: 'var(--bg-surface-elevated)',
            borderColor: 'var(--border-subtle)',
          }}
        >
          <div className="flex items-center justify-between mb-2">
            <span className="text-xs font-bold tracking-wide uppercase" style={{ color: 'var(--text-secondary)' }}>
              Requisitos de Estrutura (AST)
            </span>
            <Sparkles className="w-3.5 h-3.5" style={{ color: 'var(--accent-primary)' }} />
          </div>
          <ul className="space-y-1.5">
            {level.must_contain_nodes.map((node) => {
              const present = checkAstHintPresent(node, code);
              return (
                <li key={node} className="flex items-center gap-2 text-xs">
                  {present ? (
                    <CheckCircle2 className="w-3.5 h-3.5 shrink-0" style={{ color: 'var(--status-success)' }} />
                  ) : (
                    <Circle className="w-3.5 h-3.5 shrink-0" style={{ color: 'var(--text-secondary)' }} />
                  )}
                  <span style={{ color: present ? 'var(--text-primary)' : 'var(--text-secondary)' }}>
                    {NODE_LABELS[node] ?? `Usar estrutura ${node}`}
                  </span>
                </li>
              );
            })}
          </ul>
          {level.forbidden_nodes.length > 0 && (
            <div className="mt-2.5 pt-2 border-t text-[11px] flex items-center gap-1.5" style={{ borderColor: 'var(--border-subtle)', color: 'var(--text-secondary)' }}>
              <AlertTriangle className="w-3 h-3 shrink-0" style={{ color: 'var(--status-warning)' }} />
              <span>Proibido: {level.forbidden_nodes.join(', ')}</span>
            </div>
          )}
        </div>

        {/* Socratic Mentor Chat */}
        <div className="space-y-2.5">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-1.5 text-xs font-bold uppercase tracking-wide" style={{ color: 'var(--text-secondary)' }}>
              <Bot className="w-3.5 h-3.5" style={{ color: 'var(--accent-primary)' }} />
              Mentor Socrático
            </div>
            <button
              type="button"
              disabled={isBoss || isRequestingHint}
              onClick={onRequestHint}
              title={
                isBoss
                  ? 'Pistas socráticas bloqueadas no Desafio Chefão'
                  : 'Pedir Pista Socrática (Alt+H)'
              }
              className="inline-flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg text-xs font-semibold transition-opacity cursor-pointer disabled:opacity-45 disabled:cursor-not-allowed"
              style={{
                backgroundColor: isBoss ? 'var(--bg-surface-elevated)' : 'var(--accent-gamification)',
                color: isBoss ? 'var(--text-secondary)' : '#FFFFFF',
              }}
            >
              {isBoss ? <Lock className="w-3.5 h-3.5" /> : <HelpCircle className="w-3.5 h-3.5" />}
              {isBoss ? 'Pistas Bloqueadas (Boss)' : 'Pedir Pista (Alt+H)'}
            </button>
          </div>

          {isRequestingHint && (
            <div
              className="p-3 rounded-xl border text-xs flex items-center gap-2 animate-pulse"
              style={{
                backgroundColor: 'var(--bg-surface-elevated)',
                borderColor: 'var(--accent-gamification)',
                color: 'var(--text-primary)',
              }}
            >
              <Bot className="w-4 h-4 shrink-0" style={{ color: 'var(--accent-gamification)' }} />
              <span>Mentor analisando sua lógica...</span>
            </div>
          )}

          {messages.length === 0 && !isRequestingHint && (
            <div
              className="p-3 rounded-xl border text-xs leading-relaxed"
              style={{
                backgroundColor: 'var(--bg-canvas)',
                borderColor: 'var(--border-subtle)',
                color: 'var(--text-secondary)',
              }}
            >
              {isBoss
                ? 'Modo Desafio Chefão ativo: resolva com autonomia total. Pistas e colagem externa estão desativadas.'
                : 'Se tiver dúvidas ou alguma falha nos testes, o Mentor Socrático guiará seu raciocínio sem entregar o código pronto.'}
            </div>
          )}

          {messages.map((msg) => (
            <div
              key={msg.id}
              className="p-3.5 rounded-xl border space-y-2"
              style={{
                backgroundColor: 'var(--bg-surface-elevated)',
                borderColor: 'var(--status-warning)',
              }}
            >
              <div className="flex items-center justify-between text-[10px]">
                <span
                  className="px-1.5 py-0.5 rounded font-mono-code font-bold"
                  style={{
                    backgroundColor: 'rgba(245, 158, 11, 0.18)',
                    color: 'var(--status-warning)',
                  }}
                >
                  {msg.hint.diagnostic_category}
                </span>
                <span style={{ color: 'var(--text-secondary)' }}>
                  {msg.hint.provider_used} · {msg.timestamp}
                </span>
              </div>

              <p className="text-xs leading-relaxed">{msg.hint.friendly_explanation}</p>

              <div
                className="p-2.5 rounded-lg border text-xs font-medium leading-relaxed"
                style={{
                  backgroundColor: 'var(--bg-canvas)',
                  borderColor: 'var(--border-subtle)',
                  color: 'var(--accent-primary)',
                }}
              >
                {msg.hint.socratic_question}
              </div>

              <div className="flex items-center justify-between text-[11px]" style={{ color: 'var(--text-secondary)' }}>
                <span className="italic">{msg.hint.conceptual_reminder}</span>
                {onJumpToLine && (
                  <button
                    type="button"
                    onClick={() => onJumpToLine(msg.hint.focus_line)}
                    className="ml-2 shrink-0 underline font-mono-code cursor-pointer"
                    style={{ color: 'var(--status-warning)' }}
                  >
                    Linha {msg.hint.focus_line}
                  </button>
                )}
              </div>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
};
