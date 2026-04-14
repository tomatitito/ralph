import { spawn } from "node:child_process";

import type { CommandConfig } from "../config/config-types.ts";

export interface CommandExecutionResult {
  command: string;
  cwd: string | null;
  exitCode: number | null;
  stdout: string;
  stderr: string;
  timedOut: boolean;
}

export type ExecuteCommand = (commandConfig: CommandConfig) => Promise<CommandExecutionResult>;

export interface CommandRunnerOptions {
  cwd?: string;
  env?: NodeJS.ProcessEnv;
}

function resolveEnvironment(baseEnv: NodeJS.ProcessEnv | undefined, commandEnv: Record<string, string>): NodeJS.ProcessEnv {
  return {
    ...(baseEnv ?? process.env),
    ...commandEnv,
  };
}

function buildFailedExecution(commandConfig: CommandConfig, stderr: string): CommandExecutionResult {
  return {
    command: commandConfig.command,
    cwd: commandConfig.cwd,
    exitCode: null,
    stdout: "",
    stderr,
    timedOut: false,
  };
}

async function executeCommand(
  commandConfig: CommandConfig,
  options: CommandRunnerOptions = {},
): Promise<CommandExecutionResult> {
  const cwd = commandConfig.cwd ?? options.cwd ?? process.cwd();
  const env = resolveEnvironment(options.env, commandConfig.env);
  const timeoutMs = commandConfig.timeoutSeconds === null ? null : commandConfig.timeoutSeconds * 1000;

  return await new Promise<CommandExecutionResult>((resolve) => {
    let stdout = "";
    let stderr = "";
    let timedOut = false;
    let resolved = false;
    let timeoutHandle: ReturnType<typeof setTimeout> | null = null;

    const finish = (exitCode: number | null) => {
      if (resolved) {
        return;
      }

      resolved = true;
      if (timeoutHandle !== null) {
        clearTimeout(timeoutHandle);
      }

      resolve({
        command: commandConfig.command,
        cwd: commandConfig.cwd,
        exitCode,
        stdout,
        stderr,
        timedOut,
      });
    };

    let child: ReturnType<typeof spawn>;
    try {
      child = spawn(commandConfig.command, {
        cwd,
        env,
        shell: true,
        stdio: ["ignore", "pipe", "pipe"],
      });
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      resolve(buildFailedExecution(commandConfig, message));
      return;
    }

    child.stdout?.setEncoding("utf8");
    child.stderr?.setEncoding("utf8");

    child.stdout?.on("data", (chunk: string | Buffer) => {
      stdout += chunk.toString();
    });

    child.stderr?.on("data", (chunk: string | Buffer) => {
      stderr += chunk.toString();
    });

    child.on("error", (error) => {
      if (resolved) {
        return;
      }

      const message = error instanceof Error ? error.message : String(error);
      stderr += stderr.length > 0 ? `\n${message}` : message;
      finish(null);
    });

    child.on("close", (exitCode) => {
      finish(exitCode);
    });

    if (timeoutMs !== null) {
      timeoutHandle = setTimeout(() => {
        if (resolved) {
          return;
        }

        timedOut = true;
        child.kill("SIGKILL");
      }, timeoutMs);
    }
  });
}

export function createCommandRunner(options: CommandRunnerOptions = {}): ExecuteCommand {
  return (commandConfig) => executeCommand(commandConfig, options);
}
