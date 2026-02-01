import { create } from 'zustand';
import { getWebSocket } from '@/api/websocket';
import type { ExtractRequest, ExtractProgress, ExtractComplete } from '@/api/types';

interface ExtractionState {
  isExtracting: boolean;
  progress: ExtractProgress | null;
  result: ExtractComplete | null;
  error: string | null;
}

interface ExtractionActions {
  extract: (request: ExtractRequest) => Promise<ExtractComplete>;
  reset: () => void;
}

type ExtractionStore = ExtractionState & ExtractionActions;

const initialState: ExtractionState = {
  isExtracting: false,
  progress: null,
  result: null,
  error: null,
};

export const useExtractionStore = create<ExtractionStore>((set, get) => {
  const ws = getWebSocket();

  // Subscribe to progress updates
  ws.on('extract_progress', (message) => {
    if (get().isExtracting) {
      set({ progress: message.payload as ExtractProgress });
    }
  });

  return {
    ...initialState,

    extract: async (request: ExtractRequest): Promise<ExtractComplete> => {
      set({ isExtracting: true, progress: null, result: null, error: null });

      try {
        const response = await ws.request<ExtractRequest, ExtractComplete>(
          'extract_request',
          request
        );
        const result = response.payload as ExtractComplete;
        set({ isExtracting: false, result });
        return result;
      } catch (error) {
        const message = error instanceof Error ? error.message : 'Extraction failed';
        set({ isExtracting: false, error: message });
        throw error;
      }
    },

    reset: () => set(initialState),
  };
});
