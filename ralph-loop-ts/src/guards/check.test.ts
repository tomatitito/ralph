import { describe, expect, test } from "bun:test";

import type { CommandExecutionResult, ExecuteCommand } from "./command.ts";
import { createChecksRunner } from "./check.ts";

function execution(command: string, overrides: Partial<CommandExecutionResult> = {}): CommandExecutionResult {
  return {
    command,
    cwd: null,
    exitCode: 0,
    stdout: "",
    stderr: "",
    timedOut: false,
    ...overrides,
  };
}

describe("check runner", () => {
  test("runs after_iteration checks in file order without short-circuiting", async () => {
    const seenCommands: string[] = [];
    const commandRunner: ExecuteCommand = async (commandConfig) => {
      seenCommands.push(commandConfig.name);

      if (commandConfig.name === "first") {
        return execution(commandConfig.command, { stdout: "ok", exitCode: 0 });
      }

      if (commandConfig.name === "second") {
        return execution(commandConfig.command, {
          stdout: "missing",
          stderr: "stderr-match",
          exitCode: 1,
        });
      }

      return execution(commandConfig.command, {
        stdout: "extra",
        stderr: "extra stderr",
        exitCode: 0,
      });
    };

    const runner = createChecksRunner(
      {
        afterIteration: [
          {
            name: "first",
            command: "first-cmd",
            cwd: null,
            timeoutSeconds: null,
            requiredExitCode: 0,
            requiredStdout: "ok",
            requiredStderr: null,
            env: {},
          },
          {
            name: "second",
            command: "second-cmd",
            cwd: null,
            timeoutSeconds: null,
            requiredExitCode: 0,
            requiredStdout: "will-not-match",
            requiredStderr: "stderr-match",
            env: {},
          },
        ],
      },
      commandRunner,
    );

    const afterIteration = await runner();

    expect(afterIteration.hook).toBe("after_iteration");
    expect(afterIteration.executed).toBe(true);
    expect(afterIteration.passed).toBe(false);
    expect(afterIteration.results.map((result) => result.name)).toEqual(["first", "second"]);
    expect(afterIteration.results.map((result) => result.passed)).toEqual([true, false]);

    expect(seenCommands).toEqual(["first", "second"]);
  });

  test("fails timed out commands even when exit code and output match", async () => {
    const runner = createChecksRunner(
      {
        afterIteration: [
          {
            name: "timeout",
            command: "timeout-cmd",
            cwd: null,
            timeoutSeconds: null,
            requiredExitCode: 0,
            requiredStdout: "matched",
            requiredStderr: "matched",
            env: {},
          },
        ],
      },
      async (commandConfig) =>
        execution(commandConfig.command, {
          stdout: "matched",
          stderr: "matched",
          timedOut: true,
          exitCode: 0,
        }),
    );

    const result = await runner();

    expect(result.passed).toBe(false);
    expect(result.results[0]?.passed).toBe(false);
  });
});
