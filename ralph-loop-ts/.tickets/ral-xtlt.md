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
# Replace the mock runtime with a Pi SDK runtime for fresh-session iterations

Replace the temporary mock runtime in `src/runtime/pi-runtime.ts` with a real Pi-backed implementation that creates fresh sessions for each Ralph iteration and feeds handoff summaries forward.

## Acceptance Criteria

- Pi is embedded through the SDK, not via pi CLI subprocess orchestration
- the existing `IterationRuntime` seam remains the controller-facing contract
- the temporary mock behavior in `src/runtime/pi-runtime.ts` is replaced or cleanly split behind the same function-shaped runtime contract
- one fresh session is created per iteration
- the original objective plus handoff summary are injected each time
- the runtime treats `agent_end` as the canonical end-of-iteration event
- runtime exit reasons are normalized to the controller-facing contract
- session metadata is returned on a best-effort basis and missing metadata becomes diagnostics rather than hard failure
- controller tests can continue to use fake/mock runtimes without requiring live Pi

## Implementation Notes

- `rlt-w903` already established a temporary mock vertical slice and the function-shaped runtime seam; this ticket upgrades that seam to a real Pi implementation rather than introducing it from scratch.
- Define a thin runtime boundary so the controller does not depend directly on raw Pi SDK details.
- Hide Pi SDK details behind a narrow adapter interface and keep Pi-specific types from leaking into controller logic.
- Preserve the existing iteration-level contract shape, for example:
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

- Keep using a fake or adapter-backed runtime in controller tests so lifecycle work does not require live Pi integration.
- Add at least one integration-style test proving fresh-session-per-iteration behavior for the real Pi-backed adapter.

## Suggested Implementation Checklist

1. Start from the seam introduced by `rlt-w903`, especially:
   - `IterationInput`
   - `IterationRuntimeResult`
   - `IterationRuntime`
2. Keep the mock/fake runtime path available for tests so downstream controller work can proceed without live Pi integration.
3. Add a failing contract test for real-runtime behavior:
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
10. Add at least one integration-style test against the real adapter boundary while keeping the adapter small and heavily contract-tested.

## Definition of Done Heuristic

This ticket is done when the temporary mock runtime has been replaced by a real Pi-backed runtime behind the same controller-friendly interface, Pi details remain hidden from callers, dependency boundaries still hold, and the runtime is covered by contract tests plus at least one focused integration-style test.
