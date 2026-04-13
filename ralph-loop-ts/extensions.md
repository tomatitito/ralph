# Extensions

Version 1 uses separate internal Pi extensions.

These extensions are part of `ralph-loop-ts` itself. They are not primarily designed as reusable public packages.

## Extension set

### 1. Context monitor extension
Purpose:
- inspect context usage during the iteration
- remember the highest observed usage
- determine whether the configured context limit was crossed
- request graceful shutdown when the limit is hit

Likely hooks:
- `turn_end`
- `agent_end`

Relevant Pi API:
- `ctx.getContextUsage()`
- `ctx.shutdown()`

### 2. Lifecycle marker extension
Purpose:
- inject marker instructions into the effective system prompt
- detect structured task and loop completion markers in assistant output
- persist iteration-local marker state

Likely hooks:
- `before_agent_start`
- `message_end`
- `agent_end`

Responsibilities:
- detect `<ralph:task-complete/>`
- detect `<ralph:loop-complete/>`
- record whether either marker appeared during the iteration

## Canonical extension-controller contract

At the end of an iteration, the controller must be able to read this normalized iteration state:

```ts
interface IterationExtensionState {
  contextLimitHit: boolean;
  finalContextTokens: number | null;
  peakContextTokens: number | null;
  taskComplete: boolean;
  loopComplete: boolean;
  diagnostics: string[];
}
```

This contract should be treated as canonical for v1.

## State handoff mechanism

Version 1 should use an in-process shared state bridge owned by the Ralph controller and passed to the extensions at creation time.

Why this is preferred over session-entry persistence:
- simpler to read synchronously at iteration end
- no need to reconstruct transient iteration state from session history
- these values are controller/runtime facts, not user-visible conversational history

Session entries may still be used later for debugging or persistence, but are not the primary contract for v1.

## Context monitor semantics

The context monitor extension must:
- call `ctx.getContextUsage()` at each relevant boundary
- update `finalContextTokens` with the latest observed value
- update `peakContextTokens` with the maximum observed value
- set `contextLimitHit = true` once the configured threshold is crossed
- call `ctx.shutdown()` to stop the session gracefully

### Important semantic note
This replaces the Rust implementation's concurrent monitor process, but preserves the behavioral contract:
- once context usage is too high, the current iteration should stop
- the next iteration should restart from a summary

## Marker extension semantics

The marker extension must:
- teach the agent about the two Ralph markers before work begins
- inspect assistant text content across the entire iteration
- set `taskComplete = true` if `<ralph:task-complete/>` appears anywhere
- set `loopComplete = true` if `<ralph:loop-complete/>` appears anywhere
- allow both values to be true in the same iteration

### Marker guidance in prompt
The extension should add instructions equivalent to:
- emit `<ralph:task-complete/>` when a meaningful task boundary has been reached and the next task should begin in a fresh iteration
- emit `<ralph:loop-complete/>` only when the entire user objective is complete
- do not emit the loop-complete marker unless the whole objective is done

## Error and diagnostic handling

Extensions should accumulate human-readable diagnostics when useful, for example:
- context limit crossed at observed token count X
- malformed marker-like text ignored
- no context usage available from Pi for this turn

These diagnostics should be exposed to the controller and written into artifacts.

## Non-goals for v1

- custom TUI UI
- interactive confirmation dialogs
- extension package publishing
- generalized workflow engine inside Pi

The extensions are narrow Ralph runtime components.
