# Internal Contracts and Architecture Boundaries

This page defines the concrete internal contracts that let `ralph-loop-ts` evolve without collapsing its layering.

It is intentionally more implementation-shaped than the other wiki pages.

## Goals

- keep Pi SDK details behind a narrow runtime boundary
- make controller decisions testable without live Pi sessions
- avoid spreading process-level globals and logging concerns throughout the codebase
- make architectural boundaries enforceable with `dependency-cruiser`
- support red/green TDD with small, injectable units

## Layering rules

Preferred production layering:

```text
cli/controller
  -> config
  -> runtime
  -> checks
  -> completion
  -> artifacts

runtime
  -> extensions

extensions
  -> shared extension types

checks/completion/artifacts/config
  -> shared domain types/helpers
```

Rules:
- controller may depend on runtime; runtime must not depend on controller
- runtime may depend on extensions; extensions must not depend on controller
- checks, completion, and artifacts should not depend on runtime internals
- direct process/platform/logging dependencies should stay at the controller boundary whenever possible
- non-controller modules should receive concrete dependencies explicitly instead of importing globals directly

## Dependency injection guidance

Prefer passing explicit function types or plain-object data dependencies over classes.

Default style for `ralph-loop-ts`:
- use plain data objects for inputs and outputs
- use exported function types for behavior seams
- prefer `type Foo = (...) => ...` over single-method interfaces or classes
- use object-shaped interfaces only when a dependency truly has multiple cohesive operations

Examples:
- `Logger`
- `Clock`
- `FileSystem`
- `CommandExecutor`
- `IterationRuntime`
- `ArtifactWriter`

This keeps leaf modules testable and reduces hidden coupling.

## Dependency-cruiser guidance

At minimum, enforce rules equivalent to:
- forbid imports from `src/runtime/**` to `src/controller/**`
- forbid imports from `src/extensions/**` to `src/controller/**`
- only allow production imports of `src/runtime/**` from the controller layer
- only allow production imports of platform/adaptor modules from the controller layer
- forbid circular dependencies

If a dedicated platform/adaptor directory is introduced, for example `src/platform/`, dependency-cruiser should treat it as restricted infrastructure.

Illustrative ruleset shape:

```js
module.exports = {
  forbidden: [
    {
      name: "runtime-to-controller",
      from: { path: "^src/runtime" },
      to: { path: "^src/controller" },
    },
    {
      name: "extensions-to-controller",
      from: { path: "^src/extensions" },
      to: { path: "^src/controller" },
    },
    {
      name: "runtime-imports-only-from-controller",
      from: {
        path: "^src/(?!controller|testing)",
        pathNot: "^src/runtime",
      },
      to: { path: "^src/runtime" },
    },
    {
      name: "platform-imports-only-from-controller",
      from: { path: "^src/(?!controller|testing)" },
      to: { path: "^src/platform" },
    },
    {
      name: "no-node-process-outside-controller",
      from: { path: "^src/(?!controller|testing)" },
      to: { path: "^node:process$" },
    },
  ],
};
```

Treat this as a starting point and tune it to the actual source layout.

## Core domain contracts

These are suggested internal contracts, not wire formats.

### Prompt source

```ts
export type PromptSource =
  | { kind: "inline"; text: string }
  | { kind: "file"; path: string; text: string };
```

### Resolved run config

```ts
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
```

Notes:
- this is the normalized config the controller should consume
- path resolution and CLI override rules belong upstream of this contract

## Runtime contracts

### Iteration input

```ts
export interface IterationInput {
  iterationNumber: number;
  objective: string;
  handoffSummary: string | null;
  provider: string | null;
  model: string | null;
  thinking: string | null;
  contextLimit: number;
}
```

### Runtime exit reason

```ts
export type RuntimeExitReason =
  | "agent_end"
  | "context_limit_requested"
  | "interrupted"
  | "error";
```

### Iteration runtime result

```ts
export interface IterationRuntimeResult {
  sessionId: string | null;
  exitReason: RuntimeExitReason;
  assistantText: string | null;
  diagnostics: string[];
  extensionState: CombinedExtensionState;
}
```

### Runtime function type

```ts
export type IterationRuntime = (
  input: IterationInput,
) => Promise<IterationRuntimeResult>;
```

Notes:
- Pi SDK types should stay inside the runtime implementation where practical
- controller code should operate on `IterationRuntime`, not raw Pi sessions
- prefer a plain function implementation such as `runPiIteration` over a runtime class

## Extension contracts

### Context extension state

```ts
export interface ContextExtensionState {
  peakTokenCount: number | null;
  finalTokenCount: number | null;
  contextLimitHit: boolean;
  diagnostics: string[];
}
```

### Lifecycle marker state

