import { describe, expect, test } from "bun:test";

import { PLACEHOLDER_MESSAGE, runCli } from "../cli.ts";

describe("runCli", () => {
  test("returns the deterministic scaffold placeholder message", () => {
    expect(runCli([])).toBe(PLACEHOLDER_MESSAGE);
  });
});
