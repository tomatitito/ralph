# Architecture

## System Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                      Main Controller                             │
│                      (LoopController)                            │
└───────────────────────────┬─────────────────────────────────────┘
                            │
        ┌───────────────────┼───────────────────┐
        ▼                   ▼                   ▼
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│ Process Task │    │ Stdout       │    │ Stderr       │
│ (subprocess) │    │ Monitor      │    │ Monitor      │
└──────┬───────┘    └──────┬───────┘    └──────┬───────┘
       │                   │                   │
       │                   └─────────┬─────────┘
       │                             ▼
       │                   ┌──────────────────┐
       │                   │  Token Counter   │
       │                   │  + Promise Check │
       │                   └────────┬─────────┘
       │                            │
       └────────────────────────────┼──────────────────────────────┐
                                    ▼                              │
                          ┌──────────────────┐                     │
                          │   SharedState    │◄────────────────────┘
                          │  (RwLock-based)  │
                          └──────────────────┘
```

## Concurrency Pattern

The main loop uses `tokio::select!` to handle multiple concurrent events:

```rust
tokio::select! {
    // Wait for Claude to exit naturally
    exit_status = process.wait() => {
        // Check if promise was found in output
        if *state.promise_found.read().await {
            return Ok(IterationResult::PromiseFound);
        }
        return Ok(IterationResult::ProcessExited);
    }

    // Or receive kill command from monitor (context limit)
    Some(ProcessCommand::Kill) = cmd_rx.recv() => {
        process.kill().await?;
        return Ok(IterationResult::ContextLimitReached);
    }

    // Or receive shutdown signal (Ctrl+C)
    _ = shutdown_rx.recv() => {
        process.kill().await?;
        return Err(RalphError::ShutdownRequested);
    }
}
```

## Main Flow

```
iteration = 0
loop:
    iteration += 1

    if max_iterations.is_some() && iteration > max_iterations.unwrap():
        return MaxIterationsReached

    reset_state()
    spawn_claude(prompt)
    spawn_monitors(stdout, stderr)

    select!:
        process exits → check promise_found
        shutdown signal → kill and exit

    if promise_found:
        return Success
    else:
        continue (restart fresh)
```
