"""Before-ready freshness against disposable bare origins (precheck.md, 5)."""

import sys
sys.dont_write_bytecode = True

from copy import deepcopy
import importlib.machinery
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

DIRECTORY = Path(__file__).resolve().parent
LOADER = importlib.machinery.SourceFileLoader("ready_check", str(DIRECTORY / "ready-check"))
SPEC = importlib.util.spec_from_loader(LOADER.name, LOADER)
ready = importlib.util.module_from_spec(SPEC)
LOADER.exec_module(ready)


class ReadyCheckTests(unittest.TestCase):
    def setUp(self):
        # Origin, writer, candidate, and evidence are siblings outside the checked source.
        self.temp = tempfile.TemporaryDirectory(prefix=".ready-check-test-", dir=DIRECTORY)
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.origin = self.root / "origin.git"
        self.writer = self.root / "writer"
        self.repo = self.root / "candidate ; $(touch BAD) space"
        self.result = self.root / "result.json"
        self.env = dict(os.environ, GIT_CONFIG_GLOBAL="/dev/null", GIT_CONFIG_NOSYSTEM="1",
                        GIT_TERMINAL_PROMPT="0")
        self.environment = patch.dict(os.environ, self.env)
        self.environment.start()
        self.addCleanup(self.environment.stop)
        self.git(self.root, "init", "--bare", "--initial-branch=main", str(self.origin))
        self.git(self.root, "clone", str(self.origin), str(self.writer))
        self.git(self.writer, "config", "user.name", "Fixture")
        self.git(self.writer, "config", "user.email", "fixture@example.invalid")
        self.write(self.writer, "touched.md", "base\n")
        self.write(self.writer, "other.md", "base\n")
        self.commit(self.writer)
        self.base = self.git(self.writer, "rev-parse", "HEAD")
        self.git(self.writer, "push", "origin", "HEAD:refs/heads/main")
        self.git(self.root, "clone", str(self.origin), str(self.repo))
        self.git(self.repo, "config", "user.name", "Fixture")
        self.git(self.repo, "config", "user.email", "fixture@example.invalid")
        self.git(self.repo, "switch", "-c", "lead/task")
        self.write(self.repo, "touched.md", "candidate\n")
        self.commit(self.repo)
        self.tip = self.git(self.repo, "rev-parse", "HEAD")
        self.record = {"outcome": "passed", "restored": True, "base": self.base, "tip": self.tip,
                       "original": self.tip, "slot": str(self.repo), "started": "2026-01-01T00:00:00+00:00",
                       "heavy_observation": "fixture idle", "paths": ["touched.md"], "selection": "docs",
                       "plan": [], "steps": []}
        self.save()

    def git(self, repo, *args):
        return subprocess.check_output(["git", "--no-optional-locks", "-C", str(repo), *args],
                                       stdin=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
                                       env=self.env, text=True).strip()

    def write(self, repo, name, contents):
        filename = repo / name
        filename.parent.mkdir(parents=True, exist_ok=True)
        filename.write_text(contents)

    def commit(self, repo):
        self.git(repo, "add", "--all")
        self.git(repo, "commit", "-m", "Fixture change")

    def save(self):
        self.result.write_text(json.dumps(self.record))

    def advance(self, name="other.md", contents="advance\n"):
        self.write(self.writer, name, contents)
        self.commit(self.writer)
        self.git(self.writer, "push", "origin", "HEAD:refs/heads/main")
        return self.git(self.writer, "rev-parse", "HEAD")

    def check(self):
        return ready.check(self.result, self.repo, self.tip)

    def refuse(self, expected):
        result = self.check()
        self.assertEqual(result["verdict"], "rerun-precheck", result)
        self.assertIn(expected, result["diagnostic"])
        return result

    def plan(self):
        command = ["python3", "-I", "-B", "tools/test_fixture.py"]
        self.record.update(plan=[{"name": "tools/test_fixture.py", "argv": command,
                                  "cwd": str(self.repo), "timeout": 60}],
                           steps=[{"name": "tools/test_fixture.py", "command": command,
                                   "returncode": 0, "green": True, "duration": 0.1}],
                           active_command={"argv": command, "cwd": str(self.repo),
                                           "completion": "completed", "returncode": 0})
        self.save()

    def test_unchanged_main_accepts_exact_tip(self):
        self.assertEqual(self.check(), {"verdict": "passed", "tip": self.tip, "checkedbase": self.base,
                                       "currentmain": self.base, "finalmain": self.base,
                                       "touchedscope": ["touched.md"], "overlap": [],
                                       "diagnostic": "reported touched files unchanged at observed origin/main; point-in-time only"})

    def test_remote_same_file_change_refuses_stale_tracking(self):
        current = self.advance("touched.md")
        result = self.refuse("touched.md")
        self.assertEqual((result["checkedbase"], result["currentmain"], result["overlap"]),
                         (self.base, current, ["touched.md"]))
        self.assertEqual(self.git(self.repo, "rev-parse", "origin/main"), self.base)

    def test_remote_disjoint_change_passes_without_tracking_update(self):
        current = self.advance()
        self.assertEqual((self.check()["verdict"], self.check()["currentmain"]), ("passed", current))
        self.assertEqual(self.git(self.repo, "rev-parse", "origin/main"), self.base)

    def test_remote_advance_during_check_refuses_without_sleep(self):
        original = ready.Repository.changed

        def advance_after_diff(repository, base, current):
            paths = original(repository, base, current)
            self.advance()
            return paths

        with patch.object(ready.Repository, "changed", advance_after_diff):
            result = self.refuse("advanced during check")
        self.assertNotEqual(result["currentmain"], result["finalmain"])

    def test_remote_advance_during_fetch_refuses_without_sleep(self):
        original = ready.Repository.git

        def advance_before_fetch(repository, *args, **kwargs):
            if args[0] == "fetch":
                self.advance()
            return original(repository, *args, **kwargs)

        with patch.object(ready.Repository, "git", advance_before_fetch):
            self.refuse("advanced during check")

    def test_renamed_deleted_and_literal_paths_overlap(self):
        names = ["space name.md", "$(touch BAD);*.md", ":(glob)*.md", "line\nbreak.md", "-option.md"]
        for name in names:
            self.write(self.writer, name, "base\n")
        self.commit(self.writer)
        self.git(self.writer, "push", "origin", "HEAD:main")
        self.repo = self.root / "literal candidate"
        self.git(self.root, "clone", str(self.origin), str(self.repo))
        self.git(self.repo, "config", "user.name", "Fixture")
        self.git(self.repo, "config", "user.email", "fixture@example.invalid")
        for name in names:
            self.write(self.repo, name, "candidate\n")
        self.commit(self.repo)
        self.tip = self.git(self.repo, "rev-parse", "HEAD")
        self.record.update(tip=self.tip, original=self.tip, slot=str(self.repo))
        self.record["base"] = self.git(self.writer, "rev-parse", "HEAD")
        self.record["paths"] = names
        self.save()
        self.git(self.writer, "mv", "--", names[0], "renamed.md")
        self.git(self.writer, "rm", "--", names[1])
        for name in names[2:]:
            self.write(self.writer, name, "changed\n")
        self.commit(self.writer)
        self.git(self.writer, "push", "origin", "HEAD:main")
        result = self.refuse("changed precheck files")
        self.assertEqual(result["overlap"], sorted(names))
        self.assertFalse((self.repo / "BAD").exists())
        self.record["paths"] = ["renamed.md"]
        self.save()
        self.assertEqual(self.refuse("renamed.md")["overlap"], ["renamed.md"])

    def test_reverted_main_change_still_requires_precheck(self):
        self.advance("touched.md")
        self.advance("touched.md", "base\n")
        self.refuse("touched.md")

    def test_merge_resolution_changes_are_seen(self):
        self.git(self.writer, "switch", "-c", "side")
        self.write(self.writer, "other.md", "side\n")
        self.commit(self.writer)
        self.git(self.writer, "switch", "main")
        self.write(self.writer, "extra.md", "main\n")
        self.commit(self.writer)
        self.git(self.writer, "merge", "--no-ff", "--no-commit", "side")
        self.write(self.writer, "touched.md", "merge resolution\n")
        self.commit(self.writer)
        self.git(self.writer, "push", "origin", "HEAD:main")
        self.refuse("touched.md")

    def test_rewritten_or_removed_main_refuses(self):
        self.git(self.writer, "switch", "--orphan", "replacement")
        self.write(self.writer, "unrelated.md", "unrelated\n")
        self.commit(self.writer)
        self.git(self.writer, "push", "--force", "origin", "HEAD:main")
        self.refuse("rewrote history")
        self.git(self.origin, "update-ref", "-d", "refs/heads/main")
        self.refuse("ls-remote failed")

    def test_remote_unavailable_refuses(self):
        self.git(self.repo, "remote", "set-url", "origin", str(self.root / "missing.git"))
        self.refuse("ls-remote failed")

    def test_changed_tip_and_missing_objects_refuse(self):
        self.write(self.repo, "touched.md", "new tip\n")
        self.commit(self.repo)
        tip = self.git(self.repo, "rev-parse", "HEAD")
        result = ready.check(self.result, self.repo, tip)
        self.assertIn("tip mismatch", result["diagnostic"])
        self.tip = "1" * 40
        self.record["tip"] = self.tip
        self.save()
        self.refuse("rev-parse failed")

    def test_missing_base_object_refuses(self):
        self.record["base"] = "2" * 40
        self.save()
        self.refuse("rev-parse failed")

    def test_tip_must_be_full_literal_oid(self):
        for value in ("HEAD", self.tip[:12], "--help", "$(touch BAD)", self.tip.upper(), "0" * 39):
            with self.subTest(value=value):
                self.assertIn("full lowercase", ready.check(self.result, self.repo, value)["diagnostic"])

    def test_failed_unrestored_unknown_or_incomplete_evidence_refuses(self):
        self.plan()
        valid = deepcopy(self.record)
        patches = [({"outcome": "failed"}, "passed"), ({"restored": False}, "restoration"),
                   ({"restored": 1}, "restoration"), ({"plan": [None]}, "planned"),
                   ({"steps": []}, "incomplete"), ({"active_command": None}, "active-command"),
                   ({"active_command": {"completion": "unknown"}}, "active-command"),
                   ({"selection": None}, "selection"), ({"paths": None}, "paths"),
                   ({"error": "failed"}, "unresolved"), ({"base": self.base[:12]}, "full precheck"),
                   ({"started": "tomorrow"}, "time"), ({"heavy_observation": ""}, "heavy"),
                   ({"slot": "/a/../b"}, "slot")]
        for changes, diagnostic in patches:
            with self.subTest(changes=changes):
                self.record = dict(deepcopy(valid), **changes)
                self.save()
                self.refuse(diagnostic)

    def test_step_plan_and_completion_contradictions_refuse(self):
        self.plan()
        valid = deepcopy(self.record)
        mutations = [("steps", "returncode", 137), ("steps", "green", False),
                     ("steps", "duration", -1), ("steps", "command", ["other"]),
                     ("steps", "cwd", "/wrong"), ("plan", "timeout", -1),
                     ("plan", "cwd", "/wrong")]
        for section, key, value in mutations:
            with self.subTest(section=section, key=key):
                self.record = deepcopy(valid)
                self.record[section][0][key] = value
                self.save()
                self.assertEqual(self.check()["verdict"], "rerun-precheck")
        self.record = deepcopy(valid)
        self.record["active_command"]["returncode"] = 4
        self.save()
        self.refuse("active-command")

    def test_dangerous_and_duplicate_scopes_refuse(self):
        for paths in (["../escape"], ["/absolute"], ["a/./b"], ["a//b"], [".git/config"],
                      ["a/.GiT/b"], ["nul\0name"], [""], ["same", "same"], [42]):
            with self.subTest(paths=paths):
                self.record["paths"] = paths
                self.save()
                self.assertEqual(self.check()["verdict"], "rerun-precheck")

    def test_malformed_duplicate_or_missing_json_refuses(self):
        for data in ("{", "[]", "null", '{"outcome":"passed","outcome":"passed"}',
                     '{"restored":NaN}', "{}"):
            with self.subTest(data=data):
                self.result.write_text(data)
                self.assertEqual(self.check()["verdict"], "rerun-precheck")
        self.result.unlink()
        self.refuse("No such file")

    def test_completed_ratchet_only_evidence_passes(self):
        argv = ["python3", "-I", "-B", f"{self.repo}/skein/scripts/old_forms.py", "--write"]
        self.record.update(paths=["skein/docs/example.md"],
                           steps=[{"name": "ratchet:skein", "command": argv, "returncode": 0,
                                   "green": True, "duration": 0}],
                           active_command={"argv": argv, "cwd": f"{self.repo}/skein",
                                           "completion": "completed", "returncode": 0})
        self.save()
        self.assertEqual(self.check()["verdict"], "passed")
        self.record["steps"] = []
        self.save()
        self.refuse("ratchet")

    def test_no_command_run_with_active_command_refuses(self):
        self.record["active_command"] = {"completion": "unknown"}
        self.save()
        self.refuse("active command")

    def test_incomplete_compile_plan_and_unexpected_commands_refuse(self):
        self.record["selection"] = ["fixture"]
        self.save()
        self.refuse("compile/lint plan")
        self.record["selection"] = "docs"
        self.plan()
        self.record["plan"][0]["name"] = "unknown"
        self.save()
        self.refuse("unknown planned command")
        self.plan()
        self.record["steps"].insert(0, {"name": "extra", "command": ["true"],
                                        "returncode": 0, "green": True, "duration": 0})
        self.save()
        self.refuse("outside precheck plan")

    def compile_plan(self, selection):
        self.plan()
        self.record["selection"] = selection
        for name, argv, code in (("clippy", ["heavy", "cargo", "clippy", "-p", "fixture"], 0),
                                 ("build", ["heavy", "cargo", "nextest", "run", "--no-run"], 4)):
            self.record["plan"].append({"name": name, "argv": argv, "cwd": str(self.repo), "timeout": None})
            self.record["steps"].append({"name": name, "command": argv, "green": True,
                                         "returncode": code, "duration": 0.01})
            self.record["active_command"] = {"argv": argv, "cwd": str(self.repo),
                                             "completion": "completed", "returncode": code}
        self.save()

    def test_completed_planned_tools_and_build_exit_four_pass(self):
        self.plan()
        self.assertEqual(self.check()["verdict"], "passed")
        self.compile_plan(["fixture"])
        self.assertEqual(self.check()["verdict"], "passed")

    def test_full_selection_accepts_workspace_format_command(self):
        self.compile_plan("full")
        argv = ["cargo", "fmt", "--all", "--check"]
        self.record["plan"].insert(0, {"name": "fmt", "argv": argv, "cwd": str(self.repo), "timeout": None})
        self.record["steps"].insert(0, {"name": "fmt", "command": argv, "green": True,
                                        "returncode": 0, "duration": 0.01})
        self.save()
        self.assertEqual(self.check()["verdict"], "passed")

    def snapshot(self, repo):
        files = {}
        for filename in repo.rglob("*"):
            if filename.is_file() and ".git" not in filename.relative_to(repo).parts:
                files[str(filename.relative_to(repo))] = (filename.read_bytes(), filename.stat().st_mode)
        return (self.git(repo, "symbolic-ref", "HEAD"), self.git(repo, "rev-parse", "HEAD"),
                self.git(repo, "for-each-ref", "--format=%(refname) %(objectname)"),
                (repo / ".git/index").read_bytes(), files, (repo / ".git/HEAD").read_bytes())

    def test_cli_preserves_source_index_refs_and_origin(self):
        self.plan()
        self.advance()
        self.write(self.repo, "staged.md", "staged\n")
        self.git(self.repo, "add", "staged.md")
        self.write(self.repo, "touched.md", "dirty source\n")
        self.write(self.repo, "untracked ; *.md", "untracked\n")
        self.git(self.repo, "update-index", "--assume-unchanged", "other.md")
        before = self.snapshot(self.repo)
        bytecode = set((DIRECTORY / "__pycache__").glob("*"))
        remote = self.git(self.origin, "for-each-ref", "--format=%(refname) %(objectname)")
        result = subprocess.run([sys.executable, "-I", "-B", str(DIRECTORY / "ready-check"),
                                 str(self.result), "--repo", str(self.repo), "--tip", self.tip],
                                text=True, capture_output=True, stdin=subprocess.DEVNULL, env=self.env)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(json.loads(result.stdout)["verdict"], "passed")
        self.assertEqual(self.snapshot(self.repo), before)
        self.assertEqual(self.git(self.origin, "for-each-ref", "--format=%(refname) %(objectname)"), remote)
        self.assertFalse((self.repo / ".git/FETCH_HEAD").exists())
        self.assertEqual(set((DIRECTORY / "__pycache__").glob("*")), bytecode)
        self.record["restored"] = False
        self.save()
        result = subprocess.run([sys.executable, "-I", "-B", str(DIRECTORY / "ready-check"),
                                 str(self.result), "--repo", str(self.repo), "--tip", self.tip],
                                text=True, capture_output=True, stdin=subprocess.DEVNULL, env=self.env)
        self.assertEqual(result.returncode, 1)
        self.assertEqual(json.loads(result.stdout)["verdict"], "rerun-precheck")
        self.assertIn("precheckbase=", result.stderr)
        self.assertEqual(self.snapshot(self.repo), before)

    def test_shallow_replace_refs_and_git_overrides_refuse(self):
        self.git(self.repo, "replace", self.tip, self.base)
        self.refuse("replace refs")
        self.git(self.repo, "replace", "-d", self.tip)
        with patch.dict(os.environ, GIT_INDEX_FILE=str(self.root / "other.index")):
            self.refuse("Git environment override")
        self.git(self.root, "clone", "--depth=1", self.origin.as_uri(), str(self.root / "shallow"))
        self.assertIn("shallow", ready.check(self.result, self.root / "shallow", self.tip)["diagnostic"])


if __name__ == "__main__":
    unittest.main()
