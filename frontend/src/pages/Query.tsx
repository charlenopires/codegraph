import { useState } from 'react';
import { Search, Sparkles, Loader2, ThumbsUp, ThumbsDown, Copy, Check, Brain } from 'lucide-react';
import { useGenerationStore } from '@/stores/generation';
import { toast } from '@/stores/toast';
import { NarsPanel } from '@/components/reasoning/NarsPanel';
import { cn } from '@/lib/utils';

type Tab = 'html' | 'css' | 'js' | 'preview';

export function QueryPage() {
  const [query, setQuery] = useState('');
  const [limit, setLimit] = useState(5);
  const [includeReasoning, setIncludeReasoning] = useState(false);
  const [activeTab, setActiveTab] = useState<Tab>('html');
  const [copiedCode, setCopiedCode] = useState<string | null>(null);

  const {
    isQuerying,
    queryResult,
    isGenerating,
    generationResult,
    submittingFeedback,
    error,
    query: executeQuery,
    generate,
    submitFeedback,
    reset,
  } = useGenerationStore();

  // Track which elements have received feedback this session
  const [feedbackGiven, setFeedbackGiven] = useState<Map<string, 'thumbs_up' | 'thumbs_down'>>(
    new Map()
  );

  const handleFeedback = async (
    elementId: string,
    feedbackType: 'thumbs_up' | 'thumbs_down'
  ) => {
    try {
      const result = await submitFeedback(elementId, feedbackType, query);
      setFeedbackGiven((prev) => new Map(prev).set(elementId, feedbackType));
      toast.success(
        feedbackType === 'thumbs_up' ? 'Thanks for the feedback!' : 'Feedback recorded',
        `Confidence updated to ${(result.new_confidence * 100).toFixed(1)}%`
      );
    } catch {
      toast.error('Feedback failed', 'Please try again');
    }
  };

  const handleSearch = async () => {
    if (!query.trim()) return;
    try {
      await executeQuery({
        query,
        limit,
        include_reasoning: includeReasoning,
      });
    } catch {
      // Error handled in store
    }
  };

  const handleGenerate = async () => {
    if (!query.trim()) return;
    try {
      await generate({
        query,
        use_references: true,
        include_css: true,
        include_js: true,
      });
    } catch {
      // Error handled in store
    }
  };

  const copyCode = async (code: string, type: string) => {
    await navigator.clipboard.writeText(code);
    setCopiedCode(type);
    setTimeout(() => setCopiedCode(null), 2000);
  };

  const tabs: { id: Tab; label: string }[] = [
    { id: 'html', label: 'HTML' },
    { id: 'css', label: 'CSS' },
    { id: 'js', label: 'JavaScript' },
    { id: 'preview', label: 'Preview' },
  ];

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-foreground">Query & Generate</h1>
        <p className="text-muted-foreground mt-1">
          Search components and generate UI code using natural language
        </p>
      </div>

      {/* Search form */}
      <div className="bg-surface rounded-lg p-4 border border-border space-y-4">
        <div className="flex gap-4">
          <div className="flex-1">
            <input
              type="text"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder="Describe the UI component you need..."
              onKeyDown={(e) => e.key === 'Enter' && handleSearch()}
              className="w-full px-4 py-3 bg-background border border-border rounded-lg text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-primary-500"
            />
          </div>
          <button
            onClick={handleSearch}
            disabled={isQuerying || !query.trim()}
            className={cn(
              'flex items-center gap-2 px-4 py-2 rounded-lg font-medium transition-colors',
              'bg-primary-600 text-white hover:bg-primary-700',
              'disabled:opacity-50 disabled:cursor-not-allowed'
            )}
          >
            {isQuerying ? (
              <Loader2 className="w-4 h-4 animate-spin" />
            ) : (
              <Search className="w-4 h-4" />
            )}
            Search
          </button>
          <button
            onClick={handleGenerate}
            disabled={isGenerating || !query.trim()}
            className={cn(
              'flex items-center gap-2 px-4 py-2 rounded-lg font-medium transition-colors',
              'bg-accent-600 text-white hover:bg-accent-700',
              'disabled:opacity-50 disabled:cursor-not-allowed'
            )}
          >
            {isGenerating ? (
              <Loader2 className="w-4 h-4 animate-spin" />
            ) : (
              <Sparkles className="w-4 h-4" />
            )}
            Generate
          </button>
        </div>

        <div className="flex items-center gap-6 text-sm">
          <label className="flex items-center gap-2">
            <span className="text-muted-foreground">Limit:</span>
            <select
              value={limit}
              onChange={(e) => setLimit(Number(e.target.value))}
              className="px-2 py-1 bg-background border border-border rounded text-foreground"
            >
              <option value={3}>3</option>
              <option value={5}>5</option>
              <option value={10}>10</option>
            </select>
          </label>
          <label className="flex items-center gap-2 cursor-pointer">
            <input
              type="checkbox"
              checked={includeReasoning}
              onChange={(e) => setIncludeReasoning(e.target.checked)}
              className="w-4 h-4 rounded border-border"
            />
            <span className="text-muted-foreground">Include NARS reasoning</span>
          </label>
        </div>
      </div>

      {/* Error */}
      {error && (
        <div className="bg-error/10 border border-error/30 rounded-lg p-4">
          <p className="text-error">{error}</p>
          <button onClick={reset} className="text-sm text-error underline mt-2">
            Reset
          </button>
        </div>
      )}

      {/* Query results */}
      {queryResult && (
        <div className="space-y-4">
          <h2 className="text-lg font-semibold text-foreground">
            Results ({queryResult.elements.length})
            <span className="text-sm font-normal text-muted-foreground ml-2">
              {queryResult.processing_time_ms}ms
            </span>
          </h2>
          <div className="grid gap-3">
            {queryResult.elements.map((element) => (
              <div
                key={element.id}
                className="bg-surface rounded-lg p-4 border border-border hover:border-border-hover transition-colors"
              >
                <div className="flex items-start justify-between">
                  <div>
                    <h3 className="font-medium text-foreground">{element.name}</h3>
                    <p className="text-sm text-muted-foreground">
                      {element.category} • {element.design_system || 'Custom'}
                    </p>
                  </div>
                  <div className="flex items-center gap-2">
                    <div className="text-right">
                      <span className="text-sm font-medium text-primary-400">
                        {(element.score * 100).toFixed(0)}%
                      </span>
                    </div>
                    <button
                      onClick={() => handleFeedback(element.id, 'thumbs_up')}
                      disabled={
                        submittingFeedback.has(element.id) ||
                        feedbackGiven.has(element.id)
                      }
                      className={cn(
                        'p-1.5 rounded transition-colors',
                        feedbackGiven.get(element.id) === 'thumbs_up'
                          ? 'bg-success/20 text-success'
                          : 'hover:bg-surface-hover text-muted-foreground hover:text-success',
                        'disabled:opacity-50 disabled:cursor-not-allowed'
                      )}
                    >
                      {submittingFeedback.has(element.id) ? (
                        <Loader2 className="w-4 h-4 animate-spin" />
                      ) : (
                        <ThumbsUp className="w-4 h-4" />
                      )}
                    </button>
                    <button
                      onClick={() => handleFeedback(element.id, 'thumbs_down')}
                      disabled={
                        submittingFeedback.has(element.id) ||
                        feedbackGiven.has(element.id)
                      }
                      className={cn(
                        'p-1.5 rounded transition-colors',
                        feedbackGiven.get(element.id) === 'thumbs_down'
                          ? 'bg-error/20 text-error'
                          : 'hover:bg-surface-hover text-muted-foreground hover:text-error',
                        'disabled:opacity-50 disabled:cursor-not-allowed'
                      )}
                    >
                      <ThumbsDown className="w-4 h-4" />
                    </button>
                  </div>
                </div>
                <p className="text-xs text-muted-foreground mt-2">{element.match_reason}</p>
              </div>
            ))}
          </div>

          {includeReasoning && queryResult.narsese_queries && queryResult.narsese_queries.length > 0 && (
            <NarsPanel
              statements={queryResult.narsese_queries.map((stmt) => ({
                statement: stmt,
                type: 'input' as const,
              }))}
              title="NARS Query Translation"
              showExplanation={true}
            />
          )}

          {includeReasoning && queryResult.reasoning_explanation && (
            <div className="bg-surface rounded-lg border border-border p-4">
              <div className="flex items-center gap-2 mb-3">
                <Brain className="w-5 h-5 text-accent-400" />
                <h3 className="font-medium text-foreground">Reasoning Explanation</h3>
              </div>
              <p className="text-sm text-muted-foreground whitespace-pre-wrap">
                {queryResult.reasoning_explanation}
              </p>
            </div>
          )}
        </div>
      )}

      {/* Generation result */}
      {generationResult && (
        <div className="space-y-4">
          <h2 className="text-lg font-semibold text-foreground">
            Generated Code
            <span className="text-sm font-normal text-muted-foreground ml-2">
              {generationResult.generation_time_ms}ms
            </span>
          </h2>

          {/* Reference elements used */}
          {generationResult.reference_elements && generationResult.reference_elements.length > 0 && (
            <div className="bg-surface rounded-lg border border-border p-4">
              <h3 className="text-sm font-medium text-foreground mb-3">
                Reference Elements Used ({generationResult.reference_elements.length})
              </h3>
              <div className="flex flex-wrap gap-2">
                {generationResult.reference_elements.map((ref) => (
                  <div
                    key={ref.id}
                    className="inline-flex items-center gap-2 px-3 py-1.5 bg-surface-elevated rounded-lg text-sm"
                  >
                    <span className="text-foreground">{ref.name}</span>
                    <span className="text-muted-foreground text-xs">{ref.category}</span>
                    <span className="text-primary-400 text-xs">
                      {(ref.score * 100).toFixed(0)}%
                    </span>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* NARS reasoning used for generation */}
          {generationResult.narsese_reasoning && generationResult.narsese_reasoning.length > 0 && (
            <NarsPanel
              statements={generationResult.narsese_reasoning.map((stmt) => ({
                statement: stmt,
                type: 'derived' as const,
              }))}
              title="NARS Generation Context"
              showExplanation={false}
            />
          )}

          {/* Tabs */}
          <div className="bg-surface rounded-lg border border-border overflow-hidden">
            <div className="flex border-b border-border">
              {tabs.map((tab) => (
                <button
                  key={tab.id}
                  onClick={() => setActiveTab(tab.id)}
                  className={cn(
                    'px-4 py-2 text-sm font-medium transition-colors',
                    activeTab === tab.id
                      ? 'text-foreground bg-surface-elevated border-b-2 border-primary-500'
                      : 'text-muted-foreground hover:text-foreground'
                  )}
                >
                  {tab.label}
                </button>
              ))}
            </div>

            <div className="relative">
              {activeTab !== 'preview' && (
                <button
                  onClick={() => {
                    const code =
                      activeTab === 'html'
                        ? generationResult.html
                        : activeTab === 'css'
                          ? generationResult.css || ''
                          : generationResult.javascript || '';
                    copyCode(code, activeTab);
                  }}
                  className="absolute top-2 right-2 p-2 bg-surface-elevated hover:bg-surface-hover rounded-lg transition-colors"
                >
                  {copiedCode === activeTab ? (
                    <Check className="w-4 h-4 text-success" />
                  ) : (
                    <Copy className="w-4 h-4 text-muted-foreground" />
                  )}
                </button>
              )}

              {activeTab === 'html' && (
                <pre className="p-4 text-sm font-mono text-foreground overflow-x-auto">
                  {generationResult.html}
                </pre>
              )}
              {activeTab === 'css' && (
                <pre className="p-4 text-sm font-mono text-foreground overflow-x-auto">
                  {generationResult.css || '/* No CSS generated */'}
                </pre>
              )}
              {activeTab === 'js' && (
                <pre className="p-4 text-sm font-mono text-foreground overflow-x-auto">
                  {generationResult.javascript || '// No JavaScript generated'}
                </pre>
              )}
              {activeTab === 'preview' && (
                <div className="p-4">
                  <iframe
                    srcDoc={`
                      <!DOCTYPE html>
                      <html>
                        <head>
                          <style>${generationResult.css || ''}</style>
                        </head>
                        <body>
                          ${generationResult.html}
                          <script>${generationResult.javascript || ''}</script>
                        </body>
                      </html>
                    `}
                    className="w-full h-64 bg-white rounded border border-border"
                    sandbox="allow-scripts"
                  />
                </div>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
