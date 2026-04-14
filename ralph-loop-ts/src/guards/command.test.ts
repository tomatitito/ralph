import { describe, expect, test } from "bun:test";
import { mkdtempSync, realpathSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";

import { createCommandRunner } from "./command.ts";

function createTempDir(): string {
  return mkdtempSync(join(tmpdir(), "ralph-command-runner-"));
}

describe("command runner", () => {
  test("captures stdout, stderr, and exit code", async () => {
    const executeCommand = createCommandRunner();

    const result = await executeCommand({
      name: "capture",
      command: "printf 'out'; printf 'err' 1>&2; exit 7",
      cwd: null,
      timeoutSeconds: null,
      requiredExitCode: 0,
      requiredStdout: null,
      requiredStderr: null,
      env: {},
    });

    expect(result.exitCode).toBe(7);
    expect(result.stdout).toBe("out");
    expect(result.stderr).toBe("err");
    expect(result.timedOut).toBe(false);
  });

  test("honors cwd and env overrides", async () => {
    const cwd = createTempDir();
    const resolvedCwd = realpathSync(cwd);

    const runner = createCommandRunner({
      cwd,
      env: {
        RALPH_CHECKS_ENV: "expected-value",
      },
    });

    const result = await runner({
      name: "cwd-env",
      command: "printf '%s|%s' \"$PWD\" \"$RALPH_CHECKS_ENV\"",
      cwd: null,
      timeoutSeconds: null,
      requiredExitCode: 0,
      requiredStdout: null,
      requiredStderr: null,
      env: {},
    });

    expect(result.exitCode).toBe(0);
    expect(result.stdout).toBe(`${resolvedCwd}|expected-value`);
    expect(result.timedOut).toBe(false);
  });

  test("times out long-running commands", async () => {
    const executeCommand = createCommandRunner();

    const result = await executeCommand({
      name: "timeout",
      command: "bun -e \"await new Promise((resolve) => setTimeout(resolve, 2000))\"",
      cwd: null,
      timeoutSeconds: 1,
      requiredExitCode: 0,
      requiredStdout: null,
      requiredStderr: null,
      env: {},
    });

    expect(result.timedOut).toBe(true);
    expect(result.exitCode).toBeNull();
  });
});
