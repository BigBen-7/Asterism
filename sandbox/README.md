# Sandbox

A practice area kept **separate from the Asterism project**: nothing here is used by the contracts, services or SDK, and it is not part of the Cargo or pnpm workspaces.

`lab/` is the (currently empty) TypeScript workspace for sandbox tasks. Tasks are added together with matching issues labelled `sandbox`.

## Rules

- A sandbox PR may only touch task and test files (see `.github/workflows/sandbox-guard.yml`). Do not edit workflows, package files, lockfiles or anything outside `sandbox/`.
- Comment on an issue and wait to be assigned. One open task per person at a time.
- Write the solution yourself. Copy-pasted or unexplained generated code will be closed.
- Be kind. See [CODE_OF_CONDUCT.md](../CODE_OF_CONDUCT.md).

## Monitoring notice

Activity in the sandbox (claims, assignments and pull requests) is reviewed by maintainers for abuse such as claim hoarding, out-of-scope changes and spam. See [`infra/`](../infra/README.md).
