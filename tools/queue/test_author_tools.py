"""Author ownership selection through real merged Git candidates (design.md, 2.3)."""

import importlib.machinery
import importlib.util
from pathlib import Path
import unittest
from unittest.mock import patch

DIRECTORY = Path(__file__).resolve().parent
LOADER = importlib.machinery.SourceFileLoader("author_selection_fixtures", str(DIRECTORY / "test_precheck.py"))
SPEC = importlib.util.spec_from_loader(LOADER.name, LOADER)
fixtures = importlib.util.module_from_spec(SPEC)
LOADER.exec_module(fixtures)
mq = fixtures.mq


class AuthorToolsTests(fixtures.Fixture):
    def setUp(self):
        super().setUp()
        for owner in ("tiers", "codex-run", "queue", "heavy", "literal[1]"):
            self.write(f"tools/{owner}/test_own.py", "raise SystemExit(0)\n")
            self.write(f"tools/{owner}/source.md", "base\n")
        self.write("tools/test_install.py", "raise SystemExit(0)\n")
        self.write("tools/install", "base\n")
        self.write("tools/installed", "base\n")
        self.write("outside.md", "base\n")
        self.ancestor = self.commit()
        for branch in ("base", "tip"):
            self.git("branch", "-f", branch, self.ancestor)

    def check_plan(self, expected, *, production_runner=False):
        self.assertEqual(self.run_tool(production_runner=production_runner), 0, self.tool.record)
        names = [step["name"] for step in self.tool.record["plan"]]
        self.assertEqual(names, expected)
        self.assertTrue(all(step["timeout"] == 60 for step in self.tool.record["plan"]))
        self.restored()

    def test_tiers_only_runs_own_real_suite_and_excludes_failing_launcher(self):
        self.change("base", "tools/codex-run/test_own.py", "raise SystemExit('unrelated launcher')\n")
        self.change("tip", "tools/tiers/source.md", "candidate\n")
        self.check_plan(["tools/tiers/test_own.py"], production_runner=True)
        self.assertNotIn("unrelated launcher", (self.tool.log_directory / "commands.log").read_text())

    def test_multiple_tools_take_deduplicated_union(self):
        for path in ("tools/tiers/source.md", "tools/queue/source.md", "tools/tiers/added.md"):
            self.change("tip", path, "candidate\n")
        self.check_plan(["tools/queue/test_own.py", "tools/tiers/test_own.py"])

    def test_codex_change_includes_launcher(self):
        self.change("tip", "tools/codex-run/source.md", "candidate\n")
        self.check_plan(["tools/codex-run/test_own.py"])

    def test_queue_change_selects_queue_only(self):
        self.change("tip", "tools/queue/source.md", "candidate\n")
        self.check_plan(["tools/queue/test_own.py"])

    def test_deleted_source_keeps_owner(self):
        self.git("switch", "tip")
        self.git("rm", "tools/tiers/source.md")
        self.commit()
        self.git("switch", "original")
        self.check_plan(["tools/tiers/test_own.py"])

    def rename(self, source, destination):
        self.git("switch", "tip")
        self.git("mv", source, destination)
        self.commit()
        self.git("switch", "original")

    def test_rename_between_tools_checks_both_owners(self):
        self.rename("tools/tiers/source.md", "tools/queue/moved.md")
        self.check_plan(["tools/queue/test_own.py", "tools/tiers/test_own.py"])

    def test_rename_out_keeps_old_owner(self):
        self.rename("tools/tiers/source.md", "moved.md")
        self.check_plan(["tools/tiers/test_own.py"])

    def test_rename_in_selects_new_owner(self):
        self.rename("outside.md", "tools/tiers/moved.md")
        self.check_plan(["tools/tiers/test_own.py"])

    def test_installer_and_manifest_select_only_installer_suite(self):
        for path in ("tools/install", "tools/installed", "tools/test_install.py"):
            self.change("tip", path, "# candidate\n")
        self.check_plan(["tools/test_install.py"])

    def test_installer_and_tool_union(self):
        self.change("tip", "tools/install", "candidate\n")
        self.change("tip", "tools/tiers/source.md", "candidate\n")
        self.check_plan(["tools/test_install.py", "tools/tiers/test_own.py"])

    def refusal(self, diagnostic):
        self.assertEqual(self.run_tool(), 1)
        self.assertEqual(self.tool.record["outcome"], "infrastructure-error")
        self.assertIn(diagnostic, self.tool.record["error"])
        self.assertEqual(self.calls, [])
        self.restored()

    def test_unknown_top_level_owner_refuses(self):
        self.change("tip", "tools/shared.py", "candidate\n")
        self.refusal("unknown owner for 'tools/shared.py'")

    def test_new_tool_without_suite_refuses(self):
        self.change("tip", "tools/new-tool/source.md", "candidate\n")
        self.refusal("no candidate offline suites for 'tools/new-tool/'")

    def test_deleted_last_suite_refuses_instead_of_silent_success(self):
        self.git("switch", "tip")
        self.git("rm", "tools/tiers/test_own.py")
        self.commit()
        self.git("switch", "original")
        self.refusal("no candidate offline suites for 'tools/tiers/'")

    def test_new_tool_with_suite_is_selected(self):
        self.change("tip", "tools/new-tool/test_new.py", "# suite\n")
        self.check_plan(["tools/new-tool/test_new.py"])

    def test_owner_pathspec_is_literal_and_never_discovers_all_tools(self):
        self.change("tip", "tools/literal[1]/source.md", "candidate\n")
        git = mq.git
        scopes = []

        def inspect(repo, *args, **options):
            if args[:1] == ("ls-files",) and "--stage" in args:
                scopes.append(args[args.index("--") + 1:])
            return git(repo, *args, **options)

        with patch.object(mq, "git", side_effect=inspect):
            self.check_plan(["tools/literal[1]/test_own.py"])
        self.assertEqual(scopes, [(":(literal)tools/literal[1]/",)])

    def test_manual_heavy_recovery_stays_excluded(self):
        self.change("tip", "tools/heavy/test-heavy-recovery", "#!/bin/sh\nexit 1\n")
        self.git("switch", "tip")
        (self.repo / "tools/heavy/test-heavy-recovery").chmod(0o755)
        self.commit()
        self.git("switch", "original")
        self.check_plan(["tools/heavy/test_own.py"])


if __name__ == "__main__":
    unittest.main()
