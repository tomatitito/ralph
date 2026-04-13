# AGENTS.md

Instructions for building, testing, and running the `ralph-loop-ts` Bun/TypeScript project.

## Prerequisites

- [Bun](https://bun.sh/) 1.2+

## Project commands

Run all commands from this directory.

### Install

```bash
bun install
```

### Build and typecheck

```bash
bunx tsc --noEmit
```

### Test

```bash
bun test
```

### Architecture checks

```bash
bun run depcruise
bun run lint
```

### Run the CLI scaffold

```bash
bun run dev
bun run src/index.ts
```

## Project layout

- `src/index.ts` - executable entrypoint for `ralph-ts`
- `src/cli.ts` - minimal CLI scaffold
- `src/controller/` - orchestration layer
- `src/runtime/` - Pi runtime boundary
- `src/extensions/` - project-local Pi extensions
- `src/testing/` - Bun tests, fakes, and fixtures

## Architecture rules

- Production code in `src/runtime/` must not depend on `src/controller/`
- Direct imports of `src/runtime/` are limited to `src/controller/` and tests
- Direct imports of `src/platform/` are limited to `src/controller/` and tests
- Direct `process` imports are restricted to controller-oriented code and tests
- Use dependency-cruiser to enforce boundaries early
