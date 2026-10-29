"""Per-command admission against disposable Git and fake checkers, without heavy."""

from contextlib import redirect_stderr, redirect_stdout
import importlib.machinery
import importlib.util
import io
import json
from pathlib import Path
import subprocess
import sys
from types import SimpleNamespace
import unittest
from unittest.mock import patch

sys.dont_write_bytecode = True

# Import the fixture module, never its concrete test classes into discovery.
DIRECTORY = Path(__file__).resolve().parent
LOADER = importlib.machinery.SourceFileLoader("admission_fixtures", str(DIRECTORY / "test_precheck.py"))
SPEC = importlib.util.spec_from_loader(LOADER.name, LOADER)
fixtures = importlib.util.module_from_spec(SPEC)
LOADER.exec_module(fixtures)
precheck = fixtures.precheck
MINIMUM = 80_000_000_000


class AdmissionTests(fixtures.Fixture):
    def setUp(self):
        super().setUp()
        self.free_bytes = MINIMUM
        self.events = []
        self.space_effect = None

    def snapshot(self):
        self.assertEqual(self.git("--no-optional-locks", "status", "--porcelain"), "")
        paths = self.git("ls-files", "-z").split("\0")
        return {"head": (self.repo / ".git/HEAD").read_bytes(),
                "oid": self.git("rev-parse", "HEAD"), "refs": self.git("show-ref"),
                "index": (self.repo / ".git/index").read_bytes(),
                "source": {path: ((self.repo / path).read_bytes(),
                                  (self.repo / path).stat().st_mode)
                           for path in paths if path}}

    def run_tool(self, **admission):
        self.tool = precheck.Precheck(self.repo, "base", "tip", mq=fixtures.mq,
                                     always=["always"], read_by_tests=fixtures.PATTERNS,
                                     metadata=self.load_metadata, busy=lambda _: "fixture", **admission)
        real_popen = subprocess.Popen
        owner = self

        def launch(argv, **options):
            if argv[0] == "git":
                return real_popen(argv, **options)
            self.assertIn(argv[0], ("python3", "cargo", "heavy"))
            self.assertTrue(options["start_new_session"])
            active = json.loads(self.tool.marker.read_text())["active_command"]
            self.assertEqual((active["argv"], active["completion"]), (argv, "unknown"))
            self.events.append(("launch", list(argv)))

            class Child:
                def wait(self, timeout=None):
                    return owner.runner(argv, **options).returncode

            return Child()

        def usage(repo):
            self.assertEqual(repo, self.repo)
            self.events.append(("space", self.free_bytes))
            if self.space_effect:
                self.space_effect()
            return SimpleNamespace(free=self.free_bytes)

        with patch.object(precheck.shutil, "disk_usage", side_effect=usage), \
                patch.object(precheck.subprocess, "Popen", side_effect=launch), \
                redirect_stdout(io.StringIO()) as output:
            result = self.tool.run()
        self.printed = output.getvalue()
        return result

    def assert_restored(self, before):
        self.assertEqual(self.snapshot(), before)
        self.assertTrue(self.tool.record["restored"])
        self.assertFalse(self.tool.marker.exists())
        self.assertFalse(self.tool.executor.unknown)
        self.assertNotEqual(self.tool.record.get("active_command", {}).get("completion"), "unknown")
        self.assertFalse(self.tool.log_directory.is_relative_to(self.repo))
        result = json.loads((self.tool.log_directory / "result.json").read_text())
        self.assertEqual(result, self.tool.record)

    def test_drop_after_clippy_defers_build_and_restores_exact_inputs(self):
        self.change("base", "unrelated/src/lib.rs", "base change\n")
        self.change("tip", "skein/core/src/lib.rs", "candidate change\n")
        before = self.snapshot()

        def drop(argv):
            if argv[:3] == ["heavy", "cargo", "clippy"]:
                self.assertEqual((self.repo / "skein/core/src/lib.rs").read_text(), "candidate change\n")
                self.assertEqual((self.repo / "unrelated/src/lib.rs").read_text(), "base change\n")
                self.free_bytes = MINIMUM - 1

        self.effect = drop
        self.assertEqual(self.run_tool(), 1)
        heavy = [argv for argv, _ in self.calls if argv[0] == "heavy"]
        self.assertEqual([argv[2] for argv in heavy], ["clippy"])
        record = self.tool.record
        self.assertEqual(record["outcome"], "infrastructure-error")
        self.assertEqual(record["min_free_bytes"], MINIMUM)
        self.assertEqual([(item["name"], item["free_bytes"], item["admitted"])
                          for item in record["admissions"]],
                         [("clippy", MINIMUM, True), ("build", MINIMUM - 1, False)])
        self.assertEqual([step["name"] for step in record["steps"]], ["ratchet:skein", "fmt", "clippy"])
        self.assertTrue(all(step["green"] and step["returncode"] == 0 for step in record["steps"]))
        self.assertIn("build", [step["name"] for step in record["plan"]])
        self.assertEqual(record["active_command"]["argv"], heavy[0])
        self.assertEqual(record["active_command"]["completion"], "completed")
        self.assertEqual(record["active_command"]["returncode"], 0)
        for evidence in (self.printed, (self.tool.log_directory / "commands.log").read_text()):
            self.assertIn("[build] deferred", evidence)
            self.assertIn(str(MINIMUM - 1), evidence)
            self.assertIn(str(MINIMUM), evidence)
        self.assert_restored(before)

    def test_sufficient_space_runs_exact_commands_in_order_at_equality(self):
        self.change("tip", "skein/core/src/lib.rs", "candidate\n")
        before = self.snapshot()
        self.assertEqual(self.run_tool(), 0)
        scope = [arg for name in ("always", "jig-app", "skein-core", "smith-agent", "temper-app")
                 for arg in ("-p", name)]
        ratchet = ["python3", "-I", "-B", str(self.repo / "skein/scripts/old_forms.py"), "--write"]
        fmt = ["cargo", "fmt", "--check", "-p", "skein-core"]
        clippy = ["heavy", "cargo", "clippy", *scope, "--all-targets", "--", "-D", "warnings"]
        build = ["heavy", "cargo", "nextest", "run", *scope, "--no-run"]
        self.assertEqual([argv for argv, _ in self.calls], [ratchet, fmt, clippy, build])
        self.assertEqual(self.events, [("launch", ratchet), ("launch", fmt),
                                       ("space", MINIMUM), ("launch", clippy),
                                       ("space", MINIMUM), ("launch", build)])
        self.assertTrue(all(cwd == self.repo for argv, cwd in self.calls if argv != ratchet))
        self.assertEqual(self.tool.record["outcome"], "passed")
        self.assertTrue(all(item["admitted"] for item in self.tool.record["admissions"]))
        self.assert_restored(before)

    def test_first_heavy_refusal_keeps_fmt_without_unknown_ownership(self):
        self.change("tip", "Cargo.toml", "full candidate\n")
        before = self.snapshot()
        self.free_bytes = MINIMUM - 1
        self.assertEqual(self.run_tool(), 1)
        self.assertEqual([argv for argv, _ in self.calls], [["cargo", "fmt", "--all", "--check"]])
        self.assertEqual(self.tool.record["outcome"], "infrastructure-error")
        self.assertEqual(self.tool.record["admissions"][0]["name"], "clippy")
        self.assertEqual(self.tool.record["active_command"]["completion"], "completed")
        self.assert_restored(before)

    def test_explicit_policy_is_used_before_both_commands(self):
        self.change("tip", "unrelated/Cargo.toml", "candidate\n")
        before = self.snapshot()
        self.free_bytes = MINIMUM - 10
        self.assertEqual(self.run_tool(min_free_bytes=self.free_bytes), 0)
        self.assertEqual([argv[2] for argv, _ in self.calls], ["clippy", "nextest"])
        self.assertEqual(self.tool.record["min_free_bytes"], self.free_bytes)
        self.assertEqual([item["min_free_bytes"] for item in self.tool.record["admissions"]],
                         [self.free_bytes, self.free_bytes])
        self.assert_restored(before)

    def test_docs_need_no_disk_observation_or_metadata(self):
        self.change("tip", "docs/new.md", "docs\n")
        before = self.snapshot()
        self.free_bytes = 0
        self.space_effect = lambda: self.fail("docs must not request build admission")
        self.metadata_effect = lambda: self.fail("docs must not request metadata")
        self.assertEqual(self.run_tool(), 0)
        self.assertEqual(self.calls, [])
        self.assertEqual(self.tool.record["admissions"], [])
        self.assertEqual(self.tool.record["selection"], "docs")
        self.assert_restored(before)

    def test_metadata_and_fmt_finish_before_first_build_admission(self):
        self.change("tip", "skein/core/src/lib.rs", "candidate\n")
        before = self.snapshot()
        self.free_bytes = 0
        seen = []

        def metadata_ready():
            self.assertEqual(self.tool.record["admissions"], [])
            seen.append("metadata")

        self.metadata_effect = metadata_ready
        self.assertEqual(self.run_tool(), 1)
        self.assertEqual(seen, ["metadata"])
        self.assertEqual([step["name"] for step in self.tool.record["steps"]], ["ratchet:skein", "fmt"])
        self.assertFalse(any(argv[0] == "heavy" for argv, _ in self.calls))
        self.assert_restored(before)

    def test_disk_observation_failure_is_infrastructure_and_restores(self):
        self.change("tip", "unrelated/Cargo.toml", "candidate\n")
        before = self.snapshot()

        def unavailable():
            raise OSError("fixture disk observation failed")

        self.space_effect = unavailable
        self.assertEqual(self.run_tool(), 1)
        self.assertEqual(self.calls, [])
        self.assertEqual(self.tool.record["outcome"], "infrastructure-error")
        self.assertIn("fixture disk observation failed", self.tool.record["error"])
        self.assert_restored(before)

    def test_nextest_exit_four_still_passes_after_admission(self):
        self.change("tip", "unrelated/Cargo.toml", "candidate\n")
        before = self.snapshot()
        self.codes["nextest"] = 4
        self.assertEqual(self.run_tool(), 0)
        self.assertEqual(self.tool.record["steps"][-1]["returncode"], 4)
        self.assertTrue(self.tool.record["steps"][-1]["green"])
        self.assert_restored(before)


