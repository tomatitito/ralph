/**
 * Draft dependency-cruiser config for ralph-loop-ts.
 *
 * This is intended to be introduced with the initial scaffold and tightened as the
 * concrete source layout stabilizes.
 */

/** @type {import('dependency-cruiser').IConfiguration} */
module.exports = {
  options: {
    doNotFollow: {
      path: ["node_modules", "dist", "coverage"],
    },
    includeOnly: "^src",
    tsConfig: {
      fileName: "./tsconfig.json",
    },
    reporterOptions: {
      dot: {
        collapsePattern: "node_modules/[^/]+",
      },
    },
  },
  forbidden: [
    {
      name: "no-circular-dependencies",
      severity: "error",
      comment: "Circular dependencies make controller/runtime boundaries harder to reason about.",
      from: {},
      to: {
        circular: true,
      },
    },
    {
      name: "runtime-must-not-depend-on-controller",
      severity: "error",
      comment: "The controller may depend on the runtime, but never the other way around.",
      from: {
        path: "^src/runtime",
      },
      to: {
        path: "^src/controller",
      },
    },
    {
      name: "extensions-must-not-depend-on-controller",
      severity: "error",
      comment: "Extensions should expose normalized state only, not import controller logic.",
      from: {
        path: "^src/extensions",
      },
      to: {
        path: "^src/controller",
      },
    },
    {
      name: "runtime-imports-allowed-only-from-controller-and-tests",
      severity: "error",
      comment: "Only the controller layer and tests should import runtime modules directly.",
      from: {
        path: "^src/(?!controller|testing)",
      },
      to: {
        path: "^src/runtime",
      },
    },
    {
      name: "platform-imports-allowed-only-from-controller-and-tests",
      severity: "error",
      comment: "Concrete platform adapters should be wired by the controller and passed inward.",
      from: {
        path: "^src/(?!controller|testing)",
      },
      to: {
        path: "^src/platform",
      },
    },
    {
      name: "no-process-imports-outside-controller-and-tests",
      severity: "error",
      comment: "Direct process access should stay at the controller boundary.",
      from: {
        path: "^src/(?!controller|testing)",
      },
      to: {
        path: "^node:process$|^process$",
      },
    },
    {
      name: "no-console-imports-outside-controller-and-tests",
      severity: "warn",
      comment: "Prefer logger injection over direct console usage in leaf modules.",
      from: {
        path: "^src/(?!controller|testing)",
      },
      to: {
        path: "^node:console$|^console$",
      },
    },
    {
      name: "artifacts-must-not-depend-on-runtime",
      severity: "error",
      comment: "Artifact writing should consume normalized data, not runtime internals.",
      from: {
        path: "^src/artifacts",
      },
      to: {
        path: "^src/runtime",
      },
    },
    {
      name: "checks-must-not-depend-on-runtime",
      severity: "error",
      comment: "Checks should remain runtime-agnostic.",
      from: {
        path: "^src/checks",
      },
      to: {
        path: "^src/runtime",
      },
    },
    {
      name: "completion-must-not-depend-on-runtime",
      severity: "error",
      comment: "Completion validation should remain runtime-agnostic.",
      from: {
        path: "^src/completion",
      },
      to: {
        path: "^src/runtime",
      },
    },
  ],
};
