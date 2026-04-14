# Ralph Loop TS Spec Log

## [2026-04-13] wiki | initialized spec wiki
- Created wiki structure for `ralph-loop-ts`
- Added pages for overview, lifecycle, CLI, configuration, Pi integration, extensions, artifacts, and implementation plan
- Adopted the local wiki pattern from `wiki.md` as the structure for the specification space

## [2026-04-13] spec | hardened lifecycle, config, extension, and artifact pages
- Made the loop lifecycle more precise with explicit phases, evaluation order, precedence rules, and restart semantics
- Finalized a concrete v1 direction for marker strings and marker behavior
- Replaced conceptual config discussion with a concrete TOML schema direction, field definitions, precedence, and validation rules
- Defined the v1 extension-controller contract around an in-process shared state bridge
- Defined canonical run and iteration artifact layout and required metadata fields

## [2026-04-13] planning | added ticket roadmap and clarified execution order
- Added `ticket-roadmap.md` to explain the intended `tk` dependency flow
- Documented the critical path, supporting work, and priority meanings for newcomers

## [2026-04-14] architecture | prefer function-oriented seams over classes
- Updated the wiki to state a clear preference for plain functions and function types over single-method classes
- Reframed controller/runtime/checks/completion seams around function-shaped contracts
- Clarified that classes should be reserved for cases where stateful lifecycle management is materially clearer than functions and plain data
