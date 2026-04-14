---
id: ral-lp9k
status: open
deps: [ral-b9sm, ral-m4r6, ral-xtlt, ral-a62g, ral-2q2g, ral-gu4t, ral-g5ex, ral-og1z, ral-l3ri, ral-dp8r]
links: [index.md, lifecycle.md, pi-integration.md, artifacts.md, internal-contracts.md, source-layout.md]
created: 2026-04-13T20:32:09Z
type: epic
priority: 0
assignee: Jens Kouros
tags: [ralph-loop-ts, typescript, pi]
---
# Build ralph-loop-ts v1

Implement the first TypeScript version of Ralph Loop using the Pi SDK and internal Pi extensions, following the spec wiki in ralph-loop-ts/.

## Acceptance Criteria

- ralph-loop-ts project is scaffolded
- core loop behavior matches the spec wiki
- Pi SDK integration and internal extensions are implemented
- checks and completion validation work from TOML configs
- artifacts are written under ~/.ralph-loop/
- the new implementation has project-local documentation and tests

## Implementation Notes

- This epic is complete only when the scaffold, runtime, extensions, controller, checks/completion runners, artifact writer, documentation, and tests work together as one coherent TS implementation.
- The spec wiki remains the semantic source of truth; ticket-level implementation notes exist to reduce ambiguity during execution.
- `rlt-w903` established a mock vertical slice and the core function-shaped seams across CLI/config, runtime, controller, checks, and completion.
- Remaining tickets should generally refine or replace those slice implementations rather than re-introduce the same seams from scratch.
- Prefer introducing stable internal contracts early so later tickets compose cleanly:
  - resolved config contract
  - runtime/iteration result contract
  - extension state contract
  - controller decision contract
  - artifact writer contract
- Enforce architectural boundaries continuously with `dependency-cruiser`, especially:
  - controller may depend on runtime, but not vice versa
  - direct process/logging/platform dependencies stay at the controller boundary
  - Pi SDK details remain hidden behind runtime interfaces where practical

## Relevant Spec

- `index.md`
- `lifecycle.md`
- `pi-integration.md`
- `artifacts.md`
- `internal-contracts.md`

## Release Readiness Notes

Before closing the epic, verify:
- CLI/config behavior works from a clean checkout
- fresh-session iteration semantics work with the Pi SDK
- task and loop markers behave as specified
- checks and completion validators affect decisions in the correct order
- artifacts are complete enough to diagnose a failed run
- automated tests cover the key lifecycle paths
