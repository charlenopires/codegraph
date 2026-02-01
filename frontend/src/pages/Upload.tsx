import { useState, useCallback } from 'react';
import { useNavigate } from 'react-router-dom';
import {
  Play,
  Loader2,
  Eye,
  EyeOff,
  Settings,
  ChevronDown,
  ChevronRight,
  Network,
  Plus,
  Check,
} from 'lucide-react';
import { useExtractionStore } from '@/stores/extraction';
import { CodeEditor } from '@/components/code/CodeEditor';
import { NarsPanel } from '@/components/reasoning/NarsPanel';
import { TagInput } from '@/components/ui/TagInput';
import { ExtractionStepper } from '@/components/extraction/ExtractionStepper';
import { DESIGN_SYSTEMS } from '@/data/ontology';
import { cn } from '@/lib/utils';

export function UploadPage() {
  const navigate = useNavigate();

  // Code state
  const [html, setHtml] = useState('');
  const [css, setCss] = useState('');
  const [js, setJs] = useState('');
  const [name, setName] = useState('');

  // Advanced options state
  const [showAdvanced, setShowAdvanced] = useState(false);
  const [designSystem, setDesignSystem] = useState('');
  const [tags, setTags] = useState<string[]>([]);

  // Preview state
  const [showPreview, setShowPreview] = useState(false);

  const { isExtracting, progress, result, error, extract, reset } = useExtractionStore();

  const handleSubmit = useCallback(async () => {
    if (!html.trim()) return;

    try {
      await extract({
        html,
        css: css || undefined,
        js: js || undefined,
        name: name || undefined,
        tags: tags.length > 0 ? tags : undefined,
        design_system: designSystem || undefined,
      });
    } catch {
      // Error is handled in store
    }
  }, [html, css, js, name, tags, designSystem, extract]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.ctrlKey && e.key === 'Enter') {
        handleSubmit();
      }
    },
    [handleSubmit]
  );

  const handleReset = useCallback(() => {
    reset();
    setHtml('');
    setCss('');
    setJs('');
    setName('');
    setTags([]);
    setDesignSystem('');
  }, [reset]);

  return (
    <div className="space-y-6" onKeyDown={handleKeyDown}>
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-foreground">Upload Code Snippet</h1>
          <p className="text-muted-foreground mt-1">
            Extract UI elements from your code into the knowledge graph
          </p>
        </div>
        <button
          onClick={handleSubmit}
          disabled={isExtracting || !html.trim()}
          className={cn(
            'flex items-center gap-2 px-4 py-2 rounded-lg font-medium transition-colors',
            'bg-primary-600 text-white hover:bg-primary-700',
            'disabled:opacity-50 disabled:cursor-not-allowed'
          )}
        >
          {isExtracting ? (
            <>
              <Loader2 className="w-4 h-4 animate-spin" />
              Extracting...
            </>
          ) : (
            <>
              <Play className="w-4 h-4" />
              Extract
            </>
          )}
        </button>
      </div>

      {/* Metadata */}
      <div className="bg-surface rounded-lg border border-border overflow-hidden">
        <div className="p-4">
          <label className="block text-sm font-medium text-foreground mb-2">
            Snippet Name <span className="text-muted-foreground">(optional)</span>
          </label>
          <input
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="e.g., Primary Button, User Card..."
            className="w-full px-3 py-2 bg-background border border-border rounded-lg text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-primary-500"
          />
        </div>

        {/* Advanced Options Toggle */}
        <button
          onClick={() => setShowAdvanced(!showAdvanced)}
          className="w-full flex items-center justify-between px-4 py-3 border-t border-border hover:bg-surface-hover transition-colors"
        >
          <div className="flex items-center gap-2">
            <Settings className="w-4 h-4 text-muted-foreground" />
            <span className="text-sm font-medium text-foreground">Advanced Options</span>
          </div>
          {showAdvanced ? (
            <ChevronDown className="w-4 h-4 text-muted-foreground" />
          ) : (
            <ChevronRight className="w-4 h-4 text-muted-foreground" />
          )}
        </button>

        {/* Advanced Options Content */}
        {showAdvanced && (
          <div className="px-4 pb-4 space-y-4 border-t border-border pt-4">
            {/* Design System Dropdown */}
            <div>
              <label className="block text-sm font-medium text-foreground mb-2">
                Design System <span className="text-muted-foreground">(optional)</span>
              </label>
              <select
                value={designSystem}
                onChange={(e) => setDesignSystem(e.target.value)}
                className="w-full px-3 py-2 bg-background border border-border rounded-lg text-foreground focus:outline-none focus:ring-2 focus:ring-primary-500"
              >
                <option value="">Auto-detect</option>
                {DESIGN_SYSTEMS.map((ds) => (
                  <option key={ds.id} value={ds.id}>
                    {ds.name}
                  </option>
                ))}
              </select>
              <p className="text-xs text-muted-foreground mt-1">
                Leave empty to automatically detect the design system from your code
              </p>
            </div>

            {/* Tags Input */}
            <div>
              <label className="block text-sm font-medium text-foreground mb-2">
                Tags <span className="text-muted-foreground">(optional)</span>
              </label>
              <TagInput
                tags={tags}
                onTagsChange={setTags}
                placeholder="button, primary, cta..."
              />
              <p className="text-xs text-muted-foreground mt-1">
                Press Enter or comma to add tags. Tags help categorize your snippets.
              </p>
            </div>
          </div>
        )}
      </div>

      {/* Code editors */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-4">
        {/* HTML Editor */}
        <div className="bg-surface rounded-lg border border-border overflow-hidden flex flex-col">
          <div className="px-4 py-2 bg-surface-elevated border-b border-border flex items-center gap-2 shrink-0">
            <span className="text-sm font-medium text-foreground">HTML</span>
            <span className="text-xs text-error">*required</span>
          </div>
          <div className="h-64">
            <CodeEditor
              value={html}
              onChange={setHtml}
              language="html"
              placeholder="<button class='btn primary'>Click me</button>"
            />
          </div>
        </div>

        {/* CSS Editor */}
        <div className="bg-surface rounded-lg border border-border overflow-hidden flex flex-col">
          <div className="px-4 py-2 bg-surface-elevated border-b border-border shrink-0">
            <span className="text-sm font-medium text-foreground">CSS</span>
            <span className="text-xs text-muted-foreground ml-2">(optional)</span>
          </div>
          <div className="h-64">
            <CodeEditor
              value={css}
              onChange={setCss}
              language="css"
              placeholder=".btn { padding: 8px 16px; }"
            />
          </div>
        </div>

        {/* JS Editor */}
        <div className="bg-surface rounded-lg border border-border overflow-hidden flex flex-col">
          <div className="px-4 py-2 bg-surface-elevated border-b border-border shrink-0">
            <span className="text-sm font-medium text-foreground">JavaScript</span>
            <span className="text-xs text-muted-foreground ml-2">(optional)</span>
          </div>
          <div className="h-64">
            <CodeEditor
              value={js}
              onChange={setJs}
              language="javascript"
              placeholder="button.addEventListener('click', ...);"
            />
          </div>
        </div>
      </div>

      {/* Live Preview */}
      <div className="bg-surface rounded-lg border border-border overflow-hidden">
        <button
          onClick={() => setShowPreview(!showPreview)}
          className="w-full flex items-center justify-between p-4 hover:bg-surface-hover transition-colors"
        >
          <div className="flex items-center gap-2">
            {showPreview ? (
              <EyeOff className="w-4 h-4 text-muted-foreground" />
            ) : (
              <Eye className="w-4 h-4 text-muted-foreground" />
            )}
            <span className="text-sm font-medium text-foreground">Live Preview</span>
          </div>
          {showPreview ? (
            <ChevronDown className="w-4 h-4 text-muted-foreground" />
          ) : (
            <ChevronRight className="w-4 h-4 text-muted-foreground" />
          )}
        </button>

        {showPreview && (
          <div className="p-4 border-t border-border">
            {html.trim() ? (
              <iframe
                srcDoc={`
                  <!DOCTYPE html>
                  <html>
                    <head>
                      <style>
                        body { margin: 0; padding: 16px; font-family: system-ui, sans-serif; }
                        ${css}
                      </style>
                    </head>
                    <body>
                      ${html}
                      <script>${js}</script>
                    </body>
                  </html>
                `}
                className="w-full h-64 bg-white rounded border border-border"
                sandbox="allow-scripts"
                title="Code Preview"
              />
            ) : (
              <div className="w-full h-64 bg-background rounded border border-border flex items-center justify-center">
                <p className="text-muted-foreground text-sm">Enter HTML code to see preview</p>
              </div>
            )}
          </div>
        )}
      </div>

      {/* Progress */}
      {isExtracting && progress && (
        <ExtractionStepper
          currentPhase={progress.phase}
          progress={progress.progress}
          message={progress.message}
        />
      )}

      {/* Error */}
      {error && (
        <div className="bg-error/10 border border-error/30 rounded-lg p-4">
          <p className="text-error font-medium">Extraction Failed</p>
          <p className="text-error/80 text-sm mt-1">{error}</p>
          <button
            onClick={reset}
            className="mt-2 text-sm text-error underline hover:no-underline"
          >
            Try again
          </button>
        </div>
      )}

      {/* Result */}
      {result && (
        <div className="space-y-4">
          {/* Success Header */}
          <div className="bg-success/10 border border-success/30 rounded-lg p-4">
            <div className="flex items-center gap-3">
              <div className="p-2 bg-success/20 rounded-lg">
                <Check className="w-6 h-6 text-success" />
              </div>
              <div>
                <h3 className="font-semibold text-foreground">Extraction Complete</h3>
                <p className="text-sm text-muted-foreground">
                  {result.element_ids.length} element{result.element_ids.length !== 1 ? 's' : ''}{' '}
                  extracted in {result.processing_time_ms}ms
                </p>
              </div>
            </div>
          </div>

          {/* Summary Stats */}
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            <div className="bg-surface rounded-lg border border-border p-4">
              <span className="text-xs text-muted-foreground">Snippet ID</span>
              <p className="font-mono text-sm text-foreground truncate mt-1">{result.snippet_id}</p>
            </div>
            <div className="bg-surface rounded-lg border border-border p-4">
              <span className="text-xs text-muted-foreground">Elements</span>
              <p className="font-mono text-sm text-foreground mt-1">{result.element_ids.length}</p>
            </div>
            <div className="bg-surface rounded-lg border border-border p-4">
              <span className="text-xs text-muted-foreground">Design System</span>
              <p className="font-mono text-sm text-foreground mt-1 capitalize">
                {result.design_system || 'Custom'}
              </p>
            </div>
            <div className="bg-surface rounded-lg border border-border p-4">
              <span className="text-xs text-muted-foreground">Processing Time</span>
              <p className="font-mono text-sm text-foreground mt-1">{result.processing_time_ms}ms</p>
            </div>
          </div>

          {/* Narsese Statements with NarsPanel */}
          {result.narsese_statements.length > 0 && (
            <NarsPanel
              statements={result.narsese_statements.map((stmt) => ({
                statement: stmt,
                type: 'derived' as const,
              }))}
              title="Extracted Narsese Statements"
              showExplanation={true}
            />
          )}

          {/* Action Buttons */}
          <div className="flex gap-3">
            <button
              onClick={() => navigate(`/graph?highlight=${result.snippet_id}`)}
              className="flex items-center gap-2 px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors"
            >
              <Network className="w-4 h-4" />
              View in Graph
            </button>
            <button
              onClick={handleReset}
              className="flex items-center gap-2 px-4 py-2 border border-border text-foreground rounded-lg hover:bg-surface-hover transition-colors"
            >
              <Plus className="w-4 h-4" />
              Upload Another
            </button>
          </div>
        </div>
      )}

      <p className="text-xs text-muted-foreground">
        Press{' '}
        <kbd className="px-1.5 py-0.5 bg-surface border border-border rounded text-xs">
          Ctrl+Enter
        </kbd>{' '}
        to submit
      </p>
    </div>
  );
}
