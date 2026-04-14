---
id: ral-m4r6
status: closed
deps: [ral-b9sm]
links: [cli.md, configuration.md, internal-contracts.md]
created: 2026-04-13T20:32:09Z
type: task
priority: 1
assignee: Jens Kouros
parent: ral-lp9k
tags: [ralph-loop-ts, cli, config]
---
# Implement ralph-ts CLI and config loading

Implement the CLI surface and load/validate loop, checks, and completion TOML configs according to the spec.

## Acceptance Criteria

- prompt file and inline prompt are supported
- loop config is parsed and validated
- checks and completion configs are parsed and validated
- CLI flags override config values
- invalid config combinations fail with clear errors

## Implementation Notes

- Separate parsing from normalization:
  - parse raw CLI arguments
  - load TOML files
  - merge config sources with CLI override precedence
  - validate the final normalized config
- Introduce explicit normalized config types such as:
  - `LoopConfig`
  - `ChecksConfig`
  - `CompletionConfig`
  - `ResolvedRunConfig`
- Exactly one prompt source must be active after merging:
  - inline prompt
  - prompt file
  - config-provided prompt
- Resolve referenced checks/completion config paths relative to the loop config location when appropriate.
- Preserve compatibility for `--completion-promise`, but treat it as a compatibility input rather than the primary TS completion mechanism.
- Keep direct `process.argv` / `process.env` access at the CLI/controller boundary; pass normalized values inward.
- Prefer clear, user-facing validation errors over schema-only parse failures.

## Architecture Constraints

- Config parsing and validation modules should be pure where practical.
- They should not depend on Pi runtime modules, controller decision modules, or direct process-level globals.
- Any filesystem or environment access beyond the CLI boundary should be routed through explicit dependencies or narrow helper adapters.

## Relevant Spec

- `cli.md`
- `configuration.md`
- `internal-contracts.md`

## Suggested Module Shape

- `src/cli.ts` for argument parsing and process exit behavior
- `src/config/loop-config.ts`
- `src/config/checks-config.ts`
- `src/config/completion-config.ts`
- `src/config/resolve-config.ts`

## Out of Scope

- Running Pi sessions
- Executing checks or completion validators
- Loop orchestration decisions beyond config validation

## Verification Notes

- Add table-driven tests for valid and invalid config combinations.
- Test CLI override precedence explicitly.
- Test path resolution behavior for nested config files.
- Keep config tests independent from live runtime concerns.

## Suggested Implementation Checklist

1. Define the normalized config types described in `internal-contracts.md`, especially the resolved run config consumed by the controller.
2. Implement raw CLI argument parsing in `src/cli.ts` or a closely related module.
3. Implement loop-config parsing and validation.
4. Implement checks-config parsing and validation.
5. Implement completion-config parsing and validation.
6. Implement config-source merging with explicit precedence:
   - CLI flags
   - loop config file
   - defaults
7. Implement prompt-source validation so exactly one prompt source is active after resolution.
8. Resolve referenced config paths relative to the loop config location where required.
9. Start red/green TDD with table-driven tests for:
   - valid inline prompt config
   - valid prompt-file config
   - invalid multiple-prompt-source combinations
   - CLI override precedence
   - missing referenced config files
   - invalid checks/completion config structure
10. Keep direct `process.argv` / `process.env` access at the controller/CLI boundary only; pass normalized values into parsing/validation modules.
11. Verify that controller-facing code can consume a single normalized config contract without needing to know where values came from.

## Definition of Done Heuristic

This ticket is done when the CLI/config layer can produce a validated normalized run config for downstream code, reject invalid combinations with clear errors, and do so under table-driven red/green tests without pulling in runtime concerns.

## Implementation Plan

1. Define raw and normalized config types
   - flesh out `src/config/loop-config.ts`
   - flesh out `src/config/checks-config.ts`
   - flesh out `src/config/completion-config.ts`
   - keep `ResolvedRunConfig` in sync with `internal-contracts.md`
2. Add CLI argument parsing in `src/cli.ts`
   - support `--prompt-file/-f`
   - support `--prompt/-p`
   - support `--max-iterations/-m`
   - support `--completion-promise/-c`
   - support `--output-dir/-o`
   - support `--context-limit`
   - support `--config`
   - support `--checks-config`
   - support `--completion-config`
   - support `--provider`
   - support `--model`
   - support `--thinking`
3. Implement TOML loading and parsing
   - parse loop config from `ralph.toml` or `--config`
   - parse checks config referenced by loop config or `--checks-config`
   - parse completion config referenced by loop config or `--completion-config`
4. Implement merge and normalization in `src/config/resolve-config.ts`
   - precedence is CLI > loop config > defaults
   - normalize prompt source into exactly one active `PromptSource`
   - resolve checks/completion paths relative to the loop config file directory
5. Add validation with clear user-facing errors
   - reject multiple prompt sources
   - reject missing prompt source
   - reject invalid `max_iterations`
   - reject invalid `context_limit`
   - reject unsupported `thinking`
   - reject missing checks/completion config paths
   - reject invalid checks/completion config structures
6. Keep side effects at the CLI boundary
   - avoid direct `process.argv` / `process.env` access in config modules
   - route filesystem access through the CLI boundary or narrow helpers
7. Drive the work with red/green tests first
   - add `src/config/resolve-config.test.ts`
   - add `src/config/checks-config.test.ts`
   - add `src/config/completion-config.test.ts`
   - cover valid inline prompt config
   - cover valid prompt-file config
   - cover invalid multiple prompt sources
   - cover invalid missing prompt source
   - cover CLI override precedence
   - cover relative path resolution from nested config files
   - cover missing referenced config files
   - cover invalid checks/completion config shapes
8. Finish by wiring controller-facing consumption
   - ensure downstream code can consume one validated `ResolvedRunConfig`
   - keep runtime concerns out of the CLI/config layer

