# Asterism: Project Brief and Setup Instructions for Claude Code

> Read this entire file before doing anything. It is the hand-off from a planning conversation held in the Claude chat app. Everything under "Decisions already made" is settled; do not re-open it. Everything under "Verify before relying on it" must be checked against current official documentation, because it was written from memory and may be out of date.

---

## 1. How to use this file

1. Read sections 2-7 for context.
2. Execute section 8 (repository setup) step by step. Confirm with the maintainer before any destructive or irreversible action (force-push, deleting branches, publishing packages, deploying contracts to any network).
3. Execute section 9 (labels and milestones) and section 10 (issue backlog) using the `gh` CLI. Create issues in batches, show the maintainer the list first, and then create them.
4. Afterwards, this file may be kept as `CLAUDE.md` at the repo root. Trim section 8 once setup is complete so that it does not go stale.

The maintainer is a backend developer based in Kaduna, Nigeria, who is comfortable with TypeScript backends and is building this project in Rust on Stellar. Explain Rust or Soroban concepts briefly when they are non-obvious, with examples.

---

## 2. Project summary

**Name:** Asterism (working name; availability on GitHub, crates.io and npm has NOT been verified, see section 12).

**What it is:** An open-source monorepo providing a small public-goods stack on the **Stellar network (Soroban smart contracts, Rust)**. The stack is built around one shared foundation, an **attestation registry**, and four modules that produce or consume attestations:

| # | Module | Role |
|---|---|---|
| 1 | Credentials | Issues and verifies tamper-proof credentials (certificates, badges), with revocation. **Produces** attestations. |
| 2 | Reputation | Turns open-source contribution activity (GitHub) into non-transferable attestations. **Produces** attestations. |
| 3 | Quadratic funding | Round-based grant funding with quadratic matching and Sybil resistance. **Consumes** attestations. |
| 4 | Governance | Proposals, voting, timelock, and optional gasless off-chain voting. **Consumes** attestations. |

**Architectural rule:** each module depends only on the shared attestation layer, never on another module. Every module must be usable and releasable on its own.

**Level:** medium. Roughly 4-5 packages per module, one core smart-contract design problem per module.

---

## 3. Decisions already made (do not re-open)

| Topic | Decision |
|---|---|
| Fields | All four modules above, delivered in phases |
| Blockchain | Stellar, Soroban smart contracts in Rust |
| Registry | Build our own attestation registry (do not depend on an external attestation standard) |
| Licence | MIT |
| Scope discipline | Strict. Build one phase at a time. Nothing beyond the current phase starts until the previous phase has been released (see section 7.6) |
| Network | Stellar **testnet** only until explicitly decided otherwise |
| Backend language | Rust (Axum, SQLx, Postgres) |
| Client/frontend | TypeScript SDK; Next.js frontends; Stellar wallet integration (Freighter is the common choice) |
| Monorepo tooling | Cargo workspace for Rust + pnpm workspace for TypeScript, with `just` as the single command runner |
| Contribution model | Public from day one; `good first issue` items prepared before any announcement |

---

## 4. Architecture

### 4.1 The attestation layer

An attestation is a signed on-chain statement about a subject address, made by an issuer, under a registered schema.

Proposed model (refine during the Phase 0 design issue; this is a starting point, not a final spec):

- **Schema**: `{ id, name, version, fields_hash, revocable: bool, creator }`
- **Attestation**: `{ id, schema_id, issuer, subject, data_hash | merkle_root, issued_at, expires_at?, revoked_at? }`
- **Operations**: `register_schema`, `issue`, `issue_batch` (Merkle root), `revoke`, `get`, `verify`, `extend_ttl`
- **Authorization**: use Soroban's native `require_auth` for issuer actions.

### 4.2 Data flow

```
Issuers/Apps --> contracts (Soroban) --> events --> indexer (Rust) --> Postgres --> API (Rust) --> SDK (TS) --> Web apps
                                                                          ^
                                          GitHub App (webhooks) ----------+   (Phase 2)
```

### 4.3 Module dependencies

```
credentials ---+
               +--> attestation-registry <--+-- funding   (reads attestations for Sybil resistance)
reputation ----+                            +-- governance (reads attestations for voting eligibility/weight)
```

---

## 5. Repository structure

