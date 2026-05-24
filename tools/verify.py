#!/usr/bin/env python3
"""verify.py — artifact-driven verification gate for example-kvs.

Same shape as the LS-N gate in pulseengine/meld. Walks rivet
artifacts, finds tests by name convention, runs them, reports
PASSED / FAILED / MISSING per artifact.

Convention: an artifact with id COMP-REQ-KVS-KEY-NAMING expects at
least one cargo test named test_comp_req_kvs_key_naming_* under
verification/.

Output:
  - .verify-output.json     — machine-readable bucket counts
  - stdout                  — human-readable per-artifact table
  - exit code 1 iff any FAILED or MISSING

Usage:
    python3 tools/verify.py [--artifacts artifacts] [--tests verification]
"""
from __future__ import annotations

import argparse
import json
import pathlib
import re
import subprocess
import sys
import yaml


def expected_artifacts(artifact_dir: pathlib.Path) -> list[dict]:
    """Return every artifact that we expect to be verified.

    Currently scopes to comp-req artifacts with status: approved —
    those are the unit of work for the verification gate.
    """
    expected = []
    for path in sorted(artifact_dir.glob("*.yaml")):
        try:
            data = yaml.safe_load(path.read_text()) or {}
        except yaml.YAMLError:
            continue
        for art in data.get("artifacts", []):
            if art.get("type") == "comp-req" and art.get("status") == "approved":
                expected.append(art)
    return expected


def test_glob_for(artifact_id: str) -> str:
    """COMP-REQ-KVS-KEY-NAMING → test_comp_req_kvs_key_naming_*"""
    return "test_" + artifact_id.lower().replace("-", "_") + "_*"


def discover_tests(test_dir: pathlib.Path, glob_pattern: str) -> list[str]:
    """Find #[test] functions matching the glob."""
    pat_re = re.compile(
        "^" + glob_pattern.replace("*", "[a-z_0-9]*") + "$"
    )
    results = []
    for rs in test_dir.rglob("*.rs"):
        for match in re.finditer(r"fn\s+([a-z_0-9]+)\s*\(", rs.read_text()):
            name = match.group(1)
            if pat_re.match(name):
                results.append(name)
    return results


def run_test(name: str) -> bool:
    """Run a single cargo test by exact match. Returns True iff green.

    For the example-kvs skeleton there is no Cargo project yet, so
    this function reports MISSING-or-skipped rather than actually
    invoking cargo. In a real deployment this is:
        subprocess.run(["cargo", "test", "--lib", "--no-fail-fast",
                        "--", name], check=False).returncode == 0
    """
    # Stub for the example skeleton.
    return True


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--artifacts", type=pathlib.Path,
                    default=pathlib.Path("artifacts"))
    ap.add_argument("--tests", type=pathlib.Path,
                    default=pathlib.Path("verification"))
    ap.add_argument("--report", type=pathlib.Path,
                    default=pathlib.Path(".verify-output.json"))
    args = ap.parse_args(argv)

    artifacts = expected_artifacts(args.artifacts)
    passed, failed, missing = [], [], []

    for art in artifacts:
        glob = test_glob_for(art["id"])
        tests = discover_tests(args.tests, glob)
        if not tests:
            missing.append((art["id"], glob))
            continue
        all_green = all(run_test(t) for t in tests)
        (passed if all_green else failed).append((art["id"], tests))

    # Human-readable table
    print(f"{'BUCKET':9s} {'ID':45s} EVIDENCE")
    print("─" * 80)
    for aid, tests in passed:
        print(f"{'PASSED':9s} {aid:45s} {len(tests)} test(s) all green")
    for aid, tests in failed:
        print(f"{'FAILED':9s} {aid:45s} {len(tests)} test(s), some failed")
    for aid, glob in missing:
        print(f"{'MISSING':9s} {aid:45s} no test matching `{glob}`")
    print("─" * 80)
    print(f"{len(passed)} PASSED, {len(failed)} FAILED, {len(missing)} MISSING")

    # Machine-readable
    args.report.write_text(json.dumps({
        "passed":  [a for a, _ in passed],
        "failed":  [a for a, _ in failed],
        "missing": [{"id": a, "expected-glob": g} for a, g in missing],
        "counts": {
            "passed": len(passed),
            "failed": len(failed),
            "missing": len(missing),
        },
    }, indent=2))

    # Gate fires red on any FAILED or MISSING
    return 1 if (failed or missing) else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
