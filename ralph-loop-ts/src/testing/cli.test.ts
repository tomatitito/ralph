import { describe, expect, test } from "bun:test";
import { mkdtempSync, writeFileSync, mkdirSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";

import { parseCliArgs, runCli } from "../cli.ts";

describe("parseCliArgs", () => {
  test("parses the supported CLI flags", () => {
    expect(
      parseCliArgs([
        "--prompt",
        "hello",
        "--max-iterations",
        "3",
        "--completion-promise",
        "DONE",
        "--output-dir",
        ".ralph",
        "--context-limit",
        "1000",
        "--config",
        "ralph.toml",
        "--checks-config",
        "checks.toml",
        "--completion-config",
        "completion.toml",
        "--provider",
        "anthropic",
        "--model",
        "claude-sonnet-4-5",
        "--thinking",
        "medium",
      ]),
    ).toEqual({
      promptText: "hello",
      promptFile: null,
      maxIterations: 3,
      completionPromise: "DONE",
      outputDir: ".ralph",
      contextLimit: 1000,
      configPath: "ralph.toml",
      checksConfigPath: "checks.toml",
      completionConfigPath: "completion.toml",
      provider: "anthropic",
      model: "claude-sonnet-4-5",
      thinking: "medium",
    });
  });
});

describe("runCli", () => {
  test("resolves config and returns JSON", async () => {
    const root = mkdtempSync(join(tmpdir(), "ralph-cli-"));
    mkdirSync(join(root, "nested"));
    writeFileSync(join(root, "task.txt"), "ship it\n");
    writeFileSync(
      join(root, "ralph.toml"),
      [
        "[prompt]",
        'file = "task.txt"',
        "",
        "[paths]",
        'checks = "checks.toml"',
        'completion = "completion.toml"',
      ].join("\n"),
    );
    writeFileSync(join(root, "checks.toml"), '[[after_iteration]]\nname = "test"\ncommand = "bun test"\n');
    writeFileSync(
      join(root, "completion.toml"),
      '[[on_loop_complete_claim]]\nname = "done"\ncommand = "echo ok"\n',
    );

    const output = await runCli([], { cwd: root });
    const parsed = JSON.parse(output) as {
      runConfig: { prompt: { kind: string; text: string }; checksConfigPath: string; completionConfigPath: string };
    };

    expect(parsed.runConfig.prompt.kind).toBe("file");
    expect(parsed.runConfig.prompt.text).toBe("ship it\n");
    expect(parsed.runConfig.checksConfigPath).toBe(join(root, "checks.toml"));
    expect(parsed.runConfig.completionConfigPath).toBe(join(root, "completion.toml"));
  });

  test("runs the deterministic mock loop end-to-end", async () => {
    const root = mkdtempSync(join(tmpdir(), "ralph-cli-mock-"));
    writeFileSync(join(root, "checks.toml"), '[[after_iteration]]\nname = "always-pass"\ncommand = "echo ok"\n');
    writeFileSync(
      join(root, "completion.toml"),
      '[[on_loop_complete_claim]]\nname = "validate"\ncommand = "echo ok"\n',
    );

    const output = await runCli(
      [
        "--prompt",
        "ship it",
        "--provider",
        "mock",
        "--checks-config",
        join(root, "checks.toml"),
        "--completion-config",
        join(root, "completion.toml"),
      ],
      { cwd: root },
    );

    expect(output).toContain("mock task 1 completed");
    expect(output).toContain("mock task 2 completed");
    expect(output).toContain("mock task 3 completed");
    expect(output.match(/<ralph:task-complete\/>/g)?.length).toBe(3);
    expect(output.match(/<ralph:loop-complete\/>/g)?.length).toBe(1);
  });
});
