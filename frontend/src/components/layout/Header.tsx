import { useEffect } from 'react';
import { Wifi, WifiOff, RefreshCw, Sun, Moon, Monitor } from 'lucide-react';
import { useWebSocket } from '@/api/hooks/useWebSocket';
import { useSettingsStore, getEffectiveTheme, type Theme } from '@/stores/settings';
import { cn } from '@/lib/utils';

const THEME_ICONS: Record<Theme, React.ComponentType<{ className?: string }>> = {
  light: Sun,
  dark: Moon,
  system: Monitor,
};

export function Header() {
  const { state, isConnected, isConnecting, isReconnecting } = useWebSocket();
  const { theme, setTheme } = useSettingsStore();

  // Apply theme to document
  useEffect(() => {
    const effectiveTheme = getEffectiveTheme(theme);
    document.documentElement.classList.remove('light', 'dark');
    document.documentElement.classList.add(effectiveTheme);
  }, [theme]);

  // Cycle through themes
  const cycleTheme = () => {
    const themes: Theme[] = ['dark', 'light', 'system'];
    const currentIndex = themes.indexOf(theme);
    const nextIndex = (currentIndex + 1) % themes.length;
    setTheme(themes[nextIndex]);
  };

  const ThemeIcon = THEME_ICONS[theme];

  return (
    <header className="h-14 border-b border-border bg-surface flex items-center justify-between px-6">
      <div className="flex items-center gap-4">
        <h2 className="text-lg font-semibold text-foreground">Dashboard</h2>
      </div>

      <div className="flex items-center gap-4">
        {/* Theme toggle */}
        <button
          onClick={cycleTheme}
          className="p-2 hover:bg-surface-hover rounded-lg transition-colors"
          title={`Theme: ${theme}`}
        >
          <ThemeIcon className="w-5 h-5 text-muted-foreground" />
        </button>

        {/* Connection status */}
        <div
          className={cn(
            'flex items-center gap-2 px-3 py-1.5 rounded-full text-sm',
            isConnected && 'bg-success/10 text-success',
            isConnecting && 'bg-warning/10 text-warning',
            isReconnecting && 'bg-warning/10 text-warning',
            state === 'disconnected' && 'bg-error/10 text-error'
          )}
        >
          {isConnected && (
            <>
              <Wifi className="w-4 h-4" />
              <span>Connected</span>
            </>
          )}
          {isConnecting && (
            <>
              <RefreshCw className="w-4 h-4 animate-spin" />
              <span>Connecting...</span>
            </>
          )}
          {isReconnecting && (
            <>
              <RefreshCw className="w-4 h-4 animate-spin" />
              <span>Reconnecting...</span>
            </>
          )}
          {state === 'disconnected' && (
            <>
              <WifiOff className="w-4 h-4" />
              <span>Disconnected</span>
            </>
          )}
        </div>
      </div>
    </header>
  );
}
