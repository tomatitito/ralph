export type PromptSource =
  | { kind: "inline"; text: string }
  | { kind: "file"; path: string; text: string };

export type ThinkingLevel = "off" | "minimal" | "low" | "medium" | "high" | "xhigh";

export interface ResolvedRunConfig {
  prompt: PromptSource;
  maxIterations: number | null;
  contextLimit: number;
  completionPromise: string | null;
  provider: string | null;
  model: string | null;
  thinking: ThinkingLevel | null;
  outputDir: string | null;
  checksConfigPath: string | null;
  completionConfigPath: string | null;
  projectPath: string;
}

export interface RawCliArgs {
  promptText: string | null;
  promptFile: string | null;
  maxIterations: number | null;
  completionPromise: string | null;
  outputDir: string | null;
  contextLimit: number | null;
  configPath: string | null;
  checksConfigPath: string | null;
  completionConfigPath: string | null;
  provider: string | null;
  model: string | null;
  thinking: string | null;
}

export interface LoopConfig {
  promptText: string | null;
  promptFile: string | null;
  maxIterations: number | null;
  contextLimit: number | null;
  completionPromise: string | null;
  provider: string | null;
  model: string | null;
  thinking: string | null;
  outputDir: string | null;
  checksConfigPath: string | null;
  completionConfigPath: string | null;
}

export interface CommandConfig {
  name: string;
  command: string;
  cwd: string | null;
  timeoutSeconds: number | null;
  requiredExitCode: number;
  requiredStdout: string | null;
  requiredStderr: string | null;
  env: Record<string, string>;
}

export interface ChecksConfig {
  afterIteration: CommandConfig[];
  beforeFinalSuccess: CommandConfig[];
}

export interface CompletionConfig {
  onLoopCompleteClaim: CommandConfig[];
}

export interface ResolvedConfigBundle {
  runConfig: ResolvedRunConfig;
  checksConfig: ChecksConfig;
  completionConfig: CompletionConfig;
}
