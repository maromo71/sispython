import React, { useState } from 'react';
import { CheckCircle2, Cpu, Play, X } from 'lucide-react';
import { useKioskStore } from '../../store/useKioskStore';

export const CalibrationOnboarding: React.FC = () => {
  const { calibrationDone, completeCalibration } = useKioskStore();
  const [val, setVal] = useState('energia = 10 + 5');
  const [feedback, setFeedback] = useState<string | null>(null);

  if (calibrationDone) return null;

  const handleTest = () => {
    if (val.includes('=') && /\d+/.test(val)) {
      setFeedback('Circuito calibrado! Você já domina a execução no Quiosque.');
      setTimeout(() => completeCalibration(), 900);
    } else {
      setFeedback('Dica: mantenha uma atribuição com = e números, como energia = 10 + 5.');
    }
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center p-4"
      style={{ backgroundColor: 'rgba(11, 15, 25, 0.82)' }}
      role="dialog"
      aria-modal="true"
      aria-label="Calibrando seu Terminal"
    >
      <div
        className="w-full max-w-md rounded-2xl border p-6 space-y-4 shadow-2xl"
        style={{
          backgroundColor: 'var(--bg-surface-panel)',
          borderColor: 'var(--accent-primary)',
        }}
      >
        <div className="flex items-start justify-between">
          <div className="flex items-center gap-2.5">
            <Cpu className="w-5 h-5" style={{ color: 'var(--accent-primary)' }} />
            <div>
              <h2 className="text-sm font-bold">Calibrando seu Terminal (30s)</h2>
              <p className="text-xs" style={{ color: 'var(--text-secondary)' }}>
                Bem-vindo ao LogicKiosk! Teste seu primeiro comando antes da Missão 1.1.
              </p>
            </div>
          </div>
          <button
            type="button"
            onClick={completeCalibration}
            aria-label="Pular calibração"
            className="p-1 rounded cursor-pointer"
            style={{ color: 'var(--text-secondary)' }}
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        <div className="space-y-2">
          <label className="text-xs font-semibold block">
            Edite a expressão abaixo e clique em Testar Calibração:
          </label>
          <input
            type="text"
            value={val}
            onChange={(e) => setVal(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) {
                handleTest();
              }
            }}
            className="w-full px-3 py-2 rounded-lg border font-mono-code text-xs outline-none"
            style={{
              backgroundColor: 'var(--bg-canvas)',
              borderColor: 'var(--border-subtle)',
              color: 'var(--text-primary)',
            }}
          />
        </div>

        {feedback && (
          <div
            className="p-2.5 rounded-lg text-xs flex items-center gap-2"
            style={{
              backgroundColor: 'rgba(16, 185, 129, 0.14)',
              color: 'var(--status-success)',
            }}
          >
            <CheckCircle2 className="w-4 h-4 shrink-0" />
            <span>{feedback}</span>
          </div>
        )}

        <div className="flex items-center justify-between pt-2">
          <button
            type="button"
            onClick={completeCalibration}
            className="text-xs underline cursor-pointer"
            style={{ color: 'var(--text-secondary)' }}
          >
            Pular para o Quiosque
          </button>
          <button
            type="button"
            onClick={handleTest}
            className="inline-flex items-center gap-1.5 px-4 py-2 rounded-lg text-xs font-bold cursor-pointer"
            style={{
              backgroundColor: 'var(--accent-primary)',
              color: '#0B0F19',
            }}
          >
            <Play className="w-3.5 h-3.5 fill-current" />
            Testar Calibração
          </button>
        </div>
      </div>
    </div>
  );
};
