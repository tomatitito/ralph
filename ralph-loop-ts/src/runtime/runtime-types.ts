export type RuntimeExitReason =
  | "agent_end"
  | "context_limit_requested"
  | "interrupted"
  | "error";

export interface IterationInput {
  iterationNumber: number;
  objective: string;
  handoffSummary: string | null;
  provider: string | null;
  model: string | null;
  thinking: string | null;
  contextLimit: number;
}

export interface CombinedExtensionState {
  context: {
    peakTokenCount: number | null;
    finalTokenCount: number | null;
    contextLimitHit: boolean;
    diagnostics: string[];
  };
  lifecycle: {
    taskComplete: boolean;
    loopComplete: boolean;
    diagnostics: string[];
  };
}

export interface IterationRuntimeResult {
  sessionId: string | null;
  exitReason: RuntimeExitReason;
  assistantText: string | null;
  diagnostics: string[];
  extensionState: CombinedExtensionState;
}

export type IterationRuntime = (
  input: IterationInput,
) => Promise<IterationRuntimeResult>;
