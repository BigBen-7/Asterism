# Infra

Maintainer tooling that lives away from the main project. Nothing here ships with Asterism.

## `watch/sandbox_watch.py`

Reads GitHub activity on issues labelled `sandbox` and writes a report of contributors who tripped one or more signals. Signals are prompts for a human to look, **not proof of bad behaviour**. Review the linked activity before acting on anyone.

Signals:

| Code | Meaning |
|---|---|
| `MASS_CLAIM` | Commented on 5 or more distinct sandbox issues |
| `HOARDING` | Assigned to 3 or more open sandbox issues at once |
| `OUT_OF_SCOPE_PR` | PR touches files outside the allowed task and test files |
| `PROTECTED_PATH` | PR touches workflows, package files, lockfiles or `infra/` |
| `NEW_ACCOUNT` | Account younger than 14 days with a claim or PR |
| `PR_NO_ASSIGNMENT` | PR closes a sandbox issue assigned to someone else, or to no one |
| `PR_FLOOD` | 3 or more open sandbox PRs from one account |

### Run it

Use the maintainer account's token explicitly so a changed active `gh` account cannot interfere:

```bash
export GH_TOKEN=$(gh auth token --user TomikeDS)
python3 infra/watch/sandbox_watch.py --repo Chain-Explain/Asterism
```

Options: `--ignore LOGIN` (repeatable, maintainers are ignored), `--min-signals N` (default 1).

Reports are written to `infra/reports/` and are git-ignored. Never commit them: they contain people's handles and activity.
