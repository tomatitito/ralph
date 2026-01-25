# Ralph Architecture - Revised Plan

## Overview

Ralph consists of two components:
1. **ralph-loop**: Orchestrates running Claude Code in a loop until a completion promise is found
2. **ralph-viewer**: Views transcripts from ralph-loop runs

## Key Insight

Claude Code already stores comprehensive transcripts at:
```
~/.claude/projects/<project-path>/<session-id>.jsonl
```

These transcripts include all messages, tool use, thinking, token usage, etc. **Ralph-loop should not duplicate this data.**

---

## Architecture

```
┌─────────────────┐                    ┌──────────────────────────────┐
│   ralph-loop    │                    │  Claude Code                 │
│                 │ ───── runs ──────▶ │                              │
│  • orchestrates │                    │  • stores full transcripts   │
│  • tracks state │                    │    in ~/.claude/projects/    │
│  • detects done │                    │                              │
└────────┬────────┘                    └──────────────────────────────┘
         │
         │ writes metadata only
         ▼
┌──────────────────────────────────────┐
│  .ralph-loop-output/                 │
│  └── runs/<run-id>/                  │
│      └── meta.json                   │
│          (maps iterations → sessions)│
└──────────────────────────────────────┘
         │
         │ reads metadata
         ▼
┌──────────────────────────────────────┐
│  ralph-viewer                        │
│                                      │
│  • reads run metadata from           │
│    .ralph-loop-output/               │
│  • reads actual transcripts from     │
│    ~/.claude/projects/               │
│  • displays formatted output         │
└──────────────────────────────────────┘
```

---

## Part 1: Ralph-loop Output

### Directory Structure

```
.ralph-loop-output/
├── runs/
│   ├── <run-id>/
│   │   └── meta.json          # Run metadata with session mappings
│   └── <run-id>/
│       └── meta.json
└── latest -> runs/<most-recent-run-id>/
```

Note: No transcript files. Only metadata.

### Run Metadata (`meta.json`)

```json
{
  "run_id": "20250125-143022-abc123",
  "status": "completed",
  "started_at": "2025-01-25T14:30:22Z",
  "completed_at": "2025-01-25T14:45:10Z",
  "project_path": "/home/user/myproject",
  "prompt_file": "task.txt",
  "prompt_preview": "First 100 chars of prompt...",
  "completion_promise": "TASK COMPLETE",
  "exit_reason": "promise_fulfilled",
  "iterations": [
    {
      "iteration": 1,
      "session_id": "7ff71072-5080-408c-b2f0-2f140b159a7c",
      "started_at": "2025-01-25T14:30:22Z",
      "ended_at": "2025-01-25T14:35:00Z",
      "end_reason": "context_limit",
      "tokens": {
        "input": 150000,
        "output": 8000
      }
    },
    {
      "iteration": 2,
      "session_id": "abc12345-6789-4def-0123-456789abcdef",
      "started_at": "2025-01-25T14:35:05Z",
      "ended_at": "2025-01-25T14:45:10Z",
      "end_reason": "promise_found",
      "tokens": {
        "input": 45000,
        "output": 3000
      }
    }
  ]
}
```

### What ralph-loop needs to capture

From Claude Code's JSON output, ralph-loop extracts:
- **Session ID**: From `init` or `result` events
- **Token usage**: From `result` events
- **Completion detection**: Scan assistant messages for the promise text

Ralph-loop does NOT need to store the full transcript - Claude Code already does that.

---

## Part 2: Ralph-viewer

### Data Sources

The viewer reads from two locations:

1. **Run metadata**: `.ralph-loop-output/runs/<run-id>/meta.json`
   - Lists iterations and their session IDs
   - Provides run status, timing, configuration

2. **Session transcripts**: `~/.claude/projects/<project-path>/<session-id>.jsonl`
   - Full conversation history
   - All tool calls and results
   - Token usage details

### Resolving Project Path

The project path in `~/.claude/projects/` is derived from the working directory:
- `/home/sprite/ralph` → `-home-sprite-ralph`

The viewer can:
1. Use `project_path` from meta.json
2. Or derive it from the current working directory

### CLI Interface

```bash
# Default: show picker if multiple runs, auto-select if one
ralph-viewer

# View specific run
ralph-viewer --run 20250125-143022-abc123

# View specific iteration of a run
ralph-viewer --run 20250125-143022-abc123 --iteration 2

# List all runs
ralph-viewer --list

# Specify custom output directory
ralph-viewer --dir /path/to/.ralph-loop-output

# Don't follow live updates
ralph-viewer --no-follow
```

