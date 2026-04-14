import type { ResolvedConfigBundle, ResolvedRunConfig } from "../config/config-types.ts";
import type { CheckHookResult } from "../checks/check-types.ts";
import type { CompletionRunner } from "../completion/completion-runner.ts";
import type { CompletionValidationResult } from "../completion/completion-types.ts";
import type { IterationRuntime, IterationRuntimeResult } from "../runtime/runtime-types.ts";
import type { ChecksRunner } from "../checks/check-runner.ts";

export type RunExitReason = "loop_completed" | "max_iterations_exceeded" | "interrupted" | "error";

export interface IterationEvaluationInput {
  iterationNumber: number;
  runtime: IterationRuntimeResult;
  afterIterationChecks: CheckHookResult;
  completion: CompletionValidationResult;
  beforeFinalSuccessChecks: CheckHookResult | null;
}

export interface LoopControllerDependencies {
  runtime: IterationRuntime;
  runChecks: ChecksRunner;
  runCompletion: CompletionRunner;
}

export interface RunLoopControllerInput extends LoopControllerDependencies {
  runConfig: ResolvedRunConfig;
}

export interface LoopControllerResult {
  exitReason: RunExitReason;
  iterationCount: number;
  outputLines: string[];
}

export interface RunConfiguredLoopInput {
  config: ResolvedConfigBundle;
}
