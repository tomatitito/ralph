#!/bin/bash

# Ralph Loop PreCompact Hook
# Signals the outer loop to continue when compaction is triggered.

CONTINUE_FILE=".ralph-continue"

# Create continue file to signal the loop to continue
touch "$CONTINUE_FILE"
