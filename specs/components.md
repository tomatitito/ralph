# Core Components

## Configuration (`config.rs`)

```rust
pub struct Config {
    pub max_iterations: Option<u32>,   // None = infinite loop, Some(n) = limit to n iterations
    pub completion_promise: String,    // Default: "TASK COMPLETE"
    pub context_limit: ContextLimitConfig,
    pub output_dir: PathBuf,
    pub claude_path: String,           // Default: "claude"
    pub claude_args: Vec<String>,      // Default: ["--dangerously-skip-permissions"]
}

pub struct ContextLimitConfig {
    pub max_tokens: usize,             // Default: 180_000
    pub warning_threshold: usize,      // Default: 150_000
    pub estimation_method: TokenEstimationMethod,
}
```

## Shared State (`state.rs`)

```rust
pub struct SharedState {
    pub token_count: RwLock<usize>,
    pub output_buffer: RwLock<String>,
    pub promise_found: RwLock<bool>,
    pub iteration: RwLock<u32>,
}
```

## Process Management (`process.rs`)

- Spawn Claude with `tokio::process::Command`
- Pipe prompt to stdin
- Capture stdout/stderr as async streams
- Provide `kill()` method for proactive termination

## Output Monitor (`monitor.rs`)

Two concurrent tasks (stdout + stderr):
- Read lines asynchronously
- Append to shared buffer
- Update token count via TokenCounter
- Check for `<promise>TEXT</promise>` pattern
- If context limit reached → send Kill command
- If promise found → set `promise_found = true`

## Token Counter (`token_counter.rs`)

Three estimation methods:
- **Tiktoken**: Use `tiktoken-rs` with `cl100k_base` encoding (most accurate)
- **ByteRatio**: `text.len() / 4`
- **CharRatio**: `text.chars().count() / 4`

## Loop Controller (`loop_controller.rs`)

Main orchestration logic that coordinates all components and manages the iteration loop.

## Dependencies

```toml
[package]
name = "ralph-loop"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.35", features = ["full", "process", "sync", "signal"] }
clap = { version = "4.4", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
tiktoken-rs = "0.5"
thiserror = "1.0"
anyhow = "1.0"
regex = "1.10"
colored = "2.1"
```
