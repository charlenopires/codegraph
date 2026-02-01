import { useEffect, useState, useCallback } from 'react';
import { Network, Loader2, Filter, RefreshCw } from 'lucide-react';
import { useMetricsStore } from '@/stores/metrics';
import { getWebSocket } from '@/api/websocket';
import { ForceGraph } from '@/components/graph/ForceGraph';
import { ElementDetail } from '@/components/graph/ElementDetail';
import type { GraphElement, GraphElementsRequest, GraphElementsResult } from '@/api/types';
import { cn } from '@/lib/utils';

export function GraphPage() {
  const { graphStats, fetchGraphStats } = useMetricsStore();
  const [elements, setElements] = useState<GraphElement[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [filters, setFilters] = useState<{ category?: string; design_system?: string }>({});
  const [showFilters, setShowFilters] = useState(false);
  const [selectedElement, setSelectedElement] = useState<GraphElement | null>(null);

  const fetchElements = useCallback(async () => {
    setIsLoading(true);
    setError(null);

    try {
      const ws = getWebSocket();
      const response = await ws.request<GraphElementsRequest, GraphElementsResult>('graph_elements', {
        page: 1,
        per_page: 100,
        category: filters.category,
        design_system: filters.design_system,
      });
      setElements((response.payload as GraphElementsResult).elements);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to fetch elements');
    } finally {
      setIsLoading(false);
    }
  }, [filters]);

  useEffect(() => {
    fetchGraphStats();
    fetchElements();
  }, [fetchGraphStats, fetchElements]);

  const handleRefresh = () => {
    fetchGraphStats();
    fetchElements();
  };

  const categories = ['button', 'input', 'card', 'modal', 'navigation', 'layout', 'text'];
  const designSystems = ['material-ui', 'tailwind', 'chakra', 'bootstrap', 'ant-design', 'shadcn', 'custom'];

  return (
    <div className="space-y-6 h-full flex flex-col">
      <div className="flex items-center justify-between shrink-0">
        <div>
          <h1 className="text-2xl font-bold text-foreground">Knowledge Graph</h1>
          <p className="text-muted-foreground mt-1">
            Explore the UI component knowledge graph
          </p>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={() => setShowFilters(!showFilters)}
            className={cn(
              'flex items-center gap-2 px-3 py-2 rounded-lg font-medium transition-colors',
              'border border-border hover:bg-surface-hover',
              showFilters && 'bg-surface-elevated'
            )}
          >
            <Filter className="w-4 h-4" />
            Filters
          </button>
          <button
            onClick={handleRefresh}
            disabled={isLoading}
            className={cn(
              'flex items-center gap-2 px-3 py-2 rounded-lg font-medium transition-colors',
              'bg-primary-600 text-white hover:bg-primary-700',
              'disabled:opacity-50'
            )}
          >
            <RefreshCw className={cn('w-4 h-4', isLoading && 'animate-spin')} />
            Refresh
          </button>
        </div>
      </div>

      {/* Filters panel */}
      {showFilters && (
        <div className="bg-surface rounded-lg border border-border p-4 shrink-0">
          <div className="flex flex-wrap gap-4">
            <div>
              <label className="block text-sm font-medium text-foreground mb-2">Category</label>
              <select
                value={filters.category || ''}
                onChange={(e) => setFilters({ ...filters, category: e.target.value || undefined })}
                className="px-3 py-2 bg-background border border-border rounded-lg text-foreground"
              >
                <option value="">All categories</option>
                {categories.map((cat) => (
                  <option key={cat} value={cat}>{cat}</option>
                ))}
              </select>
            </div>
            <div>
              <label className="block text-sm font-medium text-foreground mb-2">Design System</label>
              <select
                value={filters.design_system || ''}
                onChange={(e) => setFilters({ ...filters, design_system: e.target.value || undefined })}
                className="px-3 py-2 bg-background border border-border rounded-lg text-foreground"
              >
                <option value="">All design systems</option>
                {designSystems.map((ds) => (
                  <option key={ds} value={ds}>{ds}</option>
                ))}
              </select>
            </div>
            <div className="flex items-end">
              <button
                onClick={() => setFilters({})}
                className="px-3 py-2 text-sm text-muted-foreground hover:text-foreground"
              >
                Clear filters
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Stats cards */}
      {graphStats && (
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4 shrink-0">
          <div className="bg-surface rounded-lg p-4 border border-border">
            <p className="text-sm text-muted-foreground">Total Nodes</p>
            <p className="text-3xl font-bold text-foreground mt-1">
              {graphStats.total_nodes.toLocaleString()}
            </p>
          </div>
          <div className="bg-surface rounded-lg p-4 border border-border">
            <p className="text-sm text-muted-foreground">Total Relationships</p>
            <p className="text-3xl font-bold text-foreground mt-1">
              {graphStats.total_relationships.toLocaleString()}
            </p>
          </div>
          <div className="bg-surface rounded-lg p-4 border border-border">
            <p className="text-sm text-muted-foreground">Average Degree</p>
            <p className="text-3xl font-bold text-foreground mt-1">
              {graphStats.avg_degree.toFixed(2)}
            </p>
          </div>
        </div>
      )}

      {/* Error */}
      {error && (
        <div className="bg-error/10 border border-error/30 rounded-lg p-4 shrink-0">
          <p className="text-error">{error}</p>
        </div>
      )}

      {/* Graph visualization */}
      <div className="bg-surface rounded-lg border border-border flex-1 min-h-[500px] overflow-hidden relative">
        {isLoading ? (
          <div className="absolute inset-0 flex items-center justify-center">
            <Loader2 className="w-8 h-8 animate-spin text-muted-foreground" />
          </div>
        ) : elements.length > 0 ? (
          <ForceGraph
            nodes={elements}
            width={1200}
            height={600}
            onNodeClick={setSelectedElement}
          />
        ) : (
          <div className="absolute inset-0 flex items-center justify-center">
            <div className="text-center text-muted-foreground">
              <Network className="w-16 h-16 mx-auto mb-4 opacity-50" />
              <p className="text-lg font-medium">No Elements Found</p>
              <p className="text-sm mt-2">
                Upload some code snippets to populate the graph
              </p>
            </div>
          </div>
        )}
      </div>

      {/* Node and relationship stats */}
      {graphStats && (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6 shrink-0">
          <div className="bg-surface rounded-lg border border-border p-4">
            <h3 className="font-medium text-foreground mb-3">Nodes by Label</h3>
            <div className="space-y-2 max-h-48 overflow-y-auto">
              {graphStats.nodes_by_label.length > 0 ? (
                graphStats.nodes_by_label.map((item) => (
                  <div key={item.label} className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">{item.label}</span>
                    <span className="text-sm font-mono text-foreground">{item.count}</span>
                  </div>
                ))
              ) : (
                <p className="text-sm text-muted-foreground">No data available</p>
              )}
            </div>
          </div>
          <div className="bg-surface rounded-lg border border-border p-4">
            <h3 className="font-medium text-foreground mb-3">Relationships by Type</h3>
            <div className="space-y-2 max-h-48 overflow-y-auto">
              {graphStats.relationships_by_type.length > 0 ? (
                graphStats.relationships_by_type.map((item) => (
                  <div key={item.rel_type} className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">{item.rel_type}</span>
                    <span className="text-sm font-mono text-foreground">{item.count}</span>
                  </div>
                ))
              ) : (
                <p className="text-sm text-muted-foreground">No data available</p>
              )}
            </div>
          </div>
        </div>
      )}

      {/* Element Detail Modal */}
      {selectedElement && (
        <ElementDetail
          element={selectedElement}
          onClose={() => setSelectedElement(null)}
        />
      )}
    </div>
  );
}
