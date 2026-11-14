import React, { useState } from 'react';
import { KeyRound, ShieldCheck, Trash2, X } from 'lucide-react';
import { useProgressionStore } from '../../store/useProgressionStore';

export const ApiKeyModal: React.FC = () => {
  const { apiKeyModalOpen, setApiKeyModalOpen, saveApiKey, curriculum } =
    useProgressionStore();
  const [keyInput, setKeyInput] = useState('');
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  if (!apiKeyModalOpen) return null;

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setSaving(true);
    setError(null);
    try {
      await saveApiKey(keyInput);
      setKeyInput('');
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setSaving(false);
    }
  };

  const handleRemove = async () => {
    setSaving(true);
    setError(null);
    try {
      await saveApiKey('');
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setSaving(false);
    }
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center p-4"
      style={{ backgroundColor: 'rgba(11, 15, 25, 0.8)' }}
      role="dialog"
      aria-modal="true"
      aria-label="Configurar Chave Gratuita Gemini AI Studio"
    >
      <div
        className="w-full max-w-md rounded-2xl border p-6 space-y-4 shadow-2xl"
        style={{
          backgroundColor: 'var(--bg-surface-panel)',
          borderColor: 'var(--border-subtle)',
        }}
      >
        <div className="flex items-start justify-between">
          <div className="flex items-center gap-2.5">
            <KeyRound className="w-5 h-5" style={{ color: 'var(--accent-primary)' }} />
            <div>
              <h2 className="text-sm font-bold">Gemini Free Tier (BYOK)</h2>
              <p className="text-xs" style={{ color: 'var(--text-secondary)' }}>
                Opcional · Cascata gratuita: gemini-3.8-flash → gemini-3.5-flash-lite → Rust Offline
              </p>
            </div>
          </div>
          <button
            type="button"
            onClick={() => setApiKeyModalOpen(false)}
            aria-label="Fechar modal"
            className="p-1 rounded cursor-pointer"
            style={{ color: 'var(--text-secondary)' }}
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        <div
          className="p-3 rounded-xl border text-xs space-y-1"
          style={{
            backgroundColor: 'var(--bg-canvas)',
            borderColor: 'var(--border-subtle)',
            color: 'var(--text-secondary)',
          }}
        >
          <div className="flex items-center gap-1.5 font-semibold" style={{ color: 'var(--status-success)' }}>
            <ShieldCheck className="w-4 h-4" />
            Armazenamento Seguro no Cofre do Sistema (Keyring)
          </div>
          <p>
            Sua chave do Google AI Studio nunca é salva no SQLite ou localStorage. Sem chave, o LogicKiosk opera 100% offline com o Motor Nativo Rust.
          </p>
        </div>

        <form onSubmit={handleSubmit} className="space-y-3">
          <div>
            <label className="block text-xs font-semibold mb-1">
              Chave API do Google AI Studio
            </label>
            <input
              type="password"
              value={keyInput}
              onChange={(e) => setKeyInput(e.target.value)}
              placeholder={
                curriculum?.ai_key_configured
                  ? '•••••••••••••••••••• (chave configurada no Keyring)'
                  : 'AIza...'
              }
              className="w-full px-3 py-2 rounded-lg border font-mono-code text-xs outline-none"
              style={{
                backgroundColor: 'var(--bg-canvas)',
                borderColor: 'var(--border-subtle)',
                color: 'var(--text-primary)',
              }}
            />
          </div>

          {error && (
            <div className="text-xs" style={{ color: 'var(--status-danger)' }}>
              {error}
            </div>
          )}

          <div className="flex items-center justify-between pt-1">
            {curriculum?.ai_key_configured ? (
              <button
                type="button"
                disabled={saving}
                onClick={handleRemove}
                className="inline-flex items-center gap-1 text-xs font-semibold cursor-pointer"
                style={{ color: 'var(--status-danger)' }}
              >
                <Trash2 className="w-3.5 h-3.5" />
                Remover Chave
              </button>
            ) : (
              <span />
            )}

            <button
              type="submit"
              disabled={saving || !keyInput.trim()}
              className="px-4 py-2 rounded-lg text-xs font-bold cursor-pointer disabled:opacity-50"
              style={{
                backgroundColor: 'var(--accent-primary)',
                color: '#0B0F19',
              }}
            >
              {saving ? 'Salvando...' : 'Salvar no Keyring'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
};
