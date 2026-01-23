---------------------------- MODULE RalphLoop ----------------------------
(***************************************************************************)
(* TLA+ Specification for the Ralph Loop Rust Implementation               *)
(*                                                                         *)
(* This spec models a concurrent application that runs Claude Code in a    *)
(* loop with real-time context monitoring. It spawns Claude as a           *)
(* subprocess and concurrently monitors output for token count and         *)
(* completion promises.                                                    *)
(*                                                                         *)
(* Key feature: MaxIterations is optional. When not set (HasMaxIterations  *)
(* = FALSE), the loop runs indefinitely until promise found or shutdown.   *)
(***************************************************************************)

EXTENDS Integers, Sequences, TLC

CONSTANTS
    HasMaxIterations,   \* Boolean: TRUE if max iterations is set, FALSE for infinite
    MaxIterations,      \* Maximum iterations (only used if HasMaxIterations = TRUE)
    ContextLimit,       \* Maximum token count before killing process
    ModelBound          \* Bound for model checking infinite loops (not part of impl)

VARIABLES
    \* Loop Controller State
    iteration,          \* Current iteration number (starts at 0)
    loopState,          \* State of the main loop: "running", "success", "failed", "shutdown"

    \* Claude Process State
    processState,       \* State of subprocess: "idle", "running", "exited", "killed"

    \* Shared State (protected by RwLock in implementation)
    tokenCount,         \* Current estimated token count
    promiseFound,       \* Boolean: has the completion promise been found?

    \* Monitor State
    monitorActive,      \* Boolean: are monitors currently running?

    \* Events/Commands (modeled as flags for simplicity)
    killRequested,      \* Kill command sent due to context limit
    shutdownSignal      \* External shutdown signal (Ctrl+C)

vars == <<iteration, loopState, processState, tokenCount, promiseFound,
          monitorActive, killRequested, shutdownSignal>>

-----------------------------------------------------------------------------
(***************************************************************************)
(* Helper Operators                                                        *)
(***************************************************************************)

(* Effective iteration limit for model checking *)
EffectiveLimit ==
    IF HasMaxIterations THEN MaxIterations ELSE ModelBound

