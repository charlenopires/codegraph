import { useEffect, useMemo } from 'react';
import { BarChart3, ThumbsUp, ThumbsDown, Clock, Zap, RefreshCw } from 'lucide-react';
import {
  PieChart,
  Pie,
  Cell,
  BarChart,
  Bar,
  XAxis,
  YAxis,
  Tooltip,
  ResponsiveContainer,
  Legend,
} from 'recharts';
import { useMetricsStore } from '@/stores/metrics';
import { cn } from '@/lib/utils';

const CATEGORY_COLORS = [
  '#f59e0b', '#10b981', '#3b82f6', '#8b5cf6', '#ec4899', '#06b6d4', '#6b7280',
];

const DESIGN_SYSTEM_COLORS: Record<string, string> = {
  'material-ui': '#2196f3',
  'tailwind': '#38bdf8',
  'chakra': '#319795',
  'bootstrap': '#7952b3',
  'ant-design': '#1890ff',
  'shadcn': '#ffffff',
  'custom': '#a855f7',
  'unknown': '#6b7280',
};

export function MetricsPage() {
  const { isSubscribed, metrics, subscribe, unsubscribe, fetchGraphStats } = useMetricsStore();

  useEffect(() => {
    subscribe();
    fetchGraphStats();

    return () => unsubscribe();
  }, [subscribe, unsubscribe, fetchGraphStats]);

  const feedbackRatio = metrics
    ? metrics.positive_feedback / Math.max(metrics.positive_feedback + metrics.negative_feedback, 1)
    : 0;

  const categoryData = useMemo(() => {
    if (!metrics) return [];
    return metrics.elements_by_category.map((item, index) => ({
      name: item.category,
      value: item.count,
      color: CATEGORY_COLORS[index % CATEGORY_COLORS.length],
    }));
  }, [metrics]);

  const designSystemData = useMemo(() => {
    if (!metrics) return [];
    return metrics.elements_by_design_system.map((item) => ({
      name: item.design_system,
      value: item.count,
      color: DESIGN_SYSTEM_COLORS[item.design_system] || DESIGN_SYSTEM_COLORS['unknown'],
    }));
  }, [metrics]);

  const latencyData = useMemo(() => {
    if (!metrics) return [];
    return [
      { name: 'Query', latency: metrics.avg_query_latency_ms, fill: '#3b82f6' },
      { name: 'Generation', latency: metrics.avg_generation_latency_ms, fill: '#8b5cf6' },
    ];
  }, [metrics]);

  const feedbackData = useMemo(() => {
    if (!metrics) return [];
    return [
      { name: 'Positive', value: metrics.positive_feedback, color: '#10b981' },
      { name: 'Negative', value: metrics.negative_feedback, color: '#ef4444' },
    ];
  }, [metrics]);

  const handleRefresh = () => {
    fetchGraphStats();
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-foreground">Metrics Dashboard</h1>
          <p className="text-muted-foreground mt-1">
            Real-time system metrics and analytics
          </p>
        </div>
        <div className="flex items-center gap-3">
          <button
            onClick={handleRefresh}
            className="flex items-center gap-2 px-3 py-2 rounded-lg border border-border hover:bg-surface-hover transition-colors"
          >
            <RefreshCw className="w-4 h-4" />
            Refresh
          </button>
          <div
            className={cn(
              'flex items-center gap-2 px-3 py-1.5 rounded-full text-sm',
              isSubscribed ? 'bg-success/10 text-success' : 'bg-muted/10 text-muted-foreground'
            )}
          >
            <span className={cn('w-2 h-2 rounded-full', isSubscribed ? 'bg-success animate-pulse' : 'bg-muted')} />
            {isSubscribed ? 'Live' : 'Not connected'}
          </div>
        </div>
      </div>

      {metrics ? (
        <>
          {/* Main stats */}
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
            <div className="bg-surface rounded-lg p-4 border border-border">
              <div className="flex items-center gap-3">
                <div className="p-2 bg-primary-600/20 rounded-lg">
                  <BarChart3 className="w-5 h-5 text-primary-400" />
                </div>
                <div>
                  <p className="text-sm text-muted-foreground">Total Elements</p>
                  <p className="text-2xl font-bold text-foreground">
                    {metrics.total_elements.toLocaleString()}
                  </p>
                </div>
              </div>
            </div>

            <div className="bg-surface rounded-lg p-4 border border-border">
              <div className="flex items-center gap-3">
                <div className="p-2 bg-accent-600/20 rounded-lg">
                  <Zap className="w-5 h-5 text-accent-400" />
                </div>
                <div>
                  <p className="text-sm text-muted-foreground">Total Queries</p>
                  <p className="text-2xl font-bold text-foreground">
                    {metrics.total_queries.toLocaleString()}
                  </p>
                </div>
              </div>
            </div>

            <div className="bg-surface rounded-lg p-4 border border-border">
              <div className="flex items-center gap-3">
                <div className="p-2 bg-success/20 rounded-lg">
                  <ThumbsUp className="w-5 h-5 text-success" />
                </div>
                <div>
                  <p className="text-sm text-muted-foreground">Feedback Ratio</p>
                  <p className="text-2xl font-bold text-foreground">
                    {(feedbackRatio * 100).toFixed(1)}%
                  </p>
                </div>
              </div>
            </div>

            <div className="bg-surface rounded-lg p-4 border border-border">
              <div className="flex items-center gap-3">
                <div className="p-2 bg-warning/20 rounded-lg">
                  <Clock className="w-5 h-5 text-warning" />
                </div>
                <div>
                  <p className="text-sm text-muted-foreground">Avg Query Latency</p>
                  <p className="text-2xl font-bold text-foreground">
                    {metrics.avg_query_latency_ms.toFixed(0)}ms
                  </p>
                </div>
              </div>
            </div>
          </div>

          {/* Charts row */}
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            {/* Latency chart */}
            <div className="bg-surface rounded-lg border border-border p-4">
              <h3 className="font-medium text-foreground mb-4">Average Latency (ms)</h3>
              <div className="h-64">
                <ResponsiveContainer width="100%" height="100%">
                  <BarChart data={latencyData}>
                    <XAxis dataKey="name" stroke="var(--color-muted-foreground)" fontSize={12} />
                    <YAxis stroke="var(--color-muted-foreground)" fontSize={12} />
                    <Tooltip
                      contentStyle={{
                        backgroundColor: 'var(--color-surface-elevated)',
                        border: '1px solid var(--color-border)',
                        borderRadius: '8px',
                      }}
                      labelStyle={{ color: 'var(--color-foreground)' }}
                    />
                    <Bar dataKey="latency" radius={[4, 4, 0, 0]}>
                      {latencyData.map((entry, index) => (
                        <Cell key={`cell-${index}`} fill={entry.fill} />
                      ))}
                    </Bar>
                  </BarChart>
                </ResponsiveContainer>
              </div>
            </div>

            {/* Feedback pie chart */}
            <div className="bg-surface rounded-lg border border-border p-4">
              <h3 className="font-medium text-foreground mb-4">Feedback Distribution</h3>
              <div className="h-64">
                <ResponsiveContainer width="100%" height="100%">
                  <PieChart>
                    <Pie
                      data={feedbackData}
                      cx="50%"
                      cy="50%"
                      innerRadius={60}
                      outerRadius={80}
                      paddingAngle={5}
                      dataKey="value"
                    >
                      {feedbackData.map((entry, index) => (
                        <Cell key={`cell-${index}`} fill={entry.color} />
                      ))}
                    </Pie>
                    <Tooltip
                      contentStyle={{
                        backgroundColor: 'var(--color-surface-elevated)',
                        border: '1px solid var(--color-border)',
                        borderRadius: '8px',
                      }}
                    />
                    <Legend />
                  </PieChart>
                </ResponsiveContainer>
              </div>
            </div>
          </div>

          {/* Category and design system charts */}
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            {/* Elements by category */}
            <div className="bg-surface rounded-lg border border-border p-4">
              <h3 className="font-medium text-foreground mb-4">Elements by Category</h3>
              {categoryData.length > 0 ? (
                <div className="h-64">
                  <ResponsiveContainer width="100%" height="100%">
                    <PieChart>
                      <Pie
                        data={categoryData}
                        cx="50%"
                        cy="50%"
                        outerRadius={80}
                        dataKey="value"
                        label={({ name, percent }) => `${name} ${((percent ?? 0) * 100).toFixed(0)}%`}
                        labelLine={false}
                      >
                        {categoryData.map((entry, index) => (
                          <Cell key={`cell-${index}`} fill={entry.color} />
                        ))}
                      </Pie>
                      <Tooltip
                        contentStyle={{
                          backgroundColor: 'var(--color-surface-elevated)',
                          border: '1px solid var(--color-border)',
                          borderRadius: '8px',
                        }}
                      />
                    </PieChart>
                  </ResponsiveContainer>
                </div>
              ) : (
                <div className="h-64 flex items-center justify-center text-muted-foreground">
                  No category data available
                </div>
              )}
            </div>

            {/* Elements by design system */}
            <div className="bg-surface rounded-lg border border-border p-4">
              <h3 className="font-medium text-foreground mb-4">Elements by Design System</h3>
              {designSystemData.length > 0 ? (
                <div className="h-64">
                  <ResponsiveContainer width="100%" height="100%">
                    <BarChart data={designSystemData} layout="vertical">
                      <XAxis type="number" stroke="var(--color-muted-foreground)" fontSize={12} />
                      <YAxis type="category" dataKey="name" stroke="var(--color-muted-foreground)" fontSize={12} width={100} />
                      <Tooltip
                        contentStyle={{
                          backgroundColor: 'var(--color-surface-elevated)',
                          border: '1px solid var(--color-border)',
                          borderRadius: '8px',
                        }}
                      />
                      <Bar dataKey="value" radius={[0, 4, 4, 0]}>
                        {designSystemData.map((entry, index) => (
                          <Cell key={`cell-${index}`} fill={entry.color} />
                        ))}
                      </Bar>
                    </BarChart>
                  </ResponsiveContainer>
                </div>
              ) : (
                <div className="h-64 flex items-center justify-center text-muted-foreground">
                  No design system data available
                </div>
              )}
            </div>
          </div>

          {/* Detailed stats */}
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div className="bg-surface rounded-lg border border-border p-4">
              <div className="flex items-center gap-2 mb-3">
                <ThumbsUp className="w-4 h-4 text-success" />
                <span className="text-sm text-muted-foreground">Positive Feedback</span>
              </div>
              <p className="text-3xl font-bold text-foreground">{metrics.positive_feedback}</p>
            </div>
            <div className="bg-surface rounded-lg border border-border p-4">
              <div className="flex items-center gap-2 mb-3">
                <ThumbsDown className="w-4 h-4 text-error" />
                <span className="text-sm text-muted-foreground">Negative Feedback</span>
              </div>
              <p className="text-3xl font-bold text-foreground">{metrics.negative_feedback}</p>
            </div>
            <div className="bg-surface rounded-lg border border-border p-4">
              <div className="flex items-center gap-2 mb-3">
                <Zap className="w-4 h-4 text-accent-400" />
                <span className="text-sm text-muted-foreground">Total Generations</span>
              </div>
              <p className="text-3xl font-bold text-foreground">{metrics.total_generations}</p>
            </div>
          </div>
        </>
      ) : (
        <div className="flex items-center justify-center h-64">
          <div className="text-center text-muted-foreground">
            <BarChart3 className="w-16 h-16 mx-auto mb-4 opacity-50" />
            <p className="text-lg font-medium">Waiting for metrics...</p>
            <p className="text-sm mt-2">Connect to the server to see real-time data</p>
          </div>
        </div>
      )}
    </div>
  );
}
