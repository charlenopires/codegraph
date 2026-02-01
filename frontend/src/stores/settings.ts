import { create } from 'zustand';
import { persist } from 'zustand/middleware';

export type Theme = 'dark' | 'light' | 'system';
export type DesignSystem = 'material-ui' | 'tailwind' | 'chakra' | 'bootstrap' | 'any';

interface SettingsState {
  // Connection settings
  websocketUrl: string;

  // NARS reasoning
  narsEnabled: boolean;

  // Default preferences
  defaultDesignSystem: DesignSystem;
  defaultQueryLimit: number;

  // Theme
  theme: Theme;

  // Advanced
  showDebugInfo: boolean;
}

interface SettingsActions {
  setWebsocketUrl: (url: string) => void;
  setNarsEnabled: (enabled: boolean) => void;
  setDefaultDesignSystem: (system: DesignSystem) => void;
  setDefaultQueryLimit: (limit: number) => void;
  setTheme: (theme: Theme) => void;
  setShowDebugInfo: (show: boolean) => void;
  resetToDefaults: () => void;
}

type SettingsStore = SettingsState & SettingsActions;

const defaultSettings: SettingsState = {
  websocketUrl: 'ws://localhost:3000/ws',
  narsEnabled: true,
  defaultDesignSystem: 'any',
  defaultQueryLimit: 10,
  theme: 'dark',
  showDebugInfo: false,
};

export const useSettingsStore = create<SettingsStore>()(
  persist(
    (set) => ({
      ...defaultSettings,

      setWebsocketUrl: (url) => set({ websocketUrl: url }),
      setNarsEnabled: (enabled) => set({ narsEnabled: enabled }),
      setDefaultDesignSystem: (system) => set({ defaultDesignSystem: system }),
      setDefaultQueryLimit: (limit) => set({ defaultQueryLimit: limit }),
      setTheme: (theme) => set({ theme: theme }),
      setShowDebugInfo: (show) => set({ showDebugInfo: show }),

      resetToDefaults: () => set(defaultSettings),
    }),
    {
      name: 'codegraph-settings',
    }
  )
);

// Helper to get the effective theme (resolving 'system')
export function getEffectiveTheme(theme: Theme): 'dark' | 'light' {
  if (theme === 'system') {
    return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
  }
  return theme;
}
