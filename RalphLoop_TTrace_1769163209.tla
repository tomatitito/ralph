---- MODULE RalphLoop_TTrace_1769163209 ----
EXTENDS Sequences, RalphLoop, TLCExt, Toolbox, Naturals, TLC

_expression ==
    LET RalphLoop_TEExpression == INSTANCE RalphLoop_TEExpression
    IN RalphLoop_TEExpression!expression
----

_trace ==
    LET RalphLoop_TETrace == INSTANCE RalphLoop_TETrace
    IN RalphLoop_TETrace!trace
----

_inv ==
    ~(
        TLCGet("level") = Len(_TETrace)
        /\
        shutdownSignal = (TRUE)
        /\
        monitorActive = (FALSE)
        /\
        processState = ("idle")
        /\
        killRequested = (FALSE)
        /\
        loopState = ("shutdown")
        /\
        tokenCount = (0)
        /\
        iteration = (0)
        /\
        promiseFound = (FALSE)
    )
----

_init ==
    /\ killRequested = _TETrace[1].killRequested
    /\ shutdownSignal = _TETrace[1].shutdownSignal
    /\ processState = _TETrace[1].processState
    /\ loopState = _TETrace[1].loopState
    /\ monitorActive = _TETrace[1].monitorActive
    /\ promiseFound = _TETrace[1].promiseFound
    /\ tokenCount = _TETrace[1].tokenCount
    /\ iteration = _TETrace[1].iteration
----

_next ==
    /\ \E i,j \in DOMAIN _TETrace:
        /\ \/ /\ j = i + 1
              /\ i = TLCGet("level")
        /\ killRequested  = _TETrace[i].killRequested
        /\ killRequested' = _TETrace[j].killRequested
        /\ shutdownSignal  = _TETrace[i].shutdownSignal
        /\ shutdownSignal' = _TETrace[j].shutdownSignal
        /\ processState  = _TETrace[i].processState
        /\ processState' = _TETrace[j].processState
        /\ loopState  = _TETrace[i].loopState
        /\ loopState' = _TETrace[j].loopState
        /\ monitorActive  = _TETrace[i].monitorActive
        /\ monitorActive' = _TETrace[j].monitorActive
        /\ promiseFound  = _TETrace[i].promiseFound
        /\ promiseFound' = _TETrace[j].promiseFound
        /\ tokenCount  = _TETrace[i].tokenCount
        /\ tokenCount' = _TETrace[j].tokenCount
        /\ iteration  = _TETrace[i].iteration
        /\ iteration' = _TETrace[j].iteration

\* Uncomment the ASSUME below to write the states of the error trace
\* to the given file in Json format. Note that you can pass any tuple
\* to `JsonSerialize`. For example, a sub-sequence of _TETrace.
    \* ASSUME
    \*     LET J == INSTANCE Json
    \*         IN J!JsonSerialize("RalphLoop_TTrace_1769163209.json", _TETrace)

=============================================================================

 Note that you can extract this module `RalphLoop_TEExpression`
  to a dedicated file to reuse `expression` (the module in the 
  dedicated `RalphLoop_TEExpression.tla` file takes precedence 
  over the module `RalphLoop_TEExpression` below).

---- MODULE RalphLoop_TEExpression ----
EXTENDS Sequences, RalphLoop, TLCExt, Toolbox, Naturals, TLC

expression == 
    [
        \* To hide variables of the `RalphLoop` spec from the error trace,
        \* remove the variables below.  The trace will be written in the order
        \* of the fields of this record.
        killRequested |-> killRequested
        ,shutdownSignal |-> shutdownSignal
        ,processState |-> processState
        ,loopState |-> loopState
        ,monitorActive |-> monitorActive
        ,promiseFound |-> promiseFound
        ,tokenCount |-> tokenCount
        ,iteration |-> iteration
        
        \* Put additional constant-, state-, and action-level expressions here:
        \* ,_stateNumber |-> _TEPosition
        \* ,_killRequestedUnchanged |-> killRequested = killRequested'
        
        \* Format the `killRequested` variable as Json value.
        \* ,_killRequestedJson |->
        \*     LET J == INSTANCE Json
        \*     IN J!ToJson(killRequested)
        
        \* Lastly, you may build expressions over arbitrary sets of states by
        \* leveraging the _TETrace operator.  For example, this is how to
        \* count the number of times a spec variable changed up to the current
        \* state in the trace.
        \* ,_killRequestedModCount |->
        \*     LET F[s \in DOMAIN _TETrace] ==
        \*         IF s = 1 THEN 0
        \*         ELSE IF _TETrace[s].killRequested # _TETrace[s-1].killRequested
        \*             THEN 1 + F[s-1] ELSE F[s-1]
        \*     IN F[_TEPosition - 1]
    ]

=============================================================================



Parsing and semantic processing can take forever if the trace below is long.
 In this case, it is advised to uncomment the module below to deserialize the
 trace from a generated binary file.

\*
\*---- MODULE RalphLoop_TETrace ----
\*EXTENDS IOUtils, RalphLoop, TLC
\*
\*trace == IODeserialize("RalphLoop_TTrace_1769163209.bin", TRUE)
\*
\*=============================================================================
\*

---- MODULE RalphLoop_TETrace ----
EXTENDS RalphLoop, TLC

trace == 
    <<
    ([shutdownSignal |-> FALSE,monitorActive |-> FALSE,processState |-> "idle",killRequested |-> FALSE,loopState |-> "init",tokenCount |-> 0,iteration |-> 0,promiseFound |-> FALSE]),
    ([shutdownSignal |-> TRUE,monitorActive |-> FALSE,processState |-> "idle",killRequested |-> FALSE,loopState |-> "init",tokenCount |-> 0,iteration |-> 0,promiseFound |-> FALSE]),
    ([shutdownSignal |-> TRUE,monitorActive |-> FALSE,processState |-> "idle",killRequested |-> FALSE,loopState |-> "shutdown",tokenCount |-> 0,iteration |-> 0,promiseFound |-> FALSE])
    >>
----


=============================================================================

---- CONFIG RalphLoop_TTrace_1769163209 ----
CONSTANTS
    HasMaxIterations = TRUE
    MaxIterations = 3
    ContextLimit = 5
    ModelBound = 5

INVARIANT
    _inv

CHECK_DEADLOCK
    \* CHECK_DEADLOCK off because of PROPERTY or INVARIANT above.
    FALSE

INIT
    _init

NEXT
    _next

CONSTANT
    _TETrace <- _trace

ALIAS
    _expression
=============================================================================
\* Generated on Fri Jan 23 10:13:30 UTC 2026