# Ralph Summary Writing on Context Limit - Implementation Plan

## Problem

Currently, when an iteration is killed due to context window limit:
- The iteration metadata is properly recorded in `.ralph-meta.json`
- The loop continues to the next iteration
- **BUT**: No summary is written for the killed iteration before restart

This means we lose visibility into what progress was made before hitting the limit.

## Solution: Option 2 (Summary Mini-Iteration After Kill)

When an iteration is killed due to context limit, immediately start a "summary mini-iteration" that asks Claude to summarize what happened in the killed iteration, then attach that summary to the killed iteration's metadata.

### Research Findings: Why We Can't Inject Mid-Execution

**stdin cannot be used for mid-execution requests** because:
1. Headless mode (`-p` flag) is designed for one-shot prompts
2. stdin is closed immediately after the initial prompt to signal EOF
3. Claude Code uses stdin closure as the signal that prompt input is complete
4. No mechanism exists to send additional messages while Claude is running

See: `ralph-loop-rs/src/process.rs:48-52` where stdin is explicitly dropped after initial write.

### High-Level Flow

```
Iteration N starts
    ↓
Claude works, tokens accumulate
    ↓
Tokens reach MAX threshold (180K)
    ↓
Kill process
    ↓
Record iteration N: end_reason = ContextLimit, session_id = ABC
    ↓
Start MINI-ITERATION (summary request)
    ↓
Prompt: "Summarize session ABC - what was accomplished, what remains?"
    ↓
Claude reads transcript and provides summary
    ↓
Capture summary response
    ↓
Attach summary to iteration N's metadata
    ↓
Start iteration N+1 with original task (normal flow)
```

### Benefits

- ✅ Gets Claude's perspective on what was accomplished (better than parsing)
- ✅ Full iteration summary (not just the tail end)
- ✅ Simple implementation (reuses existing agent infrastructure)
- ✅ Human-readable progress report
- ✅ Useful for ralph-viewer

### Costs

- Extra Claude invocation per context-limit kill (~5-10K tokens)
- Adds one more iteration to the count (or needs special "summary" iteration type)

---

## Architecture Changes

### 1. Add Summary Iteration Type

**Option A**: Don't count summary iterations
```rust
pub enum IterationType {
    Normal,      // Counts toward max_iterations
    Summary,     // Doesn't count, purely for metadata
}
```

**Option B**: Count them (simpler)
- Just treat as normal iteration
- Document that context-limit kills add an extra iteration

**Recommendation**: Start with Option B (simpler), add Option A if needed

### 2. Summary Storage

**transcript.rs** needs:
```rust
pub struct IterationMetadata {
    // ... existing fields ...
    
    /// Optional summary when iteration ended early
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}

impl TranscriptWriter {
    /// Write summary for a specific iteration (not necessarily current)
    pub fn write_iteration_summary(&mut self, iteration: usize, summary: String) -> Result<()> {
        // Find iteration by number and update its summary field
        // Write to .ralph-meta.json
    }
}
```

### 3. Loop Controller Integration

**loop_controller.rs** needs to:
1. Detect when iteration ends with `ExitReason::ContextLimit`
2. Run a summary mini-iteration
3. Attach summary to the killed iteration
4. Continue with normal next iteration

---

## Implementation Tasks

### Phase 1: Add Summary Field to Metadata

#### 1.1 Add Summary Field to IterationMetadata
**File**: `src/transcript.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterationMetadata {
    pub iteration: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    pub started_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ended_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_reason: Option<IterationEndReason>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens: Option<TokenCount>,
    
    /// Summary captured after context limit kill
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}
```

#### 1.2 Add Method to Write Summary to Specific Iteration
**File**: `src/transcript.rs`

```rust
impl TranscriptWriter {
    /// Write summary for a specific iteration number
    pub fn write_iteration_summary(&mut self, iteration_num: usize, summary: String) -> Result<()> {
        // Find the iteration by number
        if let Some(iter) = self.metadata.iterations.iter_mut()
            .find(|i| i.iteration == iteration_num) {
            iter.summary = Some(summary);
            self.write_metadata()?;
        }
        Ok(())
    }
}
```

### Phase 2: Implement Summary Mini-Iteration

#### 2.1 Add Summary Prompt Generator
**File**: `src/loop_controller.rs` (or new `src/summary.rs`)

```rust
/// Generate a prompt to summarize a killed iteration
fn create_summary_prompt(session_id: &str, original_task: &str) -> String {
    format!(
        r#"The previous iteration (session ID: {}) was terminated due to context limit.

Please read the transcript for that session and provide a concise summary covering:
1. What task you were working on
2. What progress was made / what was accomplished
3. What was in-progress when the session ended
4. What remains to be done

Original task: {}

Keep the summary brief but informative (3-5 paragraphs maximum)."#,
        session_id,
        original_task
    )
}
```

#### 2.2 Update Loop Controller to Handle Context Limit
**File**: `src/loop_controller.rs`

