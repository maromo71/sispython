import React, { useEffect, useState } from 'react';
import { AlertCircle, KeyRound, Loader2, Sparkles, X } from 'lucide-react';
import { useProgressionStore } from '../../store/useProgressionStore';
import type { ModuleDifficulty, ModuleRecord } from '../../types/ipc';

export const MIN_EXERCISES = 5;
export const MAX_EXERCISES = 20;

export function clampExerciseCount(value: number, minBound = MIN_EXERCISES): number {
  const safeMin = Math.max(MIN_EXERCISES, Math.min(MAX_EXERCISES, minBound));
  if (!Number.isFinite(value)) return safeMin;
  return Math.max(safeMin, Math.min(MAX_EXERCISES, Math.round(value)));
}

interface AiModuleGeneratorModalProps {
  open: boolean;
  mode: 'create' | 'expand';
  targetModule: ModuleRecord | null;
  currentLevelCount: number;
  onClose: () => void;
  onSuccess: (message: string) => void;
}

export const AiModuleGeneratorModal: React.FC<AiModuleGeneratorModalProps> = ({
  open,
  mode,
  targetModule,
  currentLevelCount,
  onClose,
  onSuccess,
}) => {
  const {
    curriculum,
    generateAiModule,
    expandModuleExercises,
    setApiKeyModalOpen,
  } = useProgressionStore();

  const expandMin = Math.min(MAX_EXERCISES, Math.max(MIN_EXERCISES, currentLevelCount + 1));
  const [title, setTitle] = useState('');
  const [topic, setTopic] = useState('');
  const [difficulty, setDifficulty] = useState<ModuleDifficulty>('INTERMEDIARIO');
  const [exerciseCount, setExerciseCount] = useState<number>(MIN_EXERCISES);
  const [customInstructions, setCustomInstructions] = useState('');
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!open) return;
    setError(null);
    if (mode === 'expand') {
      setExerciseCount(clampExerciseCount(Math.min(MAX_EXERCISES, Math.max(MIN_EXERCISES, currentLevelCount + 3)), expandMin));
      setTopic('');
    } else {
      const nextNum = (curriculum?.modules.length ?? 3) + 1;
      setTitle(`Módulo ${nextNum}: Trilha Prática com IA`);
      setTopic('');
      setCustomInstructions('');
      setExerciseCount(MIN_EXERCISES);
    }
  }, [open, mode, currentLevelCount, expandMin, curriculum?.modules.length]);

  if (!open) return null;

  const isExpand = mode === 'expand' && targetModule !== null;
  const minSlider = isExpand ? expandMin : MIN_EXERCISES;
  const newExercisesDelta = isExpand ? Math.max(0, exerciseCount - currentLevelCount) : exerciseCount;

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);

    if (exerciseCount < MIN_EXERCISES || exerciseCount > MAX_EXERCISES) {
      setError(`A quantidade de exercícios deve estar entre ${MIN_EXERCISES} e ${MAX_EXERCISES}.`);
      return;
    }

    setSubmitting(true);
    try {
      if (isExpand && targetModule) {
        const res = await expandModuleExercises({
          module_id: targetModule.id,
          target_exercise_count: exerciseCount,
          focus_topic: topic.trim() ? topic.trim() : null,
        });
        onSuccess(
          `+${res.generated_levels_count} novos exercícios validados no Sandbox e adicionados ao "${targetModule.title}" via ${res.provider_used}.`
        );
      } else {
        if (!title.trim() || !topic.trim()) {
          setError('Preencha o título e o tema Python do novo módulo.');
          setSubmitting(false);
          return;
        }
        const res = await generateAiModule({
          title: title.trim(),
          topic: topic.trim(),
          difficulty,
          exercise_count: exerciseCount,
          custom_instructions: customInstructions.trim() ? customInstructions.trim() : null,
        });
        onSuccess(
          `Módulo "${title.trim()}" criado com ${res.generated_levels_count} fases validadas no Sandbox via ${res.provider_used}.`
        );
      }
      onClose();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center p-4"
      style={{ backgroundColor: 'rgba(11, 15, 25, 0.82)' }}
      role="dialog"
      aria-modal="true"
      aria-label={isExpand ? 'Customizar Quantidade de Exercícios com IA' : 'Gerar Novo Módulo com IA'}
    >
      <div
        className="w-full max-w-lg rounded-2xl border p-6 space-y-4 shadow-2xl max-h-[90vh] overflow-y-auto"
        style={{
          backgroundColor: 'var(--bg-surface-panel)',
          borderColor: 'var(--border-subtle)',
        }}
      >
        <div className="flex items-start justify-between gap-3">
          <div className="flex items-center gap-2.5">
            <Sparkles className="w-5 h-5 shrink-0" style={{ color: 'var(--accent-gamification)' }} />
            <div>
              <h2 className="text-sm font-bold">
                {isExpand
                  ? `Expandir "${targetModule?.title}"`
                  : 'Gerar Novo Módulo com IA (5 a 20 Fases)'}
              </h2>
              <p className="text-xs" style={{ color: 'var(--text-secondary)' }}>
                Cada exercício gerado é validado no Inspetor AST e executado contra os testes no Sandbox antes de entrar na trilha.
              </p>
            </div>
          </div>
          <button
            type="button"
            disabled={submitting}
            onClick={onClose}
            aria-label="Fechar gerador de IA"
            className="p-1 rounded cursor-pointer disabled:opacity-40"
            style={{ color: 'var(--text-secondary)' }}
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {!curriculum?.ai_key_configured && (
          <div
            className="p-3 rounded-xl border text-xs flex flex-wrap items-center justify-between gap-2"
            style={{
              backgroundColor: 'rgba(245, 158, 11, 0.12)',
              borderColor: 'var(--status-warning)',
              color: 'var(--status-warning)',
            }}
          >
            <span>
              Chave Gemini não configurada no Keyring (ou use Ollama local com LOCAL_LLM_ENABLED).
            </span>
            <button
              type="button"
              onClick={() => setApiKeyModalOpen(true)}
              className="inline-flex items-center gap-1 px-2.5 py-1 rounded-lg font-bold cursor-pointer"
              style={{
                backgroundColor: 'var(--bg-canvas)',
                color: 'var(--text-primary)',
              }}
            >
              <KeyRound className="w-3.5 h-3.5" />
              Configurar Chave
            </button>
          </div>
        )}

        <form onSubmit={handleSubmit} className="space-y-4">
          {!isExpand ? (
            <>
              <div>
                <label htmlFor="ai-mod-title" className="block text-xs font-semibold mb-1">
                  Título do Módulo
                </label>
                <input
                  id="ai-mod-title"
                  type="text"
                  value={title}
                  onChange={(e) => setTitle(e.target.value)}
                  placeholder="Ex.: Módulo 4: Strings, Tuplas e Dicionários"
                  className="w-full px-3 py-2 rounded-lg border text-xs outline-none"
                  style={{
                    backgroundColor: 'var(--bg-canvas)',
                    borderColor: 'var(--border-subtle)',
                    color: 'var(--text-primary)',
                  }}
                />
              </div>

              <div>
                <label htmlFor="ai-mod-topic" className="block text-xs font-semibold mb-1">
                  Tema / Conteúdo Python
                </label>
                <input
                  id="ai-mod-topic"
                  type="text"
                  value={topic}
                  onChange={(e) => setTopic(e.target.value)}
                  placeholder="Ex.: Análise de logs com split/strip, dicionários e compreensão de listas"
                  className="w-full px-3 py-2 rounded-lg border text-xs outline-none"
                  style={{
                    backgroundColor: 'var(--bg-canvas)',
                    borderColor: 'var(--border-subtle)',
                    color: 'var(--text-primary)',
                  }}
                />
              </div>

              <div>
                <label htmlFor="ai-mod-diff" className="block text-xs font-semibold mb-1">
                  Dificuldade Progressiva
                </label>
                <select
                  id="ai-mod-diff"
                  value={difficulty}
                  onChange={(e) => setDifficulty(e.target.value as ModuleDifficulty)}
                  className="w-full px-3 py-2 rounded-lg border text-xs outline-none"
                  style={{
                    backgroundColor: 'var(--bg-canvas)',
                    borderColor: 'var(--border-subtle)',
                    color: 'var(--text-primary)',
                  }}
                >
                  <option value="INICIANTE">Iniciante (variáveis, condicionais e laços)</option>
                  <option value="INTERMEDIARIO">Intermediário (listas, dicionários e strings)</option>
                  <option value="AVANCADO">Avançado (algoritmos, matrizes e casos de borda)</option>
                </select>
              </div>
            </>
          ) : (
            <div>
              <label htmlFor="ai-expand-topic" className="block text-xs font-semibold mb-1">
                Foco dos Novos Exercícios (opcional)
              </label>
              <input
                id="ai-expand-topic"
                type="text"
                value={topic}
                onChange={(e) => setTopic(e.target.value)}
                placeholder={`Manter tema atual: ${targetModule?.description}`}
                className="w-full px-3 py-2 rounded-lg border text-xs outline-none"
                style={{
                  backgroundColor: 'var(--bg-canvas)',
                  borderColor: 'var(--border-subtle)',
                  color: 'var(--text-primary)',
                }}
              />
            </div>
          )}

          {/* Custom Exercise Count Control (Strict 5..=20) */}
          <div
            className="p-3.5 rounded-xl border space-y-2.5"
            style={{
              backgroundColor: 'var(--bg-canvas)',
              borderColor: 'var(--border-subtle)',
            }}
          >
            <div className="flex items-center justify-between gap-2">
              <label htmlFor="ai-exercise-count" className="text-xs font-bold">
                Total de Exercícios do Módulo (mín. {MIN_EXERCISES} · máx. {MAX_EXERCISES})
              </label>
              <span
                className="px-2 py-0.5 rounded font-mono-code text-xs font-bold"
                style={{
                  backgroundColor: 'rgba(168, 85, 247, 0.16)',
                  color: 'var(--accent-gamification)',
                }}
              >
                {exerciseCount} fases
              </span>
            </div>

            <div className="flex items-center gap-3">
              <input
                id="ai-exercise-count"
                type="range"
                min={minSlider}
                max={MAX_EXERCISES}
                step={1}
                value={exerciseCount}
                onChange={(e) =>
                  setExerciseCount(clampExerciseCount(Number(e.target.value), minSlider))
                }
                className="flex-1 cursor-pointer accent-purple-500"
              />
              <input
                type="number"
                aria-label="Número exato de exercícios"
                min={minSlider}
                max={MAX_EXERCISES}
                value={exerciseCount}
                onChange={(e) =>
                  setExerciseCount(clampExerciseCount(Number(e.target.value), minSlider))
                }
                className="w-16 px-2 py-1 rounded-lg border font-mono-code text-xs text-center outline-none"
                style={{
                  backgroundColor: 'var(--bg-surface-panel)',
                  borderColor: 'var(--border-subtle)',
                  color: 'var(--text-primary)',
                }}
              />
            </div>

            <div className="text-[11px]" style={{ color: 'var(--text-secondary)' }}>
              {isExpand
                ? `Serão adicionados +${newExercisesDelta} exercícios inéditos antes do Desafio Chefão (mantendo todo seu progresso atual).`
                : `Estrutura: ${exerciseCount - 1} fases regulares/fixação + 1 Desafio Chefão final (sem pistas).`}
            </div>
          </div>

          {!isExpand && (
            <div>
              <label htmlFor="ai-custom-notes" className="block text-xs font-semibold mb-1">
                Instruções Pedagógicas Extras (opcional)
              </label>
              <textarea
                id="ai-custom-notes"
                rows={2}
                value={customInstructions}
                onChange={(e) => setCustomInstructions(e.target.value)}
                placeholder="Ex.: Contextualize os problemas com sensores de energia ou automação escolar."
                className="w-full px-3 py-2 rounded-lg border text-xs outline-none resize-none"
                style={{
                  backgroundColor: 'var(--bg-canvas)',
                  borderColor: 'var(--border-subtle)',
                  color: 'var(--text-primary)',
                }}
              />
            </div>
          )}

          {error && (
            <div
              role="alert"
              className="p-3 rounded-xl border flex items-start gap-2 text-xs"
              style={{
                backgroundColor: 'rgba(244, 63, 94, 0.12)',
                borderColor: 'var(--status-danger)',
                color: 'var(--status-danger)',
              }}
            >
              <AlertCircle className="w-4 h-4 shrink-0 mt-0.5" />
              <span>{error}</span>
            </div>
          )}

          <div className="flex items-center justify-end gap-2 pt-1">
            <button
              type="button"
              disabled={submitting}
              onClick={onClose}
              className="px-3.5 py-2 rounded-lg border text-xs font-semibold cursor-pointer disabled:opacity-40"
              style={{
                backgroundColor: 'var(--bg-surface-elevated)',
                borderColor: 'var(--border-subtle)',
              }}
            >
              Cancelar
            </button>
            <button
              type="submit"
              disabled={submitting}
              className="inline-flex items-center gap-1.5 px-4 py-2 rounded-lg text-xs font-bold cursor-pointer disabled:opacity-60"
              style={{
                backgroundColor: 'var(--accent-gamification)',
                color: '#FFFFFF',
              }}
            >
              {submitting ? (
                <>
                  <Loader2 className="w-3.5 h-3.5 animate-spin" />
                  Gerando e Validando no Sandbox...
                </>
              ) : (
                <>
                  <Sparkles className="w-3.5 h-3.5" />
                  {isExpand ? `Expandir para ${exerciseCount} Fases` : `Gerar ${exerciseCount} Fases com IA`}
                </>
              )}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
};