```
asterism/
├── Cargo.toml                  # Rust workspace root
├── pnpm-workspace.yaml
├── package.json
├── justfile                    # single entry point for all commands
├── rust-toolchain.toml
├── LICENSE                     # MIT
├── README.md
├── CONTRIBUTING.md
├── CODE_OF_CONDUCT.md
├── SECURITY.md
├── ROADMAP.md
├── CLAUDE.md                   # this file (trimmed after setup)
├── .github/
│   ├── workflows/              # ci-rust.yml, ci-ts.yml
│   ├── ISSUE_TEMPLATE/         # bug.yml, feature.yml, good-first-issue.yml
│   └── PULL_REQUEST_TEMPLATE.md
├── contracts/
│   ├── attestation-registry/
│   ├── credentials/
│   ├── funding/
│   └── governance/
├── crates/
│   ├── asterism-types/         # shared schemas/types (used by contracts where no_std allows, and by services)
│   ├── matching-engine/        # pure Rust quadratic-funding math
│   └── asterism-client/        # Rust client for services
├── services/
│   ├── api/
│   ├── indexer/
│   ├── github-app/             # Phase 2
│   └── vote-relayer/           # Phase 4
├── packages/
│   ├── sdk/                    # TypeScript SDK
│   ├── ui/                     # shared React components
│   └── config/                 # shared tsconfig/eslint
└── apps/
    ├── web-credentials/
    ├── web-funding/
    ├── web-dao/
    └── docs/
```

Create only what the current phase needs. Empty placeholder directories for later phases are allowed only with a one-line `README.md` saying which phase they belong to.

---

## 6. Soroban and Stellar notes: verify before relying on them

These points were stated from memory by the planning assistant and may be wrong or outdated. **Check each against the current official Stellar/Soroban documentation before designing around it, and record the confirmed answer in the relevant issue.**

1. **Authorization.** Soroban uses its own authorization model (`require_auth` and authorization entries), not EIP-712. Off-chain signed voting in Phase 4 must be designed around this; custom account contracts may be relevant. Run a short design spike before any Phase 4 implementation.
2. **State archival and rent (TTL).** Contract state has a TTL that must be extended. The registry needs an explicit policy: who pays to keep records alive, and what happens when a record expires or is archived. This is a core Phase 0/1 design decision.
3. **Event retention.** The RPC is believed to retain events for only a limited window. The indexer must therefore persist all events from the first deployment. Confirm the current retention period.
4. **Resource limits.** Per-transaction limits on CPU instructions, memory and ledger entries make batching (Merkle roots) important. Confirm current limits and design benchmarks around them.
5. **CLI and SDK.** Confirm current commands for the Stellar CLI (project initialization, build, deploy, bindings generation) and the current `soroban-sdk` version before writing the setup scripts.
6. **Testnet access.** Confirm the current testnet RPC endpoint and friendbot/faucet process.
7. **`no_std`.** Contracts are `no_std`. Shared types in `asterism-types` must be designed so that the contract-facing part compiles under `no_std`.

---

## 7. Conventions

### 7.1 Git

- Default branch: `main`, protected (require PR, passing CI, one review once there is more than one maintainer).
- Branch names: `feat/<short-name>`, `fix/<short-name>`, `docs/<short-name>`, `chore/<short-name>`.
- Commits: Conventional Commits (`feat(registry): add issue_batch`). Scopes: `registry`, `credentials`, `funding`, `governance`, `indexer`, `api`, `sdk`, `ui`, `ci`, `docs`.
- Small PRs, each linked to an issue (`Closes #N`).

### 7.2 Rust

- Stable toolchain pinned in `rust-toolchain.toml`.
- `cargo fmt --check` and `cargo clippy --workspace --all-targets -- -D warnings` must pass.
- No `unwrap()`/`expect()` in non-test service code; use typed errors (`thiserror`).
- Contracts: explicit error enums, minimal storage, documented storage layout, no unbounded loops.

### 7.3 TypeScript

- Strict mode, ESLint, Prettier. Package manager: pnpm.

### 7.4 Testing

- Contracts: unit tests plus property/fuzz-style tests where practical. Every public contract function needs success and failure tests.
- Services: integration tests against a local Postgres (Docker Compose).
- `matching-engine`: property tests (for example, matching never exceeds the pool; results are order-independent).

### 7.5 Security

- Contracts that hold or move funds (funding, governance) are **unaudited**. Every README for those modules must say so prominently.
- Never commit secrets or keys. Provide `.env.example` only. Use testnet keys only.
- Maintain `SECURITY.md` with a private disclosure channel.

### 7.6 Scope discipline (strict)

