import type { ResolvedConfigBundle, ResolvedRunConfig } from "../config/config-types.ts";
import { createChecksRunner, type ChecksRunner } from "../guards/check.ts";
import {
  createCompletionRunner,
  type CompletionRunner,
  type CompletionValidationResult,
} from "../guards/completion.ts";
import { buildHandoffSummary } from "./handoff-summary.ts";
import { runPiIteration } from "../runtime/pi-runtime.ts";
import type { IterationRuntime } from "../runtime/runtime-types.ts";

export type RunExitReason = "loop_completed" | "max_iterations_exceeded" | "interrupted" | "error";

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
}

function isSuccessfulLoopCompletion(input: {
  loopCompleteClaimed: boolean;
  afterIterationPassed: boolean;
  completionPassed: boolean;
}): boolean {
  return input.loopCompleteClaimed && input.afterIterationPassed && input.completionPassed;
}

export async function runLoopController(input: RunLoopControllerInput): Promise<LoopControllerResult> {
  const outputLines: string[] = [];
  let iterationNumber = 1;
  let handoffSummary: string | null = null;

  while (true) {
    if (input.runConfig.maxIterations !== null && iterationNumber > input.runConfig.maxIterations) {
      return {
        exitReason: "max_iterations_exceeded",
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

    if (
      isSuccessfulLoopCompletion({
        loopCompleteClaimed,
        afterIterationPassed: afterIterationChecks.passed,
        completionPassed: completion.status === "passed",
      })
    ) {
      return {
        exitReason: "loop_completed",
        iterationCount: iterationNumber,
        outputLines,
      };
    }

    if (runtimeResult.exitReason === "interrupted") {
      return {
        exitReason: "interrupted",
        iterationCount: iterationNumber,
        outputLines,
      };
    }

    if (runtimeResult.exitReason === "error") {
      return {
        exitReason: "error",
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

export async function runConfiguredLoop(input: RunConfiguredLoopInput): Promise<LoopControllerResult> {
  if (input.config.runConfig.provider !== "mock") {
    throw new Error("only --provider mock is implemented for this vertical slice");
  }

  return runLoopController({
    runConfig: input.config.runConfig,
    runtime: runPiIteration,
    runChecks: createChecksRunner(input.config.checksConfig),
    runCompletion: createCompletionRunner(input.config.completionConfig),
  });
}
