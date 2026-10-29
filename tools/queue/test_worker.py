"""Local worker contract against disposable Git clones and fake check commands."""

from contextlib import redirect_stderr, redirect_stdout
import copy
import importlib.util
import io
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch


DIRECTORY = Path(__file__).resolve().parent
ROOT = DIRECTORY.parent.parent
SPEC = importlib.util.spec_from_file_location("queue_worker", DIRECTORY / "worker.py")
worker = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(worker)
mq = worker.mq


class WorkerTests(unittest.TestCase):
    def test_fixture_root_is_outside_checked_source(self):
        self.assertNotIn(ROOT, self.root.parents)

    def setUp(self):
        os.umask(0o077)
        self.temporary = tempfile.TemporaryDirectory(dir=ROOT.parent, prefix=".worker-test-")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.origin = self.root / "origin.git"
        self.prep = self.root / "preparation"
        self.repo = self.root / "worker checkout ; literal"
        self.artifacts = self.root / "artifacts"
        self.artifacts.mkdir(mode=0o700)
        mq.git(self.root, "init", "--bare", str(self.origin))
        mq.git(self.root, "clone", str(self.origin), str(self.prep))
        mq.git(self.prep, "config", "user.name", "Fixture")
        mq.git(self.prep, "config", "user.email", "fixture@example.invalid")
        mq.git(self.prep, "switch", "-c", "main")
        self.names = ["core", "consumer", *worker.frozen_lines("always.txt")]
        self.write(".gitignore", "/target/\n")
        for name in self.names:
            self.write(f"{name}/Cargo.toml", f'[package]\nname = "{name}"\nversion = "1.0.0"\n')
            self.write(f"{name}/src/lib.rs", "")
        self.write("core/src/deps.json", "[]")
        self.write("Cargo.lock", 'version = 4\n[[package]]\nname = "external"\nversion = "1.0.0"\n')
        self.commit("base")
        self.parent = self.tip()
        mq.git(self.prep, "push", "origin", "main")
        mq.git(self.origin, "symbolic-ref", "HEAD", "refs/heads/main")
        mq.git(self.root, "clone", str(self.origin), str(self.repo))
        mq.git(self.repo, "config", "user.name", "Fixture")
        mq.git(self.repo, "config", "user.email", "fixture@example.invalid")
        self.calls = []
        self.metadata_heads = []
        self.sequence = 0

    def write(self, name, text):
        path = self.prep / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text)

    def commit(self, message):
        mq.git(self.prep, "add", ".")
        mq.git(self.prep, "commit", "-m", message)

    def tip(self, repo=None):
        return mq.git(repo or self.prep, "rev-parse", "HEAD").stdout.strip()

    def metadata(self, repo):
        self.metadata_heads.append(self.tip(repo))
        packages = [{"id": name, "name": name, "version": "1.0.0", "source": None,
                     "manifest_path": str(repo / name / "Cargo.toml")} for name in self.names]
        edges = json.loads((repo / "core/src/deps.json").read_text())
        return {"workspace_root": str(repo), "workspace_members": list(self.names), "packages": packages,
                "resolve": {"nodes": [{"id": name, "dependencies": edges if name == "consumer" else []}
                                       for name in self.names]}}

    def transfer(self):
        # Transfer exact objects; neither worker code nor tests fetch origin to obtain candidates.
        bundle = self.root / "candidate.bundle"
        mq.git(self.prep, "bundle", "create", str(bundle), "HEAD")
        mq.git(self.repo, "bundle", "unbundle", str(bundle))

    def request(self, changes=None, *, selected=None, parent=None, branches=True):
        self.sequence += 1
        for name, text in (changes or {"README.md": "candidate\n"}).items():
            self.write(name, text)
        self.commit("candidate")
        self.transfer()
        candidate = self.tip()
        paths = mq.changed_paths(self.prep, candidate, base=parent or self.parent)
        data = self.metadata(self.prep)
        computed = mq.selection(paths, data, always=worker.frozen_lines("always.txt"),
                                read_by_tests=worker.frozen_lines("read-by-tests.txt"))
        selected = selected if selected is not None else (sorted(computed) if isinstance(computed, set) else computed)
        return {"version": 1, "attempt": f"attempt-{self.sequence}", "generation": "generation-1",
                "gate": "local-test", "parent": parent or self.parent, "candidate": candidate,
                "branches": [{"name": "ready/ops/input", "tip": candidate}] if branches else [],
                "selection": selected, "runtime": worker.runtime_identity(),
                "plan": worker.check_plan(self.prep, selected, paths, data)}

    def runner(self, argv, **kwargs):
        self.calls.append(argv)
        self.assertEqual(kwargs["stdin"], subprocess.DEVNULL)
        kwargs["stdout"].write("fixture command completed\n")
        return subprocess.CompletedProcess(argv, 0)

    def run_job(self, request, *, runner=None, checker=None, metadata_loader=None, free=100):
        def checks(*args, **kwargs):
            return mq.run_checks(*args, runner=runner or self.runner, **kwargs)
        with patch.object(mq, "free_gb", return_value=free):
            return worker.run_job(self.repo, self.artifacts, request, gate="local-test",
                                  checker=checker or checks, metadata_loader=metadata_loader or self.metadata)

    def assert_infra(self, result, text):
        self.assertEqual(result["outcome"], "INFRA", result)
        self.assertIn(text, result["error"])

    def test_exact_transferred_identity_private_atomic_evidence_and_no_authority_effects(self):
        request = self.request({"tools/test_fixture.py": "print('fixture')\n"})
        origin_refs = mq.git(self.origin, "show-ref").stdout
        remote_refs = mq.git(self.repo, "for-each-ref", "refs/remotes/").stdout
        with patch.object(mq, "git", wraps=mq.git) as git_calls, \
                patch.object(mq, "post", side_effect=AssertionError("post")), \
                patch.object(mq, "push_and_cleanup", side_effect=AssertionError("promotion")):
            result = self.run_job(request)
        self.assertFalse({call.args[1] for call in git_calls.call_args_list} &
                         {"fetch", "push", "ls-remote", "update-ref", "merge", "commit", "reset"})
        self.assertEqual(result["outcome"], "green", result)
        self.assertEqual(result["requested"], request)
        self.assertEqual(result["before"], result["after"])
        self.assertEqual(result["before"]["head"], request["candidate"])
        self.assertEqual(result["before"]["tree"], mq.git(self.prep, "rev-parse", "HEAD^{tree}").stdout.strip())
        self.assertEqual([s["name"] for s in result["steps"]], ["tools"])
        self.assertEqual(result["steps"][0]["suites"][0]["command"],
                         ["python3", "-I", "-B", "tools/test_fixture.py"])
        self.assertEqual(mq.git(self.origin, "show-ref").stdout, origin_refs)
        self.assertEqual(mq.git(self.repo, "for-each-ref", "refs/remotes/").stdout, remote_refs)
        self.assertEqual(self.tip(self.repo), request["candidate"])
        self.assertEqual(mq.git(self.repo, "symbolic-ref", "-q", "HEAD", check=False).returncode, 1)
        job = self.artifacts / request["attempt"]
        self.assertEqual(job.stat().st_mode & 0o777, 0o700)
        for path in job.iterdir():
            self.assertEqual(path.stat().st_mode & 0o777, 0o600)
        self.assertEqual(json.loads((job / "result.json").read_text()), result)
        log = Path(result["log"]["path"]).read_bytes()
        self.assertEqual(result["log"]["sha256"], worker.digest(log))
        self.assertEqual(result["log"]["bytes"], len(log))
        self.assertEqual(sorted(p.name for p in job.iterdir()), ["checks.log", "result.json"])
        self.assertFalse((self.repo / ".git/mq-worker-unresolved.json").exists())
        self.assertFalse(self.repo.with_suffix(".state.json").exists())
        with self.assertRaises(FileExistsError):
            self.run_job(request)

    def test_selection_and_metadata_are_candidate_graph_with_explicit_parent_not_origin_main(self):
        self.write("core/src/deps.json", '["core"]')
        self.commit("speculative parent graph")
        parent = self.tip()
        request = self.request({"core/src/lib.rs": "// child\n"}, parent=parent)
        self.assertIn("consumer", request["selection"])
        self.metadata_heads.clear()
        result = self.run_job(request)
        self.assertEqual(result["outcome"], "green", result)
        self.assertEqual(self.metadata_heads, [request["candidate"]])
        self.assertEqual(result["changed_paths"], ["core/src/lib.rs"])
        self.assertEqual(mq.git(self.repo, "rev-parse", "origin/main").stdout.strip(), self.parent)
        self.assertEqual([s["name"] for s in result["steps"]], ["fmt", "clippy", "build", "default", "fuzzy"])
        self.assertIn("--gate", result["steps"][1]["command"])
        self.assertEqual(result["steps"][-1]["command"][-2:], ["--profile", "fuzzy"])
        self.assertEqual(result["metadata"]["sha256"], worker.graph_identity(self.prep, self.metadata(self.prep))["sha256"])

    def test_check_only_ratchet_runs_real_fixture_without_writing_prepared_commit(self):
        self.write("jig/scripts/old_forms.py", (ROOT / "jig/scripts/old_forms.py").read_text())
        baseline = "max_out 0\nJournalRoom 0\ntakes 0\nworst_case 0\n"
        request = self.request({"jig/scripts/old-forms.txt": baseline}, selected="full")
        def runner(argv, **kwargs):
            if "old_forms.py" in " ".join(argv):
                return subprocess.run(argv, **kwargs)
            return self.runner(argv, **kwargs)
        result = self.run_job(request, runner=runner)
        self.assertEqual(result["outcome"], "green", result)
        self.assertEqual(result["steps"][0]["command"][-1], "./jig/scripts/old_forms.py")
        self.assertEqual(self.tip(self.repo), request["candidate"])
        self.assertEqual((self.repo / "jig/scripts/old-forms.txt").read_text(), baseline)

    def test_docs_empty_plan_is_valid_and_metadata_not_loaded(self):
        result = self.run_job(self.request(), metadata_loader=lambda repo: self.fail("metadata for docs"))
        self.assertEqual((result["outcome"], result["plan"], result["steps"]), ("green", [], []))

    def test_candidate_tools_suites_use_literal_paths_and_pinned_branch_names(self):
        request = self.request({"tools/test_$(literal);name.py": "print('literal suite path')\n"})
        request["branches"][0]["name"] = "ready/ops/$(literal);name"
        result = self.run_job(request, runner=subprocess.run)
        self.assertEqual(result["outcome"], "green", result)
        self.assertIn("literal suite path", Path(result["log"]["path"]).read_text())
        self.assertEqual(result["requested"]["branches"], request["branches"])

    def test_idle_full_checks_accept_parent_equal_candidate_with_no_inputs(self):
        request = self.request(selected="full", branches=False)
        request["parent"] = request["candidate"]
        request["plan"] = worker.check_plan(self.prep, "full", [], self.metadata(self.prep))
        result = self.run_job(request)
        self.assertEqual(result["outcome"], "green", result)
        self.assertEqual(result["changed_paths"], [])
        self.assertEqual([s["name"] for s in result["steps"]],
                         ["tools", "fmt", "clippy", "build", "default", "fuzzy"])

    def test_ordinary_nonzero_is_red_and_nextest_four_is_green(self):
        request = self.request({"core/src/lib.rs": "// source\n"})
        def runner(argv, **kwargs):
            return subprocess.CompletedProcess(argv, 1 if "clippy" in argv else 0)
        result = self.run_job(request, runner=runner)
        self.assertEqual(result["outcome"], "red", result)
        self.assertEqual([s["name"] for s in result["steps"]], ["fmt", "clippy"])
        request["attempt"] = "nextest-four"
        result = self.run_job(request, runner=lambda argv, **kw: subprocess.CompletedProcess(
            argv, 4 if "nextest" in argv else 0))
        self.assertEqual(result["outcome"], "green", result)

    def test_tools_timeout_is_red_runs_all_suites_and_missing_interpreter_is_infra(self):
        request = self.request({"tools/test_first.py": "", "tools/test_second.py": ""})
        def runner(argv, **kwargs):
            self.assertEqual(kwargs["timeout"], 60)
            self.calls.append(argv)
            if "first" in argv[-1]:
                raise subprocess.TimeoutExpired(argv, 60)
            return subprocess.CompletedProcess(argv, 0)
        result = self.run_job(request, runner=runner)
        self.assertEqual(result["outcome"], "red", result)
        self.assertEqual(len(self.calls), 2)
        request["attempt"] = "missing-interpreter"
        def missing(*args, **kwargs):
            raise FileNotFoundError("missing interpreter")
        result = self.run_job(request, runner=missing)
        self.assert_infra(result, "missing interpreter")
        self.assertTrue(result["completion_known"])
        self.assertFalse(result["reconciliation_required"])
        self.assertEqual(result["events"][0]["returncode"], 127)

    def test_preflight_errors_are_infra_without_checker_or_candidate_mutation(self):
        request = self.request()
        initial = self.tip(self.repo)
        cases = [("disk", {}, "disk floor"), ("runtime", {"runtime": {}}, "identity mismatch"),
                 ("object", {"candidate": "0" * 40}, "cat-file"),
                 ("ancestry", {"parent": request["candidate"], "candidate": self.parent}, "not contained"),
                 ("gate", {"gate": "other"}, "identity mismatch")]
        for label, updates, text in cases:
            with self.subTest(case=label):
                changed = dict(request, **updates, attempt=label)
                result = self.run_job(changed, free=1 if label == "disk" else 100,
                                      checker=lambda *a, **kw: self.fail("preflight ran checks"))
                self.assert_infra(result, text)
                self.assertEqual(self.tip(self.repo), initial)
        (self.repo / "dirty").write_text("dirty")
        result = self.run_job(dict(request, attempt="dirty"))
        self.assert_infra(result, "uncommitted or untracked")

    def test_missing_and_uncontained_ready_tips_are_infra(self):
        request = self.request()
        request["branches"][0]["tip"] = "0" * 40
        self.assert_infra(self.run_job(request), "cat-file")
        mq.git(self.prep, "switch", "--detach", self.parent)
        self.write("elsewhere.md", "unrelated\n")
        self.commit("uncontained ready tip")
        self.transfer()
        request["attempt"] = "uncontained-tip"
        request["branches"][0]["tip"] = self.tip()
        self.assert_infra(self.run_job(request), "not contained")

    def test_local_history_overrides_and_index_flags_cannot_hide_checked_inputs(self):
        request = self.request()
        mq.git(self.repo, "replace", request["candidate"], self.parent)
        self.assert_infra(self.run_job(request), "history overrides")
        mq.git(self.repo, "replace", "-d", request["candidate"])
        mq.git(self.repo, "update-index", "--assume-unchanged", "core/src/lib.rs")
        (self.repo / "core/src/lib.rs").write_text("hidden dirty contents")
        self.assert_infra(self.run_job(dict(request, attempt="hidden-index")), "index flags")

    def test_wrong_selection_incomplete_plan_and_metadata_error_are_infra(self):
        request = self.request({"core/src/lib.rs": "// source\n"})
        for label, mutate, loader, text in [
                ("selection", lambda r: r.update(selection=[]), None, "selection mismatch"),
                ("plan", lambda r: r.update(plan=r["plan"][:-1]), None, "plan mismatch"),
                ("metadata", lambda r: None, lambda repo: (_ for _ in ()).throw(ValueError("metadata failed")), "metadata failed"),
                ("graph-root", lambda r: None, lambda repo: self.metadata(self.prep), "workspace root")]:
            with self.subTest(case=label):
                changed = copy.deepcopy(request)
                changed["attempt"] = label
                mutate(changed)
                result = self.run_job(changed, metadata_loader=loader)
                self.assert_infra(result, text)
                self.assertEqual(self.calls, [])
        request = dict(request, selection="full", attempt="empty-metadata")
        self.assert_infra(self.run_job(request, metadata_loader=lambda repo: {}), "KeyError")

    def test_heavy_capacity_and_launch_errors_are_infra(self):
        request = self.request({"core/src/lib.rs": "// source\n"})
        def runner(argv, **kwargs):
            if argv[0] == "heavy":
                kwargs["stdout"].write("builder below the disk floor\n")
                return subprocess.CompletedProcess(argv, 2)
            return self.runner(argv, **kwargs)
        result = self.run_job(request, runner=runner)
        self.assert_infra(result, "below the disk floor")
        request["attempt"] = "launch-error"
        def missing(*args, **kwargs):
            raise OSError("checker unavailable")
        self.assert_infra(self.run_job(request, runner=missing), "checker unavailable")

    def test_unexpected_tracked_untracked_or_committed_write_is_infra_and_blocks_reuse(self):
        request = self.request({"core/src/lib.rs": "// source\n"})
        def runner(argv, **kwargs):
            (self.repo / "core/src/lib.rs").write_text("// rewritten\n")
            mq.git(self.repo, "add", ".")
            mq.git(self.repo, "commit", "-m", "unexpected rewrite")
            return subprocess.CompletedProcess(argv, 0)
        result = self.run_job(request, runner=runner)
        self.assert_infra(result, "HEAD/tree/worktree changed")
        self.assertTrue(result["after"]["clean"])
        self.assertNotEqual(result["after"]["head"], request["candidate"])
        self.assertTrue(result["reconciliation_required"])
        with self.assertRaisesRegex(RuntimeError, "reconciliation"):
            self.run_job(dict(request, attempt="reuse"))

    def test_dirty_write_is_recorded_and_never_reset_by_worker(self):
        request = self.request({"core/src/lib.rs": "// source\n"})
        def runner(argv, **kwargs):
            (self.repo / "new-untracked").write_text("evidence")
            return subprocess.CompletedProcess(argv, 0)
        result = self.run_job(request, runner=runner)
        self.assert_infra(result, "worktree changed")
        self.assertIn("new-untracked", result["after"]["status"])
        self.assertTrue((self.repo / "new-untracked").exists())
        self.assertIn("new-untracked", result["identity_violation"]["status"])

    def test_checker_exception_is_infra_completion_unknown_and_requires_reconciliation(self):
        request = self.request()
        def checker(*args, **kwargs):
            raise RuntimeError("checker lost")
        result = self.run_job(request, checker=checker)
        self.assert_infra(result, "checker lost")
        self.assertFalse(result["completion_known"])
        self.assertTrue(result["reconciliation_required"])
        self.assertIn("skipped", result["target_cleanup"])
        with self.assertRaisesRegex(RuntimeError, "reconciliation"):
            self.run_job(dict(request, attempt="reuse"))

    def test_lost_heavy_client_keeps_checkout_unavailable(self):
        request = self.request({"core/src/lib.rs": "// source\n"})
        def runner(argv, **kwargs):
            return subprocess.CompletedProcess(argv, -9 if argv[0] == "heavy" else 0)
        result = self.run_job(request, runner=runner)
        self.assert_infra(result, "heavy client terminated")
        self.assertFalse(result["completion_known"])
        self.assertTrue(result["reconciliation_required"])
        self.assertIn("skipped", result["target_cleanup"])

    def test_missing_or_corrupt_step_and_suite_evidence_cannot_certify_green(self):
        request = self.request({"tools/test_fixture.py": ""})
        for label, checker in [("empty", lambda *a, **kw: []),
                               ("suites", lambda *a, **kw: [{"name": "tools", "green": True,
                                                            "duration": 0, "suites": []}])]:
            with self.subTest(case=label):
                self.assert_infra(self.run_job(dict(request, attempt=label), checker=checker), "incomplete")

    def test_command_cwd_returncode_duration_and_step_order_must_match(self):
        request = self.request({"core/src/lib.rs": "// source\n"})
        good = self.run_job(request)
        self.assertEqual(good["outcome"], "green", good)
        for field, value in [("command", ["true"]), ("cwd", "core"),
                             ("returncode", True), ("green", False), ("duration", float("nan"))]:
            with self.subTest(field=field):
                steps = copy.deepcopy(good["steps"])
                steps[0][field] = value
                with self.assertRaises((ValueError, RuntimeError)):
                    worker.validate_steps(self.repo, good["plan"], steps)
        with self.assertRaises(ValueError):
            worker.validate_steps(self.repo, good["plan"], list(reversed(good["steps"])))

    def test_competing_request_is_excluded_while_first_checker_holds_checkout(self):
        request = self.request()
        def checker(*args, **kwargs):
            with self.assertRaisesRegex(RuntimeError, "holds this checkout"):
                self.run_job(dict(request, attempt="competitor"))
            self.assertFalse((self.artifacts / "competitor").exists())
            path = self.root / "competing-request.json"
            path.write_text(json.dumps(dict(request, attempt="external-competitor")))
            process = subprocess.run([sys.executable, "-I", "-B", str(DIRECTORY / "worker.py"), "run",
                                      "--enable-local", "--repo", str(self.repo),
                                      "--artifacts", str(self.artifacts), "--request", str(path),
                                      "--gate", "local-test"], stdin=subprocess.DEVNULL,
                                     capture_output=True, text=True)
            self.assertEqual(process.returncode, 2)
            self.assertIn("holds this checkout", process.stderr)
            self.assertFalse((self.artifacts / "external-competitor").exists())
            return []
        result = self.run_job(request, checker=checker)
        self.assertEqual(result["outcome"], "green", result)

    def test_crash_marker_is_durable_and_not_cleared_by_finally(self):
        request = self.request()
        def checker(*args, **kwargs):
            raise KeyboardInterrupt()
        with self.assertRaises(KeyboardInterrupt):
            self.run_job(request, checker=checker)
        marker = self.repo / ".git/mq-worker-unresolved.json"
        self.assertTrue(marker.exists())
        self.assertEqual(marker.stat().st_mode & 0o777, 0o600)
        with self.assertRaisesRegex(RuntimeError, "reconciliation"):
            self.run_job(dict(request, attempt="after-crash"))

    def test_process_death_without_finally_leaves_unresolved_marker_and_no_result(self):
        request = self.request()
        path = self.root / "crash-request.json"
        path.write_text(json.dumps(request))
        code = """import importlib.util, os, pathlib, sys
spec = importlib.util.spec_from_file_location('worker', sys.argv[1])
worker = importlib.util.module_from_spec(spec)
spec.loader.exec_module(worker)
worker.mq.free_gb = lambda repo: 100
worker.run_job(pathlib.Path(sys.argv[2]), pathlib.Path(sys.argv[3]),
               worker.read_request(pathlib.Path(sys.argv[4])), gate='local-test',
               checker=lambda *args, **kwargs: os._exit(23))
"""
        process = subprocess.run([sys.executable, "-I", "-B", "-c", code,
                                  str(DIRECTORY / "worker.py"), str(self.repo), str(self.artifacts), str(path)],
                                 stdin=subprocess.DEVNULL, capture_output=True, text=True)
        self.assertEqual(process.returncode, 23, process.stderr)
        self.assertTrue((self.repo / ".git/mq-worker-unresolved.json").is_file())
        self.assertFalse((self.artifacts / request["attempt"] / "result.json").exists())
        with self.assertRaisesRegex(RuntimeError, "reconciliation"):
            self.run_job(dict(request, attempt="after-hard-crash"))

    def test_publication_failure_preserves_unresolved_ownership(self):
        request = self.request()
        publish = worker.atomic_json
        def fail_result(path, value):
            if path.name == "result.json":
                raise OSError("result publication failed")
            return publish(path, value)
        with patch.object(worker, "atomic_json", side_effect=fail_result), \
                self.assertRaisesRegex(OSError, "publication failed"):
            self.run_job(request)
        with self.assertRaisesRegex(RuntimeError, "reconciliation"):
            self.run_job(dict(request, attempt="after-publication-failure"))

    def test_unexpected_ratchet_write_does_not_commit_a_replacement_candidate(self):
        self.write("jig/scripts/old_forms.py", """from pathlib import Path
Path('scripts/old-forms.txt').write_text('rewritten baseline')
""")
        request = self.request({"jig/scripts/old-forms.txt": "prepared baseline"}, selected="full")
        result = self.run_job(request, runner=subprocess.run)
        self.assert_infra(result, "worktree changed")
        self.assertEqual(self.tip(self.repo), request["candidate"])
        self.assertIn("jig/scripts/old-forms.txt", result["after"]["status"])
        self.assertTrue(result["reconciliation_required"])

    def test_checkout_root_symlink_linked_worktree_and_artifact_modes_refused(self):
        request = self.request()
        link = self.root / "symlink"
        link.symlink_to(self.repo, target_is_directory=True)
        for repo, artifacts, message in [(link, self.artifacts, "symlink"),
                                         (self.repo / "core", self.artifacts, "checkout root")]:
            with self.subTest(repo=repo), self.assertRaisesRegex(ValueError, message):
                worker.run_job(repo, artifacts, request, gate="local-test")
        self.artifacts.chmod(0o755)
        with self.assertRaisesRegex(ValueError, "0700"):
            self.run_job(request)
        self.artifacts.chmod(0o700)
        linked = self.root / "linked"
        mq.git(self.repo, "worktree", "add", "--detach", str(linked), "HEAD")
        with self.assertRaisesRegex(ValueError, "independent clone"):
            worker.run_job(linked, self.artifacts, request, gate="local-test")

    def test_malformed_requests_are_rejected_without_artifacts(self):
        request = self.request()
        malformed = [{"unexpected": 1}, {"version": True}, {"attempt": "../escape"},
                     {"generation": "bad; shell"}, {"candidate": "HEAD"},
                     {"selection": ["z", "a"]}, {"selection": ["a", "a"]},
                     {"selection": ["--workspace"]}, {"selection": "garbage"},
                     {"branches": [{"name": "ready/ops/x..y", "tip": request["candidate"]}]},
                     {"branches": request["branches"] * 2}, {"plan": [{}]}]
        for updates in malformed:
            with self.subTest(updates=updates), self.assertRaises(ValueError):
                self.run_job(dict(request, **updates))
        self.assertEqual(list(self.artifacts.iterdir()), [])
        path = self.root / "request.json"
        path.write_text('{"version":1,"version":1}')
        with self.assertRaisesRegex(ValueError, "duplicate"):
            worker.read_request(path)

    def test_atomic_publication_does_not_expose_partial_snapshot(self):
        result = self.artifacts / "snapshot.json"
        worker.atomic_json(result, {"old": True})
        replace = os.replace
        def inspect(source, destination):
            self.assertEqual(json.loads(result.read_text()), {"old": True})
            self.assertEqual(json.loads(Path(source).read_text()), {"new": True})
            replace(source, destination)
        with patch.object(os, "replace", side_effect=inspect):
            worker.atomic_json(result, {"new": True})
        self.assertEqual(json.loads(result.read_text()), {"new": True})

    def test_runtime_identity_is_path_independent_and_detects_changed_checker_or_lists(self):
        identity = worker.runtime_identity()
        self.assertEqual(set(identity["files"]), set(worker.RUNTIME_FILES))
        self.assertNotIn(str(DIRECTORY), json.dumps(identity))
        self.assertNotEqual(worker.runtime_identity(46), identity)
        read_bytes = Path.read_bytes
        for filename in ("mq", "always.txt"):
            with self.subTest(file=filename):
                def changed(path):
                    return b"changed" if path == DIRECTORY / filename else read_bytes(path)
                with patch.object(Path, "read_bytes", changed), self.assertRaisesRegex(RuntimeError, "runtime/config"):
                    worker.runtime_unchanged()

    def test_cli_disabled_without_opt_in_and_identity_is_read_only(self):
        with patch.object(sys, "argv", ["worker.py", "run"]), redirect_stderr(io.StringIO()), \
                self.assertRaises(SystemExit) as raised:
            worker.main()
        self.assertEqual(raised.exception.code, 2)
        with patch.object(sys, "argv", ["worker.py", "identity"]), redirect_stdout(io.StringIO()) as output:
            self.assertEqual(worker.main(), 0)
        self.assertEqual(json.loads(output.getvalue()), worker.runtime_identity())


if __name__ == "__main__":
    unittest.main()
