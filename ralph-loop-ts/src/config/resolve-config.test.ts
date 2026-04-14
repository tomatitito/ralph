import { describe, expect, test } from "bun:test";
import { existsSync, mkdtempSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";

import type { RawCliArgs } from "./config-types.ts";
import { resolveConfig } from "./resolve-config.ts";

function defaultCliArgs(overrides: Partial<RawCliArgs> = {}): RawCliArgs {
  return {
    promptText: null,
    promptFile: null,
    maxIterations: null,
    completionPromise: null,
    outputDir: null,
    contextLimit: null,
    configPath: null,
    checksConfigPath: null,
    completionConfigPath: null,
    provider: null,
    model: null,
    thinking: null,
    ...overrides,
  };
}

const syncReader = {
  fileExists: existsSync,
  readText: (path: string) => readFileSync(path, "utf8"),
};

function createFixtureRoot(): string {
  const root = mkdtempSync(join(tmpdir(), "ralph-config-"));
  mkdirSync(join(root, "config"), { recursive: true });
  writeFileSync(join(root, "config", "task.txt"), "config prompt\n");
  writeFileSync(
    join(root, "config", "ralph.toml"),
    [
      "[prompt]",
      'file = "task.txt"',
      "",
      "[loop]",
      "max_iterations = 4",
      "context_limit = 2000",
      'completion_promise = "TASK COMPLETE"',
      "",
      "[model]",
      'provider = "anthropic"',
      'model = "claude"',
      'thinking = "low"',
      "",
      "[artifacts]",
      'base_dir = ".ralph-loop"',
      "",
      "[paths]",
      'checks = "checks.toml"',
      'completion = "completion.toml"',
    ].join("\n"),
  );
  writeFileSync(
    join(root, "config", "checks.toml"),
    [
      "[[after_iteration]]",
      'name = "test"',
      'command = "bun test"',
    ].join("\n"),
  );
  writeFileSync(
    join(root, "config", "completion.toml"),
    '[[on_loop_complete_claim]]\nname = "done"\ncommand = "echo ok"\n',
  );
  writeFileSync(join(root, "bad-checks.toml"), '[[after_iteration]]\nname = 1\ncommand = "bun test"\n');
  writeFileSync(
    join(root, "bad-completion.toml"),
    '[[on_loop_complete_claim]]\nname = "done"\ncommand = 1\n',
  );
  return root;
}

describe("resolveConfig", () => {
  test("supports a valid inline prompt via CLI", () => {
    const root = createFixtureRoot();

    const resolved = resolveConfig({
      cliArgs: defaultCliArgs({
        promptText: "inline prompt",
        checksConfigPath: join(root, "config", "checks.toml"),
        completionConfigPath: join(root, "config", "completion.toml"),
      }),
      cwd: root,
      reader: syncReader,
    });

    expect(resolved.runConfig.prompt).toEqual({ kind: "inline", text: "inline prompt" });
  });

  test("supports a valid prompt-file config and resolves referenced paths relative to the loop config", () => {
    const root = createFixtureRoot();

    const resolved = resolveConfig({
      cliArgs: defaultCliArgs({ configPath: join(root, "config", "ralph.toml") }),
      cwd: root,
      reader: syncReader,
    });

    expect(resolved.runConfig.prompt.kind).toBe("file");
    expect(resolved.runConfig.checksConfigPath).toBe(join(root, "config", "checks.toml"));
    expect(resolved.runConfig.completionConfigPath).toBe(join(root, "config", "completion.toml"));
    expect(resolved.checksConfig.afterIteration).toHaveLength(1);
    expect(resolved.completionConfig.onLoopCompleteClaim).toHaveLength(1);
  });

  test("rejects multiple prompt sources", () => {
    const root = createFixtureRoot();

    expect(() =>
      resolveConfig({
        cliArgs: defaultCliArgs({
          promptText: "inline",
          promptFile: join(root, "config", "task.txt"),
          checksConfigPath: join(root, "config", "checks.toml"),
          completionConfigPath: join(root, "config", "completion.toml"),
        }),
        cwd: root,
        reader: syncReader,
      }),
    ).toThrow(/exactly one of --prompt or --prompt-file/);
  });

  test("applies CLI override precedence", () => {
    const root = createFixtureRoot();

    const resolved = resolveConfig({
      cliArgs: defaultCliArgs({
        configPath: join(root, "config", "ralph.toml"),
        promptText: "cli prompt",
        maxIterations: 2,
        contextLimit: 999,
        provider: "openai",
        model: "gpt-5",
        thinking: "high",
      }),
      cwd: root,
      reader: syncReader,
    });

    expect(resolved.runConfig.prompt).toEqual({ kind: "inline", text: "cli prompt" });
    expect(resolved.runConfig.maxIterations).toBe(2);
    expect(resolved.runConfig.contextLimit).toBe(999);
    expect(resolved.runConfig.provider).toBe("openai");
    expect(resolved.runConfig.model).toBe("gpt-5");
    expect(resolved.runConfig.thinking).toBe("high");
  });

  test("fails when a referenced config file is missing", () => {
    const root = createFixtureRoot();

    expect(() =>
      resolveConfig({
        cliArgs: defaultCliArgs({
          promptText: "inline",
          checksConfigPath: join(root, "missing-checks.toml"),
          completionConfigPath: join(root, "config", "completion.toml"),
        }),
        cwd: root,
        reader: syncReader,
      }),
    ).toThrow(/checks config file not found/);
  });

  test("fails on invalid checks config structure", () => {
    const root = createFixtureRoot();

    expect(() =>
      resolveConfig({
        cliArgs: defaultCliArgs({
          promptText: "inline",
          checksConfigPath: join(root, "bad-checks.toml"),
          completionConfigPath: join(root, "config", "completion.toml"),
        }),
        cwd: root,
        reader: syncReader,
      }),
    ).toThrow(/invalid checks config/);
  });

  test("fails on invalid completion config structure", () => {
    const root = createFixtureRoot();

    expect(() =>
      resolveConfig({
        cliArgs: defaultCliArgs({
          promptText: "inline",
          checksConfigPath: join(root, "config", "checks.toml"),
          completionConfigPath: join(root, "bad-completion.toml"),
        }),
        cwd: root,
        reader: syncReader,
      }),
    ).toThrow(/invalid completion config/);
  });
});
