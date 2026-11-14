import { create } from 'zustand';
import { ipcClient, isTauriRuntime } from '../services/ipcClient';

export type ThemeMode = 'dark-tech' | 'light-tech';

interface ThemeState {
  theme: ThemeMode;
  soundEnabled: boolean;
  setTheme: (theme: ThemeMode, persist?: boolean) => void;
  toggleTheme: () => void;
  toggleSound: () => void;
}

function applyThemeAttr(theme: ThemeMode) {
  if (typeof document !== 'undefined') {
    document.documentElement.setAttribute('data-theme', theme);
  }
}

export const useThemeStore = create<ThemeState>((set, get) => ({
  theme: 'dark-tech',
  soundEnabled: true,

  setTheme: (theme, persist = false) => {
    applyThemeAttr(theme);
    set({ theme });
    if (persist && isTauriRuntime()) {
      const dbTheme = theme === 'dark-tech' ? 'dark_tech' : 'light_tech';
      ipcClient.toggleTheme(dbTheme).catch(() => {});
    }
  },

  toggleTheme: () => {
    const next: ThemeMode = get().theme === 'dark-tech' ? 'light-tech' : 'dark-tech';
    get().setTheme(next, true);
  },

  toggleSound: () => set((s) => ({ soundEnabled: !s.soundEnabled })),
}));
