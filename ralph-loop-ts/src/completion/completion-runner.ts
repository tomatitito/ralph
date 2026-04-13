export interface CompletionRunner {
  runOnLoopCompleteClaim(claimed: boolean): Promise<void>;
}
