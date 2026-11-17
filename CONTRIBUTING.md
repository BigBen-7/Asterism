# Contributing

Thanks for helping. Asterism is public from day one; start with issues labelled `good first issue`.

## Ground rules

- Only issues in the **current milestone** are open for work. `future` issues are planning only.
- One issue per PR, linked with `Closes #N`. Keep PRs small and tested.
- Testnet only. Never commit secrets or keys.

## Workflow

- Branches: `feat/<name>`, `fix/<name>`, `docs/<name>`, `chore/<name>`.
- Commits: [Conventional Commits](https://www.conventionalcommits.org/), e.g. `feat(registry): add issue_batch`. Scopes: `registry`, `credentials`, `funding`, `governance`, `indexer`, `api`, `sdk`, `ui`, `ci`, `docs`.
- Before opening a PR run `just lint` and `just test`.

## Code standards

- Rust: stable toolchain pinned in `rust-toolchain.toml`; `cargo fmt --check` and `cargo clippy -- -D warnings` must pass; no `unwrap()`/`expect()` in non-test service code.
- TypeScript: strict mode, ESLint, Prettier, pnpm.
- Contracts: explicit error enums, minimal storage, no unbounded loops; success and failure tests for every public function.
