import type { CheckResult } from "../checks/check-types.ts";

export type CompletionValidationStatus = "skipped" | "passed" | "failed";

export interface CompletionValidationResult {
  status: CompletionValidationStatus;
  results: CheckResult[];
}
