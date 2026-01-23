# Implementation Plan

This document describes the phased approach to implementing ralph-loop. Each phase builds on the previous one and has clear acceptance criteria.

## Phase 1: Foundation ✅ COMPLETE

**Goal**: Establish project structure with minimal compiling code.

### Files
- `Cargo.toml` - dependencies
- `src/lib.rs` - module declarations
- `src/main.rs` - minimal CLI skeleton
- `src/error.rs` - `RalphError` enum
- `src/config.rs` - `Config` struct with defaults

### Acceptance Criteria
- [x] `cargo build` succeeds
- [x] `cargo test` runs (no tests yet)
- [x] `cargo clippy` passes
- [x] `cargo fmt --check` passes

---

## Phase 2: Agent Abstraction ✅ COMPLETE

**Goal**: Define the `Agent` trait and `AgentResult` types to enable testing.

### Files
- `src/agent.rs` - `Agent` trait, `AgentResult`, `ExitReason`

### Dependencies
- Phase 1 (error types)

### Acceptance Criteria
- [x] `Agent` trait compiles with async support
- [x] `AgentResult::with_promise()` and `without_promise()` constructors work
- [x] Unit tests for `AgentResult::is_fulfilled()` pass

---

## Phase 3: Loop Controller with Mock ✅ COMPLETE

**Goal**: Implement core loop logic using a mock agent.

### Files
- `src/loop_controller.rs` - `LoopController<A: Agent>`
- `src/state.rs` - `SharedState` (simplified, no RwLock yet)

### Dependencies
- Phase 2 (Agent trait)

### Acceptance Criteria
- [x] Loop continues until promise fulfilled (mock test)
- [x] Loop stops on first iteration if promise found immediately
- [x] Loop respects `max_iterations` limit
- [x] Returns `MaxIterationsExceeded` error when limit hit without promise

---

## Phase 4: Token Counting ✅ COMPLETE

**Goal**: Implement token estimation for context tracking.

### Files
- `src/token_counter.rs` - `TokenCounter` with multiple estimation methods

### Dependencies
- None (standalone utility)

### Acceptance Criteria
- [x] Tiktoken estimation works with `cl100k_base`
- [x] ByteRatio fallback: `len / 4`
- [x] CharRatio fallback: `chars().count() / 4`
- [x] Unit tests verify estimates are within expected range

---

## Phase 5: Process Management ✅ COMPLETE

**Goal**: Spawn and manage Claude subprocess.

### Files
- `src/process.rs` - `ClaudeProcess` wrapper

### Dependencies
- Phase 1 (config for claude_path)

### Acceptance Criteria
- [x] Can spawn subprocess with piped stdin/stdout/stderr
- [x] Can write prompt to stdin
- [x] Can read output streams
- [x] `kill()` terminates process

---

## Phase 6: Output Monitoring ✅ COMPLETE

**Goal**: Concurrent monitoring of stdout/stderr for tokens and promises.

### Files
- `src/monitor.rs` - `OutputMonitor` with async line reading
- `src/state.rs` - Add `RwLock` wrappers for concurrent access

### Dependencies
- Phase 4 (token counter)
- Phase 5 (process streams)

### Acceptance Criteria
- [x] Reads stdout/stderr concurrently
- [x] Detects `<promise>TEXT</promise>` pattern
- [x] Updates shared token count
- [x] Sends kill command when context limit reached

---

## Phase 7: ClaudeAgent Implementation ✅ COMPLETE

**Goal**: Production agent that wires process + monitor together.

### Files
- `src/agent.rs` - Add `ClaudeAgent` implementation

### Dependencies
- Phase 2 (Agent trait)
- Phase 5 (process)
- Phase 6 (monitor)

### Acceptance Criteria
- [x] Implements `Agent` trait
- [x] Spawns Claude, monitors output, returns `AgentResult`
- [x] Handles context limit via kill + appropriate `ExitReason`

---

## Phase 8: CLI and Signal Handling ✅ COMPLETE

**Goal**: Complete command-line interface with graceful shutdown.

### Files
- `src/main.rs` - Full CLI with clap, signal handling

### Dependencies
- All previous phases

### Acceptance Criteria
- [x] All CLI flags from spec work (`-p`, `-f`, `-m`, `-c`, etc.)
- [x] TOML config file loading works
- [x] Ctrl+C triggers graceful shutdown
- [x] Exit codes reflect success/failure appropriately

---

## Phase 9: CI and Polish ✅ COMPLETE

**Goal**: GitHub Actions, documentation, release build.

### Files
- `.github/workflows/ci.yml`
- README updates

### Acceptance Criteria
- [x] CI runs on push/PR to main
- [x] All jobs pass: check, test, fmt, clippy
- [x] `cargo build --release` produces working binary
- [x] Integration test with mock script passes (TLA+ specification verified)

---

## TLA+ Verification ✅ COMPLETE

The TLA+ specification has been verified with TLC model checker:
- Bounded mode (`RalphLoop.cfg`): All properties pass
- Infinite mode (`RalphLoopInfinite.cfg`): All properties pass

All safety invariants and liveness properties verified:
- `TypeOK` - All variables stay within valid ranges
- `IterationBoundRespected` - Never exceeds max iterations (bounded mode)
- `ProcessImpliesMonitors` - Running process implies active monitors
- `SuccessImpliesPromise` - Success state requires promise found
- `InfiniteModeNeverFails` - Infinite mode never reaches "failed" state
- `EventualTermination` - Loop eventually terminates
- `ShutdownEventuallyHandled` - Ctrl+C is handled
- `PromiseLeadsToSuccess` - Found promise leads to success
- `ContextLimitLeadsToKill` - Exceeded limit kills process
