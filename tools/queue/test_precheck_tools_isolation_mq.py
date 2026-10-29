"""Preserve an uncommitted candidate during the complete real mq suite."""

import sys
sys.dont_write_bytecode = True

import importlib.util
from pathlib import Path
import unittest

SPEC = importlib.util.spec_from_file_location(
    "precheck_isolation_fixture", Path(__file__).with_name("precheck_isolation_fixture.py"))
support = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(support)


def load_tests(loader, tests, pattern):
    return unittest.TestSuite([support.ToolsIsolationTests(
        "test_complete_mq_suite_preserves_staged_candidate_and_unstaged_ratchet")])


if __name__ == "__main__":
    unittest.main()
