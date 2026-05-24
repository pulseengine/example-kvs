#!/usr/bin/env python3
"""verify.py — artifact-driven verification gate for example-kvs.

Same shape as the LS-N gate in pulseengine/meld. Walks every comp-req
artifact whose status is `approved`, reads its `verified-by:` field
(a list of strings of the form `<bazel-target>:<test-fn-name>`), runs
each test via `bazel test --test_arg=--exact --test_arg=<name>`, and
buckets the artifact:

  PASSED  — verified-by listed N tests; all N PASSED in bazel.
  FAILED  — verified-by listed N tests; at least one FAILED in bazel.
  MISSING — verified-by is absent / empty, OR one of the listed tests
            was not present in the bazel target's test binary.

Exit code: 0 iff no FAILED and no MISSING.

This is intentionally an honest gate. If upstream code does not honor
a requirement, the surface test fires red and the gate goes red. That
is the demonstration vs. eclipse-score's coverage-wedge approach.

Usage:
    python3 tools/verify.py [--artifacts artifacts]
                            [--report .verify-output.json]
                            [--no-run]   # discovery only, do not invoke bazel

The verified-by entry grammar:
    `<bazel-target>:<rust-test-name>`
where:
    - bazel-target starts with `//` (e.g. `//tests/surface:surface_tests`)
    - rust-test-name is the exact name reported by `bazel-bin/...
      <target> --list` (e.g. `test_comp_req_kvs_key_naming_alphabet_valid_keys_accepted`,
      or with module prefix `json_backend::json_backend_tests::test_load_ok`).
"""
from __future__ import annotations

import argparse
import json
import pathlib
import re
import shutil
import subprocess
import sys
import yaml


# ── verified-by parsing ──────────────────────────────────────────────

# A bazel label is `//<pkg-path>:<target-name>`. Exactly one ':' between
# the path and target. Everything after the next ':' is the test name
# (which itself may contain '::' module separators).
_TARGET_RE = re.compile(r"^(//[^:]+:[^:]+):(.+)$")


def parse_verified_by(entry: str) -> tuple[str, str]:
    """Split '//pkg:tgt:fn::name' into ('//pkg:tgt', 'fn::name')."""
    m = _TARGET_RE.match(entry)
    if not m:
        raise ValueError(
            f"verified-by entry must match '//<target>:<test>': {entry!r}"
        )
    return m.group(1), m.group(2)


# ── artifact discovery ───────────────────────────────────────────────


def expected_artifacts(artifact_dir: pathlib.Path) -> list[dict]:
    """Approved comp-reqs with their (possibly empty) verified-by list."""
    out = []
    for path in sorted(artifact_dir.glob("*.yaml")):
        try:
            data = yaml.safe_load(path.read_text()) or {}
        except yaml.YAMLError:
            continue
        for art in data.get("artifacts", []):
            if art.get("type") != "comp-req":
                continue
            if art.get("status") != "approved":
                continue
            verified = art.get("fields", {}).get("verified-by", []) or []
            out.append({"id": art["id"], "verified-by": list(verified)})
    return out


# ── test execution ───────────────────────────────────────────────────


def bazel_cmd() -> list[str]:
    if shutil.which("bazelisk"):
        return ["bazelisk"]
    if shutil.which("bazel"):
        return ["bazel"]
    raise RuntimeError(
        "neither bazelisk nor bazel found in PATH; "
        "install one (https://bazel.build/install) to run the verify gate"
    )


