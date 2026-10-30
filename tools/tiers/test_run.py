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
            "JIG_TEST_BUNDLE", "TEMPER_TEST_BUNDLE", "TEMPER_TEST_CLAUDE")}}) + "\\n")
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
        if args[2] == "list":
            if os.environ.get("BAD_LISTING"):
                print("not JSON")
            else:
                tests = {"one": {"filter-match": {"status": "matches"}},
                         "filtered": {"filter-match": {"status": "mismatch"}}}
                if "--workspace" in args:
                    tests["missing"] = {"filter-match": {"status": "matches"}}
                    tests["missing-too"] = {"filter-match": {"status": "matches"}}
                elif os.environ.get("MISMATCH"):
                    tests["extra"] = {"filter-match": {"status": "matches"}}
                    tests["extra-too"] = {"filter-match": {"status": "matches"}}
                else:
                    tests["missing"] = {"filter-match": {"status": "matches"}}
                    tests["missing-too"] = {"filter-match": {"status": "matches"}}
                print(json.dumps({"rust-suites": {
                    "fixture::suite": {"package-name": "fixture", "binary-id": "fixture::suite",
                                       "testcases": tests},
                    "fixture::skipped": {"status": "skipped"}}}, indent=2))
            print("heavy: task fixture exit log /fixture/list.log", flush=True)
            sys.exit(int(os.environ.get("NEXTEST_EXIT", "0")))
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
        self.temp = tempfile.TemporaryDirectory(prefix="tiers-test-", dir=RUN.parents[2])
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        (self.root / ".config").mkdir()
        self.config = self.root / ".config/nextest.toml"
        self.config.write_text(''.join(f'[profile.{name}]\n' for name in (
            "default", "fuzzy", "browser-skein", "browser-jig", "browser-temper",
            "qa-temper", "qa-smith", "process-jig", "exhaustive-skein")))
        (self.root / "tools/tiers").mkdir(parents=True)
        self.packages = self.root / "tools/tiers/packages.toml"
        self.packages.write_text(''.join(f'{name} = ["a-package", "z-package"]\n' for name in (
            "browser-skein", "browser-jig", "browser-temper", "qa-temper", "qa-smith",
            "process-jig", "exhaustive-skein")))
        self.binary_lists = (
            '[binaries]\n'
            'browser-skein = [{package="skein-ui-example-server", bin="skein-ui-example"}]\n'
            'browser-jig = [{package="jig-test", bin="jig-test"}]\n'
            'browser-temper = [{package="temper", bin="temper"}]\n'
            'qa-temper = [{package="temper", bin="temper"}, '
            '{package="temper-fake-claude", bin="fake-claude"}]\n'
            'qa-smith = []\nprocess-jig = []\nexhaustive-skein = []\n')
        with self.packages.open('a') as output:
            output.write(self.binary_lists)
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
                    "SKEIN_UI_EXAMPLE_BUNDLE", "JIG_TEST_BUNDLE", "TEMPER_TEST_BUNDLE",
                    "BAD_LISTING", "MISMATCH"):
            self.env.pop(key, None)

    def invoke(self, *arguments, **env):
        self.calls.unlink(missing_ok=True)
        result = subprocess.run([sys.executable, "-I", "-B", str(RUN), *arguments],
                                cwd=self.root, env=dict(self.env, **env),
                                stdin=subprocess.DEVNULL, text=True, capture_output=True, timeout=15)
        calls = [json.loads(line) for line in self.calls.read_text().splitlines()] if self.calls.exists() else []
        return result, calls

    def test_config_is_the_only_pair_inventory(self):
        self.config.write_text('[profile.default]\n[profile.new-tier-newtree]\n')
        self.packages.write_text('new-tier-newtree = ["a-package", "z-package"]\n'
                                 '[binaries]\nnew-tier-newtree = []\n')
        result, calls = self.invoke("newtree", "new-tier", "--", "-E", "test(=one)")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual([call['name'] for call in calls], ["df", "heavy"])
        self.assertEqual(calls[-1]['args'], ["cargo", "nextest", "run", "-p", "a-package", "-p", "z-package", "--profile",
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
                if tier == "qa":
                    self.assertEqual(calls[9]['args'], ["cargo", "build", "-p", "temper-fake-claude",
                                                       "--bin", "fake-claude"])
                    del calls[8:10]
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
                native = "skein-ui-example" if tree == "skein" else server
                self.assertEqual(calls[7]['args'], ["cargo", "build", "-p", server, "--bin", native])
                self.assertEqual(calls[9]['args'], ["cargo", "nextest", "run", "-p", "a-package", "-p", "z-package", "--profile",
                                                   f"{tier}-{tree}", "--no-fail-fast", "-E", "test(=one)"])
                self.assertEqual(calls[9]['env'][variable], str(self.target / out))
                for call in calls:
                    self.assertEqual(call['env']['NEXTEST_DOUBLE_SPAWN'], "0")
                    self.assertEqual(call['env']['TEMPER_BUNDLE'], "none")

    def test_failures_stop_at_the_named_step(self):
        cases = (({"FAIL_BUILD": "skein-ui-example-shell-web"}, "wasm module", 1),
                 ({"FAIL_BUILD": "skein-ui-bundle"}, "bundle tool", 2),
                 ({"BUNDLE_EXIT": "8"}, "browser bundle", 2),
                 ({"FAIL_BUILD": "skein-ui-example-server"},
                  "prerequisite binary skein-ui-example-server/skein-ui-example", 3),
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

    def test_native_binary_order_and_failure_stop_later_steps(self):
        self.packages.write_text(self.packages.read_text().replace(
            'process-jig = []', 'process-jig = [{package="first", bin="one"}, '
            '{package="last", bin="two"}]'))
        result, calls = self.invoke("jig", "process")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual([call['name'] for call in calls], ["df", "heavy"] * 3)
        self.assertEqual(calls[1]['args'], ["cargo", "build", "-p", "first", "--bin", "one"])
        self.assertEqual(calls[3]['args'], ["cargo", "build", "-p", "last", "--bin", "two"])
        self.assertEqual(calls[5]['args'][:3], ["cargo", "nextest", "run"])
        for package, count in (("first", 2), ("last", 4)):
            result, calls = self.invoke("jig", "process", FAIL_BUILD=package)
            self.assertEqual(result.returncode, 2, result.stderr)
            self.assertIn(f"failed step prerequisite binary {package}/", result.stderr)
            self.assertEqual(len(calls), count)
            self.assertFalse(any('nextest' in call['args'] for call in calls))

    def test_binary_validation_in_run_and_verification(self):
        original = self.packages.read_text()
        invalid = ('"not a list"', '["not a record"]', '[1]', '[{}]',
                   '[{package="p"}]', '[{package="p",bin="b",extra=1}]',
                   '[{package=1,bin="b"}]', '[{package="p",bin=""}]',
                   '[{package="--p",bin="../b"}]',
                   '[{package="z",bin="b"},{package="a",bin="a"}]',
                   '[{package="p",bin="b"},{package="p",bin="b"}]',
                   '[{package="a",bin="b"},{package="z",bin="b"}]')
        invalid += ('[{package="p",bin="b",env=1}]', '[{package="p",bin="b",env="HOME"}]',
                    '[{package="p",bin="b",env="bad_TEST_env"}]',
                    '[{package="a",bin="a",env="P_TEST_BIN"},'
                    '{package="b",bin="b",env="P_TEST_BIN"}]')
        inventories = [original.replace('qa-smith = []', f'qa-smith = {value}') for value in invalid]
        inventories += [original.split('[binaries]')[0],
                        original.replace('qa-smith = []\n', ''),
                        original + 'unknown-pair = []\n',
                        original.split('[binaries]')[0] + 'binaries = []\n']
        for contents in inventories:
            self.packages.write_text(contents)
            for args in (("smith", "qa"), ("--verify", "smith", "qa"), ("--verify-all",)):
                with self.subTest(contents=contents, args=args):
                    result, calls = self.invoke(*args)
                    self.assertEqual(result.returncode, 2, result.stderr)
                    self.assertIn("tools/tiers/packages.toml", result.stderr)
                    self.assertEqual(calls, [])

    def test_binary_environment_uses_metadata_target_after_build(self):
        self.packages.write_text(self.packages.read_text().replace(
            'process-jig = []',
            'process-jig = [{package="adapter",bin="fake-claude",env="TEMPER_TEST_CLAUDE"}]'))
        result, calls = self.invoke("jig", "process", TEMPER_TEST_CLAUDE="/stale")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual([call['name'] for call in calls], ["cargo", "df", "heavy", "df", "heavy"])
        self.assertEqual(calls[2]['args'], ["cargo", "build", "-p", "adapter", "--bin", "fake-claude"])
        self.assertEqual(calls[2]['env']['TEMPER_TEST_CLAUDE'], "/stale")
        self.assertEqual(calls[-1]['env']['TEMPER_TEST_CLAUDE'], str(self.target / "debug/fake-claude"))
        result, calls = self.invoke("--verify", "jig", "process", TEMPER_TEST_CLAUDE="/stale")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual([call['name'] for call in calls], ["df", "heavy", "df", "heavy"])

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

    def test_missing_and_invalid_package_maps_refuse_before_commands(self):
        for contents in ('', 'qa-smith = []', 'qa-smith = ["z", "a"]',
                         'qa-smith = ["a", "a"]', 'qa-smith = [1]',
                         'qa-smith = "smith"', 'qa-smith = ['):
            with self.subTest(contents=contents):
                self.packages.write_text(contents + '\n' + self.binary_lists)
                result, calls = self.invoke("smith", "qa")
                self.assertEqual(result.returncode, 2)
                self.assertIn("tools/tiers/packages.toml", result.stderr)
                self.assertEqual(calls, [])
        self.packages.unlink()
        result, calls = self.invoke("smith", "qa")
        self.assertEqual(result.returncode, 2)
        self.assertEqual(calls, [])

    def test_verify_lists_both_selections_without_prerequisites(self):
        result, calls = self.invoke("--verify", "temper", "qa")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual([call['name'] for call in calls], ["df", "heavy", "df", "heavy"])
        prefix = ["cargo", "nextest", "list"]
        suffix = ["--profile", "qa-temper", "--message-format", "json"]
        self.assertEqual(calls[1]['args'], [*prefix, "--workspace", *suffix])
        self.assertEqual(calls[3]['args'], [*prefix, "-p", "a-package", "-p", "z-package", *suffix])
        self.assertIn("workspace 3 tests; packages 3 tests (2 packages)", result.stdout)
        self.assertIn("/fixture/list.log", result.stdout)
        self.assertNotIn('"rust-suites"', result.stdout)

    def test_verify_names_every_missing_and_added_test(self):
        result, calls = self.invoke("--verify", "smith", "qa", MISMATCH="1")
        self.assertEqual(result.returncode, 1, result.stderr)
        for name in ("missing", "missing-too"):
            self.assertIn(f"package set misses fixture fixture::suite {name}\n", result.stderr)
        for name in ("extra", "extra-too"):
            self.assertIn(f"package set adds fixture fixture::suite {name}\n", result.stderr)
        self.assertNotIn("filtered", result.stderr)
        self.assertEqual(len(calls), 4)

    def test_verify_all_checks_every_configured_pair_even_after_mismatch(self):
        result, calls = self.invoke("--verify-all", MISMATCH="1")
        self.assertEqual(result.returncode, 1, result.stderr)
        heavy = [call for call in calls if call['name'] == "heavy"]
        profiles = sorted(name for name in MODULE['tomllib'].loads(self.packages.read_text())
                          if name != 'binaries')
        self.assertEqual(len(heavy), 2 * len(profiles))
        self.assertEqual([call['args'][call['args'].index('--profile') + 1] for call in heavy],
                         [profile for profile in profiles for _ in range(2)])
        self.packages.write_text('qa-smith = ["smith"]')
        result, calls = self.invoke("--verify-all")
        self.assertEqual(result.returncode, 2)
        self.assertEqual(calls, [])

    def test_verify_failures_and_arguments(self):
        for env, message in (({"BAD_LISTING": "1"}, "missing nextest JSON listing"),
                             ({"NEXTEST_EXIT": "17"}, "exit 17")):
            result, calls = self.invoke("--verify", "smith", "qa", **env)
            self.assertEqual(result.returncode, 2)
            self.assertIn(message, result.stderr)
            self.assertEqual(len(calls), 2)
        for args in (("--verify-all", "smith", "qa"), ("--verify", "smith"),
                     ("--verify", "smith", "qa", "--", "-E", "all()"),
                     ("--verify-all", "--"), ("--verify-all", "--verify")):
            result, calls = self.invoke(*args)
            self.assertEqual(result.returncode, 2)
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
