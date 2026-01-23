# TLA+ Specification

The `RalphLoop.tla` specification formally models the concurrent behavior of the ralph-loop application.

## Prerequisites

- [TLA+ Toolbox](https://github.com/tlaplus/tlaplus/releases) (GUI), or
- [TLC](https://github.com/tlaplus/tlaplus/releases) command-line model checker

## Check the Specification

Two configurations are provided:

| Config File | Mode | Description |
|-------------|------|-------------|
| `RalphLoop.cfg` | Bounded | Tests with `HasMaxIterations = TRUE` |
| `RalphLoopInfinite.cfg` | Infinite | Tests with `HasMaxIterations = FALSE` |

**Using TLA+ Toolbox (GUI):**
1. Open TLA+ Toolbox
2. File → Open Spec → Add New Spec → select `RalphLoop.tla`
3. TLC Model Checker → New Model
4. Under "What is the behavior spec?" select `Spec`
5. Import constants from the desired `.cfg` file
6. Run TLC

**Using TLC command-line:**

```bash
# Check bounded mode
java -jar tla2tools.jar -config RalphLoop.cfg RalphLoop.tla

# Check infinite mode
java -jar tla2tools.jar -config RalphLoopInfinite.cfg RalphLoop.tla
```

## Properties Verified

**Safety (Invariants):**
- `TypeOK` - All variables stay within valid ranges
- `IterationBoundRespected` - Never exceeds max iterations (bounded mode)
- `ProcessImpliesMonitors` - Running process implies active monitors
- `SuccessImpliesPromise` - Success state requires promise found
- `InfiniteModeNeverFails` - Infinite mode never reaches "failed" state

**Liveness:**
- `EventualTermination` - Loop eventually terminates
- `ShutdownEventuallyHandled` - Ctrl+C is handled
- `PromiseLeadsToSuccess` - Found promise leads to success
- `ContextLimitLeadsToKill` - Exceeded limit kills process

## Verifying Implementation Adheres to Spec

### 1. State Machine Correspondence

The implementation must mirror the TLA+ state machine:

| TLA+ Variable | Rust Equivalent |
|---------------|-----------------|
| `loopState` | `LoopController` return value / internal state |
| `processState` | `ClaudeProcess` status |
| `tokenCount` | `SharedState::token_count` |
| `promiseFound` | `SharedState::promise_found` |
| `monitorActive` | Monitor task handles |
| `killRequested` | Channel message to kill process |
| `shutdownSignal` | `tokio::signal` handler |

### 2. Unit Tests for Spec Properties

The unit tests should verify the spec's safety properties:

```bash
cd ralph-loop-rs

# Run all tests including property tests
cargo test

# Run specific property tests
cargo test spec_properties
```

Key test cases derived from the spec:
- Loop continues until promise fulfilled
- Loop stops immediately when promise found on first try
- Loop respects max iterations when promise never fulfilled
- Shutdown signal terminates the loop
- Context limit triggers process kill and restart

### 3. Trace Validation (Optional)

For deeper verification, enable trace logging and compare against TLA+ traces:

```bash
cd ralph-loop-rs

# Run with trace logging
RUST_LOG=ralph_loop=trace ./target/release/ralph-loop -p "test" -m 3 2>&1 | tee trace.log

# Trace should show state transitions matching TLA+ actions:
# - StartIteration
# - MonitorOutput (token updates, promise detection)
# - ProcessExitsNaturally / KillProcess
# - HandleIterationEnd
```

### 4. Property Checklist

Before release, verify these properties hold:

- [ ] `TypeOK`: All state values within expected ranges
- [ ] `IterationBoundRespected`: `--max-iterations` flag works correctly
- [ ] `SuccessImpliesPromise`: Only exits success when promise detected
- [ ] `InfiniteModeNeverFails`: Without `-m`, never exits with "failed"
- [ ] `ShutdownEventuallyHandled`: Ctrl+C cleanly terminates
- [ ] `ContextLimitLeadsToKill`: `--context-limit` triggers restart
