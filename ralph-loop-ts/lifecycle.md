# Loop Lifecycle

## Iteration definition

An iteration is one fresh Pi session execution from prompt submission to `agent_end`, followed by Ralph-specific evaluation.

The runtime model for v1 is:
- create a fresh Pi session
- inject the original objective plus carried-forward summary
- let Pi run until it becomes idle and emits `agent_end`
- evaluate iteration state
- either terminate or start a fresh next iteration

The loop does not keep a single long-lived Pi session across iterations.

## Canonical iteration phases

Each iteration has these phases:

1. **Prepare**
   - increment iteration counter
   - check max-iteration bound before starting work
   - assemble iteration input
   - create fresh Pi session and bind Ralph extensions

2. **Run**
   - submit the iteration prompt
   - allow Pi to run until `agent_end`
   - extensions observe markers and context usage during the run

3. **Observe**
   - read extension-produced iteration state
   - collect session reference, token/context usage, markers, and diagnostics

4. **Check**
   - run `after_iteration` checks
   - record all outputs and statuses

5. **Validate**
   - if loop completion was claimed and `after_iteration` checks passed, run completion validators
   - if completion validators pass, run `before_final_success` checks

6. **Summarize**
   - write iteration metadata
   - write logs and command results
   - write handoff summary for the next iteration if another iteration is needed

7. **Decide**
   - terminate successfully
   - terminate with failure/interruption
   - or start the next fresh iteration

## Signals

Version 1 uses these conceptual signals:

1. `task_boundary`
   - the current task or subtask is complete
2. `overall_completion_claim`
   - the agent claims the full objective is complete
3. `context_limit_hit`
   - the context monitor determined the current session should stop
4. `checks_passed`
   - all `after_iteration` checks passed
5. `completion_validated`
   - all completion validators passed
6. `final_checks_passed`
   - all `before_final_success` checks passed

## Agent markers

Version 1 uses explicit structured markers in assistant output.

Canonical marker strings:
- task complete: `<ralph:task-complete/>`
- loop complete: `<ralph:loop-complete/>`

### Marker rules

- Markers may appear anywhere in assistant text content.
- Markers are detected across the whole iteration, not just the final assistant message.
- The marker extension records booleans for whether each marker appeared.
- If both markers appear in one iteration:
  - treat that as both a task boundary and a loop completion claim
  - the loop completion path takes precedence in final decision-making
- Markers are control signals, not user-facing output requirements.
- Handoff summaries should mention marker outcomes, but should not reproduce raw markers unless useful for debugging.

## Post-iteration evaluation order

After `agent_end`, the controller must evaluate in this order:

1. read extension state
2. run `after_iteration` checks
3. if loop completion was claimed and `after_iteration` checks passed:
   - run completion validators
4. if completion validators passed:
   - run `before_final_success` checks
5. write iteration artifacts
6. decide next state

This order is canonical for v1.

## Decision rules

### Successful termination
Terminate successfully only if all of the following are true:
- loop-complete marker present
- `after_iteration` checks passed
- completion validators passed
- `before_final_success` checks passed

### Restart after task completion
Start a fresh next iteration if all of the following are true:
- task-complete marker present
- loop-complete marker absent, or present but success conditions not met
- run has not been terminated by max-iteration failure or user interruption

Checks are still run before restarting.

### Restart after failed completion attempt
Start a fresh next iteration if:
- loop-complete marker present
- but any of these fail:
  - `after_iteration` checks
  - completion validators
  - `before_final_success` checks

### Restart after context limit
Start a fresh next iteration if:
- context monitor marked the iteration as context-limited
- and the run was not otherwise terminated by user interruption or max-iteration failure

Checks still run in this case.

### Restart after incomplete iteration
Start a fresh next iteration if:
- no loop-complete success condition was met
- and the run is still within its iteration bound

This includes the case where:
- no markers appeared
- checks passed
- the agent simply stopped without giving a task or loop completion signal

## Precedence rules

When multiple conditions occur in one iteration, resolve them in this order:

1. user interruption / process shutdown
2. max-iteration bound exceeded
3. successful loop completion
4. failed loop-completion claim
5. context-limit restart
6. task-boundary restart
7. generic incomplete-iteration restart

### Examples

#### Loop marker + failed checks
- not a success
- restart fresh with summary and failure outputs

#### Context limit + task marker
- treat as context-limit restart
- preserve that the task marker was seen in artifacts
- do not skip checks

#### Loop marker + context limit
- run checks and validation based on recorded iteration state
- success still requires all success conditions
- otherwise restart

#### No marker + checks pass
- not a success
- restart until success or max iterations exceeded

## Checks and validation timing

### `after_iteration`
These checks run after every iteration, including when the iteration ended because of:
- task completion
- loop completion claim
- context-limit stop
- ordinary incomplete stop

### Completion validators
These run only when:
- the loop-complete marker is present
- and `after_iteration` checks passed

### `before_final_success`
These run only when:
- the loop-complete marker is present
- `after_iteration` checks passed
- completion validators passed

## Max iterations

If `max_iterations` is configured:
- the controller increments iteration numbers starting at 1
- before beginning iteration `n`, it must fail if `n` would exceed `max_iterations`
- reaching the bound without success is a failure

If `max_iterations` is omitted:
- the loop is unbounded and continues until success or interruption

## Restart-with-summary

The next iteration receives:
- the original objective
- a compact structured summary of prior progress
- reason the prior iteration ended
- changed files or areas of work if known
- failed checks or validator summaries if applicable
- outstanding tasks
- recommended next step

The next iteration should not receive the full prior session transcript by default.

## Handoff summary requirements

Each handoff summary should include these sections:
- objective
- iteration outcome
- completed work
- outstanding work
- check and validator results
- context-pressure notes if applicable
- recommended next action

This summary is the primary continuity mechanism between iterations.
