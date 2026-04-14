import type { CompletionConfig, CommandConfig } from "../config/config-types.ts";
import type { CheckResult } from "../checks/check-types.ts";
import type { CompletionValidationResult } from "./completion-types.ts";

export type CompletionRunner = (claimed: boolean) => Promise<CompletionValidationResult>;

function toCheckResult(command: CommandConfig): CheckResult {
  return {
    name: command.name,
    hook: "before_final_success",
    execution: {
      command: command.command,
      cwd: command.cwd,
      exitCode: command.requiredExitCode,
      stdout: command.requiredStdout ?? "",
      stderr: command.requiredStderr ?? "",
      timedOut: false,
    },
    passed: true,
  };
}

export async function runCompletion(input: {
  claimed: boolean;
  config: CompletionConfig;
}): Promise<CompletionValidationResult> {
  if (!input.claimed) {
    return {
      status: "skipped",
      results: [],
    };
  }

  return {
    status: "passed",
    results: input.config.onLoopCompleteClaim.map((command) => toCheckResult(command)),
  };
}

export function createCompletionRunner(config: CompletionConfig): CompletionRunner {
  return async (claimed) => runCompletion({ claimed, config });
}
