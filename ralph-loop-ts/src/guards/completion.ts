import type { CompletionConfig, CommandConfig } from "../config/config-types.ts";
import {
  createCommandRunner,
  type CommandExecutionResult,
  type ExecuteCommand,
} from "./command.ts";

export type CompletionValidationStatus = "skipped" | "passed" | "failed";

export interface CompletionValidatorResult {
  name: string;
  command: CommandConfig;
  execution: CommandExecutionResult | null;
  passed: boolean;
  status: CompletionValidationStatus;
}

export interface CompletionValidationResult {
  status: CompletionValidationStatus;
  results: CompletionValidatorResult[];
}

export type CompletionRunner = () => Promise<CompletionValidationResult>;

function commandPassed(command: CommandConfig, execution: CommandExecutionResult): boolean {
  return (
    execution.exitCode === command.requiredExitCode &&
    !execution.timedOut &&
    (command.requiredStdout === null || execution.stdout.includes(command.requiredStdout)) &&
    (command.requiredStderr === null || execution.stderr.includes(command.requiredStderr))
  );
}

function skippedValidatorResult(command: CommandConfig): CompletionValidatorResult {
  return {
    name: command.name,
    command: {
      ...command,
      env: { ...command.env },
    },
    execution: null,
    passed: false,
    status: "skipped",
  };
}

function failedExecutionResult(command: CommandConfig, error: unknown): CommandExecutionResult {
  return {
    command: command.command,
    cwd: command.cwd,
    exitCode: null,
    stdout: "",
    stderr: error instanceof Error ? error.message : String(error),
    timedOut: false,
  };
}

function validatedValidatorResult(command: CommandConfig, execution: CommandExecutionResult): CompletionValidatorResult {
  const passed = commandPassed(command, execution);

  return {
    name: command.name,
    command: {
      ...command,
      env: { ...command.env },
    },
    execution,
    passed,
    status: passed ? "passed" : "failed",
  };
}

export function skippedCompletionResult(config: CompletionConfig): CompletionValidationResult {
  return {
    status: "skipped",
    results: config.onLoopCompleteClaim.map((command) => skippedValidatorResult(command)),
  };
}

export async function runCompletion(input: {
  config: CompletionConfig;
  commandRunner?: ExecuteCommand;
}): Promise<CompletionValidationResult> {
  const results: CompletionValidatorResult[] = [];
  const commandRunner = input.commandRunner ?? createCommandRunner();

  for (const command of input.config.onLoopCompleteClaim) {
    let execution: CommandExecutionResult;

    try {
      execution = await commandRunner(command);
    } catch (error) {
      execution = failedExecutionResult(command, error);
    }

    results.push(validatedValidatorResult(command, execution));
  }

  return {
    status: results.every((result) => result.status === "passed") ? "passed" : "failed",
    results,
  };
}

export function createCompletionRunner(
  config: CompletionConfig,
  commandRunner: ExecuteCommand = createCommandRunner(),
): CompletionRunner {
  return async () => await runCompletion({ config, commandRunner });
}
