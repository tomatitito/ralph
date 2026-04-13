export interface ArtifactWriter {
  writeRunStart(): Promise<void>;
}
