import { useState } from 'react';
import {
  Wifi,
  Brain,
  Palette,
  RotateCcw,
  Check,
  AlertCircle,
  Sun,
  Moon,
  Monitor,
  Database,
  Sliders,
} from 'lucide-react';
import { useSettingsStore, type Theme, type DesignSystem } from '@/stores/settings';
import { cn } from '@/lib/utils';

const DESIGN_SYSTEMS: { value: DesignSystem; label: string; description: string }[] = [
  { value: 'any', label: 'Any', description: 'No preference, search all design systems' },
  { value: 'material-ui', label: 'Material UI', description: 'Google\'s Material Design components' },
  { value: 'tailwind', label: 'Tailwind CSS', description: 'Utility-first CSS framework' },
  { value: 'chakra', label: 'Chakra UI', description: 'Simple, modular component library' },
  { value: 'bootstrap', label: 'Bootstrap', description: 'Popular responsive framework' },
];

const QUERY_LIMITS = [3, 5, 10, 20, 50];

const THEMES: { value: Theme; label: string; icon: React.ComponentType<{ className?: string }> }[] = [
  { value: 'dark', label: 'Dark', icon: Moon },
  { value: 'light', label: 'Light', icon: Sun },
  { value: 'system', label: 'System', icon: Monitor },
];

