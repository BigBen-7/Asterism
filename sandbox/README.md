# Sandbox

A practice area for first-time contributors. It is **separate from the Asterism project**: nothing here is used by the contracts, services or SDK, and it is not part of the Cargo or pnpm workspaces.

## How it works

`lab/` holds 100 small TypeScript tasks. Each task is one stub file and one test file:

- `lab/src/tasks/NNN-name.ts`: the function to implement (throws `Not implemented`)
- `lab/test/NNN-name.test.ts`: tests, skipped until you start (`describe.skip`)

Every task has a matching issue labelled `sandbox`. To take one:

1. Comment on the issue and wait to be assigned. One open task per person at a time.
2. Implement the stub and change `describe.skip` to `describe` in its test file.
3. Run it locally:

   ```bash
   cd sandbox/lab
   pnpm install
   pnpm test
   pnpm typecheck
   ```

4. Open a PR that changes **only** those two files, with `Closes #<issue>` in the description.

## Rules

- A PR may only touch `sandbox/lab/src/tasks/NNN-*.ts` and `sandbox/lab/test/NNN-*.test.ts`.
- Do not edit workflows, package files, lockfiles or anything outside `sandbox/`. The `sandbox-guard` check fails such PRs.
- Write the solution yourself. Copy-pasted or unexplained generated code will be closed.
- Be kind. See [CODE_OF_CONDUCT.md](../CODE_OF_CONDUCT.md).

## Monitoring notice

Activity in the sandbox (claims, assignments and pull requests) is reviewed by maintainers for abuse such as claim hoarding, out-of-scope changes and spam. See [`infra/`](../infra/README.md).
