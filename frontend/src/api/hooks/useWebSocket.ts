import { useEffect } from 'react';
import { useConnectionStore } from '@/stores/connection';

export function useWebSocket() {
  const { state, connect, disconnect } = useConnectionStore();

  useEffect(() => {
    // Auto-connect on mount
    connect();

    // Cleanup on unmount
    return () => {
      // Don't disconnect on unmount to maintain connection across pages
    };
  }, [connect]);

  return {
    state,
    isConnected: state === 'connected',
    isConnecting: state === 'connecting',
    isReconnecting: state === 'reconnecting',
    connect,
    disconnect,
  };
}
