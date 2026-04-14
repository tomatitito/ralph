import type { ChecksConfig, CommandConfig } from "./config-types.ts";

function assertObject(value: unknown, message: string): Record<string, unknown> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new Error(message);
  }

  return value as Record<string, unknown>;
}

function optionalString(value: unknown, fieldName: string): string | null {
  if (value === undefined) {
    return null;
  }

  if (typeof value !== "string") {
    throw new Error(`${fieldName} must be a string`);
  }

  return value;
}

function optionalInteger(value: unknown, fieldName: string): number | null {
  if (value === undefined) {
    return null;
  }

  if (typeof value !== "number" || !Number.isInteger(value)) {
    throw new Error(`${fieldName} must be an integer`);
  }

  return value;
}

function parseEnv(value: unknown, fieldName: string): Record<string, string> {
  if (value === undefined) {
    return {};
  }

  const objectValue = assertObject(value, `${fieldName} must be a table`);
  const entries = Object.entries(objectValue);
  for (const [key, entryValue] of entries) {
    if (typeof entryValue !== "string") {
      throw new Error(`${fieldName}.${key} must be a string`);
    }
  }

  return Object.fromEntries(entries as Array<[string, string]>);
}

function parseCommandConfig(value: unknown, fieldName: string): CommandConfig {
  const table = assertObject(value, `${fieldName} entries must be tables`);
  const name = optionalString(table.name, `${fieldName}.name`);
  const command = optionalString(table.command, `${fieldName}.command`);

  if (name === null || name.trim() === "") {
    throw new Error(`${fieldName}.name is required`);
  }

  if (command === null || command.trim() === "") {
    throw new Error(`${fieldName}.command is required`);
  }

  return {
    name,
    command,
    cwd: optionalString(table.cwd, `${fieldName}.cwd`),
    timeoutSeconds: optionalInteger(table.timeout_seconds, `${fieldName}.timeout_seconds`),
    requiredExitCode: optionalInteger(table.required_exit_code, `${fieldName}.required_exit_code`) ?? 0,
    requiredStdout: optionalString(table.required_stdout, `${fieldName}.required_stdout`),
    requiredStderr: optionalString(table.required_stderr, `${fieldName}.required_stderr`),
    env: parseEnv(table.env, `${fieldName}.env`),
  };
}

function parseCommandArray(value: unknown, fieldName: string): CommandConfig[] {
  if (value === undefined) {
    return [];
  }

  if (!Array.isArray(value)) {
    throw new Error(`${fieldName} must be an array of tables`);
  }

  return value.map((entry, index) => parseCommandConfig(entry, `${fieldName}[${index}]`));
}

export function parseChecksConfigToml(tomlText: string): ChecksConfig {
  const parsed = Bun.TOML.parse(tomlText) as unknown;
  const root = assertObject(parsed, "checks config must be a TOML table");

  return {
    afterIteration: parseCommandArray(root.after_iteration, "after_iteration"),
  };
}
