import { dirname, isAbsolute, resolve as resolvePath } from "node:path";

import { parseChecksConfigToml } from "./checks-config.ts";
import { parseCompletionConfigToml } from "./completion-config.ts";
import type {
  LoopConfig,
  PromptSource,
  RawCliArgs,
  ResolvedConfigBundle,
  ThinkingLevel,
} from "./config-types.ts";
import { parseLoopConfigToml } from "./loop-config.ts";

const DEFAULT_CONTEXT_LIMIT = 180000;
const DEFAULT_COMPLETION_PROMISE = "TASK COMPLETE";
const DEFAULT_OUTPUT_DIR = "~/.ralph-loop";
const DEFAULT_CONFIG_FILE_NAME = "ralph.toml";
const VALID_THINKING_LEVELS = new Set<ThinkingLevel>(["off", "minimal", "low", "medium", "high", "xhigh"]);

export interface ConfigReader {
  fileExists(path: string): boolean;
  readText(path: string): string;
}

export interface ResolveConfigOptions {
  cliArgs: RawCliArgs;
  cwd: string;
  reader: ConfigReader;
}

function makeError(message: string, cause?: unknown): Error {
  return new Error(cause instanceof Error ? `${message}: ${cause.message}` : message);
}

function resolveCandidatePath(path: string, baseDir: string): string {
  return isAbsolute(path) ? path : resolvePath(baseDir, path);
}

function resolveOptionalPath(path: string | null, baseDir: string): string | null {
  if (path === null) {
    return null;
  }

  return resolveCandidatePath(path, baseDir);
}

function loadLoopConfig(options: ResolveConfigOptions): { path: string | null; dir: string; config: LoopConfig | null } {
  const explicitConfigPath = options.cliArgs.configPath;
  const discoveredConfigPath = explicitConfigPath ?? resolvePath(options.cwd, DEFAULT_CONFIG_FILE_NAME);
  const shouldLoadDiscovered = explicitConfigPath !== null || options.reader.fileExists(discoveredConfigPath);

  if (!shouldLoadDiscovered) {
    return { path: null, dir: options.cwd, config: null };
  }

  const absoluteConfigPath = resolveCandidatePath(discoveredConfigPath, options.cwd);

  if (!options.reader.fileExists(absoluteConfigPath)) {
    throw makeError(`loop config file not found at ${absoluteConfigPath}`);
  }

  try {
    return {
      path: absoluteConfigPath,
      dir: dirname(absoluteConfigPath),
      config: parseLoopConfigToml(options.reader.readText(absoluteConfigPath)),
    };
  } catch (cause) {
    throw makeError(`invalid loop config at ${absoluteConfigPath}`, cause);
  }
}

function resolvePrompt(
  cliArgs: RawCliArgs,
  loopConfig: LoopConfig | null,
  loopConfigDir: string,
  options: ResolveConfigOptions,
): PromptSource {
  if (cliArgs.promptText !== null && cliArgs.promptFile !== null) {
    throw makeError("provide exactly one of --prompt or --prompt-file");
  }

  if (loopConfig !== null && loopConfig.promptText !== null && loopConfig.promptFile !== null) {
    throw makeError("loop config must not set both prompt.text and prompt.file");
  }

  if (cliArgs.promptText !== null) {
    return { kind: "inline", text: cliArgs.promptText };
  }

  if (cliArgs.promptFile !== null) {
    const path = resolveCandidatePath(cliArgs.promptFile, options.cwd);
    if (!options.reader.fileExists(path)) {
      throw makeError(`prompt file not found at ${path}`);
    }

    return { kind: "file", path, text: options.reader.readText(path) };
  }

  if (loopConfig?.promptText !== null && loopConfig?.promptText !== undefined) {
    return { kind: "inline", text: loopConfig.promptText };
  }

  if (loopConfig?.promptFile !== null && loopConfig?.promptFile !== undefined) {
    const path = resolveCandidatePath(loopConfig.promptFile, loopConfigDir);
    if (!options.reader.fileExists(path)) {
      throw makeError(`prompt file not found at ${path}`);
    }

    return { kind: "file", path, text: options.reader.readText(path) };
  }

  throw makeError("exactly one prompt source must be configured via --prompt, --prompt-file, or [prompt] in the loop config");
}