```ts
export interface LifecycleMarkerState {
  taskComplete: boolean;
  loopComplete: boolean;
  diagnostics: string[];
}
```

### Combined extension state

```ts
export interface CombinedExtensionState {
  context: ContextExtensionState;
  lifecycle: LifecycleMarkerState;
}
```

Notes:
- extensions should expose normalized state only
- controller logic should not inspect raw Pi hook payloads

## Checks contracts

### Check hook

```ts
export type CheckHook = "after_iteration";
```

### Command result

```ts
export interface CommandExecutionResult {
  command: string;
  cwd: string | null;
  exitCode: number | null;
  stdout: string;
  stderr: string;
  timedOut: boolean;
}
```

### Check result

```ts
export interface CheckResult {
  name: string;
  hook: CheckHook;
  execution: CommandExecutionResult;
  passed: boolean;
}
```

### Hook aggregate

```ts
export interface CheckHookResult {
  hook: CheckHook;
  executed: boolean;
  passed: boolean;
  results: CheckResult[];
}
```

### Checks runner

```ts
export type ChecksRunner = () => Promise<CheckHookResult>;
```

## Completion contracts

### Completion status

```ts
export type CompletionValidationStatus = "skipped" | "passed" | "failed";
```

### Completion result

```ts
export interface CompletionValidationResult {
  status: CompletionValidationStatus;
  results: CheckResult[];
}
```

### Completion runner

```ts
export type CompletionRunner = () => Promise<CompletionValidationResult>;
```

Notes:
- completion may reuse the same command-execution result shape as checks
- `skipped` is a first-class state

## Controller contracts

### Iteration evaluation input

```ts
export interface IterationEvaluationInput {
  iterationNumber: number;
  runtime: IterationRuntimeResult;
  afterIterationChecks: CheckHookResult;
  completion: CompletionValidationResult;
}
```

### Iteration decision

```ts
export type IterationDecision =
  | "success"
  | "restart_task_boundary"
  | "restart_failed_completion"
  | "restart_context_limit"
  | "restart_incomplete"
  | "interrupted"
  | "max_iterations_exceeded"
  | "error";
```

### Run exit reason

```ts
export type RunExitReason =
  | "loop_completed"
  | "max_iterations_exceeded"
  | "interrupted"
  | "error";
```

### Iteration summary

```ts
export interface IterationSummary {
  iterationNumber: number;
  summaryText: string;
}
```

Notes:
- the decision function should be as pure as practical
- artifact writing should happen from controller-owned normalized data, not by re-deriving raw runtime state elsewhere
- the main loop controller should preferably be an exported function such as `runLoopController(...)` rather than a class with a single `run()` method

## Artifact contracts

### Run status

```ts
export type RunStatus = "running" | "completed" | "failed" | "interrupted";
```

### Artifact writer

```ts
export interface ArtifactWriter {
  writeRunStart(input: {
    config: ResolvedRunConfig;
    runId: string;
    startedAt: string;
  }): Promise<void>;

  writeIteration(input: {
    evaluation: IterationEvaluationInput;
    summary: IterationSummary | null;
  }): Promise<void>;

  writeRunEnd(input: {
    runId: string;
    status: RunStatus;
    exitReason: RunExitReason;
    completedAt: string;
  }): Promise<void>;
}
```

Notes:
- file naming/path derivation belongs inside the artifact writer layer
- callers should pass normalized data, not construct artifact paths directly

## Logging contract

Suggested minimal logger abstraction:

```ts
export interface Logger {
  debug(message: string, fields?: Record<string, unknown>): void;
  info(message: string, fields?: Record<string, unknown>): void;
  warn(message: string, fields?: Record<string, unknown>): void;
  error(message: string, fields?: Record<string, unknown>): void;
}
```

Guideline:
- do not import a concrete logger into leaf modules
- pass a `Logger` dependency when logging is needed
- if a module does not truly need logging, prefer returning diagnostics instead

## Platform dependencies

If needed, centralize platform-facing behavior in narrow adapters.

Examples:
- `ProcessInfo` for cwd/env/argv lookups
- `Clock` for timestamps
- `FileSystem` for file IO
- `CommandExecutor` for shell commands

These should be wired by the controller layer, then passed inward.

## Testing implications

These contracts are designed so tests can:
- stub `IterationRuntime` without Pi
- stub `ChecksRunner` and `CompletionRunner`
- stub `ArtifactWriter`
- validate controller decisions with table-driven inputs
- run dependency-cruiser in CI as an architecture regression check
- support red/green TDD by keeping most business logic isolated from concrete infrastructure

For the first vertical slice, prefer hardcoded function implementations over scaffolding classes. For example:
- a mock `IterationRuntime` function that emits three deterministic iterations
- a trivial `ChecksRunner` function that returns `after_iteration` success
- a trivial `CompletionRunner` function that returns success when invoked