def list_tests(target: str, _cache: dict[str, set[str]] = {}) -> set[str]:
    """Return the set of test names present in the binary for `target`."""
    if target in _cache:
        return _cache[target]
    bz = bazel_cmd()
    # Build first so bazel-bin/<path> exists.
    subprocess.run(bz + ["build", target], check=True, capture_output=True)
    # Resolve bazel-bin path.
    proc = subprocess.run(
        bz + ["cquery", target, "--output=files"],
        check=True, capture_output=True, text=True,
    )
    binary = pathlib.Path(proc.stdout.strip().splitlines()[0])
    if not binary.is_file():
        raise RuntimeError(f"cquery returned non-file: {binary}")
    listed = subprocess.run(
        [str(binary), "--list"], check=True, capture_output=True, text=True,
    ).stdout
    # Each line is `<name>: test` or `<name>: bench`. test names contain `::`
    # for the module path, so rsplit on the trailing `: test` marker.
    names = {
        line.rsplit(": test", 1)[0]
        for line in listed.splitlines()
        if line.endswith(": test")
    }
    _cache[target] = names
    return names


def run_one_test(target: str, test_name: str) -> bool:
    """Invoke `bazel test --test_arg=--exact --test_arg=<test_name>`."""
    bz = bazel_cmd()
    proc = subprocess.run(
        bz + [
            "test", target,
            "--test_arg=--exact", "--test_arg=" + test_name,
            "--test_output=errors",
        ],
        capture_output=True, text=True,
    )
    return proc.returncode == 0


# ── main ─────────────────────────────────────────────────────────────


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--artifacts", type=pathlib.Path,
                    default=pathlib.Path("artifacts"))
    ap.add_argument("--report", type=pathlib.Path,
                    default=pathlib.Path(".verify-output.json"))
    ap.add_argument("--no-run", action="store_true",
                    help="Discover only; check verified-by presence, do not invoke bazel")
    args = ap.parse_args(argv)

    artifacts = expected_artifacts(args.artifacts)
    passed: list[tuple[str, list[str]]] = []
    failed: list[tuple[str, list[str]]] = []
    missing: list[tuple[str, str]] = []

    for art in artifacts:
        aid = art["id"]
        entries = art["verified-by"]
        if not entries:
            missing.append((aid, "verified-by is absent or empty"))
            continue

        # Parse + check presence + run.
        gone: list[str] = []
        failed_here: list[str] = []
        passed_here: list[str] = []

        for entry in entries:
            try:
                target, test_name = parse_verified_by(entry)
            except ValueError as e:
                failed_here.append(f"{entry}: malformed ({e})")
                continue

            # Presence check.
            try:
                if test_name not in list_tests(target):
                    gone.append(f"{target}::{test_name}")
                    continue
            except Exception as e:
                failed_here.append(f"{entry}: discovery failed: {e}")
                continue

            if args.no_run:
                passed_here.append(entry)
                continue

            if run_one_test(target, test_name):
                passed_here.append(entry)
            else:
                failed_here.append(entry)

        if gone:
            missing.append((aid,
                f"{len(gone)} listed test(s) not present in target: " + ", ".join(gone)))
        elif failed_here:
            failed.append((aid, failed_here))
        else:
            passed.append((aid, passed_here))

    # ── human-readable table ────────────────────────────────────────
    print(f"{'BUCKET':9s} {'ID':40s} EVIDENCE")
    print("─" * 100)
    for aid, entries in passed:
        print(f"{'PASSED':9s} {aid:40s} {len(entries)} test(s) all green")
    for aid, entries in failed:
        print(f"{'FAILED':9s} {aid:40s} {len(entries)} test(s) failed:")
        for e in entries:
            print(f"{'':9s} {'':40s}   - {e}")
    for aid, reason in missing:
        print(f"{'MISSING':9s} {aid:40s} {reason}")
    print("─" * 100)
    print(f"{len(passed)} PASSED, {len(failed)} FAILED, {len(missing)} MISSING")

    # ── machine-readable report ─────────────────────────────────────
    args.report.write_text(json.dumps({
        "passed":  [{"id": a, "tests": e} for a, e in passed],
        "failed":  [{"id": a, "tests": e} for a, e in failed],
        "missing": [{"id": a, "reason": r} for a, r in missing],
        "counts": {
            "passed": len(passed),
            "failed": len(failed),
            "missing": len(missing),
        },
    }, indent=2))

    return 1 if (failed or missing) else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
