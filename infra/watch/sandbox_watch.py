#!/usr/bin/env python3
"""Report contributors who trip abuse signals on `sandbox` issues and PRs.

Signals are prompts for human review, not proof. Output goes to stdout and to
infra/reports/ (git-ignored). Needs the `gh` CLI; set GH_TOKEN to pin the account.
"""
import argparse
import json
import re
import subprocess
import sys
from collections import defaultdict
from datetime import datetime, timezone
from pathlib import Path

ALLOWED = re.compile(r"^sandbox/lab/(src/tasks/\d{3}-[a-z0-9-]+\.ts|test/\d{3}-[a-z0-9-]+\.test\.ts)$")
PROTECTED = re.compile(r"^(\.github/|infra/|sandbox/lab/(package\.json|pnpm-lock\.yaml|pnpm-workspace\.yaml|tsconfig\.json))|(^|/)(package\.json|pnpm-lock\.yaml|Cargo\.toml|Cargo\.lock)$")
CLOSES = re.compile(r"(?:close[sd]?|fix(?:e[sd])?|resolve[sd]?)\s+#(\d+)", re.I)
CLAIM_LIMIT, HOARD_LIMIT, FLOOD_LIMIT, NEW_DAYS = 5, 3, 3, 14


def gh(path):
    out = subprocess.run(["gh", "api", "--paginate", path], capture_output=True, text=True)
    if out.returncode != 0:
        sys.exit(f"gh api {path} failed: {out.stderr.strip()}")
    text = out.stdout.strip()
    if not text:
        return []
    # --paginate emits one JSON array per page; join them
    text = re.sub(r"\]\s*\[", ",", text)
    return json.loads(text)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--repo", required=True)
    ap.add_argument("--ignore", action="append", default=[], help="maintainer logins to skip")
    ap.add_argument("--min-signals", type=int, default=1)
    a = ap.parse_args()
    ignore = {x.lower() for x in a.ignore} | {"tomikeds"}
    repo = a.repo

    issues = [i for i in gh(f"repos/{repo}/issues?labels=sandbox&state=all&per_page=100") if "pull_request" not in i]
    sb = {i["number"]: i for i in issues}
    comments = gh(f"repos/{repo}/issues/comments?per_page=100")
    pulls = gh(f"repos/{repo}/pulls?state=all&per_page=100")

    signals = defaultdict(lambda: defaultdict(list))  # login -> code -> evidence

    claims = defaultdict(set)
    for c in comments:
        n = int(c["issue_url"].rsplit("/", 1)[1])
        if n in sb:
            claims[c["user"]["login"]].add(n)
    for u, ns in claims.items():
        if len(ns) >= CLAIM_LIMIT:
            signals[u]["MASS_CLAIM"].append("issues " + ", ".join(f"#{n}" for n in sorted(ns)))

    assigned = defaultdict(list)
    for n, i in sb.items():
        if i["state"] == "open":
            for asg in i.get("assignees") or []:
                assigned[asg["login"]].append(n)
    for u, ns in assigned.items():
        if len(ns) >= HOARD_LIMIT:
            signals[u]["HOARDING"].append("assigned " + ", ".join(f"#{n}" for n in sorted(ns)))

    open_prs = defaultdict(list)
    for p in pulls:
        files = [f["filename"] for f in gh(f"repos/{repo}/pulls/{p['number']}/files?per_page=100")]
        linked = {int(n) for n in CLOSES.findall((p.get("body") or "") + " " + p["title"])} & set(sb)
        touches_sb = any(f.startswith("sandbox/") for f in files)
        if not (linked or touches_sb):
            continue
        u = p["user"]["login"]
        tag = f"PR #{p['number']}"
        bad = [f for f in files if not ALLOWED.match(f)]
        if bad:
            signals[u]["OUT_OF_SCOPE_PR"].append(f"{tag}: " + ", ".join(bad[:5]))
        prot = [f for f in files if PROTECTED.search(f)]
        if prot:
            signals[u]["PROTECTED_PATH"].append(f"{tag}: " + ", ".join(prot[:5]))
        for n in linked:
            who = [x["login"] for x in sb[n].get("assignees") or []]
            if u not in who:
                signals[u]["PR_NO_ASSIGNMENT"].append(f"{tag} closes #{n} (assigned: {', '.join(who) or 'nobody'})")
        if p["state"] == "open":
            open_prs[u].append(p["number"])
    for u, ns in open_prs.items():
        if len(ns) >= FLOOD_LIMIT:
            signals[u]["PR_FLOOD"].append("open " + ", ".join(f"PR #{n}" for n in sorted(ns)))

    now = datetime.now(timezone.utc)
    for u in list(signals):
        info = subprocess.run(["gh", "api", f"users/{u}"], capture_output=True, text=True)
        if info.returncode == 0:
            created = datetime.fromisoformat(json.loads(info.stdout)["created_at"].replace("Z", "+00:00"))
            if (now - created).days < NEW_DAYS:
                signals[u]["NEW_ACCOUNT"].append(f"created {created.date()}")
    rows = []
    for u, codes in signals.items():
        if u.lower() in ignore or len(codes) < a.min_signals:
            continue
        rows.append((u, codes))
    rows.sort(key=lambda r: -len(r[1]))

    lines = [f"# Sandbox watch report ({now:%Y-%m-%d %H:%M} UTC)", "", "Signals are prompts for review, not proof.", ""]
    if not rows:
        lines.append("No contributors tripped any signal.")
    for u, codes in rows:
        lines.append(f"## {u} ({len(codes)} signal type(s))")
        for code, ev in codes.items():
            lines.append(f"- **{code}**: " + "; ".join(ev))
        lines.append("")
    report = "\n".join(lines)
    print(report)

    out_dir = Path(__file__).resolve().parents[1] / "reports"
    out_dir.mkdir(exist_ok=True)
    path = out_dir / f"sandbox-watch-{now:%Y%m%d-%H%M%S}.md"
    path.write_text(report)
    print(f"\nWrote {path}", file=sys.stderr)


if __name__ == "__main__":
    main()
