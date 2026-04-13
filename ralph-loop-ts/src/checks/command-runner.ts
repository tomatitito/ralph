export interface CommandRunner {
  run(command: string): Promise<void>;
}
