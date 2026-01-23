# Implementation Plan

This document describes the phased approach to implementing ralph-loop. Each phase builds on the previous one and has clear acceptance criteria.

## Phase 1: Foundation

**Goal**: Establish project structure with minimal compiling code.

### Files
- `Cargo.toml` - dependencies
- `src/lib.rs` - module declarations
- `src/main.rs` - minimal CLI skeleton
- `src/error.rs` - `RalphError` enum
- `src/config.rs` - `Config` struct with defaults

### Acceptance Criteria
- [ ] `cargo build` succeeds
- [ ] `cargo test` runs (no tests yet)
- [ ] `cargo clippy` passes
- [ ] `cargo fmt --check` passes

---

## Phase 2: Agent Abstraction

**Goal**: Define the `Agent` trait and `AgentResult` types to enable testing.

### Files
- `src/agent.rs` - `Agent` trait, `AgentResult`, `ExitReason`

### Dependencies
- Phase 1 (error types)

### Acceptance Criteria
- [ ] `Agent` trait compiles with async support
- [ ] `AgentResult::with_promise()` and `without_promise()` constructors work
- [ ] Unit tests for `AgentResult::is_fulfilled()` pass

---

## Phase 3: Loop Controller with Mock

**Goal**: Implement core loop logic using a mock agent.

### Files
- `src/loop_controller.rs` - `LoopController<A: Agent>`
- `src/state.rs` - `SharedState` (simplified, no RwLock yet)

### Dependencies
- Phase 2 (Agent trait)

### Acceptance Criteria
- [ ] Loop continues until promise fulfilled (mock test)
- [ ] Loop stops on first iteration if promise found immediately
- [ ] Loop respects `max_iterations` limit
- [ ] Returns `MaxIterationsExceeded` error when limit hit without promise

---

## Phase 4: Token Counting

**Goal**: Implement token estimation for context tracking.

### Files
- `src/token_counter.rs` - `TokenCounter` with multiple estimation methods

### Dependencies
- None (standalone utility)

### Acceptance Criteria
- [ ] Tiktoken estimation works with `cl100k_base`
- [ ] ByteRatio fallback: `len / 4`
- [ ] CharRatio fallback: `chars().count() / 4`
- [ ] Unit tests verify estimates are within expected range

---

## Phase 5: Process Management

**Goal**: Spawn and manage Claude subprocess.

### Files
- `src/process.rs` - `ClaudeProcess` wrapper

### Dependencies
- Phase 1 (config for claude_path)

### Acceptance Criteria
- [ ] Can spawn subprocess with piped stdin/stdout/stderr
- [ ] Can write prompt to stdin
- [ ] Can read output streams
- [ ] `kill()` terminates process

---

## Phase 6: Output Monitoring

**Goal**: Concurrent monitoring of stdout/stderr for tokens and promises.

### Files
- `src/monitor.rs` - `OutputMonitor` with async line reading
- `src/state.rs` - Add `RwLock` wrappers for concurrent access

### Dependencies
- Phase 4 (token counter)
- Phase 5 (process streams)

### Acceptance Criteria
- [ ] Reads stdout/stderr concurrently
- [ ] Detects `<promise>TEXT</promise>` pattern
- [ ] Updates shared token count
- [ ] Sends kill command when context limit reached

---

## Phase 7: ClaudeAgent Implementation

**Goal**: Production agent that wires process + monitor together.

### Files
- `src/agent.rs` - Add `ClaudeAgent` implementation

### Dependencies
- Phase 2 (Agent trait)
- Phase 5 (process)
- Phase 6 (monitor)

### Acceptance Criteria
- [ ] Implements `Agent` trait
- [ ] Spawns Claude, monitors output, returns `AgentResult`
- [ ] Handles context limit via kill + appropriate `ExitReason`

---

## Phase 8: CLI and Signal Handling

**Goal**: Complete command-line interface with graceful shutdown.

### Files
- `src/main.rs` - Full CLI with clap, signal handling

### Dependencies
- All previous phases

### Acceptance Criteria
- [ ] All CLI flags from spec work (`-p`, `-f`, `-m`, `-c`, etc.)
- [ ] TOML config file loading works
- [ ] Ctrl+C triggers graceful shutdown
- [ ] Exit codes reflect success/failure appropriately

---

## Phase 9: CI and Polish

**Goal**: GitHub Actions, documentation, release build.

### Files
- `.github/workflows/ci.yml`
- README updates

### Acceptance Criteria
- [ ] CI runs on push/PR to main
- [ ] All jobs pass: check, test, fmt, clippy
- [ ] `cargo build --release` produces working binary
- [ ] Integration test with mock script passes
