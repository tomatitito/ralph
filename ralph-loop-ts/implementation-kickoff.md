# Implementation Kickoff

This page is a practical starting checklist for the first implementation ticket, `ral-b9sm`.

It is intentionally concrete and TDD-oriented.

## Goal

Get to a minimal, runnable, testable scaffold that already enforces the key architectural boundary:
- the controller may depend on runtime
- runtime must not depend on controller
- only the controller should directly depend on process/platform/logging details in production code

## First slice to build

Aim for the smallest useful vertical slice:
- Bun project boots
- `ralph-ts` runs and prints a deterministic placeholder message
- one test passes
- dependency-cruiser passes

## Recommended order

1. Create `package.json`
   - add Bun scripts:
     - `dev`
     - `test`
     - `depcruise`
   - add `bin` entry for `ralph-ts`
2. Create `tsconfig.json`
   - ESM
   - strict mode
3. Create initial source layout from `source-layout.md`
   - at minimum:
     - `src/index.ts`
     - `src/cli.ts`
     - `src/controller/`
     - `src/runtime/`
     - `src/platform/`
     - `src/testing/`
4. Add `.dependency-cruiser.cjs`
5. Add first failing test for CLI placeholder behavior
6. Implement the minimum code to make that test pass
7. Add first architecture test or CI-style test that runs dependency-cruiser
8. Make dependency-cruiser pass
9. Add `AGENTS.md` or leave a stub if keeping that work mostly in `ral-h0n4`

## First tests to write

### Test 1: placeholder CLI behavior

Intent:
- prove the scaffold runs
- prove the executable path is wired

Example expectation:
- invoking the CLI entry returns or prints a deterministic placeholder such as `ralph-ts: not implemented yet`

### Test 2: architecture boundary check

Intent:
- prove architecture checks are part of the normal development loop from day one

Approach options:
- a test that shells out to dependency-cruiser
- or a CI step documented and run locally during the ticket

Minimum expectation:
- dependency-cruiser exits successfully on the initial scaffold

## Red/green rhythm

For `ral-b9sm`, keep the cycle tiny:

1. red: write failing placeholder CLI test
2. green: implement minimal CLI
3. red: write failing architecture-check invocation or wire depcruise script
4. green: add config/layout until it passes
5. refactor: clean file layout and scripts while tests stay green

## Suggested first commands

```bash
bun test
bun run depcruise
bun run dev
```

## Early pitfalls to avoid

- do not let runtime import controller, even temporarily
- do not put real business logic into `src/platform/`
- do not let leaf modules read `process` directly
- do not postpone dependency-cruiser until later tickets
- do not overbuild the scaffold before the first tests exist

## Ready-to-start definition

You are ready to move from `ral-b9sm` to `ral-m4r6` when:
- scaffold files exist
- CLI placeholder runs
- at least one scaffold test is green
- dependency-cruiser is green
- the `src/` layout supports the next config ticket without restructuring
