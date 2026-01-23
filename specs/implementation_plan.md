# Implementation Plan

## Project Structure

```
ralph-loop/
├── Cargo.toml
└── src/
    ├── main.rs              # CLI entry point, signal handling
    ├── lib.rs               # Library exports
    ├── config.rs            # Configuration structures
    ├── loop_controller.rs   # Main orchestration
    ├── process.rs           # Claude subprocess management
    ├── monitor.rs           # Output monitoring (tokens + promises)
    ├── token_counter.rs     # Token estimation
    ├── state.rs             # Shared state and events
    └── error.rs             # Error types
```

## Files to Create

| File | Purpose |
|------|---------|
| `ralph-loop/Cargo.toml` | Project manifest with dependencies |
| `ralph-loop/src/main.rs` | CLI parsing, signal handling, entry point |
| `ralph-loop/src/lib.rs` | Module exports |
| `ralph-loop/src/config.rs` | Config structs and loading |
| `ralph-loop/src/error.rs` | RalphError enum |
| `ralph-loop/src/state.rs` | SharedState and events |
| `ralph-loop/src/token_counter.rs` | Token estimation |
| `ralph-loop/src/process.rs` | ClaudeProcess wrapper |
| `ralph-loop/src/monitor.rs` | OutputMonitor with concurrent readers |
| `ralph-loop/src/loop_controller.rs` | Main orchestration logic |

## Verification

### 1. Build

```bash
cargo build --release
```

### 2. Unit Tests

```bash
cargo test
```

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
