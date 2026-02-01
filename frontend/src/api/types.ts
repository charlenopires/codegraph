// WebSocket Protocol Types

export type MessageType =
  // Client → Server
  | 'extract_request'
  | 'query_request'
  | 'generate_request'
  | 'feedback_submit'
  | 'graph_stats'
  | 'graph_elements'
  | 'metrics_subscribe'
  | 'metrics_unsubscribe'
  | 'ping'
  // Server → Client
  | 'extract_progress'
  | 'extract_complete'
  | 'query_result'
  | 'generate_streaming'
  | 'generate_complete'
  | 'feedback_ack'
  | 'graph_stats_result'
  | 'graph_elements_result'
  | 'metrics_update'
  | 'pong'
  | 'error';

export interface WsMessage<T = unknown> {
  id: string;
  type: MessageType;
  timestamp: number;
  payload: T;
}

// Request payloads
export interface ExtractRequest {
  html: string;
  css?: string;
  js?: string;
  name?: string;
  tags?: string[];
  design_system?: string;
}

export interface QueryRequest {
  query: string;
  design_system?: string;
  limit: number;
  include_reasoning: boolean;
}

export interface GenerateRequest {
  query: string;
  design_system?: string;
  use_references: boolean;
  include_css: boolean;
  include_js: boolean;
}

export interface FeedbackSubmit {
  element_id: string;
  feedback_type: 'thumbs_up' | 'thumbs_down';
  query_context?: string;
  comment?: string;
}

export interface GraphElementsRequest {
  page: number;
  per_page: number;
  category?: string;
  design_system?: string;
}

// Response payloads
export interface ExtractProgress {
  phase: 'parsing' | 'detection' | 'ontology' | 'narsese' | 'embedding' | 'storing';
  progress: number;
  message: string;
}

export interface ExtractComplete {
  snippet_id: string;
  element_ids: string[];
  narsese_statements: string[];
  design_system: string;
  processing_time_ms: number;
}

export interface ElementWithScore {
  id: string;
  name: string;
  category: string;
  design_system: string;
  score: number;
  match_reason: string;
}

export interface QueryResult {
  elements: ElementWithScore[];
  narsese_queries: string[];
  reasoning_explanation?: string;
  processing_time_ms: number;
}

export interface GenerateStreaming {
  chunk: string;
  chunk_type: 'html' | 'css' | 'javascript';
  is_final: boolean;
}

export interface GenerateComplete {
  html: string;
  css?: string;
  javascript?: string;
  reference_elements: ElementWithScore[];
  narsese_reasoning: string[];
  generation_time_ms: number;
}

export interface FeedbackAck {
  feedback_id: string;
  element_id: string;
  new_confidence: number;
}

export interface LabelCount {
  label: string;
  count: number;
}

export interface RelTypeCount {
  rel_type: string;
  count: number;
}

export interface GraphStatsResult {
  total_nodes: number;
  total_relationships: number;
  nodes_by_label: LabelCount[];
  relationships_by_type: RelTypeCount[];
  avg_degree: number;
}

export interface GraphElement {
  id: string;
  name: string;
  category: string;
  design_system: string;
  connections: number;
}

export interface GraphElementsResult {
  elements: GraphElement[];
  total: number;
  page: number;
  per_page: number;
}

export interface CategoryCount {
  category: string;
  count: number;
}

export interface DesignSystemCount {
  design_system: string;
  count: number;
}

export interface MetricsUpdate {
  total_elements: number;
  total_queries: number;
  total_generations: number;
  positive_feedback: number;
  negative_feedback: number;
  avg_query_latency_ms: number;
  avg_generation_latency_ms: number;
  elements_by_category: CategoryCount[];
  elements_by_design_system: DesignSystemCount[];
}

export interface ErrorPayload {
  code: number;
  message: string;
}
