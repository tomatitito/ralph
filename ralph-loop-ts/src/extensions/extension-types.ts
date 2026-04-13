export interface ContextExtensionState {
  peakTokenCount: number | null;
  finalTokenCount: number | null;
  contextLimitHit: boolean;
  diagnostics: string[];
}

export interface LifecycleMarkerState {
  taskComplete: boolean;
  loopComplete: boolean;
  diagnostics: string[];
}
