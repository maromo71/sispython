import React from 'react';
import {
  Award,
  Flame,
  KeyRound,
  Map,
  Moon,
  Sparkles,
  Sun,
  Terminal,
  Volume2,
  VolumeX,
  Wifi,
  WifiOff,
} from 'lucide-react';
import { motion, AnimatePresence } from 'framer-motion';
import { LogoMarkIcon } from '../icons/LogicKioskIcons';
import { useThemeStore } from '../../store/useThemeStore';
import { useProgressionStore } from '../../store/useProgressionStore';

export const TopHudBar: React.FC = () => {
  const { theme, toggleTheme, soundEnabled, toggleSound } = useThemeStore();
  const {
    curriculum,
    activeView,
    setActiveView,
    setApiKeyModalOpen,
    xpGainPulse,
    clearXpPulse,
  } = useProgressionStore();

  const profile = curriculum?.profile;
  const levels = curriculum?.levels ?? [];
  const completedCount = levels.filter((l) => l.status === 'COMPLETED_MASTERY').length;
  const progressPct =
    levels.length > 0 ? Math.round((completedCount / levels.length) * 100) : 0;

  const isOnlineAi = curriculum?.ai_key_configured ?? false;

  return (
    <header
      className="flex flex-wrap items-center justify-between gap-2 px-4 py-2.5 border-b select-none"
      style={{
        backgroundColor: 'var(--bg-surface-panel)',
        borderColor: 'var(--border-subtle)',
      }}
    >
      {/* Brand + Module Progress */}
      <div className="flex items-center gap-4 min-w-0">
        <div className="flex items-center gap-2">
          <div
            className="w-8 h-8 rounded-lg flex items-center justify-center border"
            style={{
              backgroundColor: 'var(--bg-surface-elevated)',
              borderColor: 'var(--accent-primary)',
              color: 'var(--accent-primary)',
            }}
          >
            <LogoMarkIcon className="w-6 h-6" />
          </div>
          <div>
            <div className="font-bold text-sm tracking-tight leading-none">
              LogicKiosk
            </div>
            <div className="text-[11px] mt-0.5" style={{ color: 'var(--text-secondary)' }}>
              Módulo 1 · {progressPct}% Concluído
            </div>
          </div>
        </div>

        {/* Progress Bar */}
        <div className="hidden md:flex items-center gap-2">
          <div
            className="w-28 h-2 rounded-full overflow-hidden border"
            style={{
              backgroundColor: 'var(--bg-canvas)',
              borderColor: 'var(--border-subtle)',
            }}
            aria-label={`Progresso do módulo: ${progressPct}%`}
          >
            <div
              className="h-full transition-all duration-500"
              style={{
                width: `${progressPct}%`,
                backgroundColor: 'var(--status-success)',
              }}
            />
          </div>
          <span className="text-xs font-mono-code" style={{ color: 'var(--text-secondary)' }}>
            {completedCount}/{levels.length}
          </span>
        </div>
      </div>

      {/* Navigation Tabs */}
      <nav className="flex items-center gap-1.5" aria-label="Navegação principal">
        <button
          type="button"
          onClick={() => setActiveView('kiosk')}
          className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold transition-colors cursor-pointer"
          style={{
            backgroundColor:
              activeView === 'kiosk' ? 'var(--accent-primary)' : 'var(--bg-surface-elevated)',
            color: activeView === 'kiosk' ? '#0B0F19' : 'var(--text-primary)',
          }}
        >
          <Terminal className="w-3.5 h-3.5" />
          Quiosque
        </button>

        <button
          type="button"
          onClick={() => setActiveView('map')}
          className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold transition-colors cursor-pointer"
          style={{
            backgroundColor:
              activeView === 'map' ? 'var(--accent-primary)' : 'var(--bg-surface-elevated)',
            color: activeView === 'map' ? '#0B0F19' : 'var(--text-primary)',
          }}
        >
          <Map className="w-3.5 h-3.5" />
          Habilidades
        </button>

        <button
          type="button"
          onClick={() => setActiveView('wallet')}
          className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold transition-colors cursor-pointer"
          style={{
            backgroundColor:
              activeView === 'wallet' ? 'var(--accent-gamification)' : 'var(--bg-surface-elevated)',
            color: activeView === 'wallet' ? '#FFFFFF' : 'var(--text-primary)',
          }}
        >
          <Award className="w-3.5 h-3.5" />
          Carteira SHA-256
        </button>
      </nav>

      {/* Telemetry & Controls */}
      <div className="flex items-center gap-2">
        {/* XP Pill */}
        <div
          className="relative flex items-center gap-1.5 px-2.5 py-1 rounded-lg border text-xs font-semibold"
          style={{
            backgroundColor: 'var(--bg-surface-elevated)',
            borderColor: 'var(--border-subtle)',
          }}
          title="Total de XP e Nível do Perfil"
        >
          <Sparkles className="w-3.5 h-3.5" style={{ color: 'var(--accent-gamification)' }} />
          <span className="font-mono-code">{profile?.total_xp ?? 0} XP</span>
          <span className="text-[11px]" style={{ color: 'var(--text-secondary)' }}>
            · Nv.{profile?.current_level_rank ?? 1}
          </span>
          <AnimatePresence>
            {xpGainPulse && (
              <motion.span
                initial={{ opacity: 0, y: 12, scale: 0.8 }}
                animate={{ opacity: 1, y: -18, scale: 1.05 }}
                exit={{ opacity: 0, y: -28 }}
                transition={{ duration: 1.2 }}
                onAnimationComplete={clearXpPulse}
                className="absolute -top-2 right-0 px-1.5 py-0.5 rounded text-[11px] font-bold pointer-events-none"
                style={{
                  backgroundColor: 'var(--status-success)',
                  color: '#0B0F19',
                }}
              >
                +{xpGainPulse} XP
              </motion.span>
            )}
          </AnimatePresence>
        </div>

        {/* Streak Pill */}
        <div
          className="flex items-center gap-1 px-2.5 py-1 rounded-lg border text-xs font-semibold"
          style={{
            backgroundColor: 'var(--bg-surface-elevated)',
            borderColor: 'var(--border-subtle)',
          }}
          title="Ofensiva de dias consecutivos"
        >
          <Flame className="w-3.5 h-3.5" style={{ color: 'var(--status-warning)' }} />
          <span className="font-mono-code">{profile?.current_streak_days ?? 1}d</span>
        </div>

        {/* AI Connection Status + BYOK button */}
        <button
          type="button"
          onClick={() => setApiKeyModalOpen(true)}
          className="hidden sm:flex items-center gap-1.5 px-2.5 py-1 rounded-lg border text-xs font-medium cursor-pointer transition-colors"
          style={{
            backgroundColor: 'var(--bg-surface-elevated)',
            borderColor: isOnlineAi ? 'var(--status-success)' : 'var(--border-subtle)',
            color: isOnlineAi ? 'var(--status-success)' : 'var(--text-secondary)',
          }}
          title="Configurar Chave Gratuita Gemini AI Studio (BYOK)"
        >
          {isOnlineAi ? <Wifi className="w-3.5 h-3.5" /> : <WifiOff className="w-3.5 h-3.5" />}
          <span>{curriculum?.ai_status_label ?? 'Offline: Motor Nativo Rust'}</span>
          <KeyRound className="w-3 h-3 ml-0.5 opacity-80" />
        </button>

        {/* Sound toggle */}
        <button
          type="button"
          onClick={toggleSound}
          aria-label={soundEnabled ? 'Desativar som de sucesso' : 'Ativar som de sucesso'}
          className="p-1.5 rounded-lg border cursor-pointer"
          style={{
            backgroundColor: 'var(--bg-surface-elevated)',
            borderColor: 'var(--border-subtle)',
            color: 'var(--text-secondary)',
          }}
        >
          {soundEnabled ? <Volume2 className="w-4 h-4" /> : <VolumeX className="w-4 h-4" />}
        </button>

        {/* Theme Toggle (Alt+T) */}
        <button
          type="button"
          onClick={toggleTheme}
          aria-label="Alternar tema Dark/Light (Alt+T)"
          title="Alternar tema Dark/Light (Alt+T)"
          className="flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg border text-xs font-semibold cursor-pointer"
          style={{
            backgroundColor: 'var(--bg-surface-elevated)',
            borderColor: 'var(--border-subtle)',
            color: 'var(--text-primary)',
          }}
        >
          {theme === 'dark-tech' ? (
            <>
              <Sun className="w-3.5 h-3.5" style={{ color: 'var(--status-warning)' }} />
              <span className="hidden lg:inline">Light</span>
            </>
          ) : (
            <>
              <Moon className="w-3.5 h-3.5" style={{ color: 'var(--accent-primary)' }} />
              <span className="hidden lg:inline">Dark</span>
            </>
          )}
        </button>
      </div>
    </header>
  );
};
