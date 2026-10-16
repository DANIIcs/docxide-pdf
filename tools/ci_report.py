#!/usr/bin/env python3
"""Markdown summary of an engine comparison run, compared with the previous published run.

    python3 tools/ci_report.py <previous report.json> <manifest.json> <new report.json>

Prints the summary on stdout (CI appends it to $GITHUB_STEP_SUMMARY) and writes a slim
report.json, published with the site, for the next run to compare against. A missing or
unreadable previous report just leaves out the deltas.
"""
from __future__ import annotations

import json
import os
import sys
from pathlib import Path

MOVE = 1.0   # Jaccard points a case must move to count as up/down


def slim(manifest: dict) -> dict:
    cases = {f'{c["group"]}/{c["case"]}': {"scores": {e: {"jaccard": s.get("jaccard"), "ssim": s.get("ssim")}
                                                      for e, s in c.get("scores", {}).items()},
                                           "times": c.get("times", {}),
                                           "engines": [e for e, p in c.get("pages", {}).items() if p]}
             for c in manifest["cases"]}
    return {"versions": manifest["versions"], "cases": cases}


def mean(xs: list) -> float | None:
    return sum(xs) / len(xs) if xs else None


def fmt(x, delta=False) -> str:
    if x is None:
        return "–"
    return f"{x:+.2f}" if delta else f"{x:.1f}"


def jac(report: dict, case: str, engine: str):
    return report["cases"].get(case, {}).get("scores", {}).get(engine, {}).get("jaccard")


def version_cell(old: str | None, new: str) -> str:
    if old is None or old == new:
        return new
    repo = os.environ.get("GITHUB_REPOSITORY")
    if repo and "@" in old and "@" in new:   # our own build: link the commit range
        a, b = (v.split("@")[1].removesuffix("+dirty") for v in (old, new))
        return f"[{old} → {new}](https://github.com/{repo}/compare/{a}...{b})"
    return f"**{old} → {new}**"


def report(prev: dict, cur: dict) -> str:
    out = []
    cases, old_cases = set(cur["cases"]), set(prev.get("cases", {}))
    line = f"{len(cases)} cases"
    if old_cases:
        added, removed = sorted(cases - old_cases), sorted(old_cases - cases)
        line += f" ({len(added)} new, {len(removed)} removed since the last run)"
        line += "".join(f"\n- {label}: {', '.join(xs[:20])}{' …' if len(xs) > 20 else ''}"
                        for label, xs in (("New", added), ("Removed", removed)) if xs)
    else:
        line += " (no previous report to compare with)"
    out.append(line)

    old_v = prev.get("versions", {})
    changed = [e for e in cur["versions"] if e in old_v and old_v[e] != cur["versions"][e] and e != "generated"]
    if changed:
        out.append("\n**Engine versions changed:** " + ", ".join(f"{e} {old_v[e]} → {cur['versions'][e]}" for e in changed))

    common = cases & old_cases
    out.append("\n| Engine | Version | Converted | Mean Jaccard | Δ (same cases) | ↑ / ↓ cases | Mean SSIM | Mean s/doc |")
    out.append("|---|---|---|---|---|---|---|---|")
    for e, v in cur["versions"].items():
        if e == "reference":
            out.append(f"| {e} | {v} | | | | | | |")
            continue
        conv = sum(e in c["engines"] for c in cur["cases"].values())
        j = mean([x for c in cases if (x := jac(cur, c, e)) is not None])
        ssim = mean([s for c in cur["cases"].values() if (s := c["scores"].get(e, {}).get("ssim")) is not None])
        t = mean([s for c in cur["cases"].values() if (s := c["times"].get(e)) is not None])
        pairs = [(a, b) for c in common if (a := jac(prev, c, e)) is not None and (b := jac(cur, c, e)) is not None]
        d = mean([b for _, b in pairs]) - mean([a for a, _ in pairs]) if pairs else None
        up = sum(b - a >= MOVE for a, b in pairs)
        down = sum(a - b >= MOVE for a, b in pairs)
        old_conv = sum(e in c.get("engines", []) for c in prev.get("cases", {}).values())
        conv_cell = f"{conv}/{len(cases)}" + (f" (was {old_conv}/{len(old_cases)})" if old_cases and old_conv != conv else "")
        moves = f"{up} / {down}" if pairs else "–"
        out.append(f"| {e} | {version_cell(old_v.get(e), v)} | {conv_cell} | {fmt(j)} | {fmt(d, True)} | {moves} "
                   f"| {fmt(ssim)} | {fmt(t)} |")

    movers = sorted(((b - a, c, a, b) for c in common
                     if (a := jac(prev, c, "generated")) is not None and (b := jac(cur, c, "generated")) is not None
                     and abs(b - a) >= MOVE), key=lambda m: -abs(m[0]))
    if movers:
        out.append(f"\n<details><summary>docxide-pdf: {len(movers)} cases moved ≥ {MOVE} Jaccard</summary>\n")
        out.append("| Case | Before | After | Δ |\n|---|---|---|---|")
        out += [f"| {c} | {a:.1f} | {b:.1f} | {d:+.1f} |" for d, c, a, b in movers[:30]]
        out.append("\n</details>")
    return "\n".join(out)


def main() -> None:
    prev_path, manifest_path, out_path = map(Path, sys.argv[1:4])
    try:
        prev = json.loads(prev_path.read_text())
    except (OSError, ValueError):
        prev = {}
    cur = slim(json.loads(manifest_path.read_text()))
    out_path.write_text(json.dumps(cur, separators=(",", ":")))
    print("## Engine comparison\n")
    print(report(prev, cur))


if __name__ == "__main__":
    main()
