---
id: ral-xtlt
status: open
deps: [ral-b9sm, ral-m4r6]
links: [pi-integration.md, lifecycle.md, internal-contracts.md]
created: 2026-04-13T20:32:09Z
type: task
priority: 1
assignee: Jens Kouros
parent: ral-lp9k
tags: [ralph-loop-ts, pi, sdk, runtime]
---
# Embed Pi SDK runtime for fresh-session iterations

Build the core runtime that creates fresh Pi sessions for each Ralph iteration and feeds handoff summaries forward.

## Acceptance Criteria

- Pi is embedded through the SDK, not via pi CLI subprocess orchestration
- one fresh session is created per iteration
- the original objective plus handoff summary are injected each time
- the runtime treats `agent_end` as the canonical end-of-iteration event
- runtime exit reasons are normalized to the controller-facing contract
- session metadata is returned on a best-effort basis and missing metadata becomes diagnostics rather than hard failure
- iteration count and max-iteration failure behavior follow the spec

## Implementation Notes

- Define a thin runtime boundary so the controller does not depend directly on raw Pi SDK details.
- Hide Pi SDK details behind a narrow adapter interface and keep Pi-specific types from leaking into controller logic.
- Introduce explicit iteration-level types, for example:
  - `IterationInput { iterationNumber, objective, handoffSummary, provider, model, thinking }`
  - `IterationRuntimeResult { sessionId, assistantText?, exitReason, diagnostics }`
- The runtime should own:
  - Pi runtime creation
  - fresh session creation per iteration
  - extension registration/binding
  - waiting for `agent_end`
  - extracting session identifiers and other run-local metadata
- Canonical end-of-iteration semantics:
  - the controller evaluates an iteration only after the runtime observes `agent_end`
  - extension-triggered shutdown requests, such as context-limit shutdown, are causes of termination but are not themselves the iteration boundary
  - any final extension state should be read after `agent_end` so the controller sees the settled iteration result
- Normalize runtime completion into only these controller-facing reasons:
  - `agent_end`
  - `context_limit_requested`
  - `interrupted`
  - `error`
- Session metadata contract:
  - `sessionId` should be populated when available
  - any session file/path/export reference is optional
  - missing session metadata must not fail the iteration; emit diagnostics instead
- The controller should own:
  - iteration counting
  - restart decisions
  - handoff summary generation/forwarding
  - max-iteration enforcement
- Make handoff summary injection deterministic and explicit so later tests can assert on it.

## Architecture Constraints

- `src/runtime/**` must not import from `src/controller/**`.
- Only the controller layer should depend on the runtime package directly; runtime should expose an interface, not reach upward for decisions.
- Avoid direct use of `process`, ad hoc logging, or other host globals inside runtime internals; inject any required logger or platform helpers.
- Add or extend dependency-cruiser rules to enforce the controller/runtime layering and Pi-detail encapsulation.

## Relevant Spec

- `pi-integration.md`
- `lifecycle.md`
- `internal-contracts.md`

## Suggested Module Shape

- `src/runtime/pi-runtime.ts`
- `src/runtime/iteration-runner.ts`
- `src/runtime/session-input.ts`
- `src/runtime/runtime-types.ts`

## Resolved Decisions

- The canonical end of an iteration is Pi `agent_end`.
- Context-limit shutdown requests are recorded in extension state and may influence normalized runtime exit reason, but controller evaluation still begins only after `agent_end`.
- Non-successful session termination is normalized into the controller-friendly exit reasons defined in `internal-contracts.md`.
- Session metadata for artifacts is best-effort rather than guaranteed; `sessionId` is preferred when available and any missing metadata is surfaced as diagnostics instead of hard failure.

## Out of Scope

- Post-iteration checks/completion decisions
- Artifact persistence beyond returning data needed by later layers
- Marker/context logic beyond binding the required extensions

## Verification Notes

- Use a fake or adapter-backed runtime in tests so controller tests do not require live Pi integration.
- Add at least one integration-style test proving fresh-session-per-iteration behavior.

## Suggested Implementation Checklist

1. Define the runtime-facing interfaces from `internal-contracts.md`, especially:
   - `IterationInput`
   - `IterationRuntimeResult`
   - `IterationRuntime`
2. Create a fake runtime implementation first for tests so downstream controller work can proceed without live Pi integration.
3. Add a failing contract test for `IterationRuntime` behavior:
   - one invocation corresponds to one fresh iteration/session result
   - handoff summary is accepted as input
   - normalized exit reasons are returned
4. Implement a thin Pi adapter layer that owns Pi SDK setup and hides raw Pi types.
5. Implement per-iteration fresh session creation.
6. Bind the internal Ralph extensions through the runtime layer.
7. Normalize Pi session completion into the runtime result contract, including:
   - session id when available
   - diagnostics
   - extension state bundle
   - normalized exit reason
8. Ensure the runtime does not import controller code and does not reach for process/logging globals directly.
9. Add or tighten dependency-cruiser coverage if new runtime submodules or platform adapters are introduced.
10. Add at least one integration-style test against the real adapter boundary if feasible; otherwise keep the adapter small and heavily contract-tested.

## Definition of Done Heuristic

This ticket is done when the runtime can run a single fresh iteration through a controller-friendly interface, hide Pi details from callers, satisfy the dependency boundaries, and be driven primarily through red/green contract tests.

