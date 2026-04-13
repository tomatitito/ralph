---
id: ral-2q2g
status: open
deps: [ral-b9sm, ral-xtlt]
links: [extensions.md, lifecycle.md, pi-integration.md, internal-contracts.md]
created: 2026-04-13T20:32:10Z
type: task
priority: 1
assignee: Jens Kouros
parent: ral-lp9k
tags: [ralph-loop-ts, pi, extension, lifecycle]
---
# Implement lifecycle marker extension

Implement the internal Pi extension that injects Ralph marker instructions and records task/loop completion markers seen in assistant output.

## Acceptance Criteria

- extension injects marker instructions before agent start
- <ralph:task-complete/> is detected across the iteration
- <ralph:loop-complete/> is detected across the iteration
- normalized iteration state is exposed to the controller

## Implementation Notes

- Implement detection over all assistant text content observed during the iteration, not just the final message.
- Expose normalized lifecycle state such as:
  - `taskComplete: boolean`
  - `loopComplete: boolean`
  - `diagnostics: string[]`
- Allow both markers to be true in the same iteration.
- Ignore malformed marker-like text and add diagnostics when useful.
- Keep the actual prompt injection text centralized so it can be reused in tests and reviewed against the spec.

## Architecture Constraints

- The extension should depend on Pi/runtime hook surfaces, but not on controller logic or artifact-writing concerns.
- Expose only normalized marker state and diagnostics to upstream consumers.
- Avoid direct logging or process-global access from the extension.

## Relevant Spec

- `extensions.md`
- `lifecycle.md`
- `pi-integration.md`
- `internal-contracts.md`

## Suggested Module Shape

- `src/extensions/lifecycle-markers.ts`
- `src/extensions/marker-instructions.ts`
- `src/extensions/extension-types.ts`

## Dependencies on Other Tickets

- Depends on the Pi runtime abstraction from `ral-xtlt`.
- The controller-facing state contract should be compatible with the orchestration work in `ral-gu4t`.

## Out of Scope

- Final success decisions
- Completion validator execution
- Artifact writing

## Verification Notes

- Add tests for task marker only, loop marker only, both markers, and malformed marker-like text.
- Verify markers are detected when emitted in earlier assistant messages within the same iteration.

## Suggested Implementation Checklist

1. Define the normalized lifecycle-marker state contract from `internal-contracts.md`.
2. Centralize the marker instruction text in one module.
3. Start with red/green tests for marker detection semantics:
   - task marker only
   - loop marker only
   - both markers
   - malformed marker-like text ignored
   - markers appearing in earlier assistant messages during the same iteration
4. Implement the smallest marker-detection accumulator that passes those tests.
5. Add the Pi-facing prompt-injection and output-observation hooks around that accumulator.
6. Ensure the extension exposes only normalized marker state and diagnostics.
7. Keep controller logic, artifact concerns, and direct process/logging dependencies out of the extension.
8. Add a focused integration-style test if needed to verify prompt injection and observed assistant text flow through the runtime boundary.
9. Re-run dependency-cruiser to confirm layering still holds.

## Definition of Done Heuristic

This ticket is done when marker instructions are injected consistently, markers are detected across the whole iteration, both markers can co-exist in one iteration state, and the implementation is primarily driven by red/green tests on normalized state behavior.

