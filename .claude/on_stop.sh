#!/bin/bash

# Ralph Loop Stop Hook
# Checks if the completion promise was fulfilled in the transcript (received via stdin).
# Signals to the outer loop whether to continue or stop.

PROMISE_FILE=".ralph-promise"
CONTINUE_FILE=".ralph-continue"

# Read the expected promise text, default to "TASK COMPLETE"
if [[ -f "$PROMISE_FILE" ]]; then
    PROMISE=$(cat "$PROMISE_FILE")
else
    PROMISE="TASK COMPLETE"
fi

# Check if the promise tag exists in the transcript (stdin)
if grep -q "<promise>${PROMISE}</promise>"; then
    # Promise fulfilled - delete continue file if it exists
    rm -f "$CONTINUE_FILE"
else
    # Promise not fulfilled - create continue file
    touch "$CONTINUE_FILE"
fi
