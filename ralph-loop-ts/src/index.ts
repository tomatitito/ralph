#!/usr/bin/env bun

import { runCli } from "./cli.ts";

const output = runCli(Bun.argv.slice(2));
console.log(output);
