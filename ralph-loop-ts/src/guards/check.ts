import type { ChecksConfig, CommandConfig } from "../config/config-types.ts";
import {
  createCommandRunner,
  type CommandExecutionResult,
  type ExecuteCommand,
} from "./command.ts";

export type CheckHook = "after_iteration";

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

export type ChecksRunner = () => Promise<CheckHookResult>;

function commandsForHook(config: ChecksConfig): readonly CommandConfig[] {
  return config.afterIteration;
}

function commandPassed(command: CommandConfig, execution: CommandExecutionResult): boolean {
  if (execution.timedOut || execution.exitCode !== command.requiredExitCode) {
    return false;
  }

  if (command.requiredStdout !== null && !execution.stdout.includes(command.requiredStdout)) {
    return false;
  }

  if (command.requiredStderr !== null && !execution.stderr.includes(command.requiredStderr)) {
    return false;
  }

  return true;
}

function toCheckResult(hook: CheckHook, command: CommandConfig, execution: CommandExecutionResult): CheckResult {
  return {
    name: command.name,
    hook,
    execution,
    passed: commandPassed(command, execution),
  };
}

async function runChecks(
  config: ChecksConfig,
  executeCommand: ExecuteCommand,
): Promise<CheckHookResult> {
  const hook: CheckHook = "after_iteration";
  const results: CheckResult[] = [];

  for (const command of commandsForHook(config)) {
    let execution: CommandExecutionResult;

    try {
      execution = await executeCommand(command);
    } catch (error) {
      execution = {
        command: command.command,
        cwd: command.cwd,
        exitCode: null,
        stdout: "",
        stderr: error instanceof Error ? error.message : String(error),
        timedOut: false,
      };
    }

    results.push(toCheckResult(hook, command, execution));
  }

  return {
    hook,
    executed: results.length > 0,
    passed: results.every((result) => result.passed),
    results,
  };
}

export function createChecksRunner(config: ChecksConfig, executeCommand: ExecuteCommand = createCommandRunner()): ChecksRunner {
  return async () => runChecks(config, executeCommand);
}
