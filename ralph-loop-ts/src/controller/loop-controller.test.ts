import { describe, expect, test } from "bun:test";

import type { CheckHook } from "../checks/check-types.ts";
import { runLoopController } from "./loop-controller.ts";

describe("runLoopController", () => {
  test("runs checks on every iteration and completion only on the final loop-complete claim", async () => {
    const checkHooks: CheckHook[] = [];
    const completionClaims: boolean[] = [];

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
      runChecks: async (hook) => {
        checkHooks.push(hook);
        return {
          hook,
          executed: true,
          passed: true,
          results: [],
        };
      },
      runCompletion: async (claimed) => {
        completionClaims.push(claimed);
        return {
          status: claimed ? "passed" : "skipped",
          results: [],
        };
      },
    });

    expect(result.exitReason).toBe("loop_completed");
    expect(checkHooks).toEqual([
      "after_iteration",
      "after_iteration",
      "after_iteration",
      "before_final_success",
    ]);
    expect(completionClaims).toEqual([true]);
  });
});