### Interactive Picker UI

When running `ralph-viewer` without arguments, the user is presented with interactive selection menus.

**Step 1: Select a run**

```
┌─ Select a run ─────────────────────────────────────────────────────┐
│ > 20250125-143022  (completed, 2 iters)   "Fix authentication..."  │
│   20250125-120000  (running, iter 3)      "Add user profile..."    │
│   20250124-093015  (failed, 5 iters)      "Refactor database..."   │
└────────────────────────────────────────────────────────────────────┘

Use ↑↓ to navigate, Enter to select, q to quit
```

**Step 2: Select an iteration**

After selecting a run, choose which iteration to view:

```
┌─ Select iteration ─────────────────────────────────────────────────┐
│ > [all] View entire run transcript                                  │
│   Iteration 2  (completed, 48k tokens)  - promise found             │
│   Iteration 1  (158k tokens)            - context limit             │
└─────────────────────────────────────────────────────────────────────┘
```

**Behavior:**
- If only one run exists → auto-select it, show iteration picker
- If only one iteration exists → auto-select it, show transcript
- `--run` flag → skip run picker
- `--iteration` flag → skip iteration picker

### Display

The viewer shows:
- Iteration boundaries
- Assistant messages (formatted)
- Tool calls (name, inputs)
- Tool results (summarized for large content)
- Token usage per iteration
- Timestamps

---

## Tasks

### Phase 1: Update ralph-loop metadata

- [ ] 1.1 Capture session ID from Claude Code output
  - Parse `init` or `result` events for `session_id`
  - Store in iteration metadata

- [ ] 1.2 Update meta.json schema
  - Add `project_path` field
  - Add `iterations` array with session mappings
  - Remove transcript file writing (no more `iteration_NNN.jsonl`)

- [ ] 1.3 Update tests
  - Test session ID extraction
  - Test new meta.json format

### Phase 2: Update ralph-viewer

- [ ] 2.1 Read Claude Code transcripts
  - Locate `~/.claude/projects/<project-path>/`
  - Read `<session-id>.jsonl` files
  - Parse Claude Code's transcript format

- [ ] 2.2 Map iterations to transcripts
  - Read run metadata to get session IDs
  - Look up corresponding transcript files
  - Handle missing transcript files gracefully

- [ ] 2.3 Update transcript parsing
  - Adapt to Claude Code's event format (may differ from current)
  - Handle all event types: user, assistant, tool_use, tool_result, progress, etc.

- [ ] 2.4 Update display formatting
  - Show iteration headers based on metadata
  - Stream transcript content

### Phase 3: Polish

- [ ] 3.1 Handle edge cases
  - Transcript file not found (session cleaned up)
  - Multiple projects with same run
  - Concurrent runs

- [ ] 3.2 Add helpful messages
  - Show transcript file location
  - Warn if transcript appears incomplete

- [ ] 3.3 Rename meta.json to .ralph-meta.json
  - Update ralph-loop to write `.ralph-meta.json` instead of `meta.json`
  - Update ralph-viewer to read from `.ralph-meta.json`
  - Update all documentation references

---

## Claude Code Transcript Format

Events in `~/.claude/projects/<project>/<session>.jsonl`:

```jsonl
{"type":"user","message":{"role":"user","content":"..."},"uuid":"...","timestamp":"..."}
{"type":"assistant","message":{"role":"assistant","content":[...]},"uuid":"...","timestamp":"..."}
{"type":"tool_use","..."}
{"type":"tool_result","..."}
{"type":"result","usage":{...},"total_cost_usd":0.05,"session_id":"..."}
```

Key fields:
- `type`: Event type (user, assistant, tool_use, tool_result, result, progress, etc.)
- `message`: The actual message content
- `uuid`: Unique event ID
- `parentUuid`: Links to parent event
- `timestamp`: ISO timestamp
- `sessionId`: Session identifier

---

## Benefits of This Architecture

1. **No data duplication**: Transcripts stored once by Claude Code
2. **Smaller output**: ralph-loop only stores ~1KB metadata per run
3. **Full fidelity**: Access to Claude Code's complete transcript format
4. **Future-proof**: If Claude Code adds new event types, they're automatically available
5. **Debuggable**: Can use standard tools to inspect Claude's native transcripts