export function SettingsPage() {
  const settings = useSettingsStore();
  const [tempUrl, setTempUrl] = useState(settings.websocketUrl);
  const [urlSaved, setUrlSaved] = useState(false);

  const handleSaveUrl = () => {
    settings.setWebsocketUrl(tempUrl);
    setUrlSaved(true);
    setTimeout(() => setUrlSaved(false), 2000);
  };

  const handleReset = () => {
    if (window.confirm('Are you sure you want to reset all settings to defaults?')) {
      settings.resetToDefaults();
      setTempUrl('ws://localhost:3000/ws');
    }
  };

  const handleClearStorage = () => {
    if (window.confirm('This will clear all stored data including settings. Continue?')) {
      localStorage.clear();
      sessionStorage.clear();
      window.location.reload();
    }
  };

  return (
    <div className="space-y-6 max-w-3xl">
      <div>
        <h1 className="text-2xl font-bold text-foreground">Settings</h1>
        <p className="text-muted-foreground mt-1">
          Configure CodeGraph to match your preferences
        </p>
      </div>

      {/* Connection Settings */}
      <section className="bg-surface rounded-lg border border-border p-6 space-y-4">
        <div className="flex items-center gap-3">
          <div className="p-2 bg-primary-600/20 rounded-lg">
            <Wifi className="w-5 h-5 text-primary-400" />
          </div>
          <div>
            <h2 className="text-lg font-semibold text-foreground">Connection</h2>
            <p className="text-sm text-muted-foreground">WebSocket server configuration</p>
          </div>
        </div>

        <div className="space-y-3">
          <label className="block">
            <span className="text-sm font-medium text-foreground">WebSocket URL</span>
            <div className="mt-1.5 flex gap-2">
              <input
                type="text"
                value={tempUrl}
                onChange={(e) => setTempUrl(e.target.value)}
                className="flex-1 px-3 py-2 bg-surface-elevated border border-border rounded-lg text-foreground placeholder-muted-foreground focus:ring-2 focus:ring-primary-500 focus:border-transparent"
                placeholder="ws://localhost:3000/ws"
              />
              <button
                onClick={handleSaveUrl}
                className={cn(
                  'px-4 py-2 rounded-lg font-medium transition-colors',
                  urlSaved
                    ? 'bg-success text-white'
                    : 'bg-primary-600 hover:bg-primary-700 text-white'
                )}
              >
                {urlSaved ? (
                  <Check className="w-5 h-5" />
                ) : (
                  'Save'
                )}
              </button>
            </div>
            <p className="text-xs text-muted-foreground mt-1.5">
              Changes require a page refresh to take effect
            </p>
          </label>
        </div>
      </section>

      {/* NARS Reasoning */}
      <section className="bg-surface rounded-lg border border-border p-6 space-y-4">
        <div className="flex items-center gap-3">
          <div className="p-2 bg-accent-600/20 rounded-lg">
            <Brain className="w-5 h-5 text-accent-400" />
          </div>
          <div>
            <h2 className="text-lg font-semibold text-foreground">NARS Reasoning</h2>
            <p className="text-sm text-muted-foreground">Symbolic reasoning configuration</p>
          </div>
        </div>

        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <div>
              <span className="text-sm font-medium text-foreground">Enable NARS Reasoning</span>
              <p className="text-xs text-muted-foreground mt-0.5">
                Use Non-Axiomatic Reasoning System for smarter queries
              </p>
            </div>
            <button
              onClick={() => settings.setNarsEnabled(!settings.narsEnabled)}
              className={cn(
                'relative w-12 h-6 rounded-full transition-colors',
                settings.narsEnabled ? 'bg-primary-600' : 'bg-surface-elevated border border-border'
              )}
            >
              <span
                className={cn(
                  'absolute top-1 w-4 h-4 rounded-full bg-white transition-transform',
                  settings.narsEnabled ? 'translate-x-7' : 'translate-x-1'
                )}
              />
            </button>
          </div>

          {settings.narsEnabled && (
            <div className="p-3 bg-primary-600/10 border border-primary-600/20 rounded-lg">
              <div className="flex items-start gap-2">
                <Brain className="w-4 h-4 text-primary-400 mt-0.5" />
                <div className="text-sm">
                  <p className="text-foreground font-medium">Neural proposes, symbolic disposes</p>
                  <p className="text-muted-foreground mt-1">
                    NARS provides evidential reasoning with truth-values (frequency, confidence)
                    to mitigate LLM hallucinations and improve retrieval precision.
                  </p>
                </div>
              </div>
            </div>
          )}
        </div>
      </section>

      {/* Default Preferences */}
      <section className="bg-surface rounded-lg border border-border p-6 space-y-4">
        <div className="flex items-center gap-3">
          <div className="p-2 bg-warning/20 rounded-lg">
            <Sliders className="w-5 h-5 text-warning" />
          </div>
          <div>
            <h2 className="text-lg font-semibold text-foreground">Default Preferences</h2>
            <p className="text-sm text-muted-foreground">Query and generation defaults</p>
          </div>
        </div>

        <div className="space-y-4">
          <div>
            <span className="text-sm font-medium text-foreground">Default Design System</span>
            <p className="text-xs text-muted-foreground mt-0.5 mb-2">
              Filter results by design system
            </p>
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-2">
              {DESIGN_SYSTEMS.map((ds) => (
                <button
                  key={ds.value}
                  onClick={() => settings.setDefaultDesignSystem(ds.value)}
                  className={cn(
                    'flex items-start gap-3 p-3 rounded-lg border text-left transition-colors',
                    settings.defaultDesignSystem === ds.value
                      ? 'border-primary-500 bg-primary-600/10'
                      : 'border-border hover:bg-surface-hover'
                  )}
                >
                  <div className="flex-1">
                    <span className="text-sm font-medium text-foreground">{ds.label}</span>
                    <p className="text-xs text-muted-foreground mt-0.5">{ds.description}</p>
                  </div>
                  {settings.defaultDesignSystem === ds.value && (
                    <Check className="w-4 h-4 text-primary-400 flex-shrink-0" />
                  )}
                </button>
              ))}
            </div>
          </div>

          <div>
            <span className="text-sm font-medium text-foreground">Default Query Limit</span>
            <p className="text-xs text-muted-foreground mt-0.5 mb-2">
              Maximum number of results to return
            </p>
            <div className="flex gap-2">
              {QUERY_LIMITS.map((limit) => (
                <button
                  key={limit}
                  onClick={() => settings.setDefaultQueryLimit(limit)}
                  className={cn(
                    'px-4 py-2 rounded-lg font-medium transition-colors',
                    settings.defaultQueryLimit === limit
                      ? 'bg-primary-600 text-white'
                      : 'bg-surface-elevated border border-border text-foreground hover:bg-surface-hover'
                  )}
                >
                  {limit}
                </button>
              ))}
            </div>
          </div>
        </div>
      </section>

      {/* Appearance */}
      <section className="bg-surface rounded-lg border border-border p-6 space-y-4">
        <div className="flex items-center gap-3">
          <div className="p-2 bg-success/20 rounded-lg">
            <Palette className="w-5 h-5 text-success" />
          </div>
          <div>
            <h2 className="text-lg font-semibold text-foreground">Appearance</h2>
            <p className="text-sm text-muted-foreground">Theme and display preferences</p>
          </div>
        </div>

        <div className="space-y-4">
          <div>
            <span className="text-sm font-medium text-foreground">Theme</span>
            <div className="flex gap-2 mt-2">
              {THEMES.map((t) => {
                const Icon = t.icon;
                return (
                  <button
                    key={t.value}
                    onClick={() => settings.setTheme(t.value)}
                    className={cn(
                      'flex items-center gap-2 px-4 py-2 rounded-lg font-medium transition-colors',
                      settings.theme === t.value
                        ? 'bg-primary-600 text-white'
                        : 'bg-surface-elevated border border-border text-foreground hover:bg-surface-hover'
                    )}
                  >
                    <Icon className="w-4 h-4" />
                    {t.label}
                  </button>
                );
              })}
            </div>
          </div>

          <div className="flex items-center justify-between">
            <div>
              <span className="text-sm font-medium text-foreground">Show Debug Info</span>
              <p className="text-xs text-muted-foreground mt-0.5">
                Display additional technical details in the UI
              </p>
            </div>
            <button
              onClick={() => settings.setShowDebugInfo(!settings.showDebugInfo)}
              className={cn(
                'relative w-12 h-6 rounded-full transition-colors',
                settings.showDebugInfo ? 'bg-primary-600' : 'bg-surface-elevated border border-border'
              )}
            >
              <span
                className={cn(
                  'absolute top-1 w-4 h-4 rounded-full bg-white transition-transform',
                  settings.showDebugInfo ? 'translate-x-7' : 'translate-x-1'
                )}
              />
            </button>
          </div>
        </div>
      </section>

      {/* Danger Zone */}
      <section className="bg-surface rounded-lg border border-error/30 p-6 space-y-4">
        <div className="flex items-center gap-3">
          <div className="p-2 bg-error/20 rounded-lg">
            <AlertCircle className="w-5 h-5 text-error" />
          </div>
          <div>
            <h2 className="text-lg font-semibold text-foreground">Data Management</h2>
            <p className="text-sm text-muted-foreground">Reset and clear options</p>
          </div>
        </div>

        <div className="flex flex-wrap gap-3">
          <button
            onClick={handleReset}
            className="flex items-center gap-2 px-4 py-2 rounded-lg border border-border text-foreground hover:bg-surface-hover transition-colors"
          >
            <RotateCcw className="w-4 h-4" />
            Reset to Defaults
          </button>
          <button
            onClick={handleClearStorage}
            className="flex items-center gap-2 px-4 py-2 rounded-lg bg-error/10 border border-error/30 text-error hover:bg-error/20 transition-colors"
          >
            <Database className="w-4 h-4" />
            Clear All Storage
          </button>
        </div>
      </section>
    </div>
  );
}
