import type { CommandConfig } from "../config/config-types.ts";

export type CheckHook = "after_iteration" | "before_final_success";

export interface CommandExecutionResult {
  command: string;
  cwd: string | null;
  exitCode: number | null;
  stdout: string;
  stderr: string;
  timedOut: boolean;
}

export interface CheckResult {
  name: string;
  hook: CheckHook;
  execution: CommandExecutionResult;
  passed: boolean;
}

export interface CheckHookResult {
  hook: CheckHook;
  executed: boolean;
  passed: boolean;
  results: CheckResult[];
}

export interface RunChecksInput {
  hook: CheckHook;
  commands: readonly CommandConfig[];
}
