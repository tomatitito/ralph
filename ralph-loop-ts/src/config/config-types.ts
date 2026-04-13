export type PromptSource =
  | { kind: "inline"; text: string }
  | { kind: "file"; path: string; text: string };

export interface ResolvedRunConfig {
  prompt: PromptSource;
  maxIterations: number | null;
  contextLimit: number;
  completionPromise: string | null;
  provider: string | null;
  model: string | null;
  thinking: string | null;
  outputDir: string | null;
  checksConfigPath: string | null;
  completionConfigPath: string | null;
  projectPath: string;
}
