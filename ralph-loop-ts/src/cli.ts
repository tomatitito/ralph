import { readFileSync, existsSync } from "node:fs";
import { resolve as resolvePath } from "node:path";

import type { RawCliArgs } from "./config/config-types.ts";
import { resolveConfig } from "./config/resolve-config.ts";

export interface RunCliOptions {
  cwd?: string;
}

function parseIntegerFlag(flagName: string, rawValue: string): number {
  const parsed = Number.parseInt(rawValue, 10);
  if (!Number.isInteger(parsed)) {
    throw new Error(`${flagName} must be an integer`);
  }

  return parsed;
}

export function parseCliArgs(args: readonly string[]): RawCliArgs {
  const parsed: RawCliArgs = {
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
  };

  for (let index = 0; index < args.length; index += 1) {
    const arg = args[index];
    if (arg === undefined) {
      continue;
    }

    const next = args[index + 1];
    const requireValue = (flagName: string): string => {
      if (next === undefined) {
        throw new Error(`${flagName} requires a value`);
      }
      index += 1;
      return next;
    };

    switch (arg) {
      case "-p":
      case "--prompt":
        parsed.promptText = requireValue(arg);
        break;
      case "-f":
      case "--prompt-file":
        parsed.promptFile = requireValue(arg);
        break;
      case "-m":
      case "--max-iterations":
        parsed.maxIterations = parseIntegerFlag(arg, requireValue(arg));
        break;
      case "-c":
      case "--completion-promise":
        parsed.completionPromise = requireValue(arg);
        break;
      case "-o":
      case "--output-dir":
        parsed.outputDir = requireValue(arg);
        break;
      case "--context-limit":
        parsed.contextLimit = parseIntegerFlag(arg, requireValue(arg));
        break;
      case "--config":
        parsed.configPath = requireValue(arg);
        break;
      case "--checks-config":
        parsed.checksConfigPath = requireValue(arg);
        break;
      case "--completion-config":
        parsed.completionConfigPath = requireValue(arg);
        break;
      case "--provider":
        parsed.provider = requireValue(arg);
        break;
      case "--model":
        parsed.model = requireValue(arg);
        break;
      case "--thinking":
        parsed.thinking = requireValue(arg);
        break;
      default:
        throw new Error(`unknown argument: ${arg}`);
    }
  }

  return parsed;
}

export function runCli(args: readonly string[] = [], options: RunCliOptions = {}): string {
  const cliArgs = parseCliArgs(args);
  const resolved = resolveConfig({
    cliArgs,
    cwd: options.cwd ?? resolvePath("."),
    reader: {
      fileExists: existsSync,
      readText: (path) => readFileSync(path, "utf8"),
    },
  });

  return JSON.stringify(resolved, null, 2);
}
