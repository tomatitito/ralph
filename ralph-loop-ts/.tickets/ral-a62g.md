---
id: ral-a62g
status: open
deps: [ral-b9sm, ral-xtlt]
links: [extensions.md, lifecycle.md, pi-integration.md, internal-contracts.md]
created: 2026-04-13T20:32:10Z
type: task
priority: 1
assignee: Jens Kouros
parent: ral-lp9k
tags: [ralph-loop-ts, pi, extension, context]
---
# Implement context monitor extension

Implement the internal Pi extension that watches context usage, records peak/final values, and requests shutdown when the configured limit is hit.

## Acceptance Criteria

- extension reads context usage from Pi hooks at `turn_end` and `agent_end`
- peak and final token counts are recorded
- `contextLimitHit` becomes true when observed usage is greater than or equal to the configured limit
- contextLimitHit is exposed through the controller-extension contract
- hitting the threshold causes graceful session shutdown
- missing usage observations are surfaced as diagnostics rather than hard failure

## Implementation Notes

- Define a normalized extension state contract rather than exposing raw Pi hook payloads.
- The context extension should expose fields such as:
  - `peakTokenCount: number | null`
  - `finalTokenCount: number | null`
  - `contextLimitHit: boolean`
  - `diagnostics: string[]`
- Sample context usage at each `turn_end` and again at `agent_end`.
- Record both peak and final values even when the limit is not hit.
- Treat the threshold as inclusive: the limit is hit when observed usage is `>= contextLimit`.
- If Pi does not provide usage for a given observation point, record a diagnostic rather than failing silently.
- Missing usage data is diagnostic-only and should not fail the iteration by itself.
- Shutdown requests should be idempotent; once the threshold is crossed, repeated observations should not change the semantic outcome beyond updating diagnostics if useful.
- "Graceful shutdown" should mean requesting session termination through the Pi runtime path intended for orderly completion, not abruptly killing an external process.

## Architecture Constraints

- Keep Pi hook payload handling localized to the extension/runtime boundary.
- The extension should expose normalized state only and should not import controller decision logic.
- Avoid direct logging or process-global access from the extension; surface diagnostics through return state instead.

## Relevant Spec

- `extensions.md`
- `lifecycle.md`
- `pi-integration.md`
- `internal-contracts.md`

## Suggested Module Shape

- `src/extensions/context-monitor.ts`
- `src/extensions/extension-types.ts`

## Dependencies on Other Tickets

- Depends on the Pi runtime abstraction from `ral-xtlt`; avoid coupling this extension directly to the controller.

## Out of Scope

- Deciding whether the loop restarts after a context-limit hit
- Artifact writing
- Marker detection

## Verification Notes

- Add tests for: no usage observed, usage observed below threshold, and threshold hit.
- Verify that repeated usage updates preserve the maximum observed value.

## Suggested Implementation Checklist

1. Define the normalized context-extension state contract from `internal-contracts.md`.
2. Start with red/green tests for state accumulation:
   - no usage observed
   - single usage observation
   - multiple usage observations with peak tracking
   - threshold hit
   - missing usage diagnostics
3. Implement the smallest state accumulator that passes those tests.
4. Add the Pi-facing extension hook integration around that accumulator.
5. Implement graceful shutdown request behavior when the configured threshold is crossed.
6. Ensure the extension returns normalized state and diagnostics only; do not import controller logic or artifact-writing concerns.
7. Keep direct logging/process-global access out of the extension.
8. Add a focused integration-style test if needed to confirm the extension can receive usage data from the runtime hook surface.
9. Re-run dependency-cruiser to confirm extension/controller boundaries remain intact.

## Definition of Done Heuristic

This ticket is done when context observations are normalized into a controller-friendly state contract, threshold crossing reliably triggers a graceful shutdown request, and most behavior is covered by red/green tests without depending on controller logic.

