# Ralph Viewer Analysis

## Error Diagnosis

When running `ralph-viewer` from the terminal, the following warning is displayed:

```
Transcripts from: /Users/dusty/.claude/projects/Volumes-sourcecode-personal-ralph

⚠ No session ID recorded for iteration 1
```

### Root Cause

The error occurs because the `.ralph-meta.json` file contains a `null` value for `session_id` in the iteration metadata:

```json
{
  "iterations": [
    {
      "iteration": 1,
      "session_id": null,
      "started_at": "2026-02-04T15:58:43.663831Z"
    }
  ]
}
```

This happens because:

1. **Timing Issue**: The `ralph-loop` controller writes iteration metadata via `TranscriptWriter::start_iteration()` *before* the Claude process runs and produces any output.

2. **Session ID Capture**: The session ID is only available after Claude Code starts and emits an `init` or `result` JSON event. The `JsonEventMonitor` in `monitor.rs:159-164` captures the session ID from these events.

3. **Late Update**: The session ID is updated via `TranscriptWriter::set_session_id()` in `loop_controller.rs:137-144`, but this happens *after* the iteration has started. If the viewer is launched while ralph-loop is still in its first iteration (before any Claude output), the session_id will be `null`.

4. **Current Run**: The metadata file being read is from a currently running ralph-loop session, which hasn't completed its first iteration yet.

### Code Flow

1. `LoopController::run()` calls `writer.start_iteration()` → writes iteration with `session_id: null`
2. `ClaudeAgent::run()` spawns Claude process
3. `JsonEventMonitor` waits for stdout JSON events
4. On `init` or `result` event, session_id is captured
5. Back in `LoopController`, `writer.set_session_id()` updates the metadata
6. If viewer reads before step 5, it sees `null`

### Not Actually an Error

This is expected behavior for in-progress iterations. The warning is informational, indicating that the transcript file cannot be located yet because Claude Code hasn't produced its session ID.

## Required Enhancement

Per user request, enhance `ralph-viewer` to show:

1. **Run Summary List**: Display all ralph-loops in the current directory with:
   - Why they finished (promise found, max iterations, interrupted, etc.)
   - How long they ran (duration)
   - How many tokens were used (total across all iterations)

2. **Currently Running Loop**: Show information about any active ralph-loop:
   - How long it has been running
   - How many tokens have been used so far

### Implementation Plan

1. **Add a `--summary` flag** (or make it the default behavior) that shows a table of runs with:
   - Run ID
   - Status (running/completed/failed/interrupted)
   - Exit reason (for completed runs)
   - Duration (elapsed time)
   - Token count (input + output)
   - Iteration count
   - Brief prompt preview

2. **Highlight currently running runs** with special formatting and live duration.

3. **Format output** as a readable table or list.

### Files to Modify

- `ralph-viewer/src/main.rs` - Add new CLI flag and summary mode
- `ralph-viewer/src/run.rs` - Add methods for duration calculation and exit reason display
- `ralph-viewer/src/formatter.rs` - Add summary display formatting (or new module)

### Testing

- Add tests for duration calculation
- Add tests for exit reason display
- Add tests for summary formatting