function validatePositiveInteger(value: number | null, fieldName: string): number | null {
  if (value === null) {
    return null;
  }

  if (!Number.isInteger(value) || value < 1) {
    throw makeError(`${fieldName} must be an integer greater than or equal to 1`);
  }

  return value;
}

function validateThinking(thinking: string | null): ThinkingLevel | null {
  if (thinking === null) {
    return null;
  }

  if (!VALID_THINKING_LEVELS.has(thinking as ThinkingLevel)) {
    throw makeError(`thinking must be one of: ${Array.from(VALID_THINKING_LEVELS).join(", ")}`);
  }

  return thinking as ThinkingLevel;
}

function resolveReferencedConfigPath(
  cliPath: string | null,
  loopConfigPath: string | null,
  loopPath: string | null,
  cwd: string,
  fieldName: string,
): string {
  const rawPath = cliPath ?? loopPath;
  if (rawPath === null) {
    throw makeError(`${fieldName} config path is required`);
  }

  const baseDir = cliPath !== null || loopConfigPath === null ? cwd : dirname(loopConfigPath);
  return resolveCandidatePath(rawPath, baseDir);
}

export function resolveConfig(options: ResolveConfigOptions): ResolvedConfigBundle {
  const loopConfigState = loadLoopConfig(options);
  const loopConfig = loopConfigState.config;
  const prompt = resolvePrompt(options.cliArgs, loopConfig, loopConfigState.dir, options);

  const maxIterations = validatePositiveInteger(
    options.cliArgs.maxIterations ?? loopConfig?.maxIterations ?? null,
    "max_iterations",
  );
  const contextLimit =
    validatePositiveInteger(options.cliArgs.contextLimit ?? loopConfig?.contextLimit ?? DEFAULT_CONTEXT_LIMIT, "context_limit") ??
    DEFAULT_CONTEXT_LIMIT;
  const completionPromise = options.cliArgs.completionPromise ?? loopConfig?.completionPromise ?? DEFAULT_COMPLETION_PROMISE;
  const thinking = validateThinking(options.cliArgs.thinking ?? loopConfig?.thinking ?? null);
  const outputDir = options.cliArgs.outputDir ?? loopConfig?.outputDir ?? DEFAULT_OUTPUT_DIR;
  const checksConfigPath = resolveReferencedConfigPath(
    options.cliArgs.checksConfigPath,
    loopConfigState.path,
    loopConfig?.checksConfigPath ?? null,
    options.cwd,
    "checks",
  );
  const completionConfigPath = resolveReferencedConfigPath(
    options.cliArgs.completionConfigPath,
    loopConfigState.path,
    loopConfig?.completionConfigPath ?? null,
    options.cwd,
    "completion",
  );

  if (!options.reader.fileExists(checksConfigPath)) {
    throw makeError(`checks config file not found at ${checksConfigPath}`);
  }

  if (!options.reader.fileExists(completionConfigPath)) {
    throw makeError(`completion config file not found at ${completionConfigPath}`);
  }

  let checksConfig;
  try {
    checksConfig = parseChecksConfigToml(options.reader.readText(checksConfigPath));
  } catch (cause) {
    throw makeError(`invalid checks config at ${checksConfigPath}`, cause);
  }

  let completionConfig;
  try {
    completionConfig = parseCompletionConfigToml(options.reader.readText(completionConfigPath));
  } catch (cause) {
    throw makeError(`invalid completion config at ${completionConfigPath}`, cause);
  }

  return {
    runConfig: {
      prompt,
      maxIterations,
      contextLimit,
      completionPromise,
      provider: options.cliArgs.provider ?? loopConfig?.provider ?? null,
      model: options.cliArgs.model ?? loopConfig?.model ?? null,
      thinking,
      outputDir,
      checksConfigPath,
      completionConfigPath,
      projectPath: options.cwd,
    },
    checksConfig,
    completionConfig,
  };
}

export function defaultConfigPathForCwd(cwd: string): string {
  return resolvePath(cwd, DEFAULT_CONFIG_FILE_NAME);
}

export function resolveOutputDir(outputDir: string | null, cwd: string): string | null {
  return outputDir === null ? null : resolveOptionalPath(outputDir, cwd);
}
