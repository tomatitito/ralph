---
id: ral-og1z
status: closed
deps: [ral-m4r6]
links: [configuration.md, lifecycle.md, internal-contracts.md]
created: 2026-04-13T20:32:10Z
type: task
priority: 2
assignee: Jens Kouros
parent: ral-lp9k
tags: [ralph-loop-ts, completion, commands]
---
# Replace the mock completion runner with real validation from ralph-completion.toml

Upgrade the function-shaped completion runner introduced by the mock vertical slice so it executes completion validators when the loop-complete marker is claimed and aggregates their outcomes.

## Acceptance Criteria

- on_loop_complete_claim validators are supported
- validator success rules match the config spec
- aggregate completionValidated state is computed
- validator results are returned in a persistable structured form even when skipped or failed
- validators run in file order without short-circuiting after failures

## Implementation Notes

- `rlt-w903` already introduced the function-shaped completion seam and a trivial happy-path implementation; this ticket replaces that stub with real validation semantics.
- Reuse the same command-execution semantics as the checks runner where practical.
- This ticket should focus on the completion-specific hook and aggregate result semantics:
  - validators run only on a loop-complete claim
  - validators run in file order
  - validators do not short-circuit on failure
  - overall completion validation passes only if all validators pass
- Introduce an explicit skipped state for iterations where no loop-complete claim occurred.
- Return a normalized result that the controller can interpret without needing to know command details.
- Artifact persistence is owned by `ral-l3ri`; this ticket owns validation execution and normalized result production only.

## Architecture Constraints

- Completion validation should reuse shared command-execution abstractions where practical.
- It should not depend on runtime internals, marker-detection internals, or direct process-global state.
- If diagnostics or logging are needed, pass those dependencies explicitly.

## Relevant Spec

- `configuration.md`
- `lifecycle.md`
- `internal-contracts.md`

## Suggested Module Shape

- `src/completion/completion-runner.ts`
- `src/completion/completion-types.ts`
- shared command runner reused from checks if appropriate

## Dependencies on Other Tickets

- Consumes normalized completion config from `ral-m4r6`.
- Builds on the seam established by `rlt-w903`.
- Its output feeds controller decisions in `ral-gu4t` and persistence in `ral-l3ri`.

## Out of Scope

- Detecting loop-complete markers
- Deciding final run success outside of exposing aggregate validation outcome

## Verification Notes

- Add tests for skipped, passing, and failing completion-validation runs.
- Verify results are still emitted when validation is skipped due to no loop-complete claim.

## Suggested Implementation Checklist

1. Start from the completion-validation result contract already introduced in the vertical slice, including the explicit `skipped` state.
2. Reuse the shared command-execution abstraction from the checks runner where practical.
3. Add red/green tests for completion-specific semantics:
   - skipped when no loop-complete claim is present
   - pass when all validators pass
   - fail when any validator fails
   - execution continues across failing validators
4. Implement the smallest completion runner that satisfies those tests.
5. Keep the completion runner focused on validation semantics only; marker detection remains outside this layer.
6. Return normalized results that the controller and artifact writer can consume directly.
7. Verify dependency boundaries remain intact and no runtime/global dependencies leaked in.

## Definition of Done Heuristic

This ticket is done when the current stub completion runner has been replaced by deterministic validator execution, skipped/passed/failed cases behave correctly, shared command execution is reused where sensible, and the controller can consume the normalized result without knowing command-level details.

## Notes

**2026-04-14T22:33:45Z**

Implemented real command-backed completion validation under src/guards, simplified controller invocation to thunk-based runner, updated docs/tests, committed in 6902f2a and pushed to origin/main.
