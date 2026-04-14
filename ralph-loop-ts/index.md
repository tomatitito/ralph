# Ralph Loop TS Wiki

This directory contains the working design wiki for the TypeScript implementation of Ralph Loop.

The wiki is the canonical design artifact for `ralph-loop-ts`. It captures behavior, architecture, configuration, lifecycle rules, and implementation planning.

## Pages

### Core
- [Project Overview](./overview.md) — goals, scope, parity target, and major design decisions
- [Loop Lifecycle](./lifecycle.md) — iteration semantics, restart conditions, success/failure rules
- [CLI Specification](./cli.md) — CLI shape, flags, and compatibility with `ralph-loop-rs`
- [Configuration](./configuration.md) — loop, checks, and completion config files
- [Pi Integration](./pi-integration.md) — SDK usage, session runtime, and extension boundaries
- [Extensions](./extensions.md) — separate extension responsibilities and hook behavior
- [Artifacts and State](./artifacts.md) — `~/.ralph-loop/` layout, iteration summaries, metadata
- [Implementation Plan](./implementation-plan.md) — phased delivery plan and milestones
- [Internal Contracts](./internal-contracts.md) — module boundaries, dependency-injection guidance, and core TS interfaces
- [Source Layout Contract](./source-layout.md) — intended `src/` layout, import rules, and TDD-friendly project structure
- [Implementation Kickoff](./implementation-kickoff.md) — practical first steps, first tests, and day-one boundary checks
- [Ticket Roadmap](./ticket-roadmap.md) — `tk` execution order and how to pick the next task

### Reference
- [Source Idea File](./wiki.md) — original wiki-pattern note used to structure this spec space
- [Change Log](./log.md) — chronological record of major spec changes

## Conventions
- Prefer short, interlinked pages over one giant document.
- Prefer plain functions and function types over classes when either would work.
- When a design choice changes, update the relevant page and append a note to [log.md](./log.md).
- Treat this wiki as the source of truth for `ralph-loop-ts` design.
- Later, implementation tasks can be derived into `tk` tickets from these pages.
