import { create } from 'zustand';
import { ipcClient, isTauriRuntime } from '../services/ipcClient';
import type { CurriculumPayload, LevelProgressRecord } from '../types/ipc';
import { useThemeStore } from './useThemeStore';
import { useKioskStore } from './useKioskStore';

export type ActiveView = 'kiosk' | 'map' | 'wallet';

interface ProgressionState {
  curriculum: CurriculumPayload | null;
  activeView: ActiveView;
  apiKeyModalOpen: boolean;
  loading: boolean;
  error: string | null;
  gatekeeperNotice: string | null;
  clockTamperAlert: boolean;
  xpGainPulse: number | null;

  setActiveView: (view: ActiveView) => void;
  setApiKeyModalOpen: (open: boolean) => void;
  setGatekeeperNotice: (msg: string | null) => void;
  clearXpPulse: () => void;

  loadCurriculum: () => Promise<void>;
  submitActiveCode: (level: LevelProgressRecord, code: string) => Promise<void>;
  requestHint: (level: LevelProgressRecord, code: string) => Promise<void>;
  saveApiKey: (apiKey: string) => Promise<void>;
  updateStudentName: (name: string) => Promise<void>;
}

/** Verifica se o código contém apenas espaços em branco ou comentários # */
export function isCodeEmptyOrCommentsOnly(code: string): boolean {
  const lines = code.split(/\r?\n/);
  return lines.every((line) => {
    const trimmed = line.trim();
    return trimmed.length === 0 || trimmed.startsWith('#');
  });
}

export const useProgressionStore = create<ProgressionState>((set, get) => ({
  curriculum: null,
  activeView: 'kiosk',
  apiKeyModalOpen: false,
  loading: false,
  error: null,
  gatekeeperNotice: null,
  clockTamperAlert: false,
  xpGainPulse: null,

  setActiveView: (activeView) => set({ activeView, gatekeeperNotice: null }),
  setApiKeyModalOpen: (apiKeyModalOpen) => set({ apiKeyModalOpen }),
  setGatekeeperNotice: (gatekeeperNotice) => set({ gatekeeperNotice }),
  clearXpPulse: () => set({ xpGainPulse: null }),

  loadCurriculum: async () => {
    if (!isTauriRuntime()) {
      return;
    }
    set({ loading: true, error: null });
    try {
      const data = await ipcClient.getCurriculum();
      const mappedTheme =
        data.profile.theme_preference === 'light_tech' ? 'light-tech' : 'dark-tech';
      useThemeStore.getState().setTheme(mappedTheme, false);

      const kiosk = useKioskStore.getState();
      if (!kiosk.activeLevelId && data.levels.length > 0) {
        const firstPlayable =
          data.levels.find((l) => l.status === 'UNLOCKED') ?? data.levels[0];
        kiosk.setActiveLevel(
          firstPlayable.id,
          firstPlayable.last_submitted_code ?? firstPlayable.starter_code
        );
      }
      set({ curriculum: data, loading: false });
    } catch (err) {
      set({
        error: err instanceof Error ? err.message : String(err),
        loading: false,
      });
    }
  },

  submitActiveCode: async (level, code) => {
    const kiosk = useKioskStore.getState();
    if (isCodeEmptyOrCommentsOnly(code)) {
      kiosk.setEmptySubmissionWarning(
        'Seu editor contém apenas espaços ou comentários (#). Escreva uma função Python antes de executar.'
      );
      return;
    }

    kiosk.setIsRunning(true);
    kiosk.setEmptySubmissionWarning(null);
    const startedAt = kiosk.levelStartTimestamps[level.id] ?? Date.now();
    const elapsedSeconds = Math.max(1, Math.round((Date.now() - startedAt) / 1000));

    try {
      const res = await ipcClient.submitCode(level.id, code, elapsedSeconds);
      kiosk.setSubmissionResult(level.id, res);
      if (res.clock_tamper_detected) {
        set({ clockTamperAlert: true });
      }
      if (res.reward && res.reward.earned_xp > 0) {
        set({ xpGainPulse: res.reward.earned_xp });
      }
      await get().loadCurriculum();
    } catch (err) {
      set({ error: err instanceof Error ? err.message : String(err) });
    } finally {
      kiosk.setIsRunning(false);
    }
  },

  requestHint: async (level, code) => {
    const kiosk = useKioskStore.getState();
    if (level.level_type === 'BOSS') return;
    kiosk.setIsRequestingHint(true);
    try {
      const lastResult = kiosk.lastResultByLevel[level.id];
      const failedTag =
        lastResult?.execution_summary?.first_failed_concept_tag ?? null;
      const hint = await ipcClient.requestSocraticHint(level.id, code, failedTag);
      kiosk.appendSocraticMessage(level.id, hint);
      await get().loadCurriculum();
    } catch (err) {
      set({ error: err instanceof Error ? err.message : String(err) });
    } finally {
      kiosk.setIsRequestingHint(false);
    }
  },

  saveApiKey: async (apiKey) => {
    await ipcClient.saveApiKey(apiKey);
    await get().loadCurriculum();
    set({ apiKeyModalOpen: false });
  },

  updateStudentName: async (name) => {
    await ipcClient.updateStudentName(name);
    await get().loadCurriculum();
  },
}));
