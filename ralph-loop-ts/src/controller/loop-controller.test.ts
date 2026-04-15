import { describe, expect, test } from "bun:test";

import type { ResolvedRunConfig } from "../config/config-types.ts";
import type { IterationInput, IterationRuntimeResult } from "../runtime/runtime-types.ts";
import { RunExitReason, runLoopController } from "./loop-controller.ts";

const baseRunConfig: ResolvedRunConfig = {
  prompt: { kind: "inline", text: "ship it" },
  maxIterations: null,
  contextLimit: 1000,
  completionPromise: "TASK COMPLETE",
  provider: "mock",
  model: null,
  thinking: null,
  outputDir: null,
  checksConfigPath: null,
  completionConfigPath: null,
  projectPath: "/tmp/project",
};

function makeRuntimeResult(overrides: Partial<IterationRuntimeResult> = {}): IterationRuntimeResult {
  const { extensionState, ...rest } = overrides;

  return {
    sessionId: "mock-session",
    exitReason: "agent_end",
    assistantText: "assistant output",
    diagnostics: [],
    ...rest,
    extensionState: {
      context: {
        peakTokenCount: null,
        finalTokenCount: null,
        contextLimitHit: false,
        diagnostics: [],
        ...extensionState?.context,
      },
      lifecycle: {
        taskComplete: false,
        loopComplete: false,
        diagnostics: [],
        ...extensionState?.lifecycle,
      },
    },
  };
}

