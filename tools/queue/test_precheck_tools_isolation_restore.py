"""Run the complete real mq suite through precheck and verify restoration."""

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
        "test_precheck_runs_real_mq_suite_after_ratchet_and_restores_exactly")])


if __name__ == "__main__":
    unittest.main()
