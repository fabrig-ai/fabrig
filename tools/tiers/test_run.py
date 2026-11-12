#!/usr/bin/env python3
"""Offline checks for tools/tiers/run (design.md, 5)."""

import contextlib
import io
import json
import os
from pathlib import Path
import runpy
import shutil
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
            "NEXTEST_DOUBLE_SPAWN", "TEMPER_INTERNAL_BUNDLE", "SKEIN_UI_EXAMPLE_BUNDLE",
            "JIG_TEST_BUNDLE", "TEMPER_TEST_BUNDLE", "TEMPER_TEST_PLAYGROUND",
            "CARGO_TARGET_DIR", "DEMO_TEST_CLAUDE")}}) + "\\n")
if name == "cargo" and args[:1] == ["metadata"]:
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
if name == "bundle.sh":
    sys.exit(19 if os.environ.get("FAIL_BUILD") == "temper/scripts/bundle.sh" else 0)
if name in ("wrap", "cargo", "cc"):
    if name == "wrap":
        if args[:2] == ["--flag", "one"]:
            args = args[2:]
    else:
        args = [name, *args]
    print("wrap: task fixture log /fixture/wrap.log", flush=True)
    if args[:1] == ["temper/scripts/bundle.sh"]:
        if os.environ.get("FAIL_BUILD") in args:
            sys.exit(19)
        sys.exit(0)
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
            print("wrap: task fixture exit log /fixture/list.log", flush=True)
            sys.exit(int(os.environ.get("NEXTEST_EXIT", "0")))
        if "--tool-config-file" in args:
            import tomllib
            overlay = Path(args[args.index("--tool-config-file") + 1].split(":", 1)[1])
            config = tomllib.loads(overlay.read_text())
            profile = args[args.index("--profile") + 1]
            Path(config['profile'][profile]['junit']['path']).write_text('<testsuites/>')
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
        self.bundles = self.root / "tools/tiers/bundles.toml"
        shutil.copyfile(RUN.with_name("bundles.toml"), self.bundles)
        (self.root / "tools/tiers/budgets.toml").write_text('')
        self.packages = self.root / "tools/tiers/packages.toml"
        self.packages.write_text(''.join(f'{name} = ["a-package", "z-package"]\n' for name in (
            "browser-skein", "browser-jig", "browser-temper", "qa-temper", "qa-smith",
            "process-jig", "exhaustive-skein")))
        self.binary_lists = (
            '[binaries]\n'
            'browser-skein = [{package="skein-ui-example-server", bin="skein-ui-example"}]\n'
            'browser-jig = [{package="jig-test", bin="jig-test"}]\n'
            'browser-temper = [{package="temper-box", bin="temper-box"}]\n'
            'qa-temper = [{package="smith-fake-claude", bin="fake-claude"}, '
            '{package="temper-box", bin="temper-box"}]\n'
            'qa-smith = []\nprocess-jig = []\nexhaustive-skein = []\n')
        with self.packages.open('a') as output:
            output.write(self.binary_lists)
        self.bin = self.root / "bin"
        self.bin.mkdir()
        self.target = self.root / "relocated artifacts"
        (self.target / "debug").mkdir(parents=True)
        playground_script = self.root / "temper/scripts/bundle.sh"
        playground_script.parent.mkdir(parents=True)
        for path in [*(self.bin / name for name in ("cargo", "cc", "wrap", "df")),
                     self.target / "debug/skein-ui-bundle", playground_script]:
            path.write_text(FAKE)
            path.chmod(0o700)
        self.calls = self.root / "calls.jsonl"
        self.env = dict(os.environ, PATH=f"{self.bin}:{os.environ['PATH']}",
                        ARTIFACTS=str(self.target), CALLS=str(self.calls),
                        TEMPER_INTERNAL_BUNDLE="none", NEXTEST_DOUBLE_SPAWN="1")
        for key in ("FAIL_BUILD", "NEXTEST_EXIT", "BUNDLE_EXIT", "BAD_METADATA", "METADATA_EXIT",
                    "SKEIN_UI_EXAMPLE_BUNDLE", "JIG_TEST_BUNDLE", "TEMPER_TEST_BUNDLE", "TEMPER_TEST_PLAYGROUND",
                    "BAD_LISTING", "MISMATCH", "TIERS_WRAPPER", "TIERS_DISK_BYTES"):
            self.env.pop(key, None)
        self.env.update(TIERS_WRAPPER="wrap", TIERS_DISK_BYTES="80000000000")

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
        self.assertEqual([call['name'] for call in calls], ["df", "wrap"])
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
                self.assertEqual([call['name'] for call in calls], ["df", "wrap"])
                self.assertEqual(calls[-1]['env']['NEXTEST_DOUBLE_SPAWN'], "0")
                self.assertIn("/fixture/wrap.log", result.stdout)
                self.assertIn("Summary: fixture tests", result.stdout)

    def test_qa_temper_has_native_prerequisites_without_bundle(self):
        result, calls = self.invoke("temper", "qa")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual([call['name'] for call in calls], ["df", "wrap"] * 3)
        self.assertEqual(calls[1]['args'], ["cargo", "build", "-p", "smith-fake-claude", "--bin", "fake-claude"])
        self.assertEqual(calls[3]['args'], ["cargo", "build", "-p", "temper-box", "--bin", "temper-box"])
        self.assertEqual(calls[5]['args'][:3], ["cargo", "nextest", "run"])
        self.assertIsNone(calls[5]['env']['TEMPER_TEST_BUNDLE'])

    def test_all_bundle_prerequisites(self):
        cases = (
            ("skein", "browser", "skein-ui-example-shell-web", "skein-ui-example-server",
             "SKEIN_UI_EXAMPLE_BUNDLE", "ui-example-bundle", [],
             ["--stylesheet", "skein/examples/ui/shell-web/notes.css",
              "--placeholder", "skein/examples/ui/shell-web/loading.html",
              "--placeholder-stylesheet", "skein/examples/ui/shell-web/loading.css",
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
        shell = "temper/crates/temper-client-shell-web"
        cases += (("temper", "browser", "temper-client-web-page", "temper-box", "TEMPER_TEST_BUNDLE",
                   "temper-bundle", [],
                   ["--title", "temper", "--placeholder", f"{shell}/loading.html",
                    "--placeholder-stylesheet", f"{shell}/loading.css",
                    "--stylesheet", f"{shell}/temper.css", "--stylesheet", f"{shell}/task-pages.css",
                    "--files", f"{shell}/files", "--noscript", "temper needs WebAssembly and scripts."]),)
        for tree, tier, package, server, variable, out, features, options in cases:
            with self.subTest(tree=tree, tier=tier):
                result, calls = self.invoke(tree, tier, "--", "-E", "test(=one)", **{variable: "/stale"})
                self.assertEqual(result.returncode, 0, result.stderr)
                if tree == "temper" and tier == "browser":
                    self.assertEqual([call['name'] for call in calls[6:8]], ["df", "wrap"])
                    self.assertEqual(calls[7]['args'], ["temper/scripts/bundle.sh", "--playground",
                                                       str(self.target / "temper-playground")])
                    self.assertEqual(calls[7]['env']['CARGO_TARGET_DIR'], str(self.target))
                    self.assertEqual(calls[-1]['env']['TEMPER_TEST_PLAYGROUND'],
                                     str(self.target / "temper-playground"))
                    self.assertIn("playground bundle built in", result.stdout)
                    del calls[6:8]
                self.assertEqual([call['name'] for call in calls],
                                 ["cargo", "df", "wrap", "df", "wrap", "skein-ui-bundle",
                                  "df", "wrap", "df", "wrap"])
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
                    self.assertEqual(call['env']['TEMPER_INTERNAL_BUNDLE'], "none")

    def test_playground_bundle_replaces_stale_path_only_for_browser_temper(self):
        for tier in ("browser", "qa"):
            result, calls = self.invoke("temper", tier, TEMPER_TEST_PLAYGROUND="/stale")
            self.assertEqual(result.returncode, 0, result.stderr)
            expected = str(self.target / "temper-playground") if tier == "browser" else "/stale"
            self.assertEqual(calls[-1]['env']['TEMPER_TEST_PLAYGROUND'], expected)
            builds = [call for call in calls if call['name'] == "wrap"
                      and call['args'][0] == "temper/scripts/bundle.sh"]
            self.assertEqual(len(builds), int(tier == "browser"))

    def test_playground_bundle_failure_stops_before_native_builds_and_tests(self):
        result, calls = self.invoke("temper", "browser", FAIL_BUILD="temper/scripts/bundle.sh")
        self.assertEqual(result.returncode, 2, result.stderr)
        self.assertIn("failed step playground bundle: exit 19", result.stderr)
        self.assertEqual(calls[-1]['args'][0], "temper/scripts/bundle.sh")
        self.assertFalse(any('nextest' in call['args'] for call in calls))

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
                wrap = [call for call in calls if call['name'] == "wrap"]
                self.assertEqual(len(wrap), count)
                self.assertFalse(any('nextest' in call['args'] for call in wrap))

    def test_native_binary_order_and_failure_stop_later_steps(self):
        self.packages.write_text(self.packages.read_text().replace(
            'process-jig = []', 'process-jig = [{package="first", bin="one"}, '
            '{package="last", bin="two"}]'))
        result, calls = self.invoke("jig", "process")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual([call['name'] for call in calls], ["df", "wrap"] * 3)
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

    def test_process_temper_builds_fake_claude_without_environment_setting(self):
        shutil.copyfile(RUN.parents[2] / '.config/nextest.toml', self.config)
        shutil.copyfile(RUN.with_name('packages.toml'), self.packages)
        inventory = MODULE['tomllib'].loads(self.packages.read_text())
        adapter = [binary for binary in inventory['binaries']['process-temper']
                   if binary['bin'] == 'fake-claude']
        self.assertEqual(adapter, [{'package': 'smith-fake-claude', 'bin': 'fake-claude'}])
        result, calls = self.invoke('temper', 'process')
        self.assertEqual(result.returncode, 0, result.stderr)
        builds = [call['args'] for call in calls if call['args'][:2] == ['cargo', 'build']]
        self.assertIn(['cargo', 'build', '-p', 'smith-fake-claude', '--bin', 'fake-claude'], builds)
        self.assertEqual(calls[-1]['args'][:3], ['cargo', 'nextest', 'run'])
        self.assertNotIn('cargo', [call['name'] for call in calls], 'no target lookup for an env setting')

    def test_binary_environment_uses_metadata_target_after_build(self):
        self.packages.write_text(self.packages.read_text().replace(
            'process-jig = []',
            'process-jig = [{package="adapter",bin="fake-claude",env="DEMO_TEST_CLAUDE"}]'))
        result, calls = self.invoke("jig", "process", DEMO_TEST_CLAUDE="/stale")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual([call['name'] for call in calls], ["cargo", "df", "wrap", "df", "wrap"])
        self.assertEqual(calls[2]['args'], ["cargo", "build", "-p", "adapter", "--bin", "fake-claude"])
        self.assertEqual(calls[2]['env']['DEMO_TEST_CLAUDE'], "/stale")
        self.assertEqual(calls[-1]['env']['DEMO_TEST_CLAUDE'], str(self.target / "debug/fake-claude"))
        result, calls = self.invoke("--verify", "jig", "process", DEMO_TEST_CLAUDE="/stale")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual([call['name'] for call in calls], ["df", "wrap", "df", "wrap"])

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

    def test_bundle_list_refusals_name_the_file_and_field(self):
        fields = {"package": '"temper-client-shell-web"', "out": '"temper-bundle"',
                  "variable": '"TEMPER_TEST_BUNDLE"', "options": '[]', "features": '[]'}

        def table(**changes):
            values = dict(fields, **changes)
            return "[temper]\n" + ''.join(f'{key} = {value}\n' for key, value in values.items()
                                         if value is not None)

        metadata = [["metadata", "--format-version", "1", "--no-deps"]]
        cases = (
            ("", "has no [temper] table"),
            ("temper = 1\n", "has no [temper] table"),
            ("[temper\n", "read tools/tiers/bundles.toml from the checkout root"),
            (table(title='"temper"'), "temper.title is no bundle field"),
            (table(zebra='1', alpha='1', package=None), "temper.alpha is no bundle field"),
            (table(package=None), "temper.package must be a non-empty string"),
            (table(package='1'), "temper.package must be a non-empty string"),
            (table(package='""'), "temper.package must be a non-empty string"),
            (table(package='""', out=None), "temper.package must be a non-empty string"),
            (table(variable=None), "temper.variable must be a non-empty string"),
            (table(variable='[]'), "temper.variable must be a non-empty string"),
            (table(variable='""'), "temper.variable must be a non-empty string"),
            (table(out=None), "temper.out must be a plain directory name"),
            (table(out='1'), "temper.out must be a plain directory name"),
            (table(out='""'), "temper.out must be a plain directory name"),
            (table(out='"."'), "temper.out must be a plain directory name"),
            (table(out='".."'), "temper.out must be a plain directory name"),
            (table(out='"a/b"'), "temper.out must be a plain directory name"),
            (table(out='"/abs"'), "temper.out must be a plain directory name"),
            (table(out=None, variable=None), "temper.out must be a plain directory name"),
            (table(options=None), "temper.options must be an array of strings"),
            (table(options='"--title"'), "temper.options must be an array of strings"),
            (table(options='[1]'), "temper.options must be an array of strings"),
            (table(variable=None, options=None), "temper.variable must be a non-empty string"),
            (table(options=None, features=None), "temper.options must be an array of strings"),
            (table(features=None), "temper.features must be an array of strings"),
            (table(features='"small-window"'), "temper.features must be an array of strings"),
            (table(features='[true]'), "temper.features must be an array of strings"),
        )
        for contents, message in cases:
            with self.subTest(contents=contents):
                self.bundles.write_text(contents)
                result, calls = self.invoke("temper", "browser")
                self.assertEqual(result.returncode, 2)
                self.assertIn("tiers: failed step bundle list: ", result.stderr)
                self.assertIn("tools/tiers/bundles.toml", result.stderr)
                self.assertIn(message, result.stderr)
                self.assertEqual([call['args'] for call in calls], metadata)
        self.bundles.unlink()
        result, calls = self.invoke("temper", "browser")
        self.assertEqual(result.returncode, 2)
        self.assertIn("tiers: failed step bundle list: ", result.stderr)
        self.assertIn("tools/tiers/bundles.toml", result.stderr)
        self.assertIn("read tools/tiers/bundles.toml from the checkout root", result.stderr)
        self.assertEqual([call['args'] for call in calls], metadata)

    def test_verify_lists_both_selections_without_prerequisites(self):
        result, calls = self.invoke("--verify", "temper", "qa")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual([call['name'] for call in calls], ["df", "wrap", "df", "wrap"])
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
        wrap = [call for call in calls if call['name'] == "wrap"]
        profiles = sorted(name for name in MODULE['tomllib'].loads(self.packages.read_text())
                          if name != 'binaries')
        self.assertEqual(len(wrap), 2 * len(profiles))
        self.assertEqual([call['args'][call['args'].index('--profile') + 1] for call in wrap],
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

    def test_cpu_overlay_and_native_prerequisite_are_opt_in(self):
        directory = self.root / 'CPU results'
        result, calls = self.invoke('--cpu', str(directory), 'smith', 'qa')
        self.assertEqual(result.returncode, 0, result.stderr + result.stdout)
        self.assertEqual([call['name'] for call in calls], ['cargo', 'df', 'wrap', 'df', 'wrap'])
        self.assertEqual(calls[2]['args'], ['cc', '-std=c11', '-O2', '-static', '-Wall', '-Wextra', '-Werror',
                                          'tools/tiers/cpu-wrapper.c', '-o', str(self.target / 'tiers-cpu-wrapper')])
        self.assertEqual(calls[-1]['args'], ['cargo', 'nextest', 'run', '--config-file', str(directory / 'repository.toml'),
                                            '--tool-config-file', f'tiers-cpu:{directory}/nextest.toml',
                                            '-p', 'a-package', '-p', 'z-package', '--profile', 'qa-smith', '--no-fail-fast'])
        self.assertEqual(calls[-1]['env']['NEXTEST_DOUBLE_SPAWN'], '0')
        result, calls = self.invoke('--cpu', str(directory), 'smith', 'qa')
        self.assertEqual(result.returncode, 2)
        self.assertEqual(calls, [])
        for flag in ('--profile', '--config-file', '--stress-count', '--stress-duration', '--no-run', '--debugger'):
            result, calls = self.invoke('--cpu', str(self.root / 'fresh'), 'smith', 'qa', '--', flag, 'other')
            self.assertEqual(result.returncode, 2)
            self.assertEqual(calls, [])

    def test_cpu_interruption_stops_the_command_session(self):
        import signal
        for sig in (signal.SIGTERM, signal.SIGHUP):
            with self.subTest(signal=sig):
                command_pid = self.root / f'command-{sig}'
                child_pid = self.root / f'child-{sig}'
                child = ('import os,time\nfrom pathlib import Path\n'
                         f'Path({str(child_pid)!r}).write_text(str(os.getpid()))\n'
                         'time.sleep(60)\n')
                source = ('import os,subprocess,time\nfrom pathlib import Path\n'
                          'if sys.argv[1:3]==["cargo","nextest"]:\n'
                          f'    subprocess.Popen([sys.executable,"-I","-c",{child!r}])\n'
                          f'    Path({str(command_pid)!r}).write_text(str(os.getpid()))\n'
                          '    time.sleep(60)\n')
                (self.bin / 'wrap').write_text(FAKE.replace('name = Path', source + 'name = Path', 1))
                process = subprocess.Popen(
                    [sys.executable, '-I', '-B', str(RUN), '--cpu', str(self.root / f'CPU-{sig}'), 'smith', 'qa'],
                    cwd=self.root, env=self.env, stdin=subprocess.DEVNULL,
                    stdout=subprocess.PIPE, stderr=subprocess.PIPE)
                try:
                    deadline = time.monotonic() + 3
                    while not (command_pid.exists() and child_pid.exists()):
                        self.assertIsNone(process.poll())
                        self.assertLess(time.monotonic(), deadline, 'fake command did not start')
                        time.sleep(0.005)
                    self.assertNotEqual(os.getsid(int(command_pid.read_text())), os.getsid(process.pid))
                    process.send_signal(sig)
                    stdout, stderr = process.communicate(timeout=3)
                    self.assertNotEqual(process.returncode, 0, stdout + stderr)
                    self.assertIn(b'KeyboardInterrupt', stderr)
                    for path in (command_pid, child_pid):
                        pid = int(path.read_text())
                        try:
                            stat = Path(f'/proc/{pid}/stat').read_bytes().rsplit(b')', 1)[1].split()
                        except FileNotFoundError:
                            continue
                        self.assertEqual(stat[0], b'Z', f'command process {pid} still alive')
                finally:
                    for path in (command_pid, child_pid):
                        if path.exists():
                            try:
                                os.kill(int(path.read_text()), signal.SIGKILL)
                            except ProcessLookupError:
                                pass
                    if process.poll() is None:
                        process.kill()
                    process.communicate(timeout=3)

    def test_cpu_preserves_native_prerequisites_and_reuses_metadata(self):
        self.packages.write_text(self.packages.read_text().replace(
            'qa-smith = []',
            'qa-smith = [{package="adapter",bin="fake-claude",env="DEMO_TEST_CLAUDE"}]'))
        directory = self.root / 'CPU with native'
        result, calls = self.invoke('--cpu', str(directory), 'smith', 'qa', DEMO_TEST_CLAUDE='/stale')
        self.assertEqual(result.returncode, 0, result.stderr + result.stdout)
        self.assertEqual([call['name'] for call in calls], ['cargo', 'df', 'wrap', 'df', 'wrap', 'df', 'wrap'])
        self.assertEqual(calls[2]['args'], ['cargo', 'build', '-p', 'adapter', '--bin', 'fake-claude'])
        self.assertEqual(calls[4]['args'][0], 'cc')
        self.assertEqual(calls[6]['args'][:3], ['cargo', 'nextest', 'run'])
        self.assertEqual(calls[2]['env']['DEMO_TEST_CLAUDE'], '/stale')
        for call in (calls[4], calls[6]):
            self.assertEqual(call['env']['DEMO_TEST_CLAUDE'], str(self.target / 'debug/fake-claude'))
        result, calls = self.invoke('--cpu', str(self.root / 'native-failure'), 'smith', 'qa', FAIL_BUILD='adapter')
        self.assertEqual(result.returncode, 2, result.stderr)
        self.assertEqual([call['name'] for call in calls], ['cargo', 'df', 'wrap'])
        result, calls = self.invoke('--cpu', str(self.root / 'wrapper-failure'), 'smith', 'qa',
                                    FAIL_BUILD='tools/tiers/cpu-wrapper.c')
        self.assertEqual(result.returncode, 2, result.stderr)
        self.assertIn('failed step CPU wrapper: exit 19', result.stderr)
        self.assertEqual([call['name'] for call in calls], ['cargo', 'df', 'wrap', 'df', 'wrap'])

    def test_verify_all_accepts_gate_budget(self):
        path = self.root / 'tools/tiers/budgets.toml'
        path.write_text('[gate.hearth.default]\ncpu_suite_s=30')
        result, calls = self.invoke('--verify-all')
        self.assertEqual(result.returncode, 0, result.stderr)
        commands = [call for call in calls if call['name'] == 'wrap']
        self.assertTrue(commands)
        self.assertTrue(all(call['args'][:3] == ['cargo', 'nextest', 'list']
                            for call in commands))

    def test_verify_all_validates_budgets_before_any_command(self):
        path = self.root / 'tools/tiers/budgets.toml'
        for content in ('[unknown]\ncpu_suite_s=1', '[qa-smith]\nwall_hard_s=0',
                        '[qa-smith]\ncpu_test_s=true', '[qa-smith]\nunknown=1', '[bad'):
            with self.subTest(content=content):
                path.write_text(content)
                result, calls = self.invoke('--verify-all')
                self.assertEqual(result.returncode, 2)
                self.assertEqual(calls, [])
        # Ordinary runs do not consult the opt-in budget file.
        result, calls = self.invoke('smith', 'qa')
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(len(calls), 2)

    def test_no_settings_run_plain_commands(self):
        self.env.pop("TIERS_WRAPPER")
        self.env.pop("TIERS_DISK_BYTES")
        cases = (("smith", "qa"), ("skein", "browser"), ("temper", "browser"), ("--verify", "smith", "qa"),
                 ("--verify-all",), ("--cpu", str(self.root / "plain CPU"), "smith", "qa"))
        for arguments in cases:
            with self.subTest(arguments=arguments):
                result, calls = self.invoke(*arguments)
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertFalse(any(call['name'] in ("wrap", "df") for call in calls))
                if "--verify" not in arguments and "--verify-all" not in arguments:
                    self.assertEqual(calls[-1]['name'], "cargo")
                    self.assertEqual(calls[-1]['args'][:2], ["nextest", "run"])
                if "--cpu" in arguments:
                    self.assertEqual([call['name'] for call in calls], ["cargo", "cc", "cargo"])

    def test_wrapper_prefixes_every_command(self):
        for index, prefix in enumerate((None, "", "wrap", "wrap --flag one")):
            if prefix is None:
                self.env.pop("TIERS_WRAPPER", None)
            else:
                self.env["TIERS_WRAPPER"] = prefix
            for arguments in (("skein", "browser"), ("temper", "browser"), ("--verify", "smith", "qa"),
                              ("--verify-all",),
                              ("--cpu", str(self.root / f"prefix CPU {index}"), "smith", "qa")):
                with self.subTest(prefix=prefix, arguments=arguments):
                    result, calls = self.invoke(*arguments)
                    self.assertEqual(result.returncode, 0, result.stderr)
                    commands = []
                    for call in calls:
                        if call['name'] in ("df", "skein-ui-bundle"):
                            continue
                        if call['name'] == "cargo" and call['args'][:1] == ["metadata"]:
                            self.assertEqual(call['args'], ["metadata", "--format-version", "1", "--no-deps"])
                            continue
                        if prefix:
                            self.assertEqual(call['name'], "wrap")
                            flags = prefix.split()[1:]
                            self.assertEqual(call['args'][:len(flags)], flags)
                            commands.append(call['args'][len(flags):])
                        else:
                            self.assertIn(call['name'], ("cargo", "cc", "bundle.sh"))
                            name = "temper/scripts/bundle.sh" if call['name'] == "bundle.sh" else call['name']
                            commands.append([name, *call['args']])
                    self.assertTrue(commands)
                    self.assertEqual(sum(call['name'] == "df" for call in calls), len(commands))
                    if "--cpu" in arguments:
                        self.assertEqual(commands[0][:1], ["cc"])
                        self.assertEqual(commands[-1][:4], ["cargo", "nextest", "run", "--config-file"])
                        self.assertEqual(commands[-1][5], "--tool-config-file")
                    if arguments == ("skein", "browser"):
                        self.assertEqual(sum(call['name'] == "skein-ui-bundle" for call in calls), 1)

    def test_disk_wait_on_and_off(self):
        events = []
        self.env["TIERS_DISK_BYTES"] = "42"
        available = iter((41, 1, 42))

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
            self.assertEqual(MODULE['step']("fixture", ["cargo", "check"], self.env), 0)
        self.assertEqual(events, [["df", "-B1", "--output=avail", "/"], ["sleep", 60],
                                  ["df", "-B1", "--output=avail", "/"], ["sleep", 60],
                                  ["df", "-B1", "--output=avail", "/"], ["wrap", "cargo", "check"]])
        self.assertEqual(output.getvalue().count("waiting for disk"), 2)
        self.assertIn("need 42;", output.getvalue())
        for minimum in (None, "", "0"):
            if minimum is None:
                self.env.pop("TIERS_DISK_BYTES", None)
            else:
                self.env["TIERS_DISK_BYTES"] = minimum
            for arguments in (("smith", "qa"), ("--verify", "smith", "qa")):
                result, calls = self.invoke(*arguments)
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertEqual(sum(call['name'] == "df" for call in calls),
                                 (2 if "--verify" in arguments else 1) if minimum == "0" else 0)
        for minimum in ("plenty", "-1", "1.5", " 1", "١"):
            for arguments, step in ((("smith", "qa"), "nextest"),
                                    (("--verify", "smith", "qa"), "list qa-smith workspace"),
                                    (("--verify-all",), "list browser-jig workspace"),
                                    (("--cpu", str(self.root / "bad disk CPU"), "smith", "qa"), "CPU wrapper")):
                with self.subTest(minimum=minimum, arguments=arguments):
                    result, calls = self.invoke(*arguments, TIERS_DISK_BYTES=minimum)
                    self.assertEqual(result.returncode, 2, result.stderr)
                    self.assertIn(f"failed step {step}", result.stderr)
                    self.assertIn("TIERS_DISK_BYTES", result.stderr)
                    self.assertEqual([call['args'] for call in calls],
                                     [["metadata", "--format-version", "1", "--no-deps"]]
                                     if "--cpu" in arguments else [])

    def test_disk_and_launch_errors_name_the_step(self):
        for error in (FileNotFoundError("missing command"),
                      subprocess.CalledProcessError(1, ["df"])):
            with mock.patch.object(subprocess, "run", side_effect=error):
                with self.assertRaisesRegex(MODULE['FailedStep'], "fixture"):
                    MODULE['step']("fixture", ["cargo", "check"], self.env)


if __name__ == "__main__":
    started = time.monotonic()
    suite = unittest.main(exit=False)
    print(f"Elapsed: {time.monotonic() - started:.3f}s", flush=True)
    sys.exit(not suite.result.wasSuccessful())
