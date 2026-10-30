"""Queue selection parity before recounts, using disposable Git and fake checkers."""

import importlib.machinery
import importlib.util
import json
from pathlib import Path
import unittest
from unittest.mock import patch

DIRECTORY = Path(__file__).resolve().parent
LOADER = importlib.machinery.SourceFileLoader("selection_fixtures", str(DIRECTORY / "test_precheck.py"))
SPEC = importlib.util.spec_from_loader(LOADER.name, LOADER)
fixtures = importlib.util.module_from_spec(SPEC)
LOADER.exec_module(fixtures)
mq = fixtures.mq


class SelectionTests(fixtures.Fixture):
    def tools_base(self):
        self.write("tools/test_existing.py", "# discovered offline suite\n")
        self.write("tools/guide.md", "tool guide\n")
        self.write("outside.md", "outside guide\n")
        self.ancestor = self.commit()
        for branch in ("base", "tip"):
            self.git("branch", "-f", branch, self.ancestor)

    def test_full_non_tools_inputs_skip_existing_suites_and_keep_compile_checks(self):
        self.tools_base()
        for path in ("Cargo.toml", ".config/nextest.toml", "agents/policy.md"):
            with self.subTest(path=path):
                self.change("tip", path, "full input\n")
                before = self.snapshot()
                self.calls.clear()
                with patch.object(mq, "tools_required", wraps=mq.tools_required) as required:
                    self.assertEqual(self.run_tool(), 0)
                required.assert_called_once_with(self.tool.record["selection_paths"])
                self.assertEqual(self.tool.record["selection"], "full")
                self.assertEqual([step["name"] for step in self.tool.record["plan"]],
                                 ["fmt", "clippy", "build"])
                self.assertEqual([argv for argv, _ in self.calls], [
                    ["cargo", "fmt", "--all", "--check"],
                    ["heavy", "cargo", "clippy", "--workspace", "--all-targets", "--", "-D", "warnings"],
                    ["heavy", "cargo", "nextest", "run", "--workspace", "--no-run"]])
                self.assert_restored(before)

    def check_tools_path_change(self, operation):
        self.tools_base()
        self.git("switch", "tip")
        if operation == "delete":
            self.git("rm", "tools/guide.md")
        elif operation == "rename-out":
            self.git("mv", "tools/guide.md", "moved.md")
        else:
            self.git("mv", "outside.md", "tools/moved.md")
        self.commit()
        self.git("switch", "original")
        before = self.snapshot()
        with patch.object(mq, "tools_required", wraps=mq.tools_required) as required:
            self.assertEqual(self.run_tool(), 0)
        required.assert_called_once_with(self.tool.record["selection_paths"])
        self.assertEqual(self.tool.record["selection"], "docs")
        self.assertEqual([step["name"] for step in self.tool.record["plan"]], ["tools/test_existing.py"])
        self.assertEqual([argv for argv, _ in self.calls], [["python3", "-I", "-B", "tools/test_existing.py"]])
        self.assertEqual(self.tool.record["selection_paths"],
                         {"delete": ["tools/guide.md"], "rename-out": ["moved.md", "tools/guide.md"],
                          "rename-in": ["outside.md", "tools/moved.md"]}[operation])
        self.assert_restored(before)

    def test_deleted_tools_doc_requires_suites(self):
        self.check_tools_path_change("delete")

    def test_renamed_tools_doc_source_requires_suites(self):
        self.check_tools_path_change("rename-out")

    def test_renamed_tools_doc_destination_requires_suites(self):
        self.check_tools_path_change("rename-in")

    def snapshot(self):
        return {"head": (self.repo / ".git/HEAD").read_bytes(),
                "refs": self.git("show-ref"), "index": (self.repo / ".git/index").read_bytes(),
                "source": {path: ((self.repo / path).read_bytes(), (self.repo / path).stat().st_mode)
                           for path in self.git("ls-files", "-z").split("\0") if path}}

    def assert_restored(self, before):
        self.assertEqual(self.snapshot(), before)
        self.restored()
        self.assertEqual(json.loads((self.tool.log_directory / "result.json").read_text()), self.tool.record)

    def lower(self, argv):
        if "--write" in argv:
            self.write("jig/scripts/old-forms.txt", "ceiling 7\n")

    def test_generated_baseline_keeps_package_scope_and_reverse_dependencies(self):
        source = "jig/app/src/lib.rs"
        baseline = "jig/scripts/old-forms.txt"
        self.change("tip", source, "candidate source\n")
        expected = mq.selection([source], fixtures.metadata(self.repo), always=["always"],
                                read_by_tests=fixtures.PATTERNS)
        self.assertEqual(expected, {"always", "jig-app", "temper-app"})
        self.effect = self.lower
        self.metadata_effect = lambda: self.assertEqual(
            (self.repo / baseline).read_text(), "ceiling 7\n")
        before = self.snapshot()
        self.assertEqual(self.run_tool(), 0)
        self.assertEqual(self.tool.record["selection"], sorted(expected))
        self.assertEqual(self.tool.record["selection_paths"], [source])
        self.assertEqual(self.tool.record["paths"], [source, baseline])
        self.assertEqual([step["name"] for step in self.tool.record["steps"]],
                         ["ratchet:jig", "fmt", "clippy", "build"])
        self.assertEqual(self.calls[1][0], ["cargo", "fmt", "--check", "-p", "jig-app"])
        scope = [arg for name in sorted(expected) for arg in ("-p", name)]
        self.assertEqual(self.calls[2][0], ["heavy", "cargo", "clippy", *scope,
                                           "--all-targets", "--", "-D", "warnings"])
        self.assertEqual(self.calls[3][0], ["heavy", "cargo", "nextest", "run", *scope, "--no-run"])
        self.assert_restored(before)

    def test_branch_supplied_baseline_still_selects_full(self):
        source = "jig/app/src/lib.rs"
        baseline = "jig/scripts/old-forms.txt"
        self.change("tip", source, "candidate source\n")
        self.change("tip", baseline, "ceiling 9\n")
        paths = [source, baseline]
        expected = mq.selection(paths, fixtures.metadata(self.repo), always=["always"],
                                read_by_tests=fixtures.PATTERNS)
        self.assertEqual(expected, "full")
        self.effect = self.lower
        before = self.snapshot()
        self.assertEqual(self.run_tool(), 0)
        self.assertEqual(self.tool.record["selection"], expected)
        self.assertEqual(self.tool.record["selection_paths"], paths)
        self.assertEqual(self.tool.record["paths"], paths)
        self.assertEqual([step["name"] for step in self.tool.record["steps"]],
                         ["ratchet:jig", "fmt", "clippy", "build"])
        self.assertTrue(all("--workspace" in argv for argv, _ in self.calls if argv[0] == "heavy"))
        self.assert_restored(before)

    def check_conflict(self, recounted):
        baseline = "jig/scripts/old-forms.txt"
        self.change("base", baseline, "ceiling 8\n")
        self.change("tip", baseline, "ceiling 9\n")
        observations = []

        def inspect(argv):
            if "--write" in argv:
                self.assertEqual((self.repo / baseline).read_text(), "ceiling 8\n")
                self.assertEqual(self.git("diff", "--name-only", "--diff-filter=U"), "")
                self.write(baseline, recounted)
            if argv[:3] == ["heavy", "cargo", "clippy"]:
                observations.append((self.repo / baseline).read_text())
                self.assertEqual(self.git("rev-parse", "HEAD"), self.tool.base)
                self.assertTrue((self.repo / ".git/MERGE_HEAD").exists())

        self.effect = inspect
        before = self.snapshot()
        self.assertEqual(self.run_tool(), 0)
        self.assertEqual(self.tool.record["selection"], "full")
        self.assertEqual(self.tool.record["selection_paths"], [baseline])
        self.assertEqual(self.tool.record["paths"], [] if recounted == "ceiling 8\n" else [baseline])
        self.assertEqual([step["name"] for step in self.tool.record["steps"]],
                         ["ratchet:jig", "fmt", "clippy", "build"])
        self.assertEqual(observations, [recounted])
        self.assert_restored(before)

    def test_baseline_conflict_recounts_before_full_lint(self):
        self.check_conflict("ceiling 7\n")

    def test_baseline_conflict_cannot_drop_selection_when_count_equals_base(self):
        self.check_conflict("ceiling 8\n")

    def test_docs_with_generated_baseline_need_no_metadata_tools_or_cargo(self):
        path = "jig/README.md"
        self.change("tip", path, "candidate docs\n")
        self.effect = self.lower
        self.metadata_effect = lambda: self.fail("docs recount must not load metadata")
        before = self.snapshot()
        self.assertEqual(self.run_tool(), 0)
        self.assertEqual(self.tool.record["selection"], mq.selection([path], {}, read_by_tests=fixtures.PATTERNS))
        self.assertEqual(self.tool.record["selection_paths"], [path])
        self.assertEqual(self.tool.record["paths"], [path, "jig/scripts/old-forms.txt"])
        self.assertEqual([step["name"] for step in self.tool.record["steps"]], ["ratchet:jig"])
        self.assert_restored(before)


if __name__ == "__main__":
    unittest.main()
