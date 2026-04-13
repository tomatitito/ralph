export interface CommandExecutor {
  run(command: string): Promise<void>;
}
