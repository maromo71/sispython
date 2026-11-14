import React, { useEffect, useRef } from 'react';
import {
  AlertTriangle,
  ArrowRight,
  CheckCircle2,
  Clock,
  Terminal,
  XCircle,
  Zap,
} from 'lucide-react';
import { motion } from 'framer-motion';
import type { LevelProgressRecord, PublicTestCaseView, SubmitCodeResponse } from '../../types/ipc';
import { VariableWatch } from './VariableWatch';
import { useThemeStore } from '../../store/useThemeStore';

interface ExecutionConsoleProps {
  level: LevelProgressRecord;
  publicTestCases: PublicTestCaseView[];
  isRunning: boolean;
  result: SubmitCodeResponse | null;
  hasNextLevel: boolean;
  onAdvanceNextLevel: () => void;
}

function playSuccessChime() {
  try {
    const AudioCtx = window.AudioContext || (window as unknown as { webkitAudioContext: typeof AudioContext }).webkitAudioContext;
    if (!AudioCtx) return;
    const ctx = new AudioCtx();
    const now = ctx.currentTime;
    [523.25, 659.25, 783.99].forEach((freq, idx) => {
      const osc = ctx.createOscillator();
      const gain = ctx.createGain();
      osc.type = 'sine';
      osc.frequency.setValueAtTime(freq, now + idx * 0.08);
      gain.gain.setValueAtTime(0.06, now + idx * 0.08);
      gain.gain.exponentialRampToValueAtTime(0.001, now + idx * 0.08 + 0.25);
      osc.connect(gain);
      gain.connect(ctx.destination);
      osc.start(now + idx * 0.08);
      osc.stop(now + idx * 0.08 + 0.26);
    });
  } catch {
    // Ignore audio context restriction
  }
}

