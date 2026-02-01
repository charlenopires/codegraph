import { create } from 'zustand';
import { getWebSocket } from '@/api/websocket';
import type {
  QueryRequest,
  QueryResult,
  GenerateRequest,
  GenerateComplete,
  GenerateStreaming,
  FeedbackSubmit,
  FeedbackAck,
} from '@/api/types';

interface FeedbackState {
  elementId: string;
  feedbackType: 'thumbs_up' | 'thumbs_down';
  newConfidence: number;
}

interface GenerationState {
  // Query state
  isQuerying: boolean;
  queryResult: QueryResult | null;

  // Generation state
  isGenerating: boolean;
  streamingHtml: string;
  streamingCss: string;
  streamingJs: string;
  generationResult: GenerateComplete | null;

  // Feedback state
  submittingFeedback: Set<string>;
  lastFeedback: FeedbackState | null;

  // Common
  error: string | null;
}

interface GenerationActions {
  query: (request: QueryRequest) => Promise<QueryResult>;
  generate: (request: GenerateRequest) => Promise<GenerateComplete>;
  submitFeedback: (
    elementId: string,
    feedbackType: 'thumbs_up' | 'thumbs_down',
    queryContext?: string,
    comment?: string
  ) => Promise<FeedbackAck>;
  reset: () => void;
}

type GenerationStore = GenerationState & GenerationActions;

const initialState: GenerationState = {
  isQuerying: false,
  queryResult: null,
  isGenerating: false,
  streamingHtml: '',
  streamingCss: '',
  streamingJs: '',
  generationResult: null,
  submittingFeedback: new Set(),
  lastFeedback: null,
  error: null,
};

export const useGenerationStore = create<GenerationStore>((set, get) => {
  const ws = getWebSocket();

  // Subscribe to streaming updates
  ws.on('generate_streaming', (message) => {
    if (get().isGenerating) {
      const chunk = message.payload as GenerateStreaming;
      switch (chunk.chunk_type) {
        case 'html':
          set((state) => ({ streamingHtml: state.streamingHtml + chunk.chunk }));
          break;
        case 'css':
          set((state) => ({ streamingCss: state.streamingCss + chunk.chunk }));
          break;
        case 'javascript':
          set((state) => ({ streamingJs: state.streamingJs + chunk.chunk }));
          break;
      }
    }
  });

  return {
    ...initialState,

    query: async (request: QueryRequest): Promise<QueryResult> => {
      set({ isQuerying: true, queryResult: null, error: null });

      try {
        const response = await ws.request<QueryRequest, QueryResult>(
          'query_request',
          request
        );
        const result = response.payload as QueryResult;
        set({ isQuerying: false, queryResult: result });
        return result;
      } catch (error) {
        const message = error instanceof Error ? error.message : 'Query failed';
        set({ isQuerying: false, error: message });
        throw error;
      }
    },

    generate: async (request: GenerateRequest): Promise<GenerateComplete> => {
      set({
        isGenerating: true,
        streamingHtml: '',
        streamingCss: '',
        streamingJs: '',
        generationResult: null,
        error: null,
      });

      try {
        const response = await ws.request<GenerateRequest, GenerateComplete>(
          'generate_request',
          request
        );
        const result = response.payload as GenerateComplete;
        set({ isGenerating: false, generationResult: result });
        return result;
      } catch (error) {
        const message = error instanceof Error ? error.message : 'Generation failed';
        set({ isGenerating: false, error: message });
        throw error;
      }
    },

    submitFeedback: async (
      elementId: string,
      feedbackType: 'thumbs_up' | 'thumbs_down',
      queryContext?: string,
      comment?: string
    ): Promise<FeedbackAck> => {
      // Add to submitting set
      set((state) => ({
        submittingFeedback: new Set([...state.submittingFeedback, elementId]),
      }));

      try {
        const request: FeedbackSubmit = {
          element_id: elementId,
          feedback_type: feedbackType,
          query_context: queryContext,
          comment,
        };

        const response = await ws.request<FeedbackSubmit, FeedbackAck>(
          'feedback_submit',
          request
        );
        const result = response.payload as FeedbackAck;

        // Remove from submitting set and store last feedback
        set((state) => {
          const newSubmitting = new Set(state.submittingFeedback);
          newSubmitting.delete(elementId);
          return {
            submittingFeedback: newSubmitting,
            lastFeedback: {
              elementId: result.element_id,
              feedbackType,
              newConfidence: result.new_confidence,
            },
          };
        });

        return result;
      } catch (error) {
        // Remove from submitting set on error
        set((state) => {
          const newSubmitting = new Set(state.submittingFeedback);
          newSubmitting.delete(elementId);
          return { submittingFeedback: newSubmitting };
        });
        throw error;
      }
    },

    reset: () => set(initialState),
  };
});
