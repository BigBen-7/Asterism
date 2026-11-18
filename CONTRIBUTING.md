# Contributing

Thanks for helping. Asterism is public from day one; start with issues labelled `good first issue`.

## Ground rules

- All issues are open for contributors, grouped by the milestone of their phase. Check that the issues an item depends on are done before you start it.
- One issue per PR, linked with `Closes #N`. Keep PRs small and tested.
- Testnet only. Never commit secrets or keys.

## Workflow

- Branches: `feat/<name>`, `fix/<name>`, `docs/<name>`, `chore/<name>`.
- Commits: [Conventional Commits](https://www.conventionalcommits.org/), e.g. `feat(registry): add issue_batch`. Scopes: `registry`, `credentials`, `funding`, `governance`, `indexer`, `api`, `sdk`, `ui`, `ci`, `docs`.
- Before opening a PR run `just lint` and `just test`.

## Issues and labels

Every issue follows the same structure: **Context**, **Task**, **Acceptance criteria**, **Notes / hints** and **Out of scope**. The issue templates in `.github/ISSUE_TEMPLATE/` use it, so please keep to it when opening one. Pull requests use the template in `.github/PULL_REQUEST_TEMPLATE.md`; fill in the summary and tick the checklist honestly.

Labels come in groups. An issue normally has one label from each of type, area, phase and size.

| Group     | Labels                                                                                                                                                                           | Meaning                                                                    |
| --------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------- |
| Type      | `type: feature`, `type: bug`, `type: docs`, `type: chore`, `type: design`, `type: security`                                                                                      | What kind of work it is. `type: design` is a design spike or RFC.          |
| Area      | `area: registry`, `area: credentials`, `area: reputation`, `area: funding`, `area: governance`, `area: indexer`, `area: api`, `area: sdk`, `area: web`, `area: ci`, `area: docs` | Which part of the codebase it touches. Matches the commit scopes.          |
| Phase     | `phase: 0` to `phase: 4`                                                                                                                                                         | Which roadmap phase it belongs to (see [ROADMAP.md](ROADMAP.md)).          |
| Status    | `blocked`, `needs-verification`, `future`                                                                                                                                        | See below.                                                                 |
| Community | `good first issue`, `help wanted`                                                                                                                                                | `good first issue`: a newcomer can finish it from the issue text alone.    |
| Size      | `size: S`, `size: M`, `size: L`                                                                                                                                                  | S is under a day, M is 1-3 days, L is 3+ days and should usually be split. |

Status labels:

- `blocked`: cannot proceed until something else is done. The issue says what.
- `needs-verification`: relies on a claim about Stellar or Soroban that has not been confirmed against current official documentation (see CLAUDE.md section 6). Confirm the claim first and record the answer in the issue or PR before building on it.
- `future`: the issue is outside the current milestone and is planning only. This label was used before all issues were opened for work and is no longer applied to new issues, but you may still see it on older ones. Ask in the issue before starting work on a `future` issue.

Some issues also carry program labels from events the project takes part in (for example `Stellar Wave`). They do not change how you contribute.

## Code standards

- Rust: stable toolchain pinned in `rust-toolchain.toml`; `cargo fmt --check` and `cargo clippy -- -D warnings` must pass; no `unwrap()`/`expect()` in non-test service code.
- TypeScript: strict mode, ESLint, Prettier, pnpm.
- Contracts: explicit error enums, minimal storage, no unbounded loops; success and failure tests for every public function.
