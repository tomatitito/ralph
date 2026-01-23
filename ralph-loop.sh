#!/bin/bash

# Ralph Loop - Fresh Instance Edition
# Runs Claude Code in a loop with fresh instances until completion promise is found
# or maximum iterations are reached.
#
# Works with the stop hook in .claude/on_stop.sh which signals via
# the .claude/.ralph-continue sentinel file.

set -euo pipefail

# Default values
MAX_ITERATIONS=10
COMPLETION_PROMISE="TASK COMPLETE"
PROMPT_FILE=""
PROMPT_TEXT=""
OUTPUT_DIR=".ralph-loop-output"

# State files
PROMISE_FILE=".claude/.ralph-promise"
CONTINUE_FILE=".claude/.ralph-continue"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

usage() {
    cat <<EOF
Usage: $(basename "$0") [OPTIONS] <PROMPT_FILE | -p "PROMPT_TEXT">

Run Claude Code in a Ralph loop with fresh instances each iteration.

Options:
    -p, --prompt TEXT           Prompt text (alternative to prompt file)
    -m, --max-iterations N      Maximum iterations (default: $MAX_ITERATIONS)
    -c, --completion-promise S  Promise text to detect completion (default: "$COMPLETION_PROMISE")
    -o, --output-dir DIR        Directory to store iteration outputs (default: $OUTPUT_DIR)
    -h, --help                  Show this help message

Examples:
    $(basename "$0") PROMPT.md
    $(basename "$0") -p "Fix all tests" -c "TESTS FIXED" -m 5
    $(basename "$0") --prompt "Refactor the auth module" --completion-promise "DONE" --max-iterations 20

The script will:
1. Start a fresh Claude Code instance with the prompt
2. The stop hook checks for <promise>COMPLETION_PROMISE</promise>
3. If found, the loop exits successfully
4. If not found, a new instance is started
5. Repeat until promise found or max iterations reached
EOF
    exit 0
}

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

cleanup() {
    log_info "Cleaning up state files..."
    rm -f "$PROMISE_FILE"
    rm -f "$CONTINUE_FILE"
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -p|--prompt)
            PROMPT_TEXT="$2"
            shift 2
            ;;
        -m|--max-iterations)
            MAX_ITERATIONS="$2"
            shift 2
            ;;
        -c|--completion-promise)
            COMPLETION_PROMISE="$2"
            shift 2
            ;;
        -o|--output-dir)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        -h|--help)
            usage
            ;;
        -*)
            log_error "Unknown option: $1"
            usage
            ;;
        *)
            PROMPT_FILE="$1"
            shift
            ;;
    esac
done

# Validate input
if [[ -z "$PROMPT_FILE" && -z "$PROMPT_TEXT" ]]; then
    log_error "Either a prompt file or prompt text (-p) is required"
    usage
fi

if [[ -n "$PROMPT_FILE" && ! -f "$PROMPT_FILE" ]]; then
    log_error "Prompt file not found: $PROMPT_FILE"
    exit 1
fi

# Get the prompt content
if [[ -n "$PROMPT_FILE" ]]; then
    PROMPT=$(cat "$PROMPT_FILE")
else
    PROMPT="$PROMPT_TEXT"
fi

# Create output directory and .claude directory
mkdir -p "$OUTPUT_DIR"
mkdir -p .claude

# Initialize state
# Write the expected promise for the stop hook to read
echo "$COMPLETION_PROMISE" > "$PROMISE_FILE"

# Remove any existing continue file (clean slate)
rm -f "$CONTINUE_FILE"

# Set up cleanup on exit
trap cleanup EXIT

log_info "Starting Ralph Loop"
log_info "Max iterations: $MAX_ITERATIONS"
log_info "Completion promise: $COMPLETION_PROMISE"
log_info "Output directory: $OUTPUT_DIR"
echo ""

# Main loop
iteration=1
while [[ $iteration -le $MAX_ITERATIONS ]]; do
    log_info "=== Iteration $iteration of $MAX_ITERATIONS ==="

    OUTPUT_FILE="${OUTPUT_DIR}/iteration-${iteration}.txt"

    # Run Claude Code with the prompt
    # The stop hook will check for the promise and signal via CONTINUE_FILE
    set +e
    echo "$PROMPT" | claude --dangerously-skip-permissions 2>&1 | tee "$OUTPUT_FILE"
    CLAUDE_EXIT_CODE=$?
    set -e

    log_info "Claude exited with code: $CLAUDE_EXIT_CODE"

    # Check if the stop hook signaled to continue
    if [[ ! -f "$CONTINUE_FILE" ]]; then
        # Continue file doesn't exist = promise was found
        log_success "Completion promise fulfilled in iteration $iteration!"
        log_success "Promise: <promise>$COMPLETION_PROMISE</promise>"
        echo ""
        log_info "Output saved to: $OUTPUT_FILE"
        log_info "All iteration outputs in: $OUTPUT_DIR"
        exit 0
    else
        log_warn "Completion promise not found in iteration $iteration"
        # Remove the continue file for the next iteration
        rm -f "$CONTINUE_FILE"
    fi

    # Increment iteration counter
    ((iteration++))

    if [[ $iteration -le $MAX_ITERATIONS ]]; then
        log_info "Starting fresh Claude instance..."
        echo ""
    fi
done

log_error "Maximum iterations ($MAX_ITERATIONS) reached without finding completion promise"
log_info "All iteration outputs saved in: $OUTPUT_DIR"
exit 1
