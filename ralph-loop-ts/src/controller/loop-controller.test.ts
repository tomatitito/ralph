import { describe, expect, test } from "bun:test";

import { runLoopController } from "./loop-controller.ts";

describe("runLoopController", () => {
  test("runs checks on every iteration and completion only on the final loop-complete claim", async () => {
    let checkRuns = 0;
    let completionRuns = 0;

    const result = await runLoopController({
      runConfig: {
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
      },
      runtime: async ({ iterationNumber }) => ({
        sessionId: `mock-${iterationNumber}`,
        exitReason: "agent_end",
        assistantText:
          iterationNumber < 3
            ? `mock task ${iterationNumber} completed\n<ralph:task-complete/>`
            : `mock task 3 completed\n<ralph:task-complete/>\n<ralph:loop-complete/>`,
        diagnostics: [],
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

    expect(result.exitReason).toBe("loop_completed");
    expect(checkRuns).toBe(3);
    expect(completionRuns).toBe(1);
  });
});
