import { expect, test } from "bun:test";

test("dependency-cruiser passes for the initial scaffold", () => {
  const result = Bun.spawnSync({
    cmd: ["bun", "run", "depcruise"],
    cwd: import.meta.dir + "/../..",
    stdout: "pipe",
    stderr: "pipe",
  });

  expect(result.exitCode).toBe(0);
});
