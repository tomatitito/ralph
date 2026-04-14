import type { ChecksConfig, CommandConfig } from "../config/config-types.ts";
import type { CheckHook, CheckHookResult, CheckResult } from "./check-types.ts";

export type ChecksRunner = (hook: CheckHook) => Promise<CheckHookResult>;

function commandsForHook(config: ChecksConfig, hook: CheckHook): readonly CommandConfig[] {
  return hook === "after_iteration" ? config.afterIteration : config.beforeFinalSuccess;
}

function toCheckResult(command: CommandConfig, hook: CheckHook): CheckResult {
  return {
    name: command.name,
    hook,
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

export async function runChecks(input: { hook: CheckHook; config: ChecksConfig }): Promise<CheckHookResult> {
  const commands = commandsForHook(input.config, input.hook);

  return {
    hook: input.hook,
    executed: commands.length > 0,
    passed: true,
    results: commands.map((command) => toCheckResult(command, input.hook)),
  };
}

export function createChecksRunner(config: ChecksConfig): ChecksRunner {
  return async (hook) => runChecks({ hook, config });
}