describe("runLoopController", () => {
  test("runs checks on every iteration and completion only on the final loop-complete claim", async () => {
    let checkRuns = 0;
    let completionRuns = 0;

    const result = await runLoopController({
      runConfig: baseRunConfig,
      runtime: async ({ iterationNumber }) =>
        makeRuntimeResult({
          assistantText:
            iterationNumber < 3
              ? `mock task ${iterationNumber} completed\n<ralph:task-complete/>`
              : `mock task 3 completed\n<ralph:task-complete/>\n<ralph:loop-complete/>`,
          extensionState: {
            context: {
              peakTokenCount: null,
              finalTokenCount: null,
              contextLimitHit: false,
              diagnostics: [],
            },
            lifecycle: {
              taskComplete: true,
              loopComplete: iterationNumber >= 3,
              diagnostics: [],
            },
          },
        }),
      runChecks: async () => {
        checkRuns += 1;
        return {
          hook: "after_iteration",
          executed: true,
          passed: true,
          results: [],
        };
      },
      runCompletion: async () => {
        completionRuns += 1;
        return {
          status: "passed",
          results: [],
        };
      },
    });

    expect(result.exitReason).toBe(RunExitReason.LoopCompleted);
    expect(result.iterationCount).toBe(3);
    expect(checkRuns).toBe(3);
    expect(completionRuns).toBe(1);
  });

  test("fails before starting an iteration that would exceed max iterations", async () => {
    let runtimeCalls = 0;

    const result = await runLoopController({
      runConfig: { ...baseRunConfig, maxIterations: 2 },
      runtime: async () => {
        runtimeCalls += 1;
        return makeRuntimeResult();
      },
      runChecks: async () => ({
        hook: "after_iteration",
        executed: true,
        passed: true,
        results: [],
      }),
      runCompletion: async () => ({ status: "passed", results: [] }),
    });

    expect(result.exitReason).toBe(RunExitReason.MaxIterationsExceeded);
    expect(result.iterationCount).toBe(2);
    expect(runtimeCalls).toBe(2);
  });

  test("restarts after a context-limit iteration and carries a handoff summary", async () => {
    const runtimeInputs: IterationInput[] = [];

    const result = await runLoopController({
      runConfig: { ...baseRunConfig, maxIterations: 2 },
      runtime: async (input) => {
        runtimeInputs.push(input);

        return input.iterationNumber === 1
          ? makeRuntimeResult({
              extensionState: {
                context: {
                  peakTokenCount: null,
                  finalTokenCount: null,
                  contextLimitHit: true,
                  diagnostics: [],
                },
                lifecycle: {
                  taskComplete: true,
                  loopComplete: false,
                  diagnostics: [],
                },
              },
            })
          : makeRuntimeResult({
              extensionState: {
                context: {
                  peakTokenCount: null,
                  finalTokenCount: null,
                  contextLimitHit: false,
                  diagnostics: [],
                },
                lifecycle: {
                  taskComplete: false,
                  loopComplete: true,
                  diagnostics: [],
                },
              },
            });
      },
      runChecks: async () => ({
        hook: "after_iteration",
        executed: true,
        passed: true,
        results: [],
      }),
      runCompletion: async () => ({ status: "passed", results: [] }),
    });

    expect(result.exitReason).toBe(RunExitReason.LoopCompleted);
    expect(runtimeInputs).toHaveLength(2);
    expect(runtimeInputs[1]?.handoffSummary).toContain("iteration 1 summary");
    expect(runtimeInputs[1]?.handoffSummary).toContain("task_complete: true");
  });

  test("restarts after a task boundary without invoking completion", async () => {
    let completionRuns = 0;

    const result = await runLoopController({
      runConfig: { ...baseRunConfig, maxIterations: 2 },
      runtime: async ({ iterationNumber }) =>
        iterationNumber === 1
          ? makeRuntimeResult({
              extensionState: {
                context: {
                  peakTokenCount: null,
                  finalTokenCount: null,
                  contextLimitHit: false,
                  diagnostics: [],
                },
                lifecycle: {
                  taskComplete: true,
                  loopComplete: false,
                  diagnostics: [],
                },
              },
            })
          : makeRuntimeResult({
              extensionState: {
                context: {
                  peakTokenCount: null,
                  finalTokenCount: null,
                  contextLimitHit: false,
                  diagnostics: [],
                },
                lifecycle: {
                  taskComplete: true,
                  loopComplete: true,
                  diagnostics: [],
                },
              },
            }),
      runChecks: async () => ({
        hook: "after_iteration",
        executed: true,
        passed: true,
        results: [],
      }),
      runCompletion: async () => {
        completionRuns += 1;
        return { status: "passed", results: [] };
      },
    });

    expect(result.exitReason).toBe(RunExitReason.LoopCompleted);
    expect(result.iterationCount).toBe(2);
    expect(completionRuns).toBe(1);
  });

  test("restarts after failed checks on a loop-complete claim without invoking completion", async () => {
    let completionRuns = 0;
    let checkRuns = 0;

    const result = await runLoopController({
      runConfig: { ...baseRunConfig, maxIterations: 2 },
      runtime: async ({ iterationNumber }) =>
        makeRuntimeResult({
          extensionState: {
            context: {
              peakTokenCount: null,
              finalTokenCount: null,
              contextLimitHit: false,
              diagnostics: [],
            },
            lifecycle: {
              taskComplete: true,
              loopComplete: true,
              diagnostics: [],
            },
          },
          assistantText: `attempt ${iterationNumber}`,
        }),
      runChecks: async () => {
        checkRuns += 1;
        return {
          hook: "after_iteration",
          executed: true,
          passed: checkRuns > 1,
          results: [],
        };
      },
      runCompletion: async () => {
        completionRuns += 1;
        return { status: "passed", results: [] };
      },
    });

    expect(result.exitReason).toBe(RunExitReason.LoopCompleted);
    expect(result.iterationCount).toBe(2);
    expect(completionRuns).toBe(1);
  });

  test("restarts after failed completion validation and tries again on the next loop-complete claim", async () => {
    let completionRuns = 0;

    const result = await runLoopController({
      runConfig: { ...baseRunConfig, maxIterations: 2 },
      runtime: async ({ iterationNumber }) =>
        makeRuntimeResult({
          assistantText: `attempt ${iterationNumber}\n<ralph:loop-complete/>`,
          extensionState: {
            context: {
              peakTokenCount: null,
              finalTokenCount: null,
              contextLimitHit: false,
              diagnostics: [],
            },
            lifecycle: {
              taskComplete: iterationNumber === 1,
              loopComplete: true,
              diagnostics: [],
            },
          },
        }),
      runChecks: async () => ({
        hook: "after_iteration",
        executed: true,
        passed: true,
        results: [],
      }),
      runCompletion: async () => {
        completionRuns += 1;
        return { status: completionRuns === 1 ? "failed" : "passed", results: [] };
      },
    });

    expect(result.exitReason).toBe(RunExitReason.LoopCompleted);
    expect(result.iterationCount).toBe(2);
    expect(completionRuns).toBe(2);
  });

  test("treats both markers in one successful iteration as loop completion", async () => {
    let completionRuns = 0;

    const result = await runLoopController({
      runConfig: { ...baseRunConfig, maxIterations: 3 },
      runtime: async () =>
        makeRuntimeResult({
          assistantText: "done\n<ralph:task-complete/>\n<ralph:loop-complete/>",
          extensionState: {
            context: {
              peakTokenCount: null,
              finalTokenCount: null,
              contextLimitHit: false,
              diagnostics: [],
            },
            lifecycle: {
              taskComplete: true,
              loopComplete: true,
              diagnostics: [],
            },
          },
        }),
      runChecks: async () => ({
        hook: "after_iteration",
        executed: true,
        passed: true,
        results: [],
      }),
      runCompletion: async () => {
        completionRuns += 1;
        return { status: "passed", results: [] };
      },
    });

    expect(result.exitReason).toBe(RunExitReason.LoopCompleted);
    expect(result.iterationCount).toBe(1);
    expect(completionRuns).toBe(1);
  });
});
