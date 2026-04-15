import { describe, expect, test } from "bun:test";

import { toIterationDecision, IterationDecision } from "./decision.ts";

describe("toIterationDecision", () => {
  test.each([
    {
      name: "returns success when loop completion validates even if both markers and context limit are present",
      input: {
        runtimeExitReason: "context_limit_requested" as const,
        contextLimitHit: true,
        taskComplete: true,
        loopCompleteClaimed: true,
        afterIterationChecksPassed: true,
        completionStatus: "passed" as const,
      },
      expected: IterationDecision.Success,
    },
    {
      name: "returns restart_failed_completion when both markers are present but checks fail",
      input: {
        runtimeExitReason: "agent_end" as const,
        contextLimitHit: false,
        taskComplete: true,
        loopCompleteClaimed: true,
        afterIterationChecksPassed: false,
        completionStatus: "skipped" as const,
      },
      expected: IterationDecision.RestartFailedCompletion,
    },
    {
      name: "returns restart_context_limit for context pressure without a valid loop completion",
      input: {
        runtimeExitReason: "context_limit_requested" as const,
        contextLimitHit: true,
        taskComplete: true,
        loopCompleteClaimed: false,
        afterIterationChecksPassed: true,
        completionStatus: "skipped" as const,
      },
      expected: IterationDecision.RestartContextLimit,
    },
    {
      name: "returns restart_task_boundary for task completion without loop completion",
      input: {
        runtimeExitReason: "agent_end" as const,
        contextLimitHit: false,
        taskComplete: true,
        loopCompleteClaimed: false,
        afterIterationChecksPassed: true,
        completionStatus: "skipped" as const,
      },
      expected: IterationDecision.RestartTaskBoundary,
    },
    {
      name: "returns restart_incomplete when no marker is present and checks pass",
      input: {
        runtimeExitReason: "agent_end" as const,
        contextLimitHit: false,
        taskComplete: false,
        loopCompleteClaimed: false,
        afterIterationChecksPassed: true,
        completionStatus: "skipped" as const,
      },
      expected: IterationDecision.RestartIncomplete,
    },
    {
      name: "returns interrupted before any restart decision",
      input: {
        runtimeExitReason: "interrupted" as const,
        contextLimitHit: true,
        taskComplete: true,
        loopCompleteClaimed: true,
        afterIterationChecksPassed: true,
        completionStatus: "passed" as const,
      },
      expected: IterationDecision.Interrupted,
    },
  ])("$name", ({ input, expected }) => {
    expect(toIterationDecision(input)).toBe(expected);
  });
});
