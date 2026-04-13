import type { LoopConfig } from "./config-types.ts";

function assertObject(value: unknown, message: string): Record<string, unknown> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new Error(message);
  }

  return value as Record<string, unknown>;
}

function optionalString(value: unknown, fieldName: string): string | null {
  if (value === undefined) {
    return null;
  }

  if (typeof value !== "string") {
    throw new Error(`${fieldName} must be a string`);
  }

  return value;
}

function optionalInteger(value: unknown, fieldName: string): number | null {
  if (value === undefined) {
    return null;
  }

  if (typeof value !== "number" || !Number.isInteger(value)) {
    throw new Error(`${fieldName} must be an integer`);
  }

  return value;
}

export function parseLoopConfigToml(tomlText: string): LoopConfig {
  const parsed = Bun.TOML.parse(tomlText) as unknown;
  const root = assertObject(parsed, "loop config must be a TOML table");

  const prompt = root.prompt === undefined ? {} : assertObject(root.prompt, "[prompt] must be a table");
  const loop = root.loop === undefined ? {} : assertObject(root.loop, "[loop] must be a table");
  const model = root.model === undefined ? {} : assertObject(root.model, "[model] must be a table");
  const artifacts =
    root.artifacts === undefined ? {} : assertObject(root.artifacts, "[artifacts] must be a table");
  const paths = root.paths === undefined ? {} : assertObject(root.paths, "[paths] must be a table");

  const config: LoopConfig = {
    promptText: optionalString(prompt.text, "prompt.text"),
    promptFile: optionalString(prompt.file, "prompt.file"),
    maxIterations: optionalInteger(loop.max_iterations, "loop.max_iterations"),
    contextLimit: optionalInteger(loop.context_limit, "loop.context_limit"),
    completionPromise: optionalString(loop.completion_promise, "loop.completion_promise"),
    provider: optionalString(model.provider, "model.provider"),
    model: optionalString(model.model, "model.model"),
    thinking: optionalString(model.thinking, "model.thinking"),
    outputDir: optionalString(artifacts.base_dir, "artifacts.base_dir"),
    checksConfigPath: optionalString(paths.checks, "paths.checks"),
    completionConfigPath: optionalString(paths.completion, "paths.completion"),
  };

  if (config.promptText !== null && config.promptFile !== null) {
    throw new Error("loop config must not set both prompt.text and prompt.file");
  }

  return config;
}
