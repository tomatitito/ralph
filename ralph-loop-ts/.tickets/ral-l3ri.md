---
id: ral-l3ri
status: open
deps: [ral-m4r6, ral-gu4t, ral-g5ex, ral-og1z]
links: [artifacts.md, lifecycle.md, internal-contracts.md]
created: 2026-04-13T20:32:10Z
type: task
priority: 2
assignee: Jens Kouros
parent: ral-lp9k
tags: [ralph-loop-ts, artifacts, state]
---
# Implement artifacts under ~/.ralph-loop

Write canonical run, iteration, checks, completion, diagnostics, and summary artifacts under ~/.ralph-loop/.

## Acceptance Criteria

- project/run directory structure follows the spec
- run.json is written and updated through run completion
- per-iteration metadata, summary, checks, completion, and diagnostics files are written
- artifacts record implementation=typescript and Pi session references when available

## Implementation Notes

- Encapsulate all path derivation and file naming in one artifact writer layer; other modules should not build artifact paths ad hoc.
- Support incremental writes so artifacts remain useful if a run is interrupted mid-execution.
- Define explicit write points for:
  - run start
  - iteration completion
  - run completion/failure/interruption
- Preserve canonical placeholder/skipped outputs where the spec expects them, especially for completion validation.
- Treat artifact schemas as part of the public runtime contract for future compatibility with the Rust implementation.

## Architecture Constraints

- The artifact writer should consume normalized controller/check/completion outputs and should not import runtime internals directly.
- File-system access should be encapsulated behind this layer instead of spread across the codebase.
- If clocks, path helpers, or logging are needed, pass them as explicit dependencies.

## Relevant Spec

- `artifacts.md`
- `lifecycle.md`
- `internal-contracts.md`

## Suggested Module Shape

- `src/artifacts/artifact-writer.ts`
- `src/artifacts/path-layout.ts`
- `src/artifacts/run-json.ts`
- `src/artifacts/iteration-files.ts`

## Dependencies on Other Tickets

- Consumes controller outputs from `ral-gu4t` and structured results from checks/completion runners.
- Should not re-derive state that already exists elsewhere.

## Out of Scope

- Making loop decisions
- Running checks or completion validators
- Rendering a custom viewer/UI for artifacts

## Verification Notes

- Add tests for canonical path generation, initial `run.json` creation, iteration-file writes, and final status updates.
- Verify interrupted/failed runs still produce coherent artifacts.

## Suggested Implementation Checklist

1. Define the artifact-writer input contracts from `internal-contracts.md`.
2. Start with red/green tests for path-layout behavior:
   - project slug derivation
   - stable project hash handling
   - run directory layout
   - iteration file naming
3. Implement the smallest path-layout helper that passes those tests.
4. Add artifact-writer tests for:
   - run start writes
   - iteration writes
   - run end writes
   - skipped completion output
   - interrupted/failed status updates
5. Keep filesystem access encapsulated inside the artifact layer and inject clocks/loggers/helpers as needed.
6. Ensure the artifact layer consumes normalized controller/check/completion outputs and does not import runtime internals.
7. Add one higher-level test that verifies a coherent set of artifacts is produced across a small multi-iteration scenario.
8. Re-run dependency-cruiser to confirm artifact/runtime separation remains intact.

## Definition of Done Heuristic

This ticket is done when artifact paths and file contents are produced through a single dedicated layer, interrupted and failed runs still leave coherent output behind, and the behavior is driven by red/green tests rather than incidental filesystem side effects.