(* Check if we've exceeded the max iterations (only applies when set) *)
MaxIterationsExceeded ==
    HasMaxIterations /\ iteration >= MaxIterations

(* In infinite mode, we only terminate on success or shutdown *)
InfiniteMode == ~HasMaxIterations

-----------------------------------------------------------------------------
(***************************************************************************)
(* Type Invariant                                                          *)
(***************************************************************************)

TypeOK ==
    /\ iteration \in 0..EffectiveLimit
    /\ loopState \in {"init", "running", "success", "failed", "shutdown"}
    /\ processState \in {"idle", "running", "exited", "killed"}
    /\ tokenCount \in 0..ContextLimit+100  \* Can exceed limit before detection (jumps up to 100)
    /\ promiseFound \in BOOLEAN
    /\ monitorActive \in BOOLEAN
    /\ killRequested \in BOOLEAN
    /\ shutdownSignal \in BOOLEAN

-----------------------------------------------------------------------------
(***************************************************************************)
(* Initial State                                                           *)
(***************************************************************************)

Init ==
    /\ iteration = 0
    /\ loopState = "init"
    /\ processState = "idle"
    /\ tokenCount = 0
    /\ promiseFound = FALSE
    /\ monitorActive = FALSE
    /\ killRequested = FALSE
    /\ shutdownSignal = FALSE

-----------------------------------------------------------------------------
(***************************************************************************)
(* Actions                                                                 *)
(***************************************************************************)

(***************************************************************************)
(* StartIteration: Begin a new iteration of the loop                       *)
(* - Resets state                                                          *)
(* - Spawns Claude process                                                 *)
(* - Activates monitors                                                    *)
(*                                                                         *)
(* In infinite mode (HasMaxIterations = FALSE), there's no iteration limit *)
(* In bounded mode, we check against MaxIterations                         *)
(***************************************************************************)
StartIteration ==
    /\ loopState \in {"init", "running"}
    /\ processState = "idle"
    /\ ~shutdownSignal
    /\ ~MaxIterationsExceeded  \* Only applies when HasMaxIterations = TRUE
    /\ iteration < EffectiveLimit  \* Model checking bound
    /\ iteration' = iteration + 1
    /\ loopState' = "running"
    /\ processState' = "running"
    /\ tokenCount' = 0          \* Reset for new iteration
    /\ promiseFound' = FALSE    \* Reset for new iteration
    /\ monitorActive' = TRUE
    /\ killRequested' = FALSE
    /\ UNCHANGED shutdownSignal

(***************************************************************************)
(* MonitorOutput: Monitors read output and update shared state             *)
(* This models the concurrent stdout/stderr monitoring                     *)
(***************************************************************************)
MonitorOutput ==
    /\ monitorActive
    /\ processState = "running"
    /\ ~shutdownSignal
    /\ \/ \* Receive more output, increment token count
          /\ tokenCount < ContextLimit
          /\ tokenCount' \in {tokenCount + 1, tokenCount + 10, tokenCount + 100}
          /\ UNCHANGED <<promiseFound, killRequested>>
       \/ \* Detect the completion promise in output
          /\ ~promiseFound
          /\ promiseFound' = TRUE
          /\ UNCHANGED <<tokenCount, killRequested>>
       \/ \* Context limit reached, request kill
          /\ tokenCount >= ContextLimit
          /\ killRequested' = TRUE
          /\ UNCHANGED <<tokenCount, promiseFound>>
    /\ UNCHANGED <<iteration, loopState, processState, monitorActive, shutdownSignal>>

(***************************************************************************)
(* ProcessExitsNaturally: Claude subprocess exits on its own               *)
(***************************************************************************)
ProcessExitsNaturally ==
    /\ processState = "running"
    /\ ~killRequested
    /\ ~shutdownSignal
    /\ processState' = "exited"
    /\ monitorActive' = FALSE
    /\ UNCHANGED <<iteration, loopState, tokenCount, promiseFound, killRequested, shutdownSignal>>

(***************************************************************************)
(* KillProcess: Process is killed due to context limit                     *)
(***************************************************************************)
KillProcess ==
    /\ processState = "running"
    /\ killRequested
    /\ processState' = "killed"
    /\ monitorActive' = FALSE
    /\ UNCHANGED <<iteration, loopState, tokenCount, promiseFound, killRequested, shutdownSignal>>

(***************************************************************************)
(* HandleIterationEnd: Process iteration result after process terminates   *)
(*                                                                         *)
(* Key behavior difference based on HasMaxIterations:                      *)
(* - When TRUE: Can transition to "failed" when max iterations reached     *)
(* - When FALSE (infinite): Never transitions to "failed", keeps looping   *)
(***************************************************************************)
HandleIterationEnd ==
    /\ loopState = "running"
    /\ processState \in {"exited", "killed"}
    /\ ~shutdownSignal
    /\ IF promiseFound
       THEN \* Success! Promise was found
            /\ loopState' = "success"
            /\ UNCHANGED <<iteration, tokenCount, promiseFound, killRequested>>
       ELSE IF HasMaxIterations /\ iteration >= MaxIterations
            THEN \* Max iterations reached without success (only in bounded mode)
                 /\ loopState' = "failed"
                 /\ UNCHANGED <<iteration, tokenCount, promiseFound, killRequested>>
            ELSE \* Continue looping (always in infinite mode, or if iterations remain)
                 /\ UNCHANGED <<iteration, loopState, tokenCount, promiseFound, killRequested>>
    /\ processState' = "idle"
    /\ monitorActive' = FALSE
    /\ UNCHANGED shutdownSignal

(***************************************************************************)
(* ShutdownRequested: External shutdown signal (e.g., Ctrl+C)              *)
(***************************************************************************)
ShutdownRequested ==
    /\ ~shutdownSignal
    /\ loopState \in {"init", "running"}
    /\ shutdownSignal' = TRUE
    /\ UNCHANGED <<iteration, loopState, processState, tokenCount, promiseFound,
                   monitorActive, killRequested>>

(***************************************************************************)
(* HandleShutdown: Process the shutdown signal                             *)
(***************************************************************************)
HandleShutdown ==
    /\ shutdownSignal
    /\ loopState \in {"init", "running"}
    /\ loopState' = "shutdown"
    /\ IF processState = "running"
       THEN processState' = "killed"
       ELSE processState' = processState
    /\ monitorActive' = FALSE
    /\ UNCHANGED <<iteration, tokenCount, promiseFound, killRequested, shutdownSignal>>

-----------------------------------------------------------------------------
(***************************************************************************)
(* Next State Relation                                                     *)
(***************************************************************************)

Next ==
    \/ StartIteration
    \/ MonitorOutput
    \/ ProcessExitsNaturally
    \/ KillProcess
    \/ HandleIterationEnd
    \/ ShutdownRequested
    \/ HandleShutdown

-----------------------------------------------------------------------------
(***************************************************************************)
(* Fairness Conditions                                                     *)
(*                                                                         *)
(* We require weak fairness on certain actions to ensure progress:         *)
(* - If we can start an iteration, we eventually will                      *)
(* - If a process can exit or be killed, it eventually will                *)
(* - If iteration ends, we eventually handle it                            *)
(***************************************************************************)

Fairness ==
    /\ WF_vars(StartIteration)
    /\ WF_vars(ProcessExitsNaturally)
    /\ WF_vars(KillProcess)
    /\ WF_vars(HandleIterationEnd)
    /\ WF_vars(HandleShutdown)

Spec == Init /\ [][Next]_vars /\ Fairness

-----------------------------------------------------------------------------
(***************************************************************************)
(* Safety Properties                                                       *)
(***************************************************************************)

(* In bounded mode, iteration never exceeds the maximum *)
IterationBoundRespected ==
    HasMaxIterations => iteration <= MaxIterations

(* A process can only be running if monitors are active *)
ProcessImpliesMonitors ==
    processState = "running" => monitorActive

(* Cannot be in success state if promise was never found *)
SuccessImpliesPromise ==
    loopState = "success" => promiseFound

(* Failed state only possible in bounded mode *)
FailedOnlyInBoundedMode ==
    loopState = "failed" => HasMaxIterations

(* Only one terminal state *)
TerminalStateExclusive ==
    ~(loopState = "success" /\ loopState = "failed")

(* In infinite mode, we never reach "failed" state *)
InfiniteModeNeverFails ==
    InfiniteMode => loopState /= "failed"

-----------------------------------------------------------------------------
(***************************************************************************)
(* Liveness Properties                                                     *)
(***************************************************************************)

(* The loop eventually terminates (different conditions based on mode) *)
(* In bounded mode: terminates with success, failed, or shutdown *)
(* In infinite mode: only terminates with success or shutdown *)
EventualTermination ==
    IF HasMaxIterations
    THEN <>(loopState \in {"success", "failed", "shutdown"})
    ELSE <>(loopState \in {"success", "shutdown"})

(* If shutdown is requested, we eventually handle it *)
ShutdownEventuallyHandled ==
    shutdownSignal ~> (loopState = "shutdown")

(* If promise is found, we eventually succeed *)
PromiseLeadsToSuccess ==
    promiseFound ~> (loopState = "success" \/ loopState = "shutdown")

(* If context limit is exceeded, process is eventually killed *)
ContextLimitLeadsToKill ==
    (killRequested /\ processState = "running") ~> (processState = "killed")

-----------------------------------------------------------------------------
(***************************************************************************)
(* Useful State Predicates for Model Checking                              *)
(***************************************************************************)

(* The loop has terminated *)
Terminated ==
    IF HasMaxIterations
    THEN loopState \in {"success", "failed", "shutdown"}
    ELSE loopState \in {"success", "shutdown"}

(* The loop succeeded *)
Succeeded == loopState = "success"

(* The loop failed after max iterations (only possible in bounded mode) *)
MaxIterationsReached == loopState = "failed"

(* Running in infinite mode *)
RunningInfinitely == InfiniteMode /\ loopState = "running"

=============================================================================
\* Modification History
\* Removed unused WarningThreshold constant (logging is a side effect, not state)
\* Updated to support optional MaxIterations (infinite loop mode)
\* Created for Ralph Loop Rust Implementation
