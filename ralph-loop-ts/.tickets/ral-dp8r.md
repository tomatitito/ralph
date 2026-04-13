---
id: ral-dp8r
status: open
deps: [ral-gu4t, ral-g5ex, ral-og1z]
links: [lifecycle.md, overview.md, internal-contracts.md]
created: 2026-04-13T20:32:10Z
type: task
priority: 2
assignee: Jens Kouros
parent: ral-lp9k
tags: [ralph-loop-ts, tests, lifecycle]
---
# Add tests for Ralph TS lifecycle semantics

Add automated tests covering max iterations, context-limit restarts, task boundaries, failed checks, and successful completion.

## Acceptance Criteria

- tests cover max-iteration failure
- tests cover context-limit restart
- tests cover task-boundary restart
- tests cover failed checks causing restart
- tests cover successful loop completion with validation

## Implementation Notes

- Prefer deterministic controller-level tests over brittle end-to-end tests for the majority of lifecycle coverage.
- Use red/green TDD for lifecycle behavior: add the failing case first, then implement only enough logic to pass it.
- Build fakes/stubs for:
  - iteration runtime
  - context extension output
  - lifecycle marker output
  - checks runner
  - completion runner
  - artifact writer
- Add a smaller number of integration-style tests around the real runtime adapter and artifact writer where valuable.
- Use table-driven cases for precedence and restart semantics from `lifecycle.md`.
- Add at least one architecture-boundary test or CI check invocation that runs dependency-cruiser.

## Relevant Spec

- `lifecycle.md`
- `overview.md`
- `internal-contracts.md`

## Suggested Test Structure

- `src/testing/fakes/`
- `src/controller/*.test.ts`
- `src/runtime/*.integration.test.ts`
- `src/artifacts/*.test.ts`

## Out of Scope

- Exhaustively testing Bun or Pi SDK internals
- Golden-testing every artifact byte unless the format is intentionally locked down

## Verification Notes

- Make the decision logic testable without starting live model sessions.
- Include at least one test covering both markers appearing in the same iteration.

## Suggested Implementation Checklist

1. Identify the highest-value lifecycle scenarios from `lifecycle.md` and express them as table-driven tests first.
2. Build or refine fakes for:
   - iteration runtime
   - checks runner
   - completion runner
   - artifact writer
   - extension-state inputs
3. Start red/green with pure controller-decision tests before adding broader orchestration cases.
4. Add orchestration-level tests for evaluation order and dependency interaction.
5. Add focused tests for max-iteration failure, context-limit restart, task-boundary restart, failed completion attempt, and successful validated completion.
6. Include at least one case where both markers appear in the same iteration and verify precedence behavior.
7. Add an architecture-boundary test or CI invocation that runs dependency-cruiser.
8. Keep integration tests selective and purposeful; most behavior should remain covered by deterministic unit or contract tests.

## Definition of Done Heuristic

This ticket is done when the key lifecycle semantics are protected by deterministic red/green tests, architecture boundaries are checked automatically, and the suite gives high confidence without relying heavily on brittle end-to-end runtime tests.

