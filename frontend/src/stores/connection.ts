import { create } from 'zustand';
import { getWebSocket, type ConnectionState } from '@/api/websocket';

interface ConnectionStore {
  state: ConnectionState;
  connect: () => void;
  disconnect: () => void;
}

export const useConnectionStore = create<ConnectionStore>((set) => {
  const ws = getWebSocket();

  // Subscribe to state changes
  ws.onStateChange((state) => {
    set({ state });
  });

  return {
    state: ws.state,
    connect: () => ws.connect(),
    disconnect: () => ws.disconnect(),
  };
});