- Only issues for the **current milestone** are open for work. Later-phase issues carry the `future` label and the milestone of their phase.
- A phase is "done" only when: its contracts are tested, deployed to testnet, documented, its SDK is released, and at least one person other than the maintainer has tried it and reported feedback.
- Phases 3 and 4 involve funds and security. Do not start them until Phases 1-2 have been used by someone besides the maintainer.
- If asked to add something outside the current phase, add it as a `future` issue instead of implementing it.

---

## 8. Repository setup tasks (Phase 0, part A)

Perform in order. After each numbered step, run the relevant check and report the result. **Verify tool versions and command syntax against current docs rather than trusting the commands below verbatim.**

1. **Prerequisites check.** Report installed versions of: `rustc`, `cargo`, the `wasm32` target for Soroban (confirm the exact target name in current docs), Stellar CLI, `node`, `pnpm`, `just`, `docker`, `gh`, `git`. List what is missing and ask before installing anything.
2. **Create the repository directory** named `asterism` (confirm the final name with the maintainer first) and run `git init -b main`.
3. **Root files:** `LICENSE` (MIT, with the maintainer's name and the current year), `README.md` (project summary, status badge placeholders, module table, "unaudited / testnet only" notice, quickstart placeholder), `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md` (Contributor Covenant), `SECURITY.md`, `ROADMAP.md` (from section 11), `.gitignore`, `.editorconfig`.
4. **Rust workspace:** root `Cargo.toml` with `[workspace]`, `resolver = "2"`, and shared `[workspace.package]` / `[workspace.dependencies]`. Add `rust-toolchain.toml`. Create `contracts/attestation-registry` (a Soroban contract; initialize with the Stellar CLI's current project-init command, or by hand if it differs) and `crates/asterism-types`.
5. **TypeScript workspace:** root `package.json`, `pnpm-workspace.yaml`, `packages/config` (shared tsconfig and ESLint), and an empty `packages/sdk` skeleton.
6. **`justfile`** with at least these recipes: `setup`, `build`, `test`, `lint`, `fmt`, `contracts-build`, `contracts-test`, `db-up`, `db-down`. Each recipe must work from the repo root.
7. **CI (GitHub Actions):**
   - `ci-rust.yml`: fmt check, clippy with `-D warnings`, `cargo test --workspace`, contract build. Cache cargo artifacts.
   - `ci-ts.yml`: install with pnpm, lint, typecheck, test.
   - Pin action versions; set minimal `permissions`.
8. **Docker Compose** for local Postgres (`docker-compose.yml`) and a `.env.example`.
9. **GitHub templates:** issue forms (bug, feature, good-first-issue) and the PR template (checklist: linked issue, tests, docs, `just lint` passes).
10. **First commit** on a branch `chore/bootstrap`, open a PR to `main` (do not push directly to `main`), and confirm CI is green.
11. **Report** a summary: files created, commands run, anything that deviated from this brief and why.

Do **not** create the GitHub repository, push, or set branch protection until the maintainer confirms the repo name and owner (user or organisation).

---

## 9. Labels and milestones

Create with `gh label create` / `gh api` after the remote repo exists.

**Labels**

| Group | Labels |
|---|---|
| Type | `type: feature`, `type: bug`, `type: docs`, `type: chore`, `type: design` (design spike / RFC), `type: security` |
| Area | `area: registry`, `area: credentials`, `area: reputation`, `area: funding`, `area: governance`, `area: indexer`, `area: api`, `area: sdk`, `area: web`, `area: ci`, `area: docs` |
| Phase | `phase: 0`, `phase: 1`, `phase: 2`, `phase: 3`, `phase: 4` |
| Status | `future` (outside current milestone), `blocked`, `needs-verification` (relies on a claim in section 6 that has not been confirmed) |
| Community | `good first issue`, `help wanted` |
| Size | `size: S` (under a day), `size: M` (1-3 days), `size: L` (3+ days; should usually be split) |

**Milestones** (one per phase; open only Phase 0 initially for work, others exist for planning):

- `v0.0 Foundation` (Phase 0)
- `v0.1 Credentials` (Phase 1)
- `v0.2 Reputation` (Phase 2)
- `v0.3 Funding` (Phase 3)
- `v0.4 Governance` (Phase 4)

Time estimates in the roadmap are rough guesses; do not put due dates on milestones until the maintainer decides on them.

---

## 10. Issue backlog

### 10.1 Issue format

Every issue must use this structure:

```
## Context
Why this exists; link to the relevant section of CLAUDE.md or ROADMAP.md.

## Task
What to do, concretely.

## Acceptance criteria
- [ ] Observable, testable outcomes

## Notes / hints
Pointers for contributors (files, docs). Mark anything unverified as `needs-verification`.

## Out of scope
What this issue must NOT include.
```

Apply: one `type:` label, one `area:` label, one `phase:` label, a size label, the milestone, and `good first issue` only if a newcomer could realistically complete it from the issue text alone.

### 10.2 Phase 0: Foundation (`v0.0`) - create all as open

| # | Title | Labels | Notes |
|---|---|---|---|
| 0.1 | Design RFC: attestation data model (schema, attestation, revocation, expiry) | `type: design`, `area: registry`, `size: M` | Deliverable: `docs/rfcs/0001-attestation-model.md`. Must decide on single vs. batch records and the on-chain/off-chain split. |
| 0.2 | Design RFC: state archival and TTL policy for the registry | `type: design`, `area: registry`, `needs-verification`, `size: M` | Depends on confirming section 6 item 2. |
| 0.3 | Implement `asterism-types` crate (no_std-compatible core types) | `type: feature`, `area: registry`, `size: M` | After 0.1. |
| 0.4 | Implement `attestation-registry`: `register_schema` | `type: feature`, `area: registry`, `size: M` | Tests required. |
| 0.5 | Implement `attestation-registry`: `issue` and `get` | `type: feature`, `area: registry`, `size: M` | |
| 0.6 | Implement `attestation-registry`: `revoke` with authorization checks | `type: feature`, `area: registry`, `size: M` | |
| 0.7 | Implement `attestation-registry`: `issue_batch` (Merkle root) and inclusion verification | `type: feature`, `area: registry`, `size: L` | Consider splitting. |
| 0.8 | Implement TTL extension functions and tests | `type: feature`, `area: registry`, `needs-verification`, `size: M` | After 0.2. |
| 0.9 | Contract benchmarks: resource usage per operation | `type: chore`, `area: registry`, `size: M` | Record against current network limits (section 6 item 4). |
| 0.10 | Testnet deployment script and docs | `type: chore`, `area: registry`, `needs-verification`, `size: M` | No mainnet. |
| 0.11 | Indexer: scaffold Rust service, Postgres schema, migrations | `type: feature`, `area: indexer`, `size: M` | |
| 0.12 | Indexer: ingest and persist registry events from RPC | `type: feature`, `area: indexer`, `needs-verification`, `size: L` | Must persist everything (section 6 item 3). |
| 0.13 | Indexer: backfill and resume-after-restart handling | `type: feature`, `area: indexer`, `size: M` | |
| 0.14 | Docs: architecture overview and local development guide | `type: docs`, `area: docs`, `good first issue`, `size: S` | |
| 0.15 | Docs: write the README module table and project status section | `type: docs`, `area: docs`, `good first issue`, `size: S` | |
| 0.16 | CI: add `cargo-deny` / dependency audit and Dependabot config | `type: chore`, `area: ci`, `good first issue`, `size: S` | |
| 0.17 | Add issue/PR template polish and labels documentation to CONTRIBUTING | `type: docs`, `area: docs`, `good first issue`, `size: S` | |

### 10.3 Phase 1: Credentials (`v0.1`): create with `future` label

| # | Title | Area | Size |
|---|---|---|---|
| 1.1 | RFC: credential schema format and optional W3C Verifiable Credentials mapping | credentials | M |
| 1.2 | `credentials` contract: issuer registration and authorization | credentials | M |
| 1.3 | `credentials` contract: issue, revoke, verify | credentials | L |
| 1.4 | `credentials` contract: batch issuance via Merkle root | credentials | L |
| 1.5 | API: issuance endpoints (single and batch) | api | L |
| 1.6 | API: verification endpoint and QR/link payload format | api | M |
| 1.7 | TypeScript SDK: `verify()` and `issue()` | sdk | L |
| 1.8 | Web: issuer dashboard (connect wallet, create schema, issue, revoke) | web | L |
| 1.9 | Web: public verification page | web | M |
| 1.10 | Docs: credentials quickstart and example integration | docs | S |
| 1.11 | Release checklist and v0.1 tag (including external-tester feedback) | docs | S |

### 10.4 Phase 2: Reputation (`v0.2`): `future`

| # | Title | Area | Size |
|---|---|---|---|
| 2.1 | RFC: scoring model, anti-gaming measures, privacy and consent | reputation | M |
| 2.2 | GitHub App: authentication, installation, webhook receiver | reputation | L |
| 2.3 | Contribution ingestion: merged PRs, reviews, releases | reputation | L |
| 2.4 | Scoring engine (pure Rust crate with property tests) | reputation | L |
| 2.5 | Account linking: prove GitHub account ownership for a Stellar address | reputation | L |
| 2.6 | Non-transferable reputation attestation issuance | reputation | M |
| 2.7 | Web: contributor profile page | web | M |
| 2.8 | Docs and v0.2 release | docs | S |

### 10.5 Phase 3: Quadratic funding (`v0.3`): `future`

| # | Title | Area | Size |
|---|---|---|---|
| 3.1 | RFC: round lifecycle, matching formula, Sybil-resistance policy | funding | M |
| 3.2 | `matching-engine` crate with property tests | funding | L |
| 3.3 | `funding` contract: rounds and pools | funding | L |
| 3.4 | `funding` contract: contributions and payouts | funding | L |
| 3.5 | Sybil resistance via registry attestations (reputation threshold) | funding | M |
| 3.6 | Indexer and API support for rounds | api | M |
| 3.7 | Web: round browsing, contribution flow, results | web | L |
| 3.8 | Security review checklist (self-review before any external audit) | funding | M |
| 3.9 | Docs and v0.3 release (with unaudited notice) | docs | S |

### 10.6 Phase 4: Governance (`v0.4`): `future`

| # | Title | Area | Size |
|---|---|---|---|
| 4.1 | Design spike: off-chain signed voting under Soroban's auth model | governance | M |
| 4.2 | RFC: proposal lifecycle, quorum, timelock, eligibility and weighting via attestations | governance | M |
| 4.3 | `governance` contract: proposals and on-chain voting | governance | L |
| 4.4 | `governance` contract: timelock and execution | governance | L |
| 4.5 | `vote-relayer` service (depends on 4.1) | governance | L |
| 4.6 | Web: DAO dashboard | web | L |
| 4.7 | Docs and v0.4 release (with unaudited notice) | docs | S |

### 10.7 Procedure for creating issues

1. Write all issues as Markdown files under a temporary directory (not committed), one per issue.
2. Show the maintainer the full title list for review, grouped by milestone.
3. After approval, create them with `gh issue create --title ... --body-file ... --label ... --milestone ...`.
4. Create Phase 0 issues first; add the `future` label to every issue in Phases 1-4.
5. Report the list of created issue numbers and URLs.
6. Optionally set up a GitHub Project board (Backlog / Ready / In progress / In review / Done) and add the issues. Ask first.

---

## 11. Roadmap (for `ROADMAP.md`)

Estimates are rough guesses and depend on the maintainer's availability.

| Phase | Milestone | Scope | Rough estimate |
|---|---|---|---|
| 0 | v0.0 Foundation | Tooling, CI, attestation registry, indexer | 1-2 weeks |
| 1 | v0.1 Credentials | Issue, revoke, verify; SDK; issuer and verifier apps | 3-4 weeks |
| 2 | v0.2 Reputation | GitHub App, scoring, profiles | about 3 weeks |
| 3 | v0.3 Funding | Rounds, quadratic matching, Sybil resistance | about 4 weeks |
| 4 | v0.4 Governance | Voting, timelock, optional off-chain voting | about 4 weeks |

---

## 12. Open questions (ask the maintainer; do not guess)

1. Final repository name and owner (personal account or organisation)? Check availability of the repo name, the crate prefix (`asterism-*`) and an npm scope (`@asterism/*`) first. Alternatives considered: `Tessera`, `Stellar Commons`.
2. Maintainer's name and email for the licence, `SECURITY.md` and the code of conduct contact?
3. Is a GitHub Project board wanted?
4. Preferred security-disclosure channel?
5. Any grant or ecosystem programme the project will be submitted to (this affects documentation priorities)?

---

## 13. Working agreement for Claude Code

- **Do not invent.** If a command, version, API or limit is not confirmed from current docs or from running it, say so and mark the item `needs-verification`.
- **Ask before irreversible actions:** pushing, publishing, deploying contracts, creating the GitHub repository, installing system packages, deleting files.
- **Stay in scope.** Implement only the current phase. Capture other ideas as `future` issues.
- **Keep changes small and tested.** One issue per PR, tests included, CI green.
- **Report honestly.** If something failed or was skipped, say so plainly in the summary.
- **Never** use mainnet keys or real funds. Testnet only.
