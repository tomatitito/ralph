import type { RuntimeExitReason } from "../runtime/runtime-types.ts";
import type { CompletionValidationStatus } from "../guards/completion.ts";

export enum IterationDecision {
  Success = "success",
  RestartTaskBoundary = "restart_task_boundary",
  RestartFailedCompletion = "restart_failed_completion",
  RestartContextLimit = "restart_context_limit",
  RestartIncomplete = "restart_incomplete",
  Interrupted = "interrupted",
  MaxIterationsExceeded = "max_iterations_exceeded",
  Error = "error",
}

export interface DecideIterationInput {
  runtimeExitReason: RuntimeExitReason;
  contextLimitHit: boolean;
  taskComplete: boolean;
  loopCompleteClaimed: boolean;
  afterIterationChecksPassed: boolean;
  completionStatus: CompletionValidationStatus;
}

export function toIterationDecision(input: DecideIterationInput): IterationDecision {
  const wasInterrupted = input.runtimeExitReason === "interrupted";
  const hadRuntimeError = input.runtimeExitReason === "error";
  const completedSuccessfully =
    input.loopCompleteClaimed &&
    input.afterIterationChecksPassed &&
    input.completionStatus === "passed";
  const failedLoopCompletionClaim =
    input.loopCompleteClaimed &&
    (!input.afterIterationChecksPassed || input.completionStatus === "failed");
  const hitContextLimit = input.contextLimitHit || input.runtimeExitReason === "context_limit_requested";
  const completedTaskBoundary = input.taskComplete;

  switch (true) {
    case wasInterrupted:
      return IterationDecision.Interrupted;
    case hadRuntimeError:
      return IterationDecision.Error;
    case completedSuccessfully:
      return IterationDecision.Success;
    case failedLoopCompletionClaim:
      return IterationDecision.RestartFailedCompletion;
    case hitContextLimit:
      return IterationDecision.RestartContextLimit;
    case completedTaskBoundary:
      return IterationDecision.RestartTaskBoundary;
    default:
      return IterationDecision.RestartIncomplete;
  }
}
