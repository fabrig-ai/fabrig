#!/usr/bin/env python3
"""Offline checks for tools/tiers/run (design.md, 5)."""

import contextlib
import io
import json
import os
from pathlib import Path
import runpy
import subprocess
import sys
import tempfile
import time
import unittest
from unittest import mock


RUN = Path(__file__).resolve().with_name("run")
MODULE = runpy.run_path(str(RUN))
FAKE = '''#!/usr/bin/env python3
import json
import os
from pathlib import Path
import sys
name = Path(sys.argv[0]).name
args = sys.argv[1:]
with open(os.environ["CALLS"], "a") as output:
    output.write(json.dumps({"name": name, "args": args, "env": {
        key: os.environ.get(key) for key in (
            "NEXTEST_DOUBLE_SPAWN", "TEMPER_BUNDLE", "SKEIN_UI_EXAMPLE_BUNDLE",
            "JIG_TEST_BUNDLE", "TEMPER_TEST_BUNDLE")}}) + "\\n")
if name == "cargo":
    if args != ["metadata", "--format-version", "1", "--no-deps"]:
        sys.exit(91)
    if os.environ.get("BAD_METADATA"):
        print("not JSON")
    else:
        print(json.dumps({"target_directory": os.environ["ARTIFACTS"]}))
    sys.exit(int(os.environ.get("METADATA_EXIT", "0")))
if name == "df":
    print("Avail\\n80000000000")
    sys.exit(0)
if name == "heavy":
    print("heavy: task fixture log /fixture/heavy.log", flush=True)
    if args[1] == "nextest":
        print("Summary: fixture tests", flush=True)
        sys.exit(int(os.environ.get("NEXTEST_EXIT", "0")))
    if os.environ.get("FAIL_BUILD") in args:
        sys.exit(19)
    sys.exit(0)
if name == "skein-ui-bundle":
    sys.exit(int(os.environ.get("BUNDLE_EXIT", "0")))
sys.exit(92)
'''


class RunnerTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="tiers-test-", dir=RUN.parents[2].parent)
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        (self.root / ".config").mkdir()
        self.config = self.root / ".config/nextest.toml"
        self.config.write_text(''.join(f'[profile.{name}]\n' for name in (
            "default", "fuzzy", "browser-skein", "browser-jig", "browser-temper",
            "qa-temper", "qa-smith", "process-jig", "exhaustive-skein")))
        self.bin = self.root / "bin"
        self.bin.mkdir()
        self.target = self.root / "relocated artifacts"
        (self.target / "debug").mkdir(parents=True)
        for path in [*(self.bin / name for name in ("cargo", "heavy", "df")),
                     self.target / "debug/skein-ui-bundle"]:
            path.write_text(FAKE)
            path.chmod(0o700)
        self.calls = self.root / "calls.jsonl"
        self.env = dict(os.environ, PATH=f"{self.bin}:{os.environ['PATH']}",
                        ARTIFACTS=str(self.target), CALLS=str(self.calls),
                        TEMPER_BUNDLE="none", NEXTEST_DOUBLE_SPAWN="1")
        for key in ("FAIL_BUILD", "NEXTEST_EXIT", "BUNDLE_EXIT", "BAD_METADATA", "METADATA_EXIT",
                    "SKEIN_UI_EXAMPLE_BUNDLE", "JIG_TEST_BUNDLE", "TEMPER_TEST_BUNDLE"):
            self.env.pop(key, None)

    def invoke(self, tree, tier, *extra, **env):
        self.calls.unlink(missing_ok=True)
        result = subprocess.run([sys.executable, "-I", "-B", str(RUN), tree, tier, *extra],
                                cwd=self.root, env=dict(self.env, **env),
                                stdin=subprocess.DEVNULL, text=True, capture_output=True, timeout=15)
        calls = [json.loads(line) for line in self.calls.read_text().splitlines()] if self.calls.exists() else []
        return result, calls

    def test_config_is_the_only_pair_inventory(self):
        self.config.write_text('[profile.default]\n[profile.new-tier-newtree]\n')
        result, calls = self.invoke("newtree", "new-tier", "--", "-E", "test(=one)")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual([call['name'] for call in calls], ["df", "heavy"])
        self.assertEqual(calls[-1]['args'], ["cargo", "nextest", "run", "--workspace", "--profile",
                                            "new-tier-newtree", "--no-fail-fast", "-E", "test(=one)"])
        result, calls = self.invoke("skein", "browser")
        self.assertEqual(result.returncode, 2)
        self.assertIn("refused unknown pair", result.stderr)
        self.assertEqual(calls, [])

    def test_no_prerequisite_pairs(self):
        for tree, tier in (("smith", "qa"), ("jig", "process"), ("skein", "exhaustive")):
            with self.subTest(tree=tree, tier=tier):
                result, calls = self.invoke(tree, tier)
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertEqual([call['name'] for call in calls], ["df", "heavy"])
                self.assertEqual(calls[-1]['env']['NEXTEST_DOUBLE_SPAWN'], "0")
                self.assertIn("/fixture/heavy.log", result.stdout)
                self.assertIn("Summary: fixture tests", result.stdout)

    def test_all_bundle_prerequisites(self):
        cases = (
            ("skein", "browser", "skein-ui-example-shell-web", "skein-ui-example-server",
             "SKEIN_UI_EXAMPLE_BUNDLE", "ui-example-bundle", [],
             ["--stylesheet", "skein/examples/ui/shell-web/notes.css",
              "--noscript", "Notes needs WebAssembly and scripts."]),
            ("jig", "browser", "jig-client-test-shell-web", "jig-test",
             "JIG_TEST_BUNDLE", "jig-test-bundle", [],
             ["--stylesheet", "jig/crates/jig-client-shell-web/jig.css",
              "--stylesheet", "jig/testing/jig-client-test-shell-web/jig_client_test_shell_web.css",
              "--pictures", "jig=jig/crates/jig-client-shell-web/pictures",
              "--pictures", "jig-test=jig/testing/jig-client-test-shell-web/pictures",
              "--files", "jig/testing/jig-client-test-shell-web/files",
              "--noscript", "jig needs WebAssembly and scripts."]),
        )
        for tier in ("browser", "qa"):
            shell = "temper/crates/temper-client-shell-web"
            cases += (("temper", tier, "temper-client-shell-web", "temper", "TEMPER_TEST_BUNDLE",
                       "temper-bundle", ["--features", "loopback-navigation"],
                       ["--stylesheet", f"{shell}/temper.css", "--stylesheet", f"{shell}/task-pages.css",
                        "--files", f"{shell}/files", "--noscript", "temper needs WebAssembly and scripts."]),)
        for tree, tier, package, server, variable, out, features, options in cases:
            with self.subTest(tree=tree, tier=tier):
                result, calls = self.invoke(tree, tier, "--", "-E", "test(=one)", **{variable: "/stale"})
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertEqual([call['name'] for call in calls],
                                 ["cargo", "df", "heavy", "df", "heavy", "skein-ui-bundle",
                                  "df", "heavy", "df", "heavy"])
                self.assertEqual(calls[2]['args'], ["cargo", "build", "--target", "wasm32-unknown-unknown",
                                                   *features, "-p", package])
                self.assertEqual(calls[4]['args'], ["cargo", "build", "-p", "skein-ui-bundle"])
                module = package.replace("-", "_")
                self.assertEqual(calls[5]['args'], [
                    "--module", str(self.target / "wasm32-unknown-unknown/debug" / f"{module}.wasm"),
                    "--name", module, "--prefix", "/bundle/", *options, "--out", str(self.target / out)])
                self.assertEqual(calls[7]['args'], ["cargo", "build", "-p", server])
                self.assertEqual(calls[9]['args'], ["cargo", "nextest", "run", "--workspace", "--profile",
                                                   f"{tier}-{tree}", "--no-fail-fast", "-E", "test(=one)"])
                self.assertEqual(calls[9]['env'][variable], str(self.target / out))
                for call in calls:
                    self.assertEqual(call['env']['NEXTEST_DOUBLE_SPAWN'], "0")
                    self.assertEqual(call['env']['TEMPER_BUNDLE'], "none")

    def test_failures_stop_at_the_named_step(self):
        cases = (({"FAIL_BUILD": "skein-ui-example-shell-web"}, "wasm module", 1),
                 ({"FAIL_BUILD": "skein-ui-bundle"}, "bundle tool", 2),
                 ({"BUNDLE_EXIT": "8"}, "browser bundle", 2),
                 ({"FAIL_BUILD": "skein-ui-example-server"}, "browser server", 3),
                 ({"METADATA_EXIT": "9"}, "cargo metadata", 0),
                 ({"BAD_METADATA": "1"}, "cargo metadata", 0))
        for env, step, count in cases:
            with self.subTest(step=step, env=env):
                result, calls = self.invoke("skein", "browser", **env)
                self.assertEqual(result.returncode, 2)
                self.assertIn(f"failed step {step}", result.stderr)
                heavy = [call for call in calls if call['name'] == "heavy"]
                self.assertEqual(len(heavy), count)
                self.assertFalse(any('nextest' in call['args'] for call in heavy))

    def test_nextest_exit_is_preserved(self):
        for code in (4, 100):
            result, calls = self.invoke("smith", "qa", NEXTEST_EXIT=str(code))
            self.assertEqual(result.returncode, code, result.stderr)
            self.assertEqual(len(calls), 2)

    def test_invalid_config_and_arguments(self):
        result, calls = self.invoke("smith", "qa", "-E", "test(=one)")
        self.assertEqual(result.returncode, 2)
        self.assertEqual(calls, [])
        self.config.write_text('[profile.bad\n')
        result, calls = self.invoke("smith", "qa")
        self.assertEqual(result.returncode, 2)
        self.assertIn(".config/nextest.toml", result.stderr)
        self.assertEqual(calls, [])

    def test_disk_wait_before_heavy(self):
        events = []
        available = iter((79_999_999_999, 1, 80_000_000_000))

        def fake_run(command, **kwargs):
            events.append(command)
            if command[0] == "df":
                return subprocess.CompletedProcess(command, 0, f"Avail\n{next(available)}\n")
            return subprocess.CompletedProcess(command, 0)

        def sleep(seconds):
            events.append(["sleep", seconds])

        output = io.StringIO()
        with mock.patch.object(subprocess, "run", side_effect=fake_run), \
                mock.patch.object(time, "sleep", side_effect=sleep), contextlib.redirect_stdout(output):
            self.assertEqual(MODULE['step']("fixture", ["heavy", "cargo", "check"], self.env), 0)
        self.assertEqual(events, [["df", "-B1", "--output=avail", "/"], ["sleep", 60],
                                  ["df", "-B1", "--output=avail", "/"], ["sleep", 60],
                                  ["df", "-B1", "--output=avail", "/"], ["heavy", "cargo", "check"]])
        self.assertEqual(output.getvalue().count("waiting for disk"), 2)

    def test_disk_and_launch_errors_name_the_step(self):
        for error in (FileNotFoundError("missing command"),
                      subprocess.CalledProcessError(1, ["df"])):
            with mock.patch.object(subprocess, "run", side_effect=error):
                with self.assertRaisesRegex(MODULE['FailedStep'], "fixture"):
                    MODULE['step']("fixture", ["heavy", "cargo", "check"], self.env)


if __name__ == "__main__":
    started = time.monotonic()
    suite = unittest.main(exit=False)
    print(f"Elapsed: {time.monotonic() - started:.3f}s", flush=True)
    sys.exit(not suite.result.wasSuccessful())
