import type { IterationRuntime } from "./runtime-types.ts";

const TASK_COMPLETE_MARKER = "<ralph:task-complete/>";
const LOOP_COMPLETE_MARKER = "<ralph:loop-complete/>";

function buildMockAssistantText(iterationNumber: number): string {
  if (iterationNumber >= 3) {
    return [
      `mock task ${iterationNumber} completed`,
      TASK_COMPLETE_MARKER,
      LOOP_COMPLETE_MARKER,
    ].join("\n");
  }

  return [`mock task ${iterationNumber} completed`, TASK_COMPLETE_MARKER].join("\n");
}

export const runPiIteration: IterationRuntime = async (input) => {
  const assistantText = buildMockAssistantText(input.iterationNumber);

  return {
    sessionId: `mock-session-${input.iterationNumber}`,
    exitReason: "agent_end",
    assistantText,
    diagnostics: [
      `provider=${input.provider ?? "unknown"}`,
      `iteration=${input.iterationNumber}`,
    ],
    extensionState: {
      context: {
        peakTokenCount: null,
        finalTokenCount: null,
        contextLimitHit: false,
        diagnostics: [],
      },
      lifecycle: {
        taskComplete: assistantText.includes(TASK_COMPLETE_MARKER),
        loopComplete: assistantText.includes(LOOP_COMPLETE_MARKER),
        diagnostics: [],
      },
    },
  };
};
