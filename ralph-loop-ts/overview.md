# Project Overview

## Goal

`ralph-loop-ts` is a TypeScript implementation of the Ralph Loop built on Pi's SDK and extension system.

It should be as close as practical to `ralph-loop-rs` in external behavior, while using Pi-native mechanisms where that yields a simpler design.

## Required capabilities

Version 1 should support:

- prompt file input
- inline prompt input
- max iterations
- completion promise / completion markers
- output directory and persistent run artifacts
- separate loop/checks/completion config files
- context/token limit handling
- restart with summary between iterations
- checks after every iteration
- fresh Pi session after each meaningful boundary

## Major design choices

### Runtime
- The agent runtime is Pi.
- The implementation is embedded via Pi SDK APIs, not by shelling out to the `pi` CLI.
- Model provider selection is configurable.
- Pi itself is the fixed agent runtime abstraction.

### Iteration model
An iteration is one Pi run from initial prompt delivery until Pi becomes idle and the iteration is evaluated.

An iteration may end because:
- the agent marks a task as complete
- the agent marks the overall loop as complete
- the context limit is reached
- checks fail
- a regular agent run ends without completion
- an error occurs

### Restart model
A fresh Pi session should be started after:
- task completion
- loop-completion claim that does not yet validate
- context-limit stop
- check failure

The next session receives a compact structured summary rather than the full previous context.

### Success model
A run is successful only if all of the following are true:
- the agent emits the loop-complete marker
- iteration checks pass
- completion validation passes
- any final-success checks pass

## Relationship to Rust implementation

`ralph-loop-rs` is the baseline for external behavior and CLI shape.

Expected differences in `ralph-loop-ts`:
- context monitoring is performed through Pi extension hooks, not a separate concurrent monitor process
- task boundaries become first-class in the loop lifecycle
- checks and completion validation become first-class features
- run artifacts live under `~/.ralph-loop/` rather than a repo-local output directory by default

## Relationship to TLA spec

The TLA specification remains the behavioral reference point for core loop semantics:
- bounded vs unbounded iteration behavior
- success only when completion conditions hold
- context-limit pressure causes shutdown/restart behavior
- max iteration bound is respected when configured

The TS implementation extends the model with:
- task-complete boundaries
- post-iteration checks
- explicit completion validation

Those extensions should preserve the spirit of the original loop semantics.
