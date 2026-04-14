import { describe, expect, test } from "bun:test";

import type { CommandExecutionResult } from "./command.ts";
import type { CompletionConfig, CommandConfig } from "../config/config-types.ts";
import { runCompletion, skippedCompletionResult } from "./completion.ts";

function makeCommand(overrides: Partial<CommandConfig> & Pick<CommandConfig, "name" | "command">): CommandConfig {
  return {
    name: overrides.name,
    command: overrides.command,
    cwd: overrides.cwd ?? null,
    timeoutSeconds: overrides.timeoutSeconds ?? null,
    requiredExitCode: overrides.requiredExitCode ?? 0,
    requiredStdout: overrides.requiredStdout ?? null,
    requiredStderr: overrides.requiredStderr ?? null,
    env: overrides.env ?? {},
  };
}

function makeExecution(command: CommandConfig, overrides: Partial<CommandExecutionResult> = {}): CommandExecutionResult {
  return {
    command: command.command,
    cwd: command.cwd,
    exitCode: command.requiredExitCode,
    stdout: command.requiredStdout ?? "",
    stderr: command.requiredStderr ?? "",
    timedOut: false,
    ...overrides,
  };
}

function makeConfig(commands: CommandConfig[]): CompletionConfig {
  return {
    onLoopCompleteClaim: commands,
  };
}

describe("runCompletion", () => {
  test("returns skipped results without executing validators when the loop-complete claim is absent", async () => {
    const commands = [
      makeCommand({ name: "first", command: "echo first" }),
      makeCommand({ name: "second", command: "echo second" }),
    ];

    const result = skippedCompletionResult(makeConfig(commands));

    expect(result).toEqual({
      status: "skipped",
      results: [
        {
          name: "first",
          command: commands[0]!,
          execution: null,
          passed: false,
          status: "skipped",
        },
        {
          name: "second",
          command: commands[1]!,
          execution: null,
          passed: false,
          status: "skipped",
        },
      ],
    });
  });

  test("marks completion passed only when all validator success rules match", async () => {
    const commands = [
      makeCommand({
        name: "first",
        command: "echo first",
        requiredExitCode: 0,
        requiredStdout: "alpha",
        requiredStderr: "beta",
      }),
      makeCommand({
        name: "second",
        command: "echo second",
        requiredExitCode: 2,
      }),
    ];

    const result = await runCompletion({
      config: makeConfig(commands),
      commandRunner: async (command) => {
        if (command.name === "first") {
          return makeExecution(command, {
            stdout: "alpha on stdout",
            stderr: "beta on stderr",
          });
        }

        return makeExecution(command, {
          exitCode: 2,
          stdout: "second output",
          stderr: "",
        });
      },
    });

    expect(result.status).toBe("passed");
    expect(result.results.map((result) => result.status)).toEqual(["passed", "passed"]);
  });

  test("returns failed when any validator does not satisfy the success rules", async () => {
    const commands = [
      makeCommand({ name: "pass", command: "echo pass" }),
      makeCommand({
        name: "fail",
        command: "echo fail",
        requiredExitCode: 0,
        requiredStdout: "needed",
      }),
      makeCommand({ name: "after-fail", command: "echo after" }),
    ];

    const result = await runCompletion({
      config: makeConfig(commands),
      commandRunner: async (command) => {
        if (command.name === "pass") {
          return makeExecution(command);
        }

        if (command.name === "fail") {
          return makeExecution(command, {
            exitCode: 1,
            stdout: "missing substring",
          });
        }

        return makeExecution(command);
      },
    });

    expect(result.status).toBe("failed");
    expect(result.results.map((result) => result.status)).toEqual(["passed", "failed", "passed"]);
    expect(result.results[1]!.execution?.exitCode).toBe(1);
  });

  test("continues executing validators after a failure", async () => {
    const commands = [
      makeCommand({ name: "first", command: "echo first" }),
      makeCommand({ name: "second", command: "echo second" }),
      makeCommand({ name: "third", command: "echo third" }),
    ];
    const executedNames: string[] = [];

    const result = await runCompletion({
      config: makeConfig(commands),
      commandRunner: async (command) => {
        executedNames.push(command.name);

        if (command.name === "second") {
          return makeExecution(command, {
            timedOut: true,
          });
        }

        return makeExecution(command);
      },
    });

    expect(executedNames).toEqual(["first", "second", "third"]);
    expect(result.status).toBe("failed");
    expect(result.results.map((result) => result.status)).toEqual(["passed", "failed", "passed"]);
  });
});
