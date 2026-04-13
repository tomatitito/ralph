export type IterationDecision =
  | "success"
  | "restart_task_boundary"
  | "restart_failed_completion"
  | "restart_context_limit"
  | "restart_incomplete"
  | "interrupted"
  | "max_iterations_exceeded"
  | "error";
