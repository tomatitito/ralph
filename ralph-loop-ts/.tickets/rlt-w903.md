---
id: rlt-w903
status: open
deps: [ral-m4r6]
links: []
created: 2026-04-14T05:19:19Z
type: task
priority: 1
assignee: Jens Kouros
parent: ral-lp9k
tags: [ralph-loop-ts, vertical-slice, mock, functions]
---
# Implement mock vertical-slice loop execution with function-based seams

Implement a minimal end-to-end vertical slice that runs without Pi/LLM and validates the wiring from CLI/config into runtime, controller, checks, and completion.

The implementation should follow the project preference for plain functions and function types instead of classes or single-method interfaces.

## Design

Implementation approach:
- keep the existing CLI/config path and use `provider=mock` to select the deterministic runtime behavior
- implement the temporary mock runtime directly in `src/runtime/pi-runtime.ts`; it can be replaced later by the real Pi integration
- prefer exported functions such as `runPiIteration(...)`, `runLoopController(...)`, `runChecks(...)`, and `runCompletion(...)`
- keep marker detection simple and deterministic for the first slice
- use this slice to prove restart semantics and end-to-end wiring before introducing live Pi or LLM behavior

Notes:
- this ticket intentionally cuts across runtime/controller/checks/completion as a vertical slice
- it should inform or unblock later work on `ral-xtlt`, `ral-gu4t`, `ral-g5ex`, and `ral-og1z`

## Acceptance Criteria

- running `ralph-ts` with an inline prompt and `--provider mock` executes a deterministic mock loop
- the mock runtime lives in `src/runtime/pi-runtime.ts` for now
- `IterationRuntime`, `ChecksRunner`, `CompletionRunner`, and the loop controller are all function-shaped seams
- iteration 1 prints `mock task 1 completed` and emits `<ralph:task-complete/>`
- iteration 2 prints `mock task 2 completed` and emits `<ralph:task-complete/>`
- iteration 3 prints `mock task 3 completed` and emits both `<ralph:task-complete/>` and `<ralph:loop-complete/>`
- the controller restarts after task-complete iterations and exits successfully after the final loop-complete iteration
- checks run on every iteration through a function-shaped mock implementation
- completion validation runs only on the final loop-complete claim through a function-shaped mock implementation
- no Pi SDK or LLM is required for this slice
- at least one automated test covers the happy-path mock run