export const ExecutionConsole: React.FC<ExecutionConsoleProps> = ({
  level,
  publicTestCases,
  isRunning,
  result,
  hasNextLevel,
  onAdvanceNextLevel,
}) => {
  const { soundEnabled } = useThemeStore();
  const lastPlayedRef = useRef<string | null>(null);

  useEffect(() => {
    if (result?.verdict === 'PASSED' && soundEnabled) {
      const key = `${level.id}-${result.updated_profile.total_xp}-${result.execution_summary?.execution_time_ms}`;
      if (lastPlayedRef.current !== key) {
        lastPlayedRef.current = key;
        playSuccessChime();
      }
    }
  }, [result, level.id, soundEnabled]);

  const isTimeout = result?.verdict === 'TIMEOUT_LOOP';

  return (
    <section
      className={`h-full flex flex-col overflow-hidden border-l ${
        isTimeout ? 'animate-timeout-pulse border-2' : ''
      }`}
      style={{
        backgroundColor: 'var(--bg-surface-panel)',
        borderColor: isTimeout ? 'var(--status-warning)' : 'var(--border-subtle)',
      }}
      aria-label="Console de Execução e Inspetor de Variáveis"
    >
      {/* Header */}
      <div
        className="px-4 py-2.5 border-b flex items-center justify-between"
        style={{ borderColor: 'var(--border-subtle)' }}
      >
        <div className="flex items-center gap-2 text-xs font-bold uppercase tracking-wide">
          <Terminal className="w-3.5 h-3.5" style={{ color: 'var(--accent-primary)' }} />
          <span>Console de Execução & Testes</span>
        </div>
        {result?.execution_summary && (
          <span
            className="text-[11px] font-mono-code"
            style={{ color: 'var(--text-secondary)' }}
          >
            {result.execution_summary.execution_time_ms} ms
          </span>
        )}
      </div>

      {/* Body */}
      <div className="flex-1 overflow-y-auto p-4 space-y-4">
        {/* Loading State */}
        {isRunning && (
          <div className="space-y-3" data-testid="console-loading-state">
            <div className="text-xs font-medium" style={{ color: 'var(--accent-primary)' }}>
              Executando em Sandbox isolado (timeout 2000ms)...
            </div>
            <div
              className="h-12 rounded-xl animate-pulse"
              style={{ backgroundColor: 'var(--bg-surface-elevated)' }}
            />
            <div
              className="h-20 rounded-xl animate-pulse"
              style={{ backgroundColor: 'var(--bg-surface-elevated)' }}
            />
            <div
              className="h-16 rounded-xl animate-pulse"
              style={{ backgroundColor: 'var(--bg-surface-elevated)' }}
            />
          </div>
        )}

        {/* Empty State */}
        {!isRunning && !result && (
          <div className="space-y-4" data-testid="console-empty-state">
            <div
              className="p-5 rounded-xl border text-center space-y-3"
              style={{
                backgroundColor: 'var(--bg-canvas)',
                borderColor: 'var(--border-subtle)',
              }}
            >
              {/* Minimalist Open-Circuit SVG Illustration */}
              <svg
                viewBox="0 0 180 64"
                className="w-36 h-14 mx-auto"
                fill="none"
                aria-hidden="true"
              >
                <line x1="12" y1="36" x2="64" y2="36" stroke="var(--accent-primary)" strokeWidth="2.5" strokeLinecap="round" />
                <line x1="116" y1="36" x2="168" y2="36" stroke="var(--accent-primary)" strokeWidth="2.5" strokeLinecap="round" />
                <circle cx="66" cy="36" r="5" stroke="var(--status-warning)" strokeWidth="2.5" />
                <circle cx="114" cy="36" r="5" stroke="var(--status-warning)" strokeWidth="2.5" />
                <line x1="70" y1="33" x2="106" y2="14" stroke="var(--status-warning)" strokeWidth="2.5" strokeLinecap="round" />
                <circle cx="16" cy="36" r="3" fill="var(--accent-primary)" />
                <circle cx="164" cy="36" r="3" fill="var(--accent-primary)" />
              </svg>
              <p className="text-xs leading-relaxed" style={{ color: 'var(--text-secondary)' }}>
                Pressione <kbd className="font-mono-code px-1.5 py-0.5 rounded border" style={{ borderColor: 'var(--border-subtle)' }}>Ctrl+Enter</kbd> ou clique em <strong>Executar</strong> para alimentar o circuito lógico.
              </p>
            </div>

            {/* Public Test Cases Preview */}
            {publicTestCases.length > 0 && (
              <div className="space-y-2">
                <div className="text-[11px] font-bold uppercase tracking-wide" style={{ color: 'var(--text-secondary)' }}>
                  Casos de Teste Públicos
                </div>
                {publicTestCases.map((tc, idx) => (
                  <div
                    key={tc.id}
                    className="p-2.5 rounded-lg border font-mono-code text-[11px] space-y-1"
                    style={{
                      backgroundColor: 'var(--bg-surface-elevated)',
                      borderColor: 'var(--border-subtle)',
                    }}
                  >
                    <div style={{ color: 'var(--text-secondary)' }}>Caso #{idx + 1}</div>
                    <div>
                      Entrada: <span style={{ color: 'var(--accent-primary)' }}>{tc.input_data_json}</span>
                    </div>
                    <div>
                      Esperado: <span style={{ color: 'var(--status-success)' }}>{tc.expected_output_json}</span>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        )}

        {/* Result State */}
        {!isRunning && result && (
          <div className="space-y-4">
            {/* Verdict Banner */}
            {result.verdict === 'PASSED' && (
              <motion.div
                initial={{ opacity: 0, scale: 0.96 }}
                animate={{ opacity: 1, scale: 1 }}
                className="p-4 rounded-xl border space-y-3"
                style={{
                  backgroundColor: 'rgba(16, 185, 129, 0.12)',
                  borderColor: 'var(--status-success)',
                }}
                data-testid="console-success-state"
              >
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2 font-bold text-sm" style={{ color: 'var(--status-success)' }}>
                    <CheckCircle2 className="w-5 h-5" />
                    <span>Maestria Comprovada!</span>
                  </div>
                  {result.reward && result.reward.earned_xp > 0 && (
                    <span
                      className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-bold"
                      style={{
                        backgroundColor: 'var(--accent-gamification)',
                        color: '#FFFFFF',
                      }}
                    >
                      <Zap className="w-3 h-3" />+{result.reward.earned_xp} XP
                    </span>
                  )}
                </div>

                <div className="grid grid-cols-2 gap-2 text-[11px] font-mono-code">
                  <div className="p-2 rounded-lg" style={{ backgroundColor: 'var(--bg-canvas)' }}>
                    Tempo: <strong>{result.execution_summary?.execution_time_ms ?? 0} ms</strong>
                  </div>
                  <div className="p-2 rounded-lg" style={{ backgroundColor: 'var(--bg-canvas)' }}>
                    Testes: <strong>100% Aprovados</strong>
                  </div>
                </div>

                {result.reward?.scheduled_fixation_review && (
                  <div
                    className="p-2.5 rounded-lg border text-xs"
                    style={{
                      backgroundColor: 'rgba(245, 158, 11, 0.12)',
                      borderColor: 'var(--status-warning)',
                      color: 'var(--status-warning)',
                    }}
                  >
                    <strong>Revisão Adaptativa Agendada:</strong> Como esta fase exigiu esforço extra, recomendamos revisar os conceitos antes do Desafio Chefão.
                  </div>
                )}

                {hasNextLevel && (
                  <button
                    type="button"
                    onClick={onAdvanceNextLevel}
                    className="w-full min-h-[48px] min-w-[48px] flex items-center justify-center gap-2 px-4 py-3 rounded-xl text-xs font-bold cursor-pointer transition-transform active:scale-99"
                    style={{
                      backgroundColor: 'var(--status-success)',
                      color: '#0B0F19',
                    }}
                  >
                    <span>Avançar para Fase Seguinte (Ctrl+Shift+Enter)</span>
                    <ArrowRight className="w-4 h-4" />
                  </button>
                )}
              </motion.div>
            )}

            {/* Timeout Loop Banner */}
            {result.verdict === 'TIMEOUT_LOOP' && (
              <div
                className="p-3.5 rounded-xl border space-y-2"
                style={{
                  backgroundColor: 'rgba(245, 158, 11, 0.14)',
                  borderColor: 'var(--status-warning)',
                }}
                data-testid="console-timeout-state"
              >
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2 font-bold text-xs" style={{ color: 'var(--status-warning)' }}>
                    <Clock className="w-4 h-4" />
                    <span>Loop Infinito Interrompido (&gt; 2000ms)</span>
                  </div>
                  <span className="font-mono-code text-[11px]" style={{ color: 'var(--status-warning)' }}>
                    {result.execution_summary?.variable_trace.length ?? 0} iterações capturadas
                  </span>
                </div>
                <p className="text-xs leading-relaxed">
                  A thread isolada encerrou a execução ao atingir o limite de segurança. Confira a pergunta socrática ao lado e os últimos valores no Step Watch abaixo.
                </p>
              </div>
            )}

            {/* AST Rejection / Syntax Error */}
            {(result.verdict === 'AST_REJECTED' || result.verdict === 'SYNTAX_ERROR') && (
              <div
                className="p-3.5 rounded-xl border space-y-2"
                style={{
                  backgroundColor: 'rgba(244, 63, 94, 0.12)',
                  borderColor: 'var(--status-danger)',
                }}
                data-testid="console-error-state"
              >
                <div className="flex items-center gap-2 font-bold text-xs" style={{ color: 'var(--status-danger)' }}>
                  <XCircle className="w-4 h-4" />
                  <span>
                    {result.verdict === 'SYNTAX_ERROR'
                      ? 'Erro de Sintaxe Detectado'
                      : 'Regra Estrutural (AST) Não Atendida'}
                  </span>
                </div>
                {result.ast_report.missing_required_nodes.length > 0 && (
                  <div className="text-xs">
                    Estruturas faltantes:{' '}
                    <span className="font-mono-code font-bold">
                      {result.ast_report.missing_required_nodes.join(', ')}
                    </span>
                  </div>
                )}
                {result.ast_report.triggered_forbidden_nodes.length > 0 && (
                  <div className="text-xs">
                    Estruturas proibidas detectadas:{' '}
                    <span className="font-mono-code font-bold">
                      {result.ast_report.triggered_forbidden_nodes.join(', ')}
                    </span>
                  </div>
                )}
                {result.ast_report.suspected_hardcode && (
                  <div className="text-xs">
                    Retorno constante detectado sem lógica computacional (anti-hardcode).
                  </div>
                )}
                {result.ast_report.syntax_error_detail && (
                  <details className="text-[11px] pt-1">
                    <summary className="cursor-pointer" style={{ color: 'var(--text-secondary)' }}>
                      Ver erro técnico original (Linha {result.ast_report.syntax_error_detail[1]})
                    </summary>
                    <pre className="mt-1 p-2 rounded font-mono-code overflow-x-auto" style={{ backgroundColor: 'var(--bg-canvas)' }}>
                      {result.ast_report.syntax_error_detail[0]}
                    </pre>
                  </details>
                )}
              </div>
            )}

            {/* Test Cases Results */}
            {result.execution_summary && result.execution_summary.test_results.length > 0 && (
              <div className="space-y-2">
                <div className="text-[11px] font-bold uppercase tracking-wide" style={{ color: 'var(--text-secondary)' }}>
                  Resultados dos Casos de Teste
                </div>
                {result.execution_summary.test_results.map((tc, idx) => (
                  <div
                    key={tc.test_case_id}
                    className="p-3 rounded-xl border text-xs space-y-1.5"
                    style={{
                      backgroundColor: 'var(--bg-surface-elevated)',
                      borderColor: tc.passed ? 'var(--status-success)' : 'var(--status-danger)',
                    }}
                  >
                    <div className="flex items-center justify-between font-semibold">
                      <span className="flex items-center gap-1.5">
                        {tc.passed ? (
                          <CheckCircle2 className="w-3.5 h-3.5" style={{ color: 'var(--status-success)' }} />
                        ) : (
                          <XCircle className="w-3.5 h-3.5" style={{ color: 'var(--status-danger)' }} />
                        )}
                        {tc.is_hidden_edge_case
                          ? `Caso Oculto de Borda #${idx + 1}`
                          : `Caso Público #${idx + 1}`}
                      </span>
                      <span
                        className="text-[10px] font-mono-code px-1.5 py-0.5 rounded"
                        style={{
                          backgroundColor: tc.passed
                            ? 'rgba(16, 185, 129, 0.15)'
                            : 'rgba(244, 63, 94, 0.15)',
                          color: tc.passed ? 'var(--status-success)' : 'var(--status-danger)',
                        }}
                      >
                        {tc.passed ? 'APROVADO' : 'FALHOU'}
                      </span>
                    </div>

                    <div className="font-mono-code text-[11px]" style={{ color: 'var(--text-secondary)' }}>
                      Entrada: <span style={{ color: 'var(--text-primary)' }}>{tc.input_preview}</span>
                    </div>

                    {!tc.is_hidden_edge_case && tc.expected_preview !== null && (
                      <div className="grid grid-cols-2 gap-2 font-mono-code text-[11px]">
                        <div>
                          Esperado: <span style={{ color: 'var(--status-success)' }}>{tc.expected_preview}</span>
                        </div>
                        <div>
                          Obtido: <span style={{ color: tc.passed ? 'var(--status-success)' : 'var(--status-danger)' }}>{tc.actual_preview ?? 'Erro'}</span>
                        </div>
                      </div>
                    )}

                    {tc.error_message && (
                      <details className="text-[11px]">
                        <summary className="cursor-pointer flex items-center gap-1" style={{ color: 'var(--status-warning)' }}>
                          <AlertTriangle className="w-3 h-3" />
                          Ver detalhe técnico
                        </summary>
                        <div className="mt-1 p-2 rounded font-mono-code" style={{ backgroundColor: 'var(--bg-canvas)' }}>
                          {tc.error_message}
                        </div>
                      </details>
                    )}
                  </div>
                ))}
              </div>
            )}

            {/* Stdout Output */}
            {result.execution_summary && result.execution_summary.stdout && (
              <div
                className="p-3 rounded-xl border"
                style={{
                  backgroundColor: 'var(--bg-canvas)',
                  borderColor: 'var(--border-subtle)',
                }}
              >
                <div className="flex items-center justify-between text-[11px] font-bold mb-1" style={{ color: 'var(--text-secondary)' }}>
                  <span>Saída (stdout)</span>
                  {result.execution_summary.stdout_truncated && (
                    <span style={{ color: 'var(--status-warning)' }}>
                      Truncado em 10.000 caracteres
                    </span>
                  )}
                </div>
                <pre className="font-mono-code text-xs whitespace-pre-wrap break-all max-h-32 overflow-y-auto">
                  {result.execution_summary.stdout}
                </pre>
              </div>
            )}

            {/* Step Variable Watch */}
            {result.execution_summary && (
              <VariableWatch trace={result.execution_summary.variable_trace} />
            )}
          </div>
        )}
      </div>
    </section>
  );
};