```rust
// After agent.run() returns (around line 104)
let result = agent.run(&prompt).await?;

// Set session ID
if let Some(session_id) = &result.session_id {
    writer.set_session_id(session_id.clone());
}

// Extract iteration end reason
let end_reason = match result.exit_reason {
    ExitReason::Natural => IterationEndReason::Natural,
    ExitReason::ContextLimit => IterationEndReason::ContextLimit,
    ExitReason::Shutdown => IterationEndReason::Shutdown,
};

// End iteration with metadata
let input_tokens = result.token_usage.as_ref().map_or(0, |u| u.input_tokens);
let output_tokens = result.token_usage.as_ref().map_or(0, |u| u.output_tokens);
writer.end_iteration(end_reason, input_tokens, output_tokens);

// NEW: If context limit was hit, run summary mini-iteration
if end_reason == IterationEndReason::ContextLimit {
    if let Some(session_id) = &result.session_id {
        info!("Running summary iteration for context-limited session {}", session_id);
        
        // Generate summary prompt
        let summary_prompt = create_summary_prompt(session_id, &original_task);
        
        // Run summary iteration (allow Read tool for transcript access)
        let summary_result = agent.run(&summary_prompt).await?;
        
        // Extract summary from output
        let summary = summary_result.output.clone();
        
        // Attach summary to the killed iteration
        writer.write_iteration_summary(iteration, summary)?;
        
        info!("Summary written for iteration {}", iteration);
    }
}

// Check if promise found
let promise_found = result.is_fulfilled();
```

### Phase 3: Handle Edge Cases

#### 3.1 Track Original Task
**File**: `src/loop_controller.rs`

Need to preserve the original task prompt across iterations:

```rust
pub struct LoopController {
    agent: Arc<dyn Agent>,
    original_task: String,  // NEW: Store original task for summary prompts
}

impl LoopController {
    pub fn new(agent: Arc<dyn Agent>, task: String) -> Self {
        Self {
            agent,
            original_task: task,
        }
    }
}
```

Update construction in `main.rs`:
```rust
let controller = LoopController::new(agent, task_from_file);
```

#### 3.2 Handle Summary Iteration Failures
**File**: `src/loop_controller.rs`

```rust
// Run summary iteration with error handling
let summary_result = match agent.run(&summary_prompt).await {
    Ok(result) => result,
    Err(e) => {
        warn!("Failed to generate summary: {}", e);
        // Write a fallback summary
        writer.write_iteration_summary(
            iteration,
            format!("Summary generation failed: {}. Session ID: {}", e, session_id)
        )?;
        continue; // Skip to next iteration
    }
};
```

#### 3.3 Summary Iteration Configuration
**File**: `src/config.rs`

Add config options for summary behavior:

```rust
pub struct Config {
    // ... existing fields ...
    
    /// Whether to generate summaries for context-limited iterations
    pub generate_summaries: bool,
    
    /// Allowed tools for summary iterations (should include Read)
    pub summary_allowed_tools: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            // ... existing defaults ...
            generate_summaries: true,
            summary_allowed_tools: vec!["Read".to_string()],
        }
    }
}
```

### Phase 4: Additional Fix - Shutdown Signal Gap

#### 4.1 Handle Shutdown in Loop Controller
**File**: `src/loop_controller.rs`

Pass shutdown receiver into loop controller:

```rust
pub async fn run(
    &mut self,
    config: Arc<Config>,
    mut shutdown_rx: tokio::sync::broadcast::Receiver<()>,
) -> Result<LoopResult> {
    // ... setup ...
    
    loop {
        // Run agent with shutdown handling
        tokio::select! {
            result = agent.run(&prompt) => {
                // ... existing iteration logic ...
            }
            _ = shutdown_rx.recv() => {
                info!("Shutdown signal received, closing transcript");
                writer.complete(TranscriptExitReason::Shutdown);
                return Err(RalphError::ShutdownRequested);
            }
        }
        
        // ... rest of loop ...
    }
}
```

#### 4.2 Update main.rs
**File**: `src/main.rs`

```rust
// Create broadcast channel for shutdown
let (shutdown_tx, shutdown_rx) = tokio::sync::broadcast::channel(1);

// Spawn shutdown listener
tokio::spawn(async move {
    tokio::signal::ctrl_c().await.ok();
    let _ = shutdown_tx.send(());
});

// Run loop controller with shutdown receiver
match loop_controller.run(config.clone(), shutdown_rx).await {
    Ok(result) => { /* ... */ },
    Err(RalphError::ShutdownRequested) => {
        info!("Shutdown completed gracefully");
        Ok(())
    },
    Err(e) => Err(e),
}
```

---

## Testing Strategy

### Unit Tests

1. **Summary field serialization**
   - Test that `IterationMetadata` with summary serializes correctly
   - Test that `None` summary is omitted from JSON
   - Test that summary appears in JSON when present

