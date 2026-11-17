"""Bucketing self-test for tools/verify.py, with rivet and bazel stubbed.

Run: python3 -m unittest discover -s tools -p 'test_verify.py'
"""
from __future__ import annotations

import json
import pathlib
import sys
import tempfile
import unittest
from unittest import mock

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import verify  # noqa: E402

T = "//tests/surface:surface_tests"


def comp_req(aid, tests, status="valid"):
    return {"id": aid, "type": "comp-req", "status": status, "tags": [],
            "links": [], "fields": {"verified-by": [f"{T}:{t}" for t in tests]}}


def waiver(target, status="valid", tag="known-gap"):
    return {"id": "DR-GAP", "type": "decision-record", "status": status,
            "tags": [tag], "fields": {},
            "links": [{"type": "satisfies", "target": target}]}


class Bucketing(unittest.TestCase):
    def run_gate(self, artifacts, present, passing, discovery_error=False):
        def list_tests(target):
            if discovery_error:
                raise RuntimeError("bazel build failed")
            return set(present)

        with tempfile.TemporaryDirectory() as d:
            report = pathlib.Path(d) / "out.json"
            with mock.patch.object(verify, "load_artifacts", return_value=artifacts), \
                 mock.patch.object(verify, "list_tests", side_effect=list_tests), \
                 mock.patch.object(verify, "run_one_test",
                                   side_effect=lambda tgt, name: name in passing), \
                 mock.patch("sys.stdout"):
                code = verify.main(["--variants", d, "--report", str(report)])
            return code, json.loads(report.read_text())["counts"]

    def test_all_pass(self):
        code, c = self.run_gate([comp_req("A", ["t1"])], ["t1"], {"t1"})
        self.assertEqual((code, c["passed"]), (0, 1))

    def test_unwaived_failure_fails(self):
        code, c = self.run_gate([comp_req("A", ["t1"])], ["t1"], set())
        self.assertEqual((code, c["failed"]), (1, 1))

    def test_waived_failure_is_known_gap(self):
        code, c = self.run_gate([comp_req("A", ["t1"]), waiver("A")], ["t1"], set())
        self.assertEqual((code, c["known_gap"], c["failed"]), (0, 1, 0))

    def test_waived_pass_is_stale(self):
        code, c = self.run_gate([comp_req("A", ["t1"]), waiver("A")], ["t1"], {"t1"})
        self.assertEqual((code, c["stale_gap"]), (1, 1))

    def test_waiver_does_not_cover_discovery_error(self):
        code, c = self.run_gate([comp_req("A", ["t1"]), waiver("A")], ["t1"], set(),
                                discovery_error=True)
        self.assertEqual((code, c["failed"], c["known_gap"]), (1, 1, 0))

    def test_waiver_does_not_cover_missing(self):
        code, c = self.run_gate([comp_req("A", ["t1"]), waiver("A")], [], set())
        self.assertEqual((code, c["missing"]), (1, 1))

    def test_draft_waiver_ignored(self):
        code, c = self.run_gate([comp_req("A", ["t1"]), waiver("A", status="draft")],
                                ["t1"], set())
        self.assertEqual((code, c["failed"]), (1, 1))

    def test_untagged_record_ignored(self):
        code, c = self.run_gate([comp_req("A", ["t1"]), waiver("A", tag="other")],
                                ["t1"], set())
        self.assertEqual((code, c["failed"]), (1, 1))

    def test_non_valid_comp_req_out_of_scope(self):
        code, c = self.run_gate([comp_req("A", ["t1"], status="draft")], ["t1"], set())
        self.assertEqual((code, sum(c.values())), (0, 0))


if __name__ == "__main__":
    unittest.main()
