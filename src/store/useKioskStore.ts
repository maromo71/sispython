import { create } from 'zustand';
import type { SocraticHintResponse, SubmitCodeResponse } from '../types/ipc';

export type MobileTab = 'mission' | 'editor' | 'console';

export interface SocraticMessage {
  id: string;
  timestamp: string;
  hint: SocraticHintResponse;
}

interface KioskState {
  activeLevelId: string | null;
  codeByLevel: Record<string, string>;
  isRunning: boolean;
  isRequestingHint: boolean;
  lastResultByLevel: Record<string, SubmitCodeResponse | null>;
  socraticMessagesByLevel: Record<string, SocraticMessage[]>;
  focusLine: number | null;
  activeMobileTab: MobileTab;
  emptySubmissionWarning: string | null;
  levelStartTimestamps: Record<string, number>;
  calibrationDone: boolean;

  setActiveLevel: (levelId: string, defaultCode: string) => void;
  setCode: (levelId: string, code: string) => void;
  setIsRunning: (running: boolean) => void;
  setIsRequestingHint: (loading: boolean) => void;
  setSubmissionResult: (levelId: string, res: SubmitCodeResponse) => void;
  appendSocraticMessage: (levelId: string, hint: SocraticHintResponse) => void;
  setFocusLine: (line: number | null) => void;
  setActiveMobileTab: (tab: MobileTab) => void;
  setEmptySubmissionWarning: (msg: string | null) => void;
  completeCalibration: () => void;
  resetLevelCode: (levelId: string, starterCode: string) => void;
}

export const useKioskStore = create<KioskState>((set, get) => ({
  activeLevelId: null,
  codeByLevel: {},
  isRunning: false,
  isRequestingHint: false,
  lastResultByLevel: {},
  socraticMessagesByLevel: {},
  focusLine: null,
  activeMobileTab: 'editor',
  emptySubmissionWarning: null,
  levelStartTimestamps: {},
  calibrationDone:
    typeof window !== 'undefined' &&
    window.localStorage?.getItem('logickiosk_calibrated') === '1',

  setActiveLevel: (levelId, defaultCode) => {
    const state = get();
    const hasCode = Object.prototype.hasOwnProperty.call(state.codeByLevel, levelId);
    const hasStart = Object.prototype.hasOwnProperty.call(state.levelStartTimestamps, levelId);
    set({
      activeLevelId: levelId,
      focusLine: null,
      emptySubmissionWarning: null,
      codeByLevel: hasCode
        ? state.codeByLevel
        : { ...state.codeByLevel, [levelId]: defaultCode },
      levelStartTimestamps: hasStart
        ? state.levelStartTimestamps
        : { ...state.levelStartTimestamps, [levelId]: Date.now() },
    });
  },

  setCode: (levelId, code) =>
    set((s) => ({
      codeByLevel: { ...s.codeByLevel, [levelId]: code },
      emptySubmissionWarning: null,
    })),

  setIsRunning: (isRunning) => set({ isRunning }),
  setIsRequestingHint: (isRequestingHint) => set({ isRequestingHint }),

  setSubmissionResult: (levelId, res) => {
    set((s) => {
      const nextMessages = { ...s.socraticMessagesByLevel };
      if (res.socratic_hint) {
        const prev = nextMessages[levelId] ?? [];
        nextMessages[levelId] = [
          ...prev,
          {
            id: `${Date.now()}-${Math.random()}`,
            timestamp: new Date().toLocaleTimeString('pt-BR', {
              hour: '2-digit',
              minute: '2-digit',
            }),
            hint: res.socratic_hint,
          },
        ];
      }
      return {
        lastResultByLevel: { ...s.lastResultByLevel, [levelId]: res },
        socraticMessagesByLevel: nextMessages,
        focusLine: res.socratic_hint?.focus_line ?? null,
      };
    });
  },

  appendSocraticMessage: (levelId, hint) =>
    set((s) => {
      const prev = s.socraticMessagesByLevel[levelId] ?? [];
      return {
        socraticMessagesByLevel: {
          ...s.socraticMessagesByLevel,
          [levelId]: [
            ...prev,
            {
              id: `${Date.now()}-${Math.random()}`,
              timestamp: new Date().toLocaleTimeString('pt-BR', {
                hour: '2-digit',
                minute: '2-digit',
              }),
              hint,
            },
          ],
        },
        focusLine: hint.focus_line,
      };
    }),

  setFocusLine: (focusLine) => set({ focusLine }),
  setActiveMobileTab: (activeMobileTab) => set({ activeMobileTab }),
  setEmptySubmissionWarning: (emptySubmissionWarning) => set({ emptySubmissionWarning }),

  completeCalibration: () => {
    if (typeof window !== 'undefined' && window.localStorage) {
      window.localStorage.setItem('logickiosk_calibrated', '1');
    }
    set({ calibrationDone: true });
  },

  resetLevelCode: (levelId, starterCode) =>
    set((s) => ({
      codeByLevel: { ...s.codeByLevel, [levelId]: starterCode },
      focusLine: null,
      emptySubmissionWarning: null,
    })),
}));