2. **Summary writing**
   - Test `write_iteration_summary()` updates correct iteration
   - Test summary persists to `.ralph-meta.json`

3. **Summary prompt generation**
   - Test `create_summary_prompt()` includes session ID
   - Test it includes original task

### Integration Tests

1. **Context limit with summary**
   - Mock agent that returns context limit after N tokens
   - Verify summary iteration is triggered
   - Verify summary is attached to killed iteration
   - Verify next normal iteration continues

2. **Summary iteration failure**
   - Mock summary iteration that fails
   - Verify fallback summary is written
   - Verify loop continues

3. **Shutdown handling**
   - Send shutdown signal during iteration
   - Verify transcript is properly closed
   - Verify run status is set correctly

### Manual Testing

1. Run ralph-loop with a long task that will hit context limit
2. Verify summary iteration runs automatically
3. Check `.ralph-meta.json` for summary content
4. Verify summary is meaningful and useful

---

## Example Output

After context limit is hit, `.ralph-meta.json` should look like:

```json
{
  "run_id": "20260125-143022-abc123",
  "iterations": [
    {
      "iteration": 1,
      "session_id": "7ff71072-5080-408c-b2f0-2f140b159a7c",
      "started_at": "2026-01-25T14:30:22Z",
      "ended_at": "2026-01-25T14:45:00Z",
      "end_reason": "context_limit",
      "tokens": {
        "input": 180000,
        "output": 8000
      },
      "summary": "I was working on implementing user authentication for the web application. Progress made:\n\n1. Created the User model with password hashing\n2. Implemented login/logout endpoints\n3. Added JWT token generation\n4. Started working on middleware for protected routes\n\nWhen the session ended, I was in the middle of implementing the auth middleware. The middleware structure is in place but not yet integrated with the route handlers.\n\nRemaining work:\n- Complete auth middleware integration\n- Add token refresh endpoint\n- Implement password reset flow\n- Write tests for auth system"
    },
    {
      "iteration": 2,
      "session_id": "abc12345-6789-4def-0123-456789abcdef",
      "started_at": "2026-01-25T14:45:10Z",
      "ended_at": "2026-01-25T15:00:00Z",
      "end_reason": "promise_found",
      "tokens": {
        "input": 45000,
        "output": 3000
      }
    }
  ]
}
```

---

## Open Questions

### Q1: Should summary iterations count toward max_iterations?

**Options**:
- **A**: Yes (simpler implementation)
- **B**: No (prevents summary from causing early termination)

**Recommendation**: Start with A, add iteration type tracking if it becomes a problem

### Q2: What tools should summary iteration have access to?

**Must have**: `Read` (to read the transcript)

**Consider**: 
- `Grep` (to search transcript)
- `Bash` (to run claude-viewer or similar?)

**Recommendation**: Start with just `Read`, expand if needed

### Q3: How detailed should summaries be?

**Guidance in prompt**:
- "Brief but informative (3-5 paragraphs maximum)"
- Can adjust based on real-world usage

**Option**: Add token limit to summary prompt to keep it concise

### Q4: Should we write summary to separate file?

**Options**:
- **A**: Inline in `.ralph-meta.json` (current plan)
- **B**: Separate file: `.ralph-loop-output/runs/<run-id>/iteration_<N>_summary.md`
- **C**: Both

**Recommendation**: Start with A (simpler). If summaries get large (>1KB), add B.

---

## Rollout Plan

### Phase 1: Basic Implementation ✅
- Add `summary` field to `IterationMetadata`
- Add `write_iteration_summary()` method
- Implement summary mini-iteration trigger
- Basic summary prompt

**Goal**: Get end-to-end flow working

### Phase 2: Refinement 🔄
- Improve summary prompt quality
- Add error handling for summary failures
- Add configuration options
- Test with real ralph-loop runs

**Goal**: Make it robust and configurable

### Phase 3: Polish ✨
- Fix shutdown signal handling
- Add comprehensive tests
- Update documentation
- Consider separate summary files if needed

**Goal**: Production-ready feature

---

## Success Criteria

1. ✅ When context limit is reached, a summary iteration runs automatically
2. ✅ Summary is written to `.ralph-meta.json` for the killed iteration
3. ✅ Summary captures meaningful progress information from Claude's perspective
4. ✅ Loop continues to next normal iteration after summary
5. ✅ Shutdown signals properly close transcript
6. ✅ Tests verify all new functionality
7. ✅ No regressions in existing behavior
8. ✅ Summary generation can be configured/disabled

---

## Implementation Estimate

**Complexity**: Medium

**Files to modify**:
- `src/transcript.rs` - Add summary field and write method
- `src/loop_controller.rs` - Add summary iteration logic
- `src/config.rs` - Add summary configuration
- `src/main.rs` - Update shutdown handling

**New code**: ~150-200 lines

**Testing**: ~100 lines

**Timeline**: Can be implemented incrementally in small, testable chunks
