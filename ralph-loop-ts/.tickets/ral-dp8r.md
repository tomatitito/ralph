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
# Expand lifecycle tests beyond the mock vertical-slice happy path

Add automated tests covering max iterations, context-limit restarts, task boundaries, failed checks, failed completion, precedence cases, and successful completion.

## Acceptance Criteria

- the existing happy-path mock vertical-slice run remains covered
- tests cover max-iteration failure
- tests cover context-limit restart
- tests cover task-boundary restart
- tests cover failed checks causing restart
- tests cover failed completion causing restart
- tests cover successful loop completion with validation
- tests cover at least one both-markers-present precedence case

## Implementation Notes

- `rlt-w903` already added a happy-path automated test and basic controller coverage; this ticket extends coverage from that baseline to the full lifecycle semantics.
- Prefer deterministic controller-level tests over brittle end-to-end tests for the majority of lifecycle coverage.
- Use red/green TDD for lifecycle behavior: add the failing case first, then implement only enough logic to pass it.
- Build or refine fakes/stubs for:
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

1. Start from the happy-path tests introduced by `rlt-w903` and identify the highest-value missing lifecycle scenarios from `lifecycle.md`.
2. Build or refine fakes for:
   - iteration runtime
   - checks runner
   - completion runner
   - artifact writer
   - extension-state inputs
3. Start red/green with pure controller-decision tests before adding broader orchestration cases.
4. Add orchestration-level tests for evaluation order and dependency interaction.
5. Add focused tests for max-iteration failure, context-limit restart, task-boundary restart, failed completion attempt, failed checks, and successful validated completion.
6. Include at least one case where both markers appear in the same iteration and verify precedence behavior.
7. Add an architecture-boundary test or CI invocation that runs dependency-cruiser.
8. Keep integration tests selective and purposeful; most behavior should remain covered by deterministic unit or contract tests.

## Definition of Done Heuristic

This ticket is done when the existing happy-path slice tests have been expanded into strong lifecycle coverage, key precedence semantics are protected by deterministic red/green tests, architecture boundaries are checked automatically, and the suite gives high confidence without relying heavily on brittle end-to-end runtime tests.
