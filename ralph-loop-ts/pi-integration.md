# Pi Integration

## Embedding model

`ralph-loop-ts` should embed Pi through the SDK.

Relevant Pi capabilities:
- `createAgentSession()`
- `createAgentSessionRuntime()`
- `DefaultResourceLoader`
- extension factories
- dynamic provider registration
- session persistence and event subscriptions

## Why SDK rather than CLI subprocess

Benefits:
- direct access to session state and events
- native extension hooks
- direct context-usage inspection
- simpler integration of restart logic
- no need to scrape CLI JSON output as an external process

## Core integration responsibilities

The Ralph TS controller should:
- call a runtime function typed as `IterationRuntime`
- construct a fresh Pi session per iteration
- bind the required Ralph extensions
- inject iteration input and carried-forward summary
- await `agent_end`
- read extension-produced state
- run checks and completion validators
- decide whether to terminate or start the next iteration

Implementation style preference:
- prefer plain exported functions over classes for controller/runtime seams
- for example, `runLoopController(...)` and `runPiIteration(...)`
- reserve classes for cases where stateful lifecycle management is materially clearer than closures or explicit state objects

## Session strategy

Version 1 should use fresh sessions per iteration.

Reasoning:
- keeps context small
- fits the desired task-boundary restart model
- avoids dependence on compaction as the primary mechanism

## Provider/model configuration

Pi is the hard-coded runtime abstraction.

What remains configurable:
- provider
- model
- thinking level
- possibly provider overrides via Pi provider registration

This should allow using Anthropic, OpenAI, or other Pi-supported providers without changing the Ralph TS architecture.

## Extension strategy

Extensions should live inside the project and be loaded programmatically by the SDK integration.

This keeps:
- behavior versioned with `ralph-loop-ts`
- runtime assumptions explicit
- testability high
