import { createChecksRunner } from "../checks/check-runner.ts";
import { createCompletionRunner } from "../completion/completion-runner.ts";
import { buildHandoffSummary } from "./handoff-summary.ts";
import { runPiIteration } from "../runtime/pi-runtime.ts";
import type {
  LoopControllerResult,
  RunConfiguredLoopInput,
  RunLoopControllerInput,
} from "./controller-types.ts";

function isSuccessfulLoopCompletion(input: {
  loopCompleteClaimed: boolean;
  afterIterationPassed: boolean;
  completionPassed: boolean;
  beforeFinalSuccessPassed: boolean;
}): boolean {
  return (
    input.loopCompleteClaimed &&
    input.afterIterationPassed &&
    input.completionPassed &&
    input.beforeFinalSuccessPassed
  );
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

    const afterIterationChecks = await input.runChecks("after_iteration");
    const loopCompleteClaimed = runtimeResult.extensionState.lifecycle.loopComplete;
    const completion = loopCompleteClaimed && afterIterationChecks.passed
      ? await input.runCompletion(true)
      : { status: "skipped", results: [] as [] };
    const beforeFinalSuccessChecks = loopCompleteClaimed && completion.status === "passed"
      ? await input.runChecks("before_final_success")
      : null;

    if (
      isSuccessfulLoopCompletion({
        loopCompleteClaimed,
        afterIterationPassed: afterIterationChecks.passed,
        completionPassed: completion.status === "passed",
        beforeFinalSuccessPassed: beforeFinalSuccessChecks?.passed ?? false,
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
