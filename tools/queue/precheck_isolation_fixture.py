"""Real mq suites preserve disposable precheck candidates and their caller's Git state."""

import sys
sys.dont_write_bytecode = True

from contextlib import redirect_stdout
import importlib.machinery
import importlib.util
import io
import json
from pathlib import Path
import subprocess
import unittest

DIRECTORY = Path(__file__).resolve().parent
ROOT = DIRECTORY.parent.parent
LOADER = importlib.machinery.SourceFileLoader("tools_isolation_fixtures", str(DIRECTORY / "test_precheck.py"))
SPEC = importlib.util.spec_from_loader(LOADER.name, LOADER)
fixtures = importlib.util.module_from_spec(SPEC)
LOADER.exec_module(fixtures)


class ToolsIsolationTests(fixtures.Fixture):
    def setUp(self):
        super().setUp()
        # Copy only the suite and its inputs: discovery must not recurse into this suite.
        for name in ("test_mq.py", "mq", "restart", "authority.py", "durable.py",
                     "worker.py", "train.py", "recovery.py", "publication.py", "coordinators.txt",
                     "always.txt", "read-by-tests.txt", "leads.txt"):
            self.write("tools/queue/" + name, (DIRECTORY / name).read_text())
        for prefix in ("", "skein/", "smith/", "jig/", "temper/"):
            self.write(prefix + ".gitignore", (ROOT / prefix / ".gitignore").read_text())
        self.write("jig/scripts/old-forms.txt", "10\n")
        self.write("jig/scripts/old_forms.py", """from pathlib import Path
import sys
assert sys.argv[1:] == ['--write']
Path('scripts/old-forms.txt').write_text('9\\n')
""")
        self.commit()
        self.ancestor = self.git("rev-parse", "HEAD")
        self.git("branch", "-f", "base", self.ancestor)
        self.git("branch", "-f", "tip", self.ancestor)

    def snapshot(self):
        tracked = self.git("ls-files", "-z").split("\0")
        untracked = self.git("ls-files", "--others", "--exclude-standard", "-z").split("\0")
        ignored = self.git("ls-files", "--others", "--ignored", "--exclude-standard", "-z").split("\0")
        return {"head": (self.repo / ".git/HEAD").read_bytes(),
                "oid": self.git("rev-parse", "HEAD"), "refs": self.git("show-ref"),
                "index": (self.repo / ".git/index").read_bytes(),
                "untracked": untracked, "ignored": ignored,
                "merge": {name: (self.repo / ".git" / name).read_bytes()
                          for name in ("MERGE_HEAD", "MERGE_MSG", "MERGE_MODE")
                          if (self.repo / ".git" / name).exists()},
                "source": {path: ((self.repo / path).read_bytes(),
                                  (self.repo / path).stat().st_mode)
                           for path in sorted(set(tracked + untracked + ignored)) if path}}

    def candidate(self):
        self.change("tip", "jig/README.md", "candidate docs\n")
        self.git("switch", "--detach", "base")
        self.git("merge", "--no-ff", "--no-commit", "tip")
        self.write("jig/scripts/old-forms.txt", "9\n")
        self.assertEqual(self.git("diff", "--cached", "--name-only"), "jig/README.md")
        self.assertEqual(self.git("diff", "--name-only"), "jig/scripts/old-forms.txt")

    def assert_unchanged(self, before):
        after = self.snapshot()
        for name in before:
            with self.subTest(state=name):
                self.assertEqual(after[name], before[name])

    def run_suite(self, *tests):
        return subprocess.run([sys.executable, "-I", "-B", "tools/queue/test_mq.py", "-v", *tests],
                              cwd=self.repo, env=self.env, stdin=subprocess.DEVNULL,
                              stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True,
                              timeout=fixtures.mq.TOOLS_TIMEOUT)

    def test_touched_tree_case_preserves_uncommitted_merge_and_lower_ratchet(self):
        self.candidate()
        before = self.snapshot()
        result = self.run_suite("CheckTests.test_tools_and_docs_keep_touched_tree_ratchets_without_cargo")
        self.assertEqual(result.returncode, 0, result.stdout)
        self.assertIn("Ran 1 test", result.stdout)
        self.assert_unchanged(before)

    def test_complete_mq_suite_preserves_staged_candidate_and_unstaged_ratchet(self):
        self.candidate()
        before = self.snapshot()
        result = self.run_suite()
        self.assertEqual(result.returncode, 0, result.stdout)
        self.assertIn("Ran 120 tests", result.stdout)
        self.assert_unchanged(before)

    def test_precheck_runs_real_mq_suite_after_ratchet_and_restores_exactly(self):
        self.change("tip", "jig/README.md", "candidate docs\n")
        self.git("switch", "tip")
        self.write("tools/candidate.md", "select tools suites\n")
        self.commit()
        self.git("switch", "original")
        before = self.snapshot()
        observations = []

        def runner(argv, **options):
            self.assertEqual(options["stdin"], subprocess.DEVNULL)
            self.assertEqual(argv[:3], ["python3", "-I", "-B"])
            # Only the real fixture ratchet and the copied mq suite can execute.
            self.assertIn(argv[3], (str(self.repo / "jig/scripts/old_forms.py"),
                                    "tools/queue/test_mq.py"))
            if argv[3] == "tools/queue/test_mq.py":
                self.assertTrue((self.repo / ".git/MERGE_HEAD").exists())
                self.assertIn("tools/candidate.md", self.git("diff", "--cached", "--name-only"))
                self.assertEqual(self.git("diff", "--name-only"), "jig/scripts/old-forms.txt")
                observations.append(self.snapshot())
            result = subprocess.run(argv, env=self.env, **options)
            if argv[3] == "tools/queue/test_mq.py":
                self.assert_unchanged(observations[-1])
            return result

        tool = fixtures.precheck.Precheck(self.repo, "base", "tip", mq=fixtures.mq,
                                          always=[], read_by_tests=fixtures.PATTERNS,
                                          runner=runner, busy=lambda _: "fixture observation",
                                          metadata=lambda _: self.fail("docs/tools need no metadata"))
        with redirect_stdout(io.StringIO()):
            result = tool.run()
        log = (tool.log_directory / "commands.log").read_text()
        self.assertEqual(result, 0, str(tool.record) + "\n" + log)
        self.assertEqual(len(observations), 1)
        self.assertIn("Ran 120 tests", log)
        self.assertEqual(tool.record["selection"], "docs")
        self.assertEqual(tool.record["selection_paths"], ["jig/README.md", "tools/candidate.md"])
        self.assertEqual(tool.record["paths"],
                         ["jig/README.md", "jig/scripts/old-forms.txt", "tools/candidate.md"])
        self.assertEqual([step["name"] for step in tool.record["steps"]],
                         ["ratchet:jig", "tools/queue/test_mq.py"])
        self.assertEqual(tool.record["outcome"], "passed")
        self.assertEqual(tool.record["min_free_bytes"], 80_000_000_000)
        self.assertEqual(tool.record["admissions"], [])
        self.assertTrue(tool.record["restored"])
        self.assertFalse(tool.marker.exists())
        self.assertEqual(json.loads((tool.log_directory / "result.json").read_text()), tool.record)
        self.assert_unchanged(before)
