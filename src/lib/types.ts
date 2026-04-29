export interface DetectedSpan {
  label: string;
  start: number;
  end: number;
  text: string;
  placeholder: string;
}

export interface RedactionResult {
  schema_version: number;
  text: string;
  redacted_text: string;
  detected_spans: DetectedSpan[];
  summary: Record<string, number>;
  latency_ms: number;
}

export type ModelStatus =
  | "not_found"
  | "ready"
  | "loading"
  | "loaded"
  | "error";

export type ServerLifecycleStatus =
  | "stopped"
  | "starting"
  | "running"
  | "stopping"
  | "error";

export interface ServerStatus {
  running: boolean;
  status: ServerLifecycleStatus;
  host: string;
  port: number;
  model_loaded: boolean;
}

export interface LoadedModelInfo {
  name: string;
  architecture: string;
  num_labels: number;
  max_position_embeddings: number;
  hidden_size: number;
  vocab_size: number;
}

export interface HttpLogEntry {
  id: number;
  timestamp: string;
  method: string;
  path: string;
  status: number;
  latency_ms: number;
  request_body?: string;
  response_body?: string;
}

export interface AppSettings {
  server_enabled: boolean;
  server_host: string;
  server_port: number;
  server_auto_start: boolean;
  server_log_limit: number;
  model_path: string;
  auto_copy_result: boolean;
  save_history: boolean;
  history_limit: number;
  app_language: string;
  global_shortcut: string;
}

export interface HistoryEntry {
  id: number;
  timestamp: string;
  input_text: string;
  redacted_text: string;
  detected_spans: DetectedSpan[];
  summary: Record<string, number>;
  latency_ms: number;
}
