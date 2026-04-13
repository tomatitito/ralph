---
id: ral-h0n4
status: closed
deps: [ral-b9sm]
links: [implementation-plan.md, internal-contracts.md, source-layout.md]
created: 2026-04-13T20:32:10Z
type: task
priority: 3
assignee: Jens Kouros
parent: ral-lp9k
tags: [ralph-loop-ts, docs, agents]
---
# Generate ralph-loop-ts/AGENTS.md

Add a TypeScript-specific AGENTS.md describing build, test, lint, and run commands for ralph-loop-ts.

## Acceptance Criteria

- AGENTS.md exists in ralph-loop-ts
- commands use Bun-first conventions
- the document is specific to the TS implementation

## Implementation Notes

- Document the commands that will actually be used during development:
  - build
  - test
  - format/lint if configured
  - run the CLI locally
- Mention the expected working directory and any Bun prerequisites.
- Keep the document implementation-specific; avoid copying Rust instructions that no longer apply.

## Relevant Spec

- `implementation-plan.md`
- `internal-contracts.md`

## Out of Scope

- End-user README content
- Architecture or design documentation already covered by the wiki

