# Ralph Loop Rust Specification

A concurrent Rust application that runs Claude Code in a loop with real-time context monitoring. It spawns Claude as a subprocess and concurrently monitors output for token count and completion promises.

## Documentation

- [Architecture](./architecture.md) - System architecture and concurrency model
- [Components](./components.md) - Core components and their responsibilities
- [CLI Interface](./cli.md) - Command-line interface and usage
- [Implementation Plan](./implementation_plan.md) - Phased implementation with acceptance criteria

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