"""Exercise the job-visible log, artifact contents and exit status together."""
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

REPORTER = Path(__file__).with_name("gate_log.py").resolve()


class GateLogTest(unittest.TestCase):
    def replay(self, raw, code, junit=None):
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            if junit is not None:
                (work / "junit.xml").write_bytes(junit)
            failed_ids = work / "failed-tests.txt"
            failed_ids.write_bytes(b"gate.default\n")
            child = "import sys; sys.stderr.buffer.write(sys.stdin.buffer.read()); sys.exit(int(sys.argv[1]))"
            result = subprocess.run(
                [sys.executable, "-I", REPORTER, sys.executable, "-c", child, str(code)],
                input=raw, capture_output=True, cwd=work,
                env={**os.environ, "CANDIDATE": "a" * 40, "BASE": "b" * 40},
            )
            self.assertEqual(result.returncode, code, result.stderr)
            self.assertEqual((work / "gate-diagnostics.log").read_bytes(), raw)
            self.assertEqual(failed_ids.read_bytes(), b"gate.default\n")
            if junit is not None:
                self.assertEqual((work / "junit.xml").read_bytes(), junit)
            return result.stdout.decode(), (work / "gate-result.txt").read_text()

    def test_green_budget_warning(self):
        warning = b"gate: default: warning: layer over budget: step 9 CPU s, budget 7 s\n"
        live, footer = self.replay(b"gate: selection: trees jig; 3 of 12 packages\n" +
                                   b"gate: default: green, exit 0, 65 s\n" + warning * 60, 0)
        self.assertIn("aaaaaaaaaaaa", live)
        self.assertIn("3 of 12 packages", live)
        self.assertIn("1m 05s", live)
        self.assertNotIn("over budget", live)
        self.assertIn("WARN  1 distinct warning", footer)
        self.assertIn("9 CPU s, budget 7 s", footer)
        self.assertIn("PASS · 1 check", footer)

    def test_red_assertion_and_skipped_check(self):
        report = b"""<testsuites><testsuite><testcase classname="render" name="size">
            <failure message="test failed">panicked at src/view.rs:41
assertion failed
left: 9
right: 7</failure></testcase></testsuite></testsuites>"""
        live, footer = self.replay(b"gate: default: RED, exit 1, 6 s\ngate: fuzzy: skipped, after default's red\n",
                                   1, report)
        self.assertIn("SKIP  Fuzzy tests", live)
        self.assertIn("render.size", footer)
        self.assertIn("src/view.rs:41", footer)
        self.assertIn("left: 9 right: 7", footer)
        self.assertIn("FAIL ·", footer)

    def test_enforced_budget(self):
        report = b"""<testsuite><testcase classname="gate" name="default">
          <failure message="layer over budget: 16 CPU s, ceiling 12 s"/></testcase></testsuite>"""
        _, footer = self.replay(b"gate: default: RED, exit 1, 90 s\n", 1, report)
        self.assertIn("FAIL  gate.default", footer)
        self.assertIn("16 CPU s, ceiling 12 s", footer)

    def test_all_failed_checks_and_instruction_counts_reach_the_failed_step(self):
        ids = [f"case{i}" for i in range(7)]
        report = ("<testsuite>" + "".join(
            f'<testcase classname="suite" name="{name}"><failure message="failed"/></testcase>'
            for name in ids) + "</testsuite>").encode()
        raw = (b"gate: default: RED, layer instruction growth: a: 118 instructions, at main 90 instructions\n"
               b"  alpha::suite: 118 instructions, +28 instructions, at main 90 instructions\n"
               b"    slow_case: 0.04 wall s, new\n"
               b"    5 more tests\n"
               b"gate: default: warning: layer over budget at main: other\n"
               b"  other::suite: 500 instructions, new\n"
               b"gate: fuzzy: RED, layer over budget: b: 91 instructions, at main 50 instructions\n"
               b"  beta::suite: 91 instructions, new\n")
        _, footer = self.replay(raw, 17, report)
        for name in ids:
            self.assertIn(f"FAIL  suite.{name}", footer)
        self.assertIn("Failed IDs:\ngate.default\n", footer)
        self.assertIn("118 instructions, +28 instructions, at main 90 instructions", footer)
        self.assertIn("beta::suite: 91 instructions, new", footer)
        self.assertNotIn("other::suite", footer)
        self.assertNotIn("slow_case", footer)
        self.assertNotIn("5 more tests", footer)
        self.assertIn("timing context", footer)

    def test_timeout_and_missing_report(self):
        live, footer = self.replay(b"gate: default (head): 299 s, past its wall limit; full stalls io 6%\n"
                                   b"FAULT: worker terminated\n", 42, b"incomplete")
        self.assertIn("past its wall limit", footer)
        self.assertIn("FAULT: worker terminated", footer)

    def test_docs_only(self):
        live, footer = self.replay(b"no-code: 3 files, none a code input: citations only\ncitations: 0 broken in HEAD\n", 0)
        self.assertIn("citations: 0 broken", live)
        self.assertTrue(footer.startswith("PASS ·"))
        self.assertNotIn("checks", footer)


if __name__ == "__main__":
    unittest.main()
