import type { MessageType, WsMessage, ErrorPayload } from './types';

export type ConnectionState = 'disconnected' | 'connecting' | 'connected' | 'reconnecting';

export interface WebSocketOptions {
  url?: string;
  reconnectDelay?: number;
  maxReconnectDelay?: number;
  heartbeatInterval?: number;
}

type MessageHandler = (message: WsMessage) => void;
type StateHandler = (state: ConnectionState) => void;
type ErrorHandler = (error: ErrorPayload) => void;

export class WebSocketManager {
  private ws: WebSocket | null = null;
  private options: Required<WebSocketOptions>;
  private reconnectAttempts = 0;
  private reconnectTimeout: ReturnType<typeof setTimeout> | null = null;
  private heartbeatTimeout: ReturnType<typeof setInterval> | null = null;
  private pendingRequests = new Map<string, {
    resolve: (value: WsMessage) => void;
    reject: (error: Error) => void;
    timeout: ReturnType<typeof setTimeout>;
  }>();
  private messageHandlers = new Map<MessageType, Set<MessageHandler>>();
  private stateHandlers = new Set<StateHandler>();
  private errorHandlers = new Set<ErrorHandler>();
  private _state: ConnectionState = 'disconnected';

  constructor(options: WebSocketOptions = {}) {
    // Use environment variable or fallback to current host
    const defaultUrl = import.meta.env.VITE_WS_URL ||
      `${window.location.protocol === 'https:' ? 'wss:' : 'ws:'}//${window.location.host}/ws`;
    this.options = {
      url: options.url || defaultUrl,
      reconnectDelay: options.reconnectDelay || 1000,
      maxReconnectDelay: options.maxReconnectDelay || 30000,
      heartbeatInterval: options.heartbeatInterval || 30000,
    };
  }

  get state(): ConnectionState {
    return this._state;
  }

  private setState(state: ConnectionState): void {
    this._state = state;
    this.stateHandlers.forEach(handler => handler(state));
  }

  connect(): void {
    if (this.ws?.readyState === WebSocket.OPEN) return;

    this.setState('connecting');

    try {
      this.ws = new WebSocket(this.options.url);

      this.ws.onopen = () => {
        this.reconnectAttempts = 0;
        this.setState('connected');
        this.startHeartbeat();
      };

      this.ws.onclose = () => {
        this.stopHeartbeat();
        this.setState('disconnected');
        this.scheduleReconnect();
      };

      this.ws.onerror = () => {
        // Error will trigger onclose
      };

      this.ws.onmessage = (event) => {
        try {
          const message = JSON.parse(event.data) as WsMessage;
          this.handleMessage(message);
        } catch {
          console.error('Failed to parse WebSocket message');
        }
      };
    } catch {
      this.setState('disconnected');
      this.scheduleReconnect();
    }
  }

  disconnect(): void {
    this.stopHeartbeat();
    if (this.reconnectTimeout) {
      clearTimeout(this.reconnectTimeout);
      this.reconnectTimeout = null;
    }
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
    this.setState('disconnected');

    // Reject all pending requests
    this.pendingRequests.forEach(({ reject, timeout }) => {
      clearTimeout(timeout);
      reject(new Error('Connection closed'));
    });
    this.pendingRequests.clear();
  }

  private scheduleReconnect(): void {
    if (this.reconnectTimeout) return;

    const delay = Math.min(
      this.options.reconnectDelay * Math.pow(2, this.reconnectAttempts),
      this.options.maxReconnectDelay
    );

    this.reconnectAttempts++;
    this.setState('reconnecting');

    this.reconnectTimeout = setTimeout(() => {
      this.reconnectTimeout = null;
      this.connect();
    }, delay);
  }

  private startHeartbeat(): void {
    this.heartbeatTimeout = setInterval(() => {
      if (this.ws?.readyState === WebSocket.OPEN) {
        // Send ping - server should respond with pong
        const pingMsg = {
          id: crypto.randomUUID(),
          type: 'ping',
          timestamp: Date.now(),
          payload: null,
        };
        this.ws.send(JSON.stringify(pingMsg));
      }
    }, this.options.heartbeatInterval);
  }

  private stopHeartbeat(): void {
    if (this.heartbeatTimeout) {
      clearInterval(this.heartbeatTimeout);
      this.heartbeatTimeout = null;
    }
  }

  private handleMessage(message: WsMessage): void {
    // Check if this is a response to a pending request
    const pending = this.pendingRequests.get(message.id);
    if (pending) {
      clearTimeout(pending.timeout);
      this.pendingRequests.delete(message.id);

      if (message.type === 'error') {
        pending.reject(new Error((message.payload as ErrorPayload).message));
      } else {
        pending.resolve(message);
      }
      return;
    }

    // Handle errors
    if (message.type === 'error') {
      this.errorHandlers.forEach(handler => handler(message.payload as ErrorPayload));
      return;
    }

    // Dispatch to message handlers
    const handlers = this.messageHandlers.get(message.type);
    if (handlers) {
      handlers.forEach(handler => handler(message));
    }
  }

  send<T>(type: MessageType, payload: T): string {
    if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
      throw new Error('WebSocket not connected');
    }

    const id = crypto.randomUUID();
    const message: WsMessage<T> = {
      id,
      type,
      timestamp: Date.now(),
      payload,
    };

    this.ws.send(JSON.stringify(message));
    return id;
  }

  request<Req, Res>(type: MessageType, payload: Req, timeout = 30000): Promise<WsMessage<Res>> {
    return new Promise((resolve, reject) => {
      try {
        const id = this.send(type, payload);

        const timeoutHandle = setTimeout(() => {
          this.pendingRequests.delete(id);
          reject(new Error('Request timeout'));
        }, timeout);

        this.pendingRequests.set(id, {
          resolve: resolve as (value: WsMessage) => void,
          reject,
          timeout: timeoutHandle,
        });
      } catch (error) {
        reject(error);
      }
    });
  }

  on(type: MessageType, handler: MessageHandler): () => void {
    if (!this.messageHandlers.has(type)) {
      this.messageHandlers.set(type, new Set());
    }
    this.messageHandlers.get(type)!.add(handler);

    return () => {
      this.messageHandlers.get(type)?.delete(handler);
    };
  }

  onStateChange(handler: StateHandler): () => void {
    this.stateHandlers.add(handler);
    return () => {
      this.stateHandlers.delete(handler);
    };
  }

  onError(handler: ErrorHandler): () => void {
    this.errorHandlers.add(handler);
    return () => {
      this.errorHandlers.delete(handler);
    };
  }
}

// Singleton instance
let instance: WebSocketManager | null = null;

export function getWebSocket(): WebSocketManager {
  if (!instance) {
    instance = new WebSocketManager();
  }
  return instance;
}
