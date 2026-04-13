---
id: ral-gu4t
status: open
deps: [ral-m4r6, ral-xtlt, ral-a62g, ral-2q2g]
links: [lifecycle.md, overview.md, artifacts.md, internal-contracts.md]
created: 2026-04-13T20:32:10Z
type: task
priority: 1
assignee: Jens Kouros
parent: ral-lp9k
tags: [ralph-loop-ts, loop, lifecycle]
---
# Implement iteration orchestration and decision logic

Implement the post-iteration evaluation order, restart semantics, precedence rules, and success/failure decisions from the spec.

## Acceptance Criteria

- after_iteration checks run after every iteration
- completion validators run only on loop-complete claims after checks pass
- before_final_success checks run only after validators pass
- restart and termination decisions follow the lifecycle spec

## Implementation Notes

- This ticket should introduce the main controller/state-machine layer for TS Ralph.
- Encode the canonical post-iteration order explicitly in code:
  1. read extension state
  2. run `after_iteration` checks
  3. run completion validators only for loop-complete claims when checks passed
  4. run `before_final_success` checks only after completion validation passed
  5. write iteration artifacts
  6. decide next state
- Introduce explicit decision/result types, for example:
  - `IterationDecision = success | restart_task_boundary | restart_failed_completion | restart_context_limit | restart_incomplete | interrupted | max_iterations_exceeded`
  - `RunExitReason = loop_completed | max_iterations_exceeded | interrupted | error`
- Keep decision logic pure where practical so it can be tested without live Pi sessions.
- The controller is the integration boundary and may own direct dependencies on runtime adapters, process termination, and logging adapters.
- The controller should consume normalized outputs from:
  - runtime/session layer
  - context extension
  - lifecycle marker extension
  - checks runner
  - completion runner
  - artifact writer

## Architecture Constraints

- The controller may depend on the runtime abstraction; the runtime must not depend on the controller.
- Direct use of process-level globals and concrete logging backends should be confined to the controller layer and passed downward as dependencies when needed elsewhere.
- Add dependency-cruiser rules that enforce:
  - no imports from `src/runtime/**` to `src/controller/**`
  - no direct imports of platform/adaptor modules from non-controller leaf modules
  - controller as the only production layer allowed to wire runtime, checks, completion, and artifacts together

## Relevant Spec

- `lifecycle.md`
- `overview.md`
- `artifacts.md`
- `internal-contracts.md`

## Suggested Module Shape

- `src/controller/loop-controller.ts`
- `src/controller/decision.ts`
- `src/controller/controller-types.ts`
- `src/controller/handoff-summary.ts`

## Dependencies on Other Tickets

- This ticket is the integration point for runtime and extension outputs.
- It should define the contracts later consumed by artifacts and tests.

## Out of Scope

- Parsing config files
- Implementing the internals of the checks/completion runners
- The detailed file-format logic of artifact persistence

## Verification Notes

- Add table-driven tests for precedence cases from `lifecycle.md`.
- Cover at least:
  - loop marker + failed checks
  - task marker only
  - loop marker + context-limit hit
  - no marker + checks pass
  - successful loop completion

## Suggested Implementation Checklist

1. Define the controller-facing contracts from `internal-contracts.md`, especially:
   - iteration evaluation input
   - iteration decision
   - run exit reason
   - iteration summary contract
2. Start with pure decision tests before wiring any real runtime behavior.
3. Add failing table-driven tests for the canonical lifecycle cases from `lifecycle.md`.
4. Implement the smallest pure decision function that passes those tests.
5. Add controller orchestration tests that verify the exact post-iteration order:
   - read extension state
   - run `after_iteration` checks
   - run completion validators when eligible
   - run `before_final_success` checks when eligible
   - write artifacts
   - decide next state
6. Introduce the loop controller that coordinates dependencies through interfaces rather than concrete implementations.
7. Ensure the controller is the only production layer wiring together:
   - runtime
   - checks runner
   - completion runner
   - artifact writer
   - platform/logging dependencies
8. Keep decision logic separate from side-effectful orchestration so most lifecycle behavior remains unit-testable.
9. Add red/green tests for interruption and max-iteration handling.
10. Re-run dependency-cruiser after wiring the controller to confirm boundary rules still hold.

## Definition of Done Heuristic

This ticket is done when lifecycle decisions are driven by table-driven tests, orchestration follows the canonical evaluation order, and the controller cleanly integrates runtime/checks/completion/artifacts without leaking those dependencies back into lower layers.

