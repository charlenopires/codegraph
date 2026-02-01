import { create } from 'zustand';
import { getWebSocket } from '@/api/websocket';
import type { MetricsUpdate, GraphStatsResult } from '@/api/types';

interface MetricsState {
  isSubscribed: boolean;
  metrics: MetricsUpdate | null;
  graphStats: GraphStatsResult | null;
  error: string | null;
}

interface MetricsActions {
  subscribe: () => void;
  unsubscribe: () => void;
  fetchGraphStats: () => Promise<GraphStatsResult>;
}

type MetricsStore = MetricsState & MetricsActions;

const initialState: MetricsState = {
  isSubscribed: false,
  metrics: null,
  graphStats: null,
  error: null,
};

export const useMetricsStore = create<MetricsStore>((set, get) => {
  const ws = getWebSocket();

  // Subscribe to metrics updates
  ws.on('metrics_update', (message) => {
    if (get().isSubscribed) {
      set({ metrics: message.payload as MetricsUpdate });
    }
  });

  return {
    ...initialState,

    subscribe: () => {
      try {
        ws.send('metrics_subscribe', {});
        set({ isSubscribed: true, error: null });
      } catch (error) {
        const message = error instanceof Error ? error.message : 'Failed to subscribe';
        set({ error: message });
      }
    },

    unsubscribe: () => {
      try {
        ws.send('metrics_unsubscribe', {});
        set({ isSubscribed: false });
      } catch {
        // Ignore errors on unsubscribe
      }
    },

    fetchGraphStats: async (): Promise<GraphStatsResult> => {
      try {
        const response = await ws.request<object, GraphStatsResult>('graph_stats', {});
        const result = response.payload as GraphStatsResult;
        set({ graphStats: result, error: null });
        return result;
      } catch (error) {
        const message = error instanceof Error ? error.message : 'Failed to fetch stats';
        set({ error: message });
        throw error;
      }
    },
  };
});
