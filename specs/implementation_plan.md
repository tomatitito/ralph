# Implementation Plan

## Project Structure

```
ralph-loop/
├── .github/
│   └── workflows/
│       └── ci.yml           # CI workflow (build, test, clippy, fmt)
├── Cargo.toml
└── src/
    ├── main.rs              # CLI entry point, signal handling
    ├── lib.rs               # Library exports
    ├── config.rs            # Configuration structures
    ├── agent.rs             # Agent trait + ClaudeAgent implementation
    ├── loop_controller.rs   # Main orchestration (generic over Agent)
    ├── process.rs           # Claude subprocess management
    ├── monitor.rs           # Output monitoring (tokens + promises)
    ├── token_counter.rs     # Token estimation
    ├── state.rs             # Shared state and events
    └── error.rs             # Error types
```

## Files to Create

| File | Purpose |
|------|---------|
| `ralph-loop/.github/workflows/ci.yml` | GitHub Actions CI workflow |
| `ralph-loop/Cargo.toml` | Project manifest with dependencies |
| `ralph-loop/src/main.rs` | CLI parsing, signal handling, entry point |
| `ralph-loop/src/lib.rs` | Module exports |
| `ralph-loop/src/config.rs` | Config structs and loading |
| `ralph-loop/src/error.rs` | RalphError enum |
| `ralph-loop/src/state.rs` | SharedState and events |
| `ralph-loop/src/token_counter.rs` | Token estimation |
| `ralph-loop/src/agent.rs` | Agent trait, AgentResult, ClaudeAgent impl |
| `ralph-loop/src/process.rs` | ClaudeProcess wrapper |
| `ralph-loop/src/monitor.rs` | OutputMonitor with concurrent readers |
| `ralph-loop/src/loop_controller.rs` | Main orchestration logic (generic over Agent) |

## GitHub Actions CI Workflow

`.github/workflows/ci.yml`:

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always
  RUSTFLAGS: -Dwarnings

jobs:
  check:
    name: Check
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo check --all-targets

  test:
    name: Test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo test --all-targets

  fmt:
    name: Format
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt
      - run: cargo fmt --all --check

  clippy:
    name: Clippy
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy
      - uses: Swatinem/rust-cache@v2
      - run: cargo clippy --all-targets
```

## Verification

### 1. Build

```bash
cargo build --release
```

### 2. Unit Tests

```bash
cargo test
```

#### Loop Control Logic Test

A unit test with a mocked agent to verify the loop continuation mechanism:

```rust
// In src/loop_controller.rs or tests/loop_control_test.rs

#[cfg(test)]
mod tests {
    use super::*;

    /// Mock agent that returns unfulfilled promise N times, then fulfilled
    struct MockAgent {
        calls_until_fulfilled: usize,
        call_count: std::cell::RefCell<usize>,
    }

    impl MockAgent {
        fn new(calls_until_fulfilled: usize) -> Self {
            Self {
                calls_until_fulfilled,
                call_count: std::cell::RefCell::new(0),
            }
        }

        fn call_count(&self) -> usize {
            *self.call_count.borrow()
        }
    }

    impl Agent for MockAgent {
        fn run(&self, _prompt: &str) -> AgentResult {
            let mut count = self.call_count.borrow_mut();
            *count += 1;
            if *count >= self.calls_until_fulfilled {
                AgentResult::with_promise("DONE")
            } else {
                AgentResult::without_promise()
            }
        }
    }

    #[test]
    fn loop_continues_until_promise_fulfilled() {
        let mock = MockAgent::new(3);
        let controller = LoopController::new(Config {
            promise_condition: "DONE".to_string(),
            max_iterations: None,
            ..Default::default()
        });

        let result = controller.run_with_agent(&mock);

        assert!(result.is_ok());
        assert_eq!(mock.call_count(), 3); // Called exactly 3 times
    }

    #[test]
    fn loop_stops_immediately_when_promise_fulfilled_first_try() {
        let mock = MockAgent::new(1);
        let controller = LoopController::new(Config {
            promise_condition: "DONE".to_string(),
            max_iterations: None,
            ..Default::default()
        });

        let result = controller.run_with_agent(&mock);

        assert!(result.is_ok());
        assert_eq!(mock.call_count(), 1); // Stopped after first call
    }

    #[test]
    fn loop_respects_max_iterations_when_promise_never_fulfilled() {
        let mock = MockAgent::new(100); // Would need 100 calls
        let controller = LoopController::new(Config {
            promise_condition: "DONE".to_string(),
            max_iterations: Some(5),
            ..Default::default()
        });

        let result = controller.run_with_agent(&mock);

        assert!(result.is_err()); // Should error: max iterations exceeded
        assert_eq!(mock.call_count(), 5); // Stopped at limit
    }
}
```

This test requires an `Agent` trait abstraction that `LoopController` can accept, enabling dependency injection for testing.

### 3. Integration Test with Mock

```bash
# Create test script that outputs promise
echo '#!/bin/bash
echo "Working..."
sleep 1
echo "<promise>DONE</promise>"' > /tmp/mock-claude.sh
chmod +x /tmp/mock-claude.sh

# Run ralph-loop with mock
./target/release/ralph-loop \
    -p "Test prompt" \
    -c "DONE" \
    --claude-path /tmp/mock-claude.sh
```

### 4. Real Test (with iteration limit)

```bash
./target/release/ralph-loop \
    -p "Say hello and output <promise>DONE</promise>" \
    -c "DONE" \
    -m 3
```

### 5. Infinite Loop Test

Runs until promise found or Ctrl+C:

```bash
./target/release/ralph-loop \
    -p "Keep trying until you succeed, then output <promise>DONE</promise>" \
    -c "DONE"
```
