# Implementation Plan

## Phase 1: design-finalization scaffold
- finalize marker format
- finalize the three TOML schemas
- scaffold Bun project
- add `dependency-cruiser` with initial architecture rules
- establish the initial `src/` layout contract
- create TS-specific `AGENTS.md`
- begin with red/green TDD for scaffold and boundary checks

## Phase 2: core runtime
- embed Pi SDK
- create loop controller
- create fresh-session-per-iteration orchestration
- support prompt file and inline prompt
- add CLI flags matching Rust where practical

## Phase 3: extensions
- implement context monitor extension
- implement lifecycle/marker extension
- define extension-controller state handoff

## Phase 4: checks and completion
- parse checks config
- run `after_iteration` checks
- parse completion config
- run completion validators on loop-complete claims
- run `before_final_success` checks

## Phase 5: artifacts and summaries
- create `~/.ralph-loop/` run directory structure
- write run and iteration metadata
- write iteration summary handoff files
- record implementation kind and Pi session references

## Phase 6: testing and parity review
- test max-iteration behavior
- test context-limit restart behavior
- test successful loop completion
- test task-boundary restarts
- test failed checks causing restart
- compare final semantics against `ralph-loop-rs` and `TLA_SPEC.md`

## Deliverable philosophy

The wiki defines the design.
Implementation tasks can later be derived into `tk` tickets from these phases.

Use red/green TDD as the default implementation style:
- write a failing test first
- make the smallest change that passes
- refactor with tests green

Architectural boundaries should be enforced continuously, not checked only at the end. In particular:
- controller may depend on runtime, but not vice versa
- process/platform/logging dependencies should stay at the controller boundary where practical
- shared internal contracts should keep Pi SDK details from leaking into unrelated layers
- dependency-cruiser should be run as an architecture guard alongside normal tests
