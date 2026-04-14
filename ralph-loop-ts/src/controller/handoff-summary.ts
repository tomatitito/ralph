export interface BuildHandoffSummaryInput {
  iterationNumber: number;
  assistantText: string | null;
  taskComplete: boolean;
  loopComplete: boolean;
}

export function buildHandoffSummary(input: BuildHandoffSummaryInput): string {
  const headline = input.assistantText?.split("\n")[0]?.trim() || `iteration ${input.iterationNumber}`;
  return [
    `iteration ${input.iterationNumber} summary`,
    `headline: ${headline}`,
    `task_complete: ${input.taskComplete}`,
    `loop_complete: ${input.loopComplete}`,
  ].join("\n");
}
