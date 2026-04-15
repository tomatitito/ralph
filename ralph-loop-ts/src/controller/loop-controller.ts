import type { ResolvedConfigBundle, ResolvedRunConfig } from "../config/config-types.ts";
import { createChecksRunner, type ChecksRunner } from "../guards/check.ts";
import {
  createCompletionRunner,
  type CompletionRunner,
  type CompletionValidationResult,
} from "../guards/completion.ts";
import { buildHandoffSummary } from "./handoff-summary.ts";
import { toIterationDecision, IterationDecision } from "./decision.ts";
import { runPiIteration } from "../runtime/pi-runtime.ts";
import type { IterationRuntime } from "../runtime/runtime-types.ts";

export enum RunExitReason {
  LoopCompleted = "loop_completed",
  MaxIterationsExceeded = "max_iterations_exceeded",
  Interrupted = "interrupted",
  Error = "error",
}

export interface RunLoopControllerInput {
  runConfig: ResolvedRunConfig;
  runtime: IterationRuntime;
  runChecks: ChecksRunner;
  runCompletion: CompletionRunner;
}

export interface LoopControllerResult {
  exitReason: RunExitReason;
  iterationCount: number;
  outputLines: string[];
}

export interface RunConfiguredLoopInput {
  config: ResolvedConfigBundle;
  runtime: IterationRuntime;
  runChecks: ChecksRunner;
  runCompletion: CompletionRunner;
}

export interface ConfiguredLoopDependencies {
  runtime: IterationRuntime;
  runChecks: ChecksRunner;
  runCompletion: CompletionRunner;
}

function toRunExitReason(decision: IterationDecision): RunExitReason | null {
  switch (decision) {
    case IterationDecision.Success:
      return RunExitReason.LoopCompleted;
    case IterationDecision.MaxIterationsExceeded:
      return RunExitReason.MaxIterationsExceeded;
    case IterationDecision.Interrupted:
      return RunExitReason.Interrupted;
    case IterationDecision.Error:
      return RunExitReason.Error;
    default:
      return null;
  }
}

export async function runLoopController(input: RunLoopControllerInput): Promise<LoopControllerResult> {
  const outputLines: string[] = [];
  let iterationNumber = 1;
  let handoffSummary: string | null = null;

  while (true) {
    if (input.runConfig.maxIterations !== null && iterationNumber > input.runConfig.maxIterations) {
      return {
        exitReason: RunExitReason.MaxIterationsExceeded,
        iterationCount: iterationNumber - 1,
        outputLines,
      };
    }

    const runtimeResult = await input.runtime({
      iterationNumber,
      objective: input.runConfig.prompt.text,
      handoffSummary,
      provider: input.runConfig.provider,
      model: input.runConfig.model,
      thinking: input.runConfig.thinking,
      contextLimit: input.runConfig.contextLimit,
    });

    if (runtimeResult.assistantText !== null && runtimeResult.assistantText.trim() !== "") {
      outputLines.push(runtimeResult.assistantText);
    }

    const afterIterationChecks = await input.runChecks();
    const loopCompleteClaimed = runtimeResult.extensionState.lifecycle.loopComplete;
    const completion: CompletionValidationResult = loopCompleteClaimed && afterIterationChecks.passed
      ? await input.runCompletion()
      : { status: "skipped", results: [] };

    const decision = toIterationDecision({
      runtimeExitReason: runtimeResult.exitReason,
      contextLimitHit: runtimeResult.extensionState.context.contextLimitHit,
      taskComplete: runtimeResult.extensionState.lifecycle.taskComplete,
      loopCompleteClaimed,
      afterIterationChecksPassed: afterIterationChecks.passed,
      completionStatus: completion.status,
    });

    const exitReason = toRunExitReason(decision);
    if (exitReason !== null) {
      return {
        exitReason,
        iterationCount: iterationNumber,
        outputLines,
      };
    }

    handoffSummary = buildHandoffSummary({
      iterationNumber,
      assistantText: runtimeResult.assistantText,
      taskComplete: runtimeResult.extensionState.lifecycle.taskComplete,
      loopComplete: runtimeResult.extensionState.lifecycle.loopComplete,
    });
    iterationNumber += 1;
  }
}

export function createConfiguredLoopDependencies(config: ResolvedConfigBundle): ConfiguredLoopDependencies {
  return {
    runtime: runPiIteration,
    runChecks: createChecksRunner(config.checksConfig),
    runCompletion: createCompletionRunner(config.completionConfig),
  };
}

export async function runConfiguredLoop(input: RunConfiguredLoopInput): Promise<LoopControllerResult> {
  if (input.config.runConfig.provider !== "mock") {
    throw new Error("only --provider mock is implemented for this vertical slice");
  }

  return runLoopController({
    runConfig: input.config.runConfig,
    runtime: input.runtime,
    runChecks: input.runChecks,
    runCompletion: input.runCompletion,
  });
}
