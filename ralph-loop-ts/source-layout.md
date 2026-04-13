# Source Layout Contract

This page defines the intended `src/` layout for the initial `ralph-loop-ts` scaffold.

It exists to make `ral-b9sm` implementation-ready and to keep future tickets aligned with the architecture boundaries.

## Design goals

- controller is the only production layer that wires everything together
- runtime hides Pi SDK details behind a narrow interface
- leaf modules stay testable through dependency injection
- source layout should map cleanly to dependency-cruiser rules
- the project should support red/green TDD from the first ticket onward

## Proposed layout

```text
src/
  index.ts
  cli.ts

  controller/
    loop-controller.ts
    decision.ts
    handoff-summary.ts
    controller-types.ts

  config/
    loop-config.ts
    checks-config.ts
    completion-config.ts
    resolve-config.ts
    config-types.ts

  runtime/
    iteration-runtime.ts
    pi-runtime.ts
    runtime-types.ts
    session-input.ts

  extensions/
    context-monitor.ts
    lifecycle-markers.ts
    marker-instructions.ts
    extension-types.ts

  checks/
    check-runner.ts
    command-runner.ts
    check-types.ts

  completion/
    completion-runner.ts
    completion-types.ts

  artifacts/
    artifact-writer.ts
    path-layout.ts
    run-json.ts
    iteration-files.ts

  platform/
    logger.ts
    filesystem.ts
    clock.ts
    command-executor.ts
    process-info.ts

  shared/
    result.ts
    errors.ts
    text.ts

  testing/
    fakes/
    fixtures/
```

## Responsibilities by directory

### `src/index.ts`
- executable entrypoint wiring
- minimal startup/bootstrap
- may call into `cli.ts`

### `src/cli.ts`
- reads CLI args
- reads process-level inputs
- reports user-facing errors and exit codes
- constructs controller dependencies

### `src/controller/`
- owns orchestration and lifecycle decisions
- is the only production layer allowed to import runtime modules directly
- is the only production layer allowed to import concrete platform adapters directly
- consumes normalized contracts from other layers

### `src/config/`
- parsing
- normalization
- validation
- path resolution
- should be mostly pure

### `src/runtime/`
- hides Pi SDK details
- creates fresh sessions per iteration
- binds extensions
- returns normalized runtime results
- must not import controller modules

### `src/extensions/`
- Pi-facing extension logic only
- marker/context state capture
- normalized state output
- must not import controller modules

### `src/checks/`
- executes checks hooks
- remains runtime-agnostic
- can depend on injected command executors/loggers if needed

### `src/completion/`
- executes completion validators
- remains runtime-agnostic
- can reuse checks command execution abstractions

### `src/artifacts/`
- path derivation
- file writing
- run/iteration artifact serialization
- consumes normalized data; does not import runtime internals

### `src/platform/`
- narrow host abstractions only
- filesystem, clock, logger, command execution, process information
- should not contain business logic
- should be imported directly only by the controller in production code

### `src/shared/`
- tiny domain-neutral helpers
- avoid turning this into a dumping ground

### `src/testing/`
- fakes
- fixtures
- test helpers
- dependency-cruiser rules may allow additional imports here for pragmatic testing

## Import rules summary

Allowed production flow:

```text
controller -> runtime
controller -> config
controller -> checks
controller -> completion
controller -> artifacts
controller -> platform
runtime -> extensions
checks -> shared
completion -> shared
artifacts -> shared
config -> shared
```

Disallowed production flow:
- runtime -> controller
- extensions -> controller
- checks -> runtime
- completion -> runtime
- artifacts -> runtime
- leaf modules -> platform
- leaf modules -> direct `process` access

## TDD guidance

Use red/green TDD by default.

Recommended rhythm:
1. write a failing test for one behavior or one boundary rule
2. implement the smallest change that makes it pass
3. refactor while keeping tests green

For early tickets:
- `ral-b9sm`: start with scaffold tests and dependency-cruiser invocation
- `ral-m4r6`: write table-driven config parsing tests first
- `ral-xtlt`: start with a fake `IterationRuntime` contract test, then implement the Pi adapter
- `ral-gu4t`: drive the controller with pure decision tests first
- `ral-g5ex` / `ral-og1z`: begin with command-runner and aggregation tests
- `ral-l3ri`: begin with path-layout and file-output tests

## Suggested initial test files

```text
src/controller/decision.test.ts
src/config/resolve-config.test.ts
src/checks/check-runner.test.ts
src/completion/completion-runner.test.ts
src/artifacts/path-layout.test.ts
src/testing/architecture.test.ts
```

## Architecture test suggestion

Add a test or CI step that executes dependency-cruiser and fails on boundary violations.

That keeps the architecture under red/green pressure just like behavior-level tests.
