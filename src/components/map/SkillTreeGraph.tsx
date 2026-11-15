import React, { useState } from 'react';
import {
  AlertCircle,
  Brain,
  CheckCircle2,
  Lock,
  Play,
  PlusCircle,
  ShieldAlert,
  ShieldCheck,
  SlidersHorizontal,
  Sparkles,
  Zap,
} from 'lucide-react';
import { useProgressionStore } from '../../store/useProgressionStore';
import { useKioskStore } from '../../store/useKioskStore';
import type { LevelProgressRecord, ModuleRecord } from '../../types/ipc';
import {
  AiModuleGeneratorModal,
  MAX_EXERCISES,
} from './AiModuleGeneratorModal';

function renderBadgeIcon(iconName: string) {
  switch (iconName) {
    case 'zap':
      return <Zap className="w-5 h-5" />;
    case 'brain':
      return <Brain className="w-5 h-5" />;
    case 'shield-check':
      return <ShieldCheck className="w-5 h-5" />;
    default:
      return <Zap className="w-5 h-5" />;
  }
}

export const SkillTreeGraph: React.FC = () => {
  const { curriculum, setActiveView, gatekeeperNotice, setGatekeeperNotice } =
    useProgressionStore();
  const { setActiveLevel, activeLevelId } = useKioskStore();

  const [aiModalOpen, setAiModalOpen] = useState(false);
  const [aiModalMode, setAiModalMode] = useState<'create' | 'expand'>('create');
  const [selectedModule, setSelectedModule] = useState<ModuleRecord | null>(null);
  const [selectedModuleCount, setSelectedModuleCount] = useState<number>(5);
  const [aiNotice, setAiNotice] = useState<string | null>(null);

  const modules = curriculum?.modules ?? [];
  const levels = curriculum?.levels ?? [];
  const badges = curriculum?.badges ?? [];

  const openCreateModal = () => {
    setAiModalMode('create');
    setSelectedModule(null);
    setSelectedModuleCount(5);
    setAiModalOpen(true);
  };

  const openExpandModal = (mod: ModuleRecord, count: number) => {
    setAiModalMode('expand');
    setSelectedModule(mod);
    setSelectedModuleCount(count);
    setAiModalOpen(true);
  };

  const handleSelectLevel = (lvl: LevelProgressRecord) => {
    if (lvl.status === 'LOCKED') {
      setGatekeeperNotice(
        `Fase "${lvl.title}" bloqueada! É necessário aprovar 100% dos casos públicos, ocultos e regras de AST da fase anterior para desbloquear.`
      );
      return;
    }
    setGatekeeperNotice(null);
    setActiveLevel(lvl.id, lvl.last_submitted_code ?? lvl.starter_code);
    setActiveView('kiosk');
  };

  return (
    <div
      className="flex-1 overflow-y-auto p-6 max-w-5xl w-full mx-auto space-y-8"
      aria-label="Mapa de Habilidades"
    >
      {/* AI Curriculum Controls Banner */}
      <div
        className="p-4 rounded-2xl border flex flex-wrap items-center justify-between gap-4"
        style={{
          backgroundColor: 'var(--bg-surface-panel)',
          borderColor: 'var(--border-subtle)',
        }}
      >
        <div className="flex items-center gap-3">
          <div
            className="p-2.5 rounded-xl shrink-0"
            style={{
              backgroundColor: 'rgba(168, 85, 247, 0.16)',
              color: 'var(--accent-gamification)',
            }}
          >
            <Sparkles className="w-5 h-5" />
          </div>
          <div>
            <h2 className="text-xs font-bold uppercase tracking-wide">
              Expansão de Trilha com IA (5 a 20 Fases por Módulo)
            </h2>
            <p className="text-xs mt-0.5" style={{ color: 'var(--text-secondary)' }}>
              Gere novos módulos temáticos ou aumente a quantidade de exercícios de qualquer módulo respeitando AST e Sandbox.
            </p>
          </div>
        </div>
        <button
          type="button"
          onClick={openCreateModal}
          className="inline-flex items-center gap-1.5 px-3.5 py-2 rounded-xl text-xs font-bold cursor-pointer"
          style={{
            backgroundColor: 'var(--accent-gamification)',
            color: '#FFFFFF',
          }}
        >
          <PlusCircle className="w-4 h-4" />
          Gerar Novo Módulo com IA
        </button>
      </div>

      {aiNotice && (
        <div
          role="status"
          className="p-4 rounded-xl border flex items-center justify-between gap-3 text-xs font-medium"
          style={{
            backgroundColor: 'rgba(16, 185, 129, 0.12)',
            borderColor: 'var(--status-success)',
            color: 'var(--status-success)',
          }}
        >
          <div className="flex items-center gap-2">
            <CheckCircle2 className="w-4 h-4 shrink-0" />
            <span>{aiNotice}</span>
          </div>
          <button
            type="button"
            onClick={() => setAiNotice(null)}
            className="underline cursor-pointer shrink-0"
          >
            Fechar
          </button>
        </div>
      )}

      {/* Hard Gatekeeping Alert */}
      {gatekeeperNotice && (
        <div
          role="alert"
          className="p-4 rounded-xl border flex items-center justify-between gap-3 text-xs font-medium"
          style={{
            backgroundColor: 'rgba(244, 63, 94, 0.12)',
            borderColor: 'var(--status-danger)',
            color: 'var(--status-danger)',
          }}
        >
          <div className="flex items-center gap-2">
            <AlertCircle className="w-4 h-4 shrink-0" />
            <span>{gatekeeperNotice}</span>
          </div>
          <button
            type="button"
            onClick={() => setGatekeeperNotice(null)}
            className="underline cursor-pointer shrink-0"
          >
            Fechar
          </button>
        </div>
      )}

      {/* Modules & Skill Tree Nodes */}
      {modules.map((mod) => {
        const modLevels = levels.filter((l) => l.module_id === mod.id);
        return (
          <section key={mod.id} className="space-y-4">
            <div
              className="p-5 rounded-2xl border flex flex-wrap items-start justify-between gap-4"
              style={{
                backgroundColor: 'var(--bg-surface-panel)',
                borderColor: 'var(--border-subtle)',
              }}
            >
              <div>
                <div className="flex items-center gap-2">
                  <h2 className="text-base font-bold">{mod.title}</h2>
                  <span
                    className="px-2 py-0.5 rounded-full text-[10px] font-mono-code font-bold"
                    style={{
                      backgroundColor: 'var(--bg-surface-elevated)',
                      color: 'var(--text-secondary)',
                    }}
                  >
                    {modLevels.length} / {MAX_EXERCISES} fases
                  </span>
                </div>
                <p className="text-xs mt-1 leading-relaxed" style={{ color: 'var(--text-secondary)' }}>
                  {mod.description}
                </p>
              </div>
              <button
                type="button"
                disabled={modLevels.length >= MAX_EXERCISES}
                onClick={() => openExpandModal(mod, modLevels.length)}
                title={
                  modLevels.length >= MAX_EXERCISES
                    ? `Limite máximo de ${MAX_EXERCISES} fases atingido`
                    : 'Aumentar quantidade de exercícios deste módulo com IA'
                }
                className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg border text-xs font-semibold cursor-pointer disabled:opacity-45 disabled:cursor-not-allowed"
                style={{
                  backgroundColor: 'var(--bg-surface-elevated)',
                  borderColor: 'var(--border-subtle)',
                  color: 'var(--text-primary)',
                }}
              >
                <SlidersHorizontal className="w-3.5 h-3.5" />
                {modLevels.length >= MAX_EXERCISES ? 'Máximo (20/20)' : 'Customizar Fases (IA)'}
              </button>
            </div>

            <div className="relative pl-6 space-y-4">
              <div
                className="absolute left-3 top-4 bottom-4 w-0.5"
                style={{ backgroundColor: 'var(--border-subtle)' }}
                aria-hidden="true"
              />

              {modLevels.map((lvl) => {
                const isBoss = lvl.level_type === 'BOSS';
                const isLocked = lvl.status === 'LOCKED';
                const isMastered = lvl.status === 'COMPLETED_MASTERY';
                const isCurrent = lvl.id === activeLevelId;

                const borderColor = isBoss
                  ? 'var(--accent-gamification)'
                  : isMastered
                  ? 'var(--status-success)'
                  : isCurrent
                  ? 'var(--accent-primary)'
                  : 'var(--border-subtle)';

                return (
                  <div key={lvl.id} className="relative">
                    {/* Node Dot */}
                    <span
                      className="absolute -left-[19px] top-6 w-3.5 h-3.5 rounded-full border-2"
                      style={{
                        backgroundColor: isMastered
                          ? 'var(--status-success)'
                          : isLocked
                          ? 'var(--bg-canvas)'
                          : isBoss
                          ? 'var(--accent-gamification)'
                          : 'var(--accent-primary)',
                        borderColor,
                      }}
                    />

                    <button
                      type="button"
                      onClick={() => handleSelectLevel(lvl)}
                      className="w-full text-left p-4 rounded-2xl border transition-all cursor-pointer flex flex-wrap items-center justify-between gap-4"
                      style={{
                        backgroundColor: 'var(--bg-surface-panel)',
                        borderColor,
                        opacity: isLocked ? 0.65 : 1,
                      }}
                    >
                      <div className="space-y-1">
                        <div className="flex items-center gap-2">
                          {isBoss && (
                            <span
                              className="inline-flex items-center gap-1 px-2 py-0.5 rounded text-[10px] font-bold uppercase"
                              style={{
                                backgroundColor: 'var(--accent-gamification)',
                                color: '#FFFFFF',
                              }}
                            >
                              <ShieldAlert className="w-3 h-3" />
                              Desafio Chefão
                            </span>
                          )}
                          <span className="text-xs font-mono-code" style={{ color: 'var(--text-secondary)' }}>
                            {lvl.base_xp_reward} XP base
                          </span>
                          {lvl.needs_fixation_review && (
                            <span
                              className="px-2 py-0.5 rounded text-[10px] font-semibold"
                              style={{
                                backgroundColor: 'rgba(245, 158, 11, 0.16)',
                                color: 'var(--status-warning)',
                              }}
                            >
                              Revisão Recomendada
                            </span>
                          )}
                        </div>
                        <h3 className="text-sm font-bold">{lvl.title}</h3>
                        <div className="text-[11px]" style={{ color: 'var(--text-secondary)' }}>
                          AST obrigatória: {lvl.must_contain_nodes.join(', ')}
                        </div>
                      </div>

                      <div className="flex items-center gap-3">
                        {isMastered && (
                          <div className="text-right text-[11px] font-mono-code">
                            <div style={{ color: 'var(--status-success)' }}>
                              +{lvl.earned_xp} XP
                            </div>
                            <div style={{ color: 'var(--text-secondary)' }}>
                              {lvl.best_execution_time_ms ?? 0} ms · {lvl.attempts_count} tent.
                            </div>
                          </div>
                        )}

                        <span
                          className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-bold"
                          style={{
                            backgroundColor: isLocked
                              ? 'var(--bg-surface-elevated)'
                              : isMastered
                              ? 'rgba(16, 185, 129, 0.16)'
                              : 'var(--accent-primary)',
                            color: isLocked
                              ? 'var(--text-secondary)'
                              : isMastered
                              ? 'var(--status-success)'
                              : '#0B0F19',
                          }}
                        >
                          {isLocked ? (
                            <>
                              <Lock className="w-3.5 h-3.5" /> Bloqueada
                            </>
                          ) : isMastered ? (
                            <>
                              <CheckCircle2 className="w-3.5 h-3.5" /> Revisar
                            </>
                          ) : (
                            <>
                              <Play className="w-3.5 h-3.5 fill-current" /> Abrir Quiosque
                            </>
                          )}
                        </span>
                      </div>
                    </button>
                  </div>
                );
              })}
            </div>
          </section>
        );
      })}

      {/* Badges Shelf */}
      <section className="space-y-3">
        <h2 className="text-sm font-bold uppercase tracking-wide" style={{ color: 'var(--text-secondary)' }}>
          Insígnias de Maestria
        </h2>
        <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
          {badges.map((bdg) => (
            <div
              key={bdg.id}
              className="p-4 rounded-2xl border flex items-start gap-3"
              style={{
                backgroundColor: 'var(--bg-surface-panel)',
                borderColor: bdg.unlocked ? 'var(--accent-gamification)' : 'var(--border-subtle)',
                opacity: bdg.unlocked ? 1 : 0.55,
              }}
            >
              <div
                className="p-2.5 rounded-xl shrink-0"
                style={{
                  backgroundColor: bdg.unlocked
                    ? 'rgba(168, 85, 247, 0.18)'
                    : 'var(--bg-surface-elevated)',
                  color: bdg.unlocked ? 'var(--accent-gamification)' : 'var(--text-secondary)',
                }}
              >
                {renderBadgeIcon(bdg.icon_name)}
              </div>
              <div>
                <div className="text-xs font-bold">{bdg.title}</div>
                <p className="text-[11px] mt-0.5 leading-relaxed" style={{ color: 'var(--text-secondary)' }}>
                  {bdg.description}
                </p>
                {bdg.unlocked && (
                  <span className="inline-block mt-1.5 text-[10px] font-semibold" style={{ color: 'var(--status-success)' }}>
                    Conquistada
                  </span>
                )}
              </div>
            </div>
          ))}
        </div>
      </section>

      <AiModuleGeneratorModal
        open={aiModalOpen}
        mode={aiModalMode}
        targetModule={selectedModule}
        currentLevelCount={selectedModuleCount}
        onClose={() => setAiModalOpen(false)}
        onSuccess={(msg) => setAiNotice(msg)}
      />
    </div>
  );
};