class CliTests(unittest.TestCase):
    def test_default_and_explicit_threshold_reach_helper(self):
        for options, expected in (([], MINIMUM), (["--min-free-bytes", "1"], 1),
                                  (["--min-free-bytes", "999999999999999999999999"], 999999999999999999999999)):
            with self.subTest(options=options), \
                    patch.object(sys, "argv", ["precheck", *options, "slot", "tip"]), \
                    patch.object(precheck, "policy", return_value=(fixtures.mq, (), ())), \
                    patch.object(precheck, "Precheck") as helper:
                helper.return_value.run.return_value = 0
                self.assertEqual(precheck.main(), 0)
                self.assertEqual(helper.call_args.kwargs["min_free_bytes"], expected)
                helper.return_value.run.assert_called_once_with()

    def test_malformed_threshold_refuses_before_policy_or_checkout(self):
        for value in ("0", "-1", "1.5", "nan", "NaN", "inf", "+inf", "-inf", "8e10", "", "wat"):
            with self.subTest(value=value), \
                    patch.object(sys, "argv", ["precheck", "--min-free-bytes=" + value, "slot", "tip"]), \
                    patch.object(precheck, "policy") as policy, \
                    patch.object(precheck, "Precheck") as helper, redirect_stderr(io.StringIO()):
                with self.assertRaises(SystemExit) as error:
                    precheck.main()
                self.assertEqual(error.exception.code, 2)
                policy.assert_not_called()
                helper.assert_not_called()

    def test_direct_helper_rejects_invalid_threshold_before_git(self):
        for value in (0, -1, float("nan"), float("inf"), -float("inf"), 1.5, "1", True, None):
            with self.subTest(value=value), patch.object(fixtures.mq, "git") as git:
                with self.assertRaisesRegex(ValueError, "positive integer"):
                    precheck.Precheck("slot", "base", "tip", mq=fixtures.mq, always=[],
                                      read_by_tests=[], min_free_bytes=value)
                git.assert_not_called()


if __name__ == "__main__":
    unittest.main()
