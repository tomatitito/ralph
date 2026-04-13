---
id: ral-b9sm
status: open
deps: []
links: [implementation-plan.md, pi-integration.md, internal-contracts.md, source-layout.md]
created: 2026-04-13T20:32:09Z
type: task
priority: 1
assignee: Jens Kouros
parent: ral-lp9k
tags: [ralph-loop-ts, scaffold, bun]
---
# Scaffold Bun/TypeScript project in ralph-loop-ts

Create the initial Bun + TypeScript + ESM project structure for ralph-loop-ts.

## Acceptance Criteria

- package.json, tsconfig, and source layout exist
- binary entrypoint is named ralph-ts
- project can run a hello-world CLI through bun
- repo-local AGENTS.md is added for the TS implementation
- a `dependency-cruiser` configuration exists with initial boundary rules for controller/runtime layering

## Implementation Notes

- Use Bun-first project conventions with ESM and strict TypeScript settings.
- Create a minimal but future-proof source layout, for example:
  - `src/cli.ts`
  - `src/index.ts`
  - `src/config/`
  - `src/runtime/`
  - `src/extensions/`
  - `src/controller/`
  - `src/checks/`
  - `src/completion/`
  - `src/artifacts/`
  - `src/testing/`
- Add scripts for `bun run dev`, `bun test`, `bun run lint` if linting is configured, and a runnable CLI script path in `package.json`.
- Add a baseline `dependency-cruiser` config early so architecture rules are enforced from the first implementation tickets onward.
- Start with red/green TDD: add a small scaffold test and an architecture-boundary check before filling in implementation details.
- Keep the initial CLI implementation minimal; detailed CLI behavior belongs to `ral-m4r6`.

## Architecture Constraints

- The controller is the only production layer that should directly depend on process-level globals.
- Other modules should receive dependencies explicitly rather than importing `process`-like facilities directly.
- Add initial dependency-cruiser rules for at least:
  - `src/runtime/**` must not depend on `src/controller/**`
  - direct runtime imports are limited to the controller layer and tests
  - platform/adaptor modules for globals should not be imported from arbitrary leaf modules

## Relevant Spec

- `implementation-plan.md`
- `pi-integration.md`
- `internal-contracts.md`
- `source-layout.md`
- `implementation-kickoff.md`

## Out of Scope

- Full CLI/config behavior
- Pi SDK embedding
- Checks, completion, or artifact persistence beyond placeholder structure
- Any follow-on documentation ticket for `AGENTS.md`; this ticket owns that document completely

## Verification Notes

- Running the binary via Bun should print a deterministic hello-world or placeholder message.
- The project layout should be ready for the follow-on tickets without requiring a later re-scaffold.
- Dependency-cruiser should run successfully against the initial scaffold.
- At least one initial failing-then-passing test should exercise the scaffold or architecture boundary setup.

## Suggested Implementation Checklist

1. Create `package.json` with:
   - Bun-first scripts for `dev`, `test`, and dependency-cruiser
   - CLI bin entry for `ralph-ts`
   - dev dependencies for TypeScript, Bun typings if needed, and `dependency-cruiser`
2. Add `tsconfig.json` with strict settings and ESM-oriented output.
3. Create the initial `src/` layout from `source-layout.md`, including empty placeholder modules where useful.
4. Add a minimal `src/index.ts` / `src/cli.ts` path that can print a deterministic placeholder message.
5. Add `.dependency-cruiser.cjs` and wire it into a script.
6. Start red/green TDD:
   - add one failing test for the placeholder CLI behavior or scaffold contract
   - add one failing architecture check or test that executes dependency-cruiser
   - implement the minimum needed to make both pass
7. Add `AGENTS.md` with Bun-first commands as part of this scaffold ticket.
8. Verify the scaffold from a clean checkout path:
   - CLI runs
   - tests pass
   - dependency-cruiser passes

## Definition of Done Heuristic

This ticket is done when the project can be cloned, dependencies installed, tests run, architecture checks run, and the placeholder CLI executed without needing structural rework for the next ticket.

