#!/usr/bin/env python3
"""Offline fixtures for macos.md, 4–5; no builds, network or credentials."""

import json
import os
from pathlib import Path
import runpy
import subprocess
import sys
import tempfile
import unittest
from unittest import mock
import xml.etree.ElementTree as ET


HELPER = Path(__file__).with_name("macos.py")
M = runpy.run_path(str(HELPER))
PACKAGES = '''process-temper = ["temper-box"]
browser-temper = ["temper-client-browser-world"]
browser-jig = ["jig-client-browser-world"]
browser-skein = ["skein-browser-world"]
live-temper = ["temper-box"]
live-skein = ["skein-llm-world"]
[test-binaries]
smith = [{ package = "smith-fake-claude", bin = "fake-claude" }]
'''


class MacosTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="macos-test-", dir=HELPER.parents[2])
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.packages = self.file("packages.toml", PACKAGES)
        self.table = M["tomllib"].loads(PACKAGES)
        self.destination = self.root / "out.xml"

    def file(self, name, text):
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text)
        return path

    def cli(self, *args, **env):
        return subprocess.run([sys.executable, "-I", "-B", str(HELPER), *map(str, args)],
                              env=dict(os.environ, **env), stdin=subprocess.DEVNULL,
                              text=True, capture_output=True, timeout=10)

    def place(self, test_set, text):
        return M["place"](test_set, text, self.table)

    def test_e2e_places_ids_and_only_the_browser_pair_holding_one(self):
        story = "temper-box::process.a_csv_change_passes_ci_and_a_blocking_agent_review_on_the_owned_forge_then_lands_and_survives_restart"
        browser = "skein-browser-world::browser_real.a_page_loads"
        parts, unheld = self.place("e2e", story + "\n" + browser)
        self.assertEqual(parts["process"], dict(run=True, ids=[story], pairs=["process-temper"]))
        self.assertEqual(parts["browser"], dict(run=True, ids=[browser], pairs=["browser-skein"]))
        self.assertEqual(unheld, [])

    def test_suites_tree_names_and_build_probe(self):
        parts, unheld = self.place("suites", "skein-lib.tests::one\nforgejo-conformance::suite.two\nbuild.temper")
        self.assertEqual(parts["skein"], dict(run=True, ids=["skein-lib.tests::one", "forgejo-conformance::suite.two"]))
        self.assertEqual(parts["temper"], dict(run=True, ids=["build.temper"]))
        self.assertFalse(parts["smith"]["run"])
        self.assertFalse(parts["jig"]["run"])
        self.assertEqual(unheld, [])
        for tree, package in (("smith", "smith"), ("smith", "smith-adapter"),
                              ("jig", "jig"), ("jig", "jig-world"),
                              ("temper", "temper"), ("skein", "github-conformance")):
            with self.subTest(package=package):
                self.assertTrue(self.place("suites", package + ".one")[0][tree]["run"])

    def test_all_holds_twice_but_never_live(self):
        parts, _ = self.place("all", "temper-box::process.x")
        self.assertEqual(parts["process"]["ids"], ["temper-box::process.x"])
        self.assertEqual(parts["temper"]["ids"], ["temper-box::process.x"])
        self.assertNotIn("live", parts)

    def test_plan_notices_unheld_and_writes_multiline_outputs(self):
        output = self.file("outputs", "")
        text = "hearth-builder-world::suite.x\nfabrig-boundary::outward.y\nbuild.live"
        result = self.cli("plan", "--packages", self.packages, SET="e2e", TESTS=text, GITHUB_OUTPUT=str(output))
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        for identifier in text.splitlines():
            self.assertIn("::notice", result.stdout)
            self.assertIn(identifier, result.stdout)
        values = output.read_text()
        self.assertRegex(values, r"process<<macos_[^\n]+\nfalse\n")
        self.assertRegex(values, r"parts<<macos_[^\n]+\n\[\]\n")

    def test_empty_or_blank_lines_run_every_part_whole(self):
        for text in ("", "\n \t \r\n"):
            parts, unheld = self.place("all", text)
            self.assertEqual(unheld, [])
            self.assertEqual(len(parts), 6)
            self.assertTrue(all(part["run"] and not part["ids"] for part in parts.values()))
            self.assertEqual(len(parts["browser"]["pairs"]), 3)
        parts, _ = self.place("live", "\n temper-box::live.one\r\n \n")
        self.assertEqual(parts["live"]["ids"], ["temper-box::live.one"])
        self.assertEqual(parts["live"]["pairs"], ["live-temper"])

    def test_filterset_splits_at_first_dot_and_omits_build(self):
        result = self.cli("filterset", IDS="build.temper\ntemper-box::bin/temper-box.fuzzy::a::b\nskein-lib.tests::x")
        self.assertEqual(result.returncode, 0, result.stdout)
        self.assertEqual(result.stdout.strip(), "binary_id(=temper-box::bin/temper-box) & test(=fuzzy::a::b) | binary_id(=skein-lib) & test(=tests::x)")
        self.assertEqual(M["filterset"](["pkg::suite.test.with.dot"]),
                         "binary_id(=pkg::suite) & test(=test.with.dot)")

    def test_compiler_red_keeps_whole_first_diagnostics_within_16k(self):
        first = "error[E0308]: wrong type\n  --> src/lib.rs:2:1\n  | expected bool\n\n"
        second = "error: another refusal\n  details\n\n"
        huge = "error: too large\n" + "x" * 17000 + "\n\n"
        log = self.file("build.log", "Compiling a\n" + first + second + huge + 'error: could not compile `a` (lib) due to 3 previous errors\n')
        result = self.cli("build-case", "temper", log, self.destination)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        root = ET.parse(self.destination).getroot()
        self.assertEqual(len(root.findall("testsuite")), 1)
        cases = list(root.iter("testcase"))
        self.assertEqual(len(cases), 1)
        self.assertEqual(cases[0].attrib, dict(classname="build", name="temper"))
        failure = cases[0].find("failure")
        self.assertEqual(failure.get("type"), "build failure")
        self.assertEqual(failure.text, first + second)
        self.assertLessEqual(len(failure.text.encode()), 16 * 1024)

    def test_build_without_compiler_refusal_is_a_fault(self):
        result = self.cli("build-case", "browser", self.file("build.log", "error: download refused\n"), self.destination)
        self.assertEqual(result.returncode, 2)
        self.assertIn("build fault", result.stdout)
        self.assertFalse(self.destination.exists())

    def test_merge_keeps_suites_cases_and_times_and_empty_part(self):
        first = self.file("pair.xml", '<testsuites><testsuite name="a" time="3"><testcase classname="a" name="one" time="1.234"><system-out>evidence</system-out></testcase></testsuite></testsuites>')
        second = self.file("suite.xml", '<testsuite name="b" time="2"><testcase classname="b" name="two" time="0.045"><failure type="test failure">no</failure></testcase></testsuite>')
        result = self.cli("merge", self.destination, first, second)
        self.assertEqual(result.returncode, 0, result.stdout)
        merged = ET.parse(self.destination).getroot()
        self.assertEqual([ET.tostring(s) for s in merged],
                         [ET.tostring(ET.parse(first).getroot()[0]), ET.tostring(ET.parse(second).getroot())])
        self.assertEqual(self.cli("merge", self.destination).returncode, 0)
        self.assertEqual(len(ET.parse(self.destination).getroot()), 0)

    def one_process(self, *, starved=False, unobserved=None, outcome="passed"):
        directory = self.root / "run"
        self.file("run/run.json", json.dumps(dict(finished=True, starved=starved)))
        self.file("run/results.json", json.dumps(dict(unobserved=unobserved or [], tests=[dict(outcome=outcome)])))
        self.file("run/junit.xml", '<testsuites><testsuite><testcase classname="a" name="one"/></testsuite></testsuites>')
        return directory

    def test_one_process_refuses_unobserved_or_starved(self):
        for kwargs in (dict(unobserved=[dict(binary_id="a", test_name="two")]),
                       dict(starved=True), dict(outcome="starved"), dict(outcome="cancelled")):
            with self.subTest(kwargs=kwargs):
                result = self.cli("check-one-process", self.one_process(**kwargs), 100)
                self.assertEqual(result.returncode, 2, result.stdout)
                self.assertIn("cut short", result.stdout)
        self.assertEqual(self.cli("check-one-process", self.one_process(), 0).returncode, 0)

    def test_run_statuses_accept_only_zero_and_100_with_junit(self):
        directory = self.one_process()
        for status in (0, 100):
            self.assertEqual(self.cli("check-one-process", directory, status).returncode, 0)
        for status in (2, 4, 101, 124):
            result = self.cli("check-one-process", directory, status)
            self.assertEqual(result.returncode, 2)
            self.assertIn(f"exit status {status}", result.stdout)
        (directory / "junit.xml").unlink()
        self.assertEqual(self.cli("check-one-process", directory, 0).returncode, 2)

    def listing(self):
        return self.file("listing.json", json.dumps({"rust-suites": {
            "a::suite": {"status": "listed", "testcases": {
                "one": {"ignored": False, "filter-match": {"status": "matches"}},
                "two": {"ignored": False, "filter-match": {"status": "matches"}},
                "ignored": {"ignored": True, "filter-match": {"status": "matches"}},
                "outside": {"ignored": False, "filter-match": {"status": "mismatch", "reason": "default-filter"}},
            }}
        }}))

    def test_nextest_listing_refuses_selected_but_unobserved_cases(self):
        listing = self.listing()
        junit = self.file("nextest.xml", '<testsuites><testsuite><testcase classname="a::suite" name="one"/></testsuite></testsuites>')
        result = self.cli("check-nextest", listing, junit, 100)
        self.assertEqual(result.returncode, 2)
        self.assertIn("a::suite.two", result.stdout)
        junit.write_text('<testsuites><testsuite><testcase classname="a::suite" name="one"/><testcase classname="a::suite" name="two"/></testsuite></testsuites>')
        self.assertEqual(self.cli("check-nextest", listing, junit, 100).returncode, 0)
        junit.write_text('<testsuites><testsuite><testcase classname="a::suite" name="ignored"/></testsuite></testsuites>')
        self.assertEqual(self.cli("check-nextest", listing, junit, 0, "--run-ignored", "only").returncode, 0)
        junit.write_text('<testsuites><testsuite><testcase classname="a::suite" name="ignored"><skipped/></testcase></testsuite></testsuites>')
        self.assertEqual(self.cli("check-nextest", listing, junit, 0, "--run-ignored", "only").returncode, 2)

    def test_report_copies_only_failures_and_errors_by_part(self):
        self.file("process.xml", '<testsuites><testsuite><testcase classname="p" name="pass" time="1"/><testcase classname="p" name="red" time="2"><failure type="test failure">details</failure><system-out>output</system-out></testcase></testsuite></testsuites>')
        self.file("browser.xml", '<testsuite><testcase classname="b" name="fault" time="3"><error type="test abort">signal</error></testcase></testsuite>')
        result = self.cli("report", self.root, self.destination, PARTS='["process", "browser"]')
        self.assertEqual(result.returncode, 0, result.stdout)
        root = ET.parse(self.destination).getroot()
        self.assertEqual([s.get("name") for s in root], ["process", "browser"])
        self.assertEqual([c.get("name") for c in root.iter("testcase")], ["red", "fault"])
        for part, index in (("process", 1), ("browser", 0)):
            original = list(ET.parse(self.root / f"{part}.xml").getroot().iter("testcase"))[index]
            copied = root.find(f"testsuite[@name='{part}']/testcase")
            self.assertEqual(ET.tostring(copied), ET.tostring(original))
        self.file("process.xml", '<testsuite><testcase classname="p" name="pass"/></testsuite>')
        self.assertEqual(self.cli("report", self.root, self.destination, PARTS='["process"]').returncode, 0)
        self.assertEqual(list(ET.parse(self.destination).getroot().iter("testcase")), [])
        nested = self.file("downloads/part-process/process.xml", '<testsuites/>')
        report = self.root / "report/junit.xml"
        self.assertEqual(self.cli("report", nested.parents[1], report, PARTS='["process"]').returncode, 0)
        self.assertEqual((report.parent / "process.xml").read_text(), nested.read_text())

    def test_report_missing_part_names_it_and_writes_nothing(self):
        self.file("process.xml", '<testsuites/>')
        self.destination.write_text("stale report")
        result = self.cli("report", self.root, self.destination, PARTS='["process", "temper"]')
        self.assertEqual(result.returncode, 2)
        self.assertIn("::error title=missing part::temper", result.stdout)
        self.assertFalse(self.destination.exists())

    def test_builds_whole_part_and_tree_binary_prerequisites(self):
        commands = M["build_commands"]("browser", self.table, None)
        self.assertEqual(commands, [["tools/tiers/run", tree, "browser", "--", "--no-run"]
                                    for tree in ("temper", "jig", "skein")])
        meta = dict(packages=[dict(name="smith"), dict(name="smith-world"), dict(name="hearth")])
        self.assertEqual(M["build_commands"]("smith", self.table, meta), [
            ["cargo", "build", "-p", "smith-fake-claude", "--bin", "fake-claude"],
            ["cargo", "nextest", "run", "--no-run", "--profile", "default", "-p", "smith", "-p", "smith-world"]])

    def test_run_selection_skips_build_only_and_intersects_live_story(self):
        with mock.patch.dict(os.environ, GIVEN="true"):
            self.assertEqual(M["run_arguments"]("process", ["build.process"], self.table), [])
            selected = M["run_arguments"]("browser", ["skein-browser-world::browser.a"], self.table)
            self.assertEqual(len(selected), 1)
            self.assertEqual(selected[0][0], "browser-skein")
            self.assertEqual(selected[0][1][-3:], ["-E", "binary_id(=skein-browser-world::browser) & test(=a)", "--no-tests=warn"])
            live = M["run_arguments"]("live", ["temper-box::live.a"], self.table)
            self.assertEqual(len(live), 1)
            self.assertIn("--ignore-default-filter", live[0][1])
            self.assertIn("--run-ignored", live[0][1])
            self.assertIn(") & (binary_id(=temper-box::live) & test(=a))", live[0][1][-2])

    def test_prune_removes_workspace_outputs_not_dependencies(self):
        target = self.root / "target"
        own = ["debug/deps/libsmith-123.rlib", "debug/deps/suite-123", "debug/.fingerprint/smith-123/x",
               "debug/build/smith-123/x", "debug/smith", "debug/smith.dSYM/x", "nextest/default/junit.xml",
               "temper-bundle/index.html", "wasm32-unknown-unknown/debug/deps/libsmith-123.rlib",
               "wasm32-unknown-unknown/web/.fingerprint/smith-123/x",
               "wasm32-unknown-unknown/web/smith.wasm",
               "wasm32-unknown-unknown/playground/deps/libsmith-123.rlib"]
        kept = self.file("target/debug/deps/libserde-123.rlib", "dependency")
        for name in own:
            self.file("target/" + name, "own")
        meta = dict(packages=[dict(name="smith", targets=[dict(name="smith", kind=["bin"]), dict(name="suite", kind=["test"])])])
        M["prune"](target, meta)
        self.assertTrue(kept.exists())
        self.assertFalse(any((target / name).exists() for name in own))

    def test_build_stops_on_compiler_red_or_fault_and_build_probe_skips_runs(self):
        calls = []
        outputs = {}
        log = self.file("logs/unused.log", "")

        def refusal(name, command):
            calls.append(command)
            log.with_name(name + ".log").write_text('error: could not compile `a` (lib)\n')
            return 2

        globals_ = M["build"].__globals__
        with mock.patch.dict(os.environ, RUNNER_TEMP=str(self.root), IDS="build.process"):
            with mock.patch.dict(globals_, package_map=lambda: self.table, checked=refusal,
                                 output=lambda name, value: outputs.update({name: value})):
                M["build"]("process", self.destination)
            self.assertEqual(len(calls), 1)
            self.assertEqual(outputs, dict(build_red="true", run_tests="false"))
            self.assertIsNotNone(ET.parse(self.destination).find(".//failure"))
            with mock.patch.dict(globals_, package_map=lambda: self.table,
                                 checked=lambda name, command: 0,
                                 output=lambda name, value: outputs.update({name: value})):
                M["build"]("process", self.destination)
            self.assertFalse(self.destination.exists())
            self.assertEqual(outputs, dict(build_red="false", run_tests="false"))

            def fault(name, command):
                log.with_name(name + ".log").write_text("download refused\n")
                return 2

            with mock.patch.dict(globals_, package_map=lambda: self.table, checked=fault):
                with self.assertRaisesRegex(ValueError, "build fault"):
                    M["build"]("process", self.destination)
            self.assertFalse(self.destination.exists())

    def test_later_run_fault_never_publishes_an_earlier_runs_cases(self):
        listing = self.file("logs/unused.log", json.dumps({"rust-suites": {
            "a": {"status": "listed", "testcases": {
                "one": {"ignored": False, "filter-match": {"status": "matches"}}
            }}
        }}))
        calls = []

        def command(name, argv):
            calls.append((name, argv))
            profile = name.removeprefix("list-").removeprefix("run-")
            if name.startswith("list-"):
                listing.with_name(name + ".log").write_text(listing.read_text())
                return 0
            self.file(f"target/nextest/{profile}/junit.xml", '<testsuites><testsuite><testcase classname="a" name="one"><failure>red</failure></testcase></testsuite></testsuites>')
            return 100 if profile == "browser-jig" else 2

        globals_ = M["run_part"].__globals__
        previous = Path.cwd()
        self.addCleanup(os.chdir, previous)
        os.chdir(self.root)
        selections = [(profile, ["-p", self.table[profile][0]], "default")
                      for profile in ("browser-jig", "browser-skein")]
        with mock.patch.dict(os.environ, RUNNER_TEMP=str(self.root), IDS=""):
            with mock.patch.dict(globals_, package_map=lambda: self.table, checked=command,
                                 run_arguments=lambda *args: selections):
                with self.assertRaisesRegex(ValueError, "exit status 2"):
                    M["run_part"]("browser", self.destination)
        self.assertFalse(self.destination.exists())
        self.assertEqual([name for name, _ in calls],
                         ["list-browser-jig", "run-browser-jig", "list-browser-skein", "run-browser-skein"])

    def test_install_checks_the_workflows_digest_before_unpacking(self):
        calls = []
        globals_ = M["install"].__globals__
        with mock.patch.dict(os.environ, RUNNER_TEMP=str(self.root), NEXTEST_VERSION="0.9.148",
                             NEXTEST_SHA256="a" * 64):
            with mock.patch.dict(globals_, require=lambda name, command: calls.append((name, command))):
                with mock.patch.object(Path, "home", return_value=self.root):
                    M["install"]()
        names = [name for name, _ in calls]
        self.assertLess(names.index("nextest-digest"), names.index("nextest-install"))
        self.assertIn("cargo-nextest-0.9.148-universal-apple-darwin.tar.gz", str(calls[names.index("nextest-download")][1]))
        self.assertEqual((self.root / "nextest.sha256").read_text(), "a" * 64 + f"  {self.root}/nextest.tar.gz\n")
        self.assertEqual(calls[names.index("nextest-digest")][1],
                         ["shasum", "-a", "256", "-c", self.root / "nextest.sha256"])

    def test_runner_preserves_status_and_annotates_log_without_shell_splicing(self):
        envfile = self.file("env", "")
        result = self.cli("setup-runner", RUNNER_TEMP=str(self.root), GITHUB_ENV=str(envfile))
        self.assertEqual(result.returncode, 0, result.stdout)
        env = dict(os.environ, RUNNER_TEMP=str(self.root), MACOS_HELPER=str(HELPER))
        result = subprocess.run([str(self.root / "run"), "refusal", sys.executable, "-c",
                                 "import sys; print('error: refused % value'); sys.exit(4)"],
                                env=env, stdin=subprocess.DEVNULL, text=True, capture_output=True, timeout=10)
        self.assertEqual(result.returncode, 4)
        self.assertIn("::error", result.stdout)
        self.assertIn("refused %25 value", result.stdout)
        self.assertEqual((self.root / "logs/refusal.log").read_text(), "error: refused % value\n")


if __name__ == "__main__":
    unittest.main()
