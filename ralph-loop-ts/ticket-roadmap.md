# Ticket Roadmap

This page explains how the `tk` tickets for `ralph-loop-ts` are structured so a newcomer can quickly determine what to implement next.

## Rule of thumb

Use:

```bash
tk ready
```

The ready list should expose the next actionable ticket on the critical path.

Dependencies, not just priorities, define the implementation order.

## Current epic

- `ral-lp9k` — Build ralph-loop-ts v1

The epic is intentionally blocked on all implementation tickets so it does not appear as the next item of work.

## Critical path

The intended implementation order is:

1. `ral-b9sm` — Scaffold Bun/TypeScript project in `ralph-loop-ts`
2. `ral-m4r6` — Implement `ralph-ts` CLI and config loading
3. `ral-xtlt` — Embed Pi SDK runtime for fresh-session iterations
4. `ral-a62g` — Implement context monitor extension
5. `ral-2q2g` — Implement lifecycle marker extension
6. `ral-gu4t` — Implement iteration orchestration and decision logic
7. `ral-g5ex` — Implement checks runner from `ralph-checks.toml`
8. `ral-og1z` — Implement completion validation runner from `ralph-completion.toml`
9. `ral-l3ri` — Implement artifacts under `~/.ralph-loop`
10. `ral-dp8r` — Add tests for Ralph TS lifecycle semantics

## Supporting work

Supporting work should not distract from the critical path:

- `ral-h0n4` — Generate `ralph-loop-ts/AGENTS.md`

This is intentionally lower priority and may be completed after the main scaffolding exists.

## Phases

### Phase 1 — project foundation
- `ral-b9sm`
- `ral-m4r6`

### Phase 2 — Pi runtime foundation
- `ral-xtlt`

### Phase 3 — iteration signal capture
- `ral-a62g`
- `ral-2q2g`

### Phase 4 — loop control
- `ral-gu4t`

### Phase 5 — command-based gates
- `ral-g5ex`
- `ral-og1z`

### Phase 6 — persistence and verification
- `ral-l3ri`
- `ral-dp8r`

### Parallel/supporting work
- `ral-h0n4`

## Priority guidance

Priorities are used to make the ready list easier to scan:
- **P1** — critical path implementation work
- **P2** — necessary but later-stage implementation work
- **P3** — supporting documentation or non-blocking work

Priority does not replace dependency tracking.

## How to choose the next task

1. Run `tk ready`
2. Prefer the highest-priority ready ticket on the critical path
3. If multiple same-priority tickets are ready, follow the order in the critical path list above unless there is a compelling implementation reason not to

## Dependency review after ticket refinement

After refining the tickets with architecture constraints, TDD notes, and implementation checklists, the current dependency graph still looks sound.

Review conclusions:
- `ral-b9sm` remains the right first ticket because it now establishes both the scaffold and architecture enforcement via `dependency-cruiser`.
- `ral-m4r6` should still come before runtime/controller work because it defines the normalized config contract consumed downstream.
- `ral-xtlt` still belongs before the extension tickets because it defines the runtime boundary they plug into.
- `ral-a62g` and `ral-2q2g` remain parallelizable after `ral-xtlt`.
- `ral-gu4t` remains the main integration point and should continue to depend on runtime plus both extension tickets.
- `ral-g5ex` and `ral-og1z` can still proceed after config loading; they are intentionally runtime-agnostic.
- `ral-l3ri` is correctly later in the graph because it should consume normalized outputs from controller/checks/completion rather than inventing them.
- `ral-dp8r` is correctly late because it should validate the stabilized lifecycle behavior, though lightweight tests should still be written earlier within each ticket as part of red/green TDD.

No dependency metadata changes are required at this time.

## Maintenance rule

If the dependency graph changes, update this page and the ticket metadata together so the roadmap stays accurate.
