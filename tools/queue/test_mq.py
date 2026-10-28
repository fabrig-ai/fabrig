"""Queue policy and effects against disposable local repositories; no real posts."""

from contextlib import redirect_stdout
import importlib.machinery
import importlib.util
import io
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch


DIRECTORY = Path(__file__).resolve().parent
ROOT = DIRECTORY.parent.parent
LOADER = importlib.machinery.SourceFileLoader("mq", str(DIRECTORY / "mq"))
SPEC = importlib.util.spec_from_loader(LOADER.name, LOADER)
mq = importlib.util.module_from_spec(SPEC)
LOADER.exec_module(mq)


def metadata(root=Path("/workspace")):
    names = ("core", "normal", "build", "dev", "transitive", "unrelated", "always", "nested")
    names += tuple(mq.lines(DIRECTORY / "always.txt"))
    packages = [{"id": name, "name": name, "manifest_path": str(root / name / "Cargo.toml")}
                for name in names]
    packages[7]["manifest_path"] = str(root / "core/examples/nested/Cargo.toml")
    edges = {"normal": ["core"], "build": ["core"], "dev": ["core"], "transitive": ["dev"]}
    return {"workspace_root": str(root), "workspace_members": list(names), "packages": packages,
            "resolve": {"nodes": [{"id": name, "dependencies": edges.get(name, [])}
                                   for name in names]}}


class SelectionTests(unittest.TestCase):
    def select(self, paths):
        return mq.selection(paths, metadata(), always=["always"], read_by_tests=["docs/contract/**"])

    def test_source_selects_owner_reverse_closure_and_always(self):
        self.assertEqual(self.select(["core/src/lib.rs"]),
                         {"core", "normal", "build", "dev", "transitive", "always"})

    def test_test_bench_example_and_build_script_select_owner(self):
        for path in ("tests/check.rs", "benches/speed.rs", "examples/demo.rs", "build.rs"):
            with self.subTest(path=path):
                self.assertEqual(self.select(["unrelated/" + path]), {"unrelated", "always"})

    def test_longest_manifest_directory_owns_the_change(self):
        self.assertEqual(self.select(["core/examples/nested/src/lib.rs"]), {"nested", "always"})

    def test_manifest_lock_config_script_fixture_and_unknown_select_full(self):
        for path in ("Cargo.toml", "Cargo.lock", ".config/nextest.toml", "core/Cargo.toml",
                     "core/scripts/check.py", "core/fixtures/input", "outside/source.rs"):
            with self.subTest(path=path):
                self.assertEqual(self.select([path]), "full")

    def test_markdown_outside_test_inputs_is_docs(self):
        self.assertEqual(self.select(["README.md", "docs/guide.md"]), "docs")

    def test_markdown_read_by_tests_is_full(self):
        self.assertEqual(self.select(["docs/contract/store.md"]), "full")

    def test_mixed_source_and_read_markdown_is_full(self):
        self.assertEqual(self.select(["core/src/lib.rs", "docs/contract/store.md"]), "full")

    def test_root_package_and_directory_prefix_are_distinct(self):
        data = metadata()
        data["packages"][0]["manifest_path"] = "/workspace/Cargo.toml"
        self.assertIn("core", mq.selection(["src/lib.rs"], data, always=[]))
        self.assertEqual(self.select(["core-other/src/lib.rs"]), "full")

    def test_missing_always_package_is_reported(self):
        with self.assertRaisesRegex(ValueError, "missing workspace packages"):
            mq.selection(["core/src/lib.rs"], metadata(), always=["absent"])

    def test_default_read_globs_cover_the_known_markdown_inputs(self):
        paths = ("temper/docs/help/reference/commands.md", "jig/docs/design/protocol/store.md",
                 "smith/docs/design/shell.md", "smith/benchmarks/README.md",
                 "smith/benchmarks/tests/fixtures/seed/README.md",
                 "smith/benchmarks/tests/recorded/tasks/valid/seed/README.md",
                 "smith/benchmarks/tasks/fixtures/seed/README.md",
                 "skein/crates/skein-ui/src/tests/markdown/01.md", "skein/docs/design/forge-github.md")
        for path in paths:
            with self.subTest(path=path):
                self.assertEqual(mq.selection([path], metadata()), "full")


class CheckTests(unittest.TestCase):
    def test_package_scope_and_ratchets(self):
        steps = mq.check_commands(ROOT, {"z", "a"}, ["smith/crates/a/src/lib.rs", "skein/docs/x.md"])
        self.assertEqual([name for name, _, _ in steps],
                         ["fmt", "clippy", "default", "fuzzy", "ratchet:skein", "ratchet:smith"])
        self.assertEqual(steps[0][1], ["cargo", "fmt", "--all", "--check"])
        for _, argv, _ in steps[1:4]:
            self.assertEqual(argv[:3], ["heavy", "--gate", "cargo"])
            self.assertEqual(argv[argv.index("-p"):argv.index("-p") + 4], ["-p", "a", "-p", "z"])
        self.assertEqual(steps[1][1][-4:], ["--all-targets", "--", "-D", "warnings"])
        self.assertIn("--no-fail-fast", steps[2][1])
        self.assertEqual(steps[3][1][-2:], ["--profile", "fuzzy"])
        self.assertEqual(steps[-1][2], ROOT / "smith")
        self.assertEqual(steps[-1][1], ["python3", "-I", "-B", str(ROOT / "smith/scripts/old_forms.py")])

    def test_full_uses_workspace_and_docs_only_fmt_and_ratchets(self):
        for _, argv, _ in mq.check_commands(ROOT, "full", [])[1:]:
            self.assertIn("--workspace", argv)
            self.assertNotIn("-p", argv)
        self.assertEqual([name for name, _, _ in mq.check_commands(ROOT, "docs", ["jig/README.md"])],
                         ["fmt", "ratchet:jig"])

    def test_empty_nextest_is_green_and_steps_have_durations(self):
        calls = []

        def runner(argv, **kwargs):
            calls.append(argv)
            self.assertEqual(kwargs["stdin"], subprocess.DEVNULL)
            return subprocess.CompletedProcess(argv, 4 if "nextest" in argv else 0)

        with tempfile.TemporaryDirectory(dir=ROOT, prefix=".mq-test-") as directory:
            log = Path(directory) / "checks.log"
            times = iter(range(8))
            steps = mq.run_checks(ROOT, {"core"}, [], log, runner=runner, clock=lambda: next(times))
            self.assertEqual(len(calls), 4)
            self.assertTrue(all(step["green"] and step["duration"] == 1 for step in steps))
            self.assertEqual(log.stat().st_mode & 0o777, 0o600)

    def test_failure_stops_checks_and_four_is_not_green_for_fmt(self):
        with tempfile.TemporaryDirectory(dir=ROOT, prefix=".mq-test-") as directory:
            steps = mq.run_checks(ROOT, "full", [], Path(directory) / "checks.log",
                                  runner=lambda *args, **kwargs: subprocess.CompletedProcess(args, 4))
            self.assertEqual(len(steps), 1)
            self.assertFalse(steps[0]["green"])


class BisectTests(unittest.TestCase):
    def test_one_red_is_reported_and_green_groups_land_in_order(self):
        landed = []

        def check(group):
            if "red" in group:
                return False
            landed.extend(group)
            return True

        green, red = mq.bisect(["a", "b", "red", "c", "d"], check)
        self.assertEqual(green, ["a", "b", "c", "d"])
        self.assertEqual(landed, green)
        self.assertEqual(red, ["red"])

    def test_single_red_is_rejected_once(self):
        calls = []
        self.assertEqual(mq.bisect(["red"], lambda group: calls.append(group) or False), ([], ["red"]))
        self.assertEqual(calls, [["red"]])

    def test_interacting_green_branches_are_checked_after_previous_landing(self):
        landed = []

        def check(group):
            if "a" in landed and "b" in group or {"a", "b"}.issubset(group):
                return False
            landed.extend(group)
            return True

        self.assertEqual(mq.bisect(["a", "b"], check), (["a"], ["b"]))


class RepositoryTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(dir=ROOT, prefix=".mq-test-")
        self.addCleanup(self.temporary.cleanup)
        self.directory = Path(self.temporary.name)
        self.repo = self.directory / "queue"
        self.origin = self.directory / "origin.git"
        self.repo.mkdir()
        mq.git(self.repo, "init", "-b", "work")
        mq.git(self.repo, "config", "user.name", "Queue fixture")
        mq.git(self.repo, "config", "user.email", "queue@example.test")
        mq.git(self.repo, "config", "commit.gpgsign", "false")
        self.write("shared", "base\n")
        self.commit("base")
        self.base = self.tip()
        mq.git(self.repo, "init", "--bare", str(self.origin))
        mq.git(self.repo, "remote", "add", "origin", str(self.origin))
        mq.git(self.repo, "push", "origin", "HEAD:main")
        mq.git(self.repo, "fetch", "origin")
        self.posts = []
        self.checks = []

    def write(self, path, text):
        target = self.repo / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(text)

    def commit(self, subject):
        mq.git(self.repo, "add", ".")
        mq.git(self.repo, "commit", "-m", subject)

    def tip(self):
        return mq.git(self.repo, "rev-parse", "HEAD").stdout.strip()

    def branch(self, task, path=None, text="change\n", *, publish=False):
        # Only fixture repositories receive ready refs; the enclosing slot never does.
        mq.git(self.repo, "switch", "--detach", self.base)
        self.write(path or task, text)
        self.commit(task)
        branch = {"name": "ready/coord/" + task, "tip": self.tip(), "timestamp": 1}
        if publish:
            mq.git(self.repo, "push", "origin", f"HEAD:refs/heads/{branch['name']}")
        return branch

    def checker(self, repo, selected, paths, log):
        self.checks.append((selected, paths, self.tip()))
        return [{"name": "fixture", "green": not (repo / "red").exists(), "duration": 0.0}]

    def poster(self, address, subject, body):
        self.assertLess(len(body.splitlines()), 15)
        self.posts.append((address, subject, body))
        return {"sent": True, "returncode": 3}

    def queue(self, **kwargs):
        return mq.Queue(self.repo, state_path=self.directory / "state.json",
                        log_path=self.directory / "queue.jsonl", checker=self.checker,
                        poster=self.poster, metadata_loader=lambda repo: metadata(repo), **kwargs)

    def records(self):
        return [json.loads(line) for line in (self.directory / "queue.jsonl").read_text().splitlines()]

    def run_cycle(self, queue):
        with redirect_stdout(io.StringIO()):
            return queue.cycle()

    def test_conflict_is_skipped_and_the_rest_are_merged(self):
        first = self.branch("first", "shared", "first\n")
        conflict = self.branch("conflict", "shared", "other\n")
        last = self.branch("last")
        merged, conflicts = mq.build_batch(self.repo, [first, conflict, last])
        self.assertEqual([b["name"] for b in merged], [first["name"], last["name"]])
        self.assertEqual([b["name"] for b in conflicts], [conflict["name"]])
        self.assertEqual((self.repo / "shared").read_text(), "first\n")
        self.assertTrue((self.repo / "last").exists())
        self.assertEqual(mq.git(self.repo, "status", "--porcelain").stdout, "")

    def test_candidates_are_pinned_ordered_and_skip_only_the_tried_tip(self):
        a = self.branch("a")
        b = self.branch("b")
        for branch in (a, b):
            mq.git(self.repo, "update-ref", "refs/remotes/origin/" + branch["name"], branch["tip"])
        listed = mq.list_candidates(self.repo, {a["name"]: a["tip"]})
        self.assertEqual([branch["name"] for branch in listed], [b["name"]])
        self.assertEqual(listed[0]["tip"], b["tip"])
        self.assertEqual([branch["name"] for branch in mq.list_candidates(self.repo, {a["name"]: "old"})],
                         [a["name"], b["name"]])

    def test_cycle_bisects_red_and_lands_green_without_retrying_red_tip(self):
        for task in ("a", "red", "z"):
            self.branch(task, publish=True)
        queue = self.queue()
        self.run_cycle(queue)
        mq.git(self.repo, "switch", "--detach", "origin/main")
        self.assertTrue((self.repo / "a").exists() and (self.repo / "z").exists())
        self.assertFalse((self.repo / "red").exists())
        self.assertEqual(mq.git(self.repo, "ls-remote", "--heads", "origin", "refs/heads/ready/*").stdout.split()[1],
                         "refs/heads/ready/coord/red")
        self.assertEqual(len([post for post in self.posts if "rejected" in post[1]]), 1)
        self.assertEqual(len([post for post in self.posts if "landed" in post[1]]), 2)
        checked = len(self.checks)
        self.assertFalse(self.run_cycle(self.queue()))
        self.assertEqual(len(self.checks), checked)
        records = self.records()
        self.assertTrue(all(record["start"] and record["end"] and record["steps"] for record in records))
        self.assertEqual(sum(record["outcome"] == "green" for record in records), 2)

    def test_conflict_is_reported_and_other_branch_lands(self):
        self.branch("a", "shared", "one\n", publish=True)
        self.branch("b", "shared", "two\n", publish=True)
        self.branch("z", publish=True)
        self.run_cycle(self.queue())
        self.assertEqual(len([post for post in self.posts if "rejected" in post[1]]), 1)
        self.assertTrue(any("merge conflict" in body for _, _, body in self.posts))
        self.assertTrue(any(record["conflicts"] for record in self.records()))
        self.assertTrue((self.repo / "z").exists())

    def test_dry_checks_and_restores_checkout_without_remote_posts_or_state_changes(self):
        branch = self.branch("a", publish=True)
        mq.git(self.repo, "switch", "work")
        before = mq.git(self.repo, "ls-remote", "origin").stdout
        queue = self.queue(dry=True)
        self.run_cycle(queue)
        self.assertEqual(mq.git(self.repo, "ls-remote", "origin").stdout, before)
        self.assertEqual(mq.git(self.repo, "branch", "--show-current").stdout.strip(), "work")
        self.assertEqual(self.tip(), self.base)
        self.assertEqual(self.posts, [])
        self.assertFalse(queue.state_path.exists())
        self.assertEqual(len(self.checks), 1)
        self.assertEqual(self.records()[0]["branches"][0]["tip"], branch["tip"])

    def test_plan_has_no_merge_push_post_or_state_write(self):
        self.branch("a", publish=True)
        mq.git(self.repo, "switch", "work")
        before = mq.git(self.repo, "ls-remote", "origin").stdout
        output = io.StringIO()
        queue = self.queue()
        with redirect_stdout(output):
            queue.plan()
        self.assertEqual(json.loads(output.getvalue())["selection"], "full")
        self.assertEqual(self.tip(), self.base)
        self.assertEqual(mq.git(self.repo, "ls-remote", "origin").stdout, before)
        self.assertEqual(self.posts, [])
        self.assertEqual(self.checks, [])
        self.assertFalse(queue.state_path.exists())

    def test_post_failure_is_logged_and_does_not_fail_landing(self):
        self.branch("a", publish=True)
        queue = self.queue()
        queue.poster = lambda *args: {"sent": False, "returncode": 2, "output": "not delivered"}
        self.run_cycle(queue)
        record = self.records()[0]
        self.assertEqual(record["outcome"], "green")
        self.assertEqual(record["posts"][0]["returncode"], 2)

    def test_rejected_branch_is_retried_when_its_tip_changes(self):
        branch = self.branch("red", publish=True)
        self.run_cycle(self.queue())
        self.assertEqual(len(self.checks), 1)
        mq.git(self.repo, "switch", "--detach", branch["tip"])
        mq.git(self.repo, "rm", "red")
        self.write("fixed", "fixed\n")
        self.commit("remove failing input")
        mq.git(self.repo, "push", "origin", f"HEAD:refs/heads/{branch['name']}")
        self.run_cycle(self.queue())
        self.assertEqual(len(self.checks), 2)
        self.assertTrue((self.repo / "fixed").exists())
        self.assertEqual(self.records()[-1]["outcome"], "green")

    def test_selected_check_and_periodic_full_are_applied_to_real_batches(self):
        self.branch("a", "core/src/lib.rs", publish=True)
        queue = self.queue(now=lambda: 10000, full_every=2)
        queue.state["last_full"] = 9999
        self.run_cycle(queue)
        self.assertEqual(self.checks[0][0],
                         {"core", "normal", "build", "dev", "transitive"} | set(mq.lines(DIRECTORY / "always.txt")))
        self.branch("z", "unrelated/src/lib.rs", publish=True)
        self.run_cycle(queue)
        self.assertEqual(self.checks[1][0], "full")
        self.assertEqual(queue.state["batches_since_full"], 0)

    def test_docs_batch_does_not_need_cargo_metadata(self):
        self.branch("a", "guide.md", publish=True)
        queue = self.queue(now=lambda: 10000)
        queue.state["last_full"] = 9999

        def no_metadata(repo):
            self.fail("docs-only selection should not invoke cargo metadata")

        queue.metadata_loader = no_metadata
        self.run_cycle(queue)
        self.assertEqual(self.checks[0][0], "docs")

    def test_manifest_batch_does_not_need_valid_cargo_metadata(self):
        self.branch("a", "Cargo.toml", "not a valid manifest\n", publish=True)
        queue = self.queue(now=lambda: 10000)
        queue.state["last_full"] = 9999

        def no_metadata(repo):
            self.fail("a manifest change must select full before metadata is loaded")

        queue.metadata_loader = no_metadata
        self.run_cycle(queue)
        self.assertEqual(self.checks[0][0], "full")

    def test_full_failure_is_bisected_with_full_checks(self):
        self.branch("a", "core/src/good.rs", publish=True)
        self.branch("z", "unrelated/src/red.rs", publish=True)
        queue = self.queue()
        selections = []

        def check(repo, selected, paths, log):
            selections.append(selected)
            green = selected != "full" or not (repo / "unrelated/src/red.rs").exists()
            return [{"name": "fixture", "green": green, "duration": 0.0}]

        queue.checker = check
        self.run_cycle(queue)
        self.assertTrue(all(selected == "full" for selected in selections))
        self.assertTrue((self.repo / "core/src/good.rs").exists())
        mq.git(self.repo, "switch", "--detach", "origin/main")
        self.assertFalse((self.repo / "unrelated/src/red.rs").exists())
        self.assertTrue(any("rejected ready/coord/z" == subject for _, subject, _ in self.posts))

    def test_dry_bisection_checks_later_groups_against_prior_green_groups(self):
        for task in ("a", "red", "z"):
            self.branch(task, publish=True)
        queue = self.queue(dry=True)
        observed = []

        def check(repo, selected, paths, log):
            observed.append({name for name in ("a", "red", "z") if (repo / name).exists()})
            return self.checker(repo, selected, paths, log)

        queue.checker = check
        self.run_cycle(queue)
        self.assertEqual(observed[-1], {"a", "z"})
        self.assertEqual(self.posts, [])

    def test_checks_cannot_leave_uncommitted_inputs_and_land(self):
        self.branch("a", publish=True)
        queue = self.queue()

        def dirty_check(repo, selected, paths, log):
            self.write("shared", "modified by check\n")
            return [{"name": "fixture", "green": True, "duration": 0.0}]

        queue.checker = dirty_check
        with self.assertRaisesRegex(RuntimeError, "uncommitted or untracked"):
            self.run_cycle(queue)
        self.assertEqual(self.records()[0]["outcome"], "error")
        self.assertEqual(mq.git(self.repo, "rev-parse", "origin/main").stdout.strip(), self.base)

    def test_merge_setup_error_is_logged_without_rejecting_a_branch(self):
        branch = self.branch("a", publish=True)
        queue = self.queue()
        with patch.object(mq, "build_batch", side_effect=RuntimeError("merge infrastructure failure")):
            with self.assertRaisesRegex(RuntimeError, "merge infrastructure failure"):
                self.run_cycle(queue)
        record = self.records()[0]
        self.assertEqual(record["branches"][0]["name"], branch["name"])
        self.assertEqual(record["outcome"], "error")
        self.assertIn("merge infrastructure failure", Path(record["log"]).read_text())
        self.assertEqual(queue.state["tried"], {})
        self.assertEqual(self.posts, [])

    def test_changed_tip_is_preserved_by_cleanup(self):
        branch = self.branch("a", publish=True)
        self.write("new", "new\n")
        self.commit("advanced ready tip")
        advanced = self.tip()
        mq.git(self.repo, "push", "origin", f"HEAD:refs/heads/{branch['name']}")
        mq.build_batch(self.repo, [branch])
        _, cleanup = mq.push_and_cleanup(self.repo, [branch])
        self.assertIn("preserved", cleanup[0]["outcome"])
        self.assertEqual(mq.git(self.repo, "ls-remote", "origin", "refs/heads/" + branch["name"]).stdout.split()[0], advanced)

    def test_push_failure_does_not_delete_ready_branch(self):
        branch = self.branch("a", publish=True)
        mq.build_batch(self.repo, [branch])
        mq.git(self.repo, "config", "remote.origin.pushurl", str(self.directory / "absent.git"))
        with self.assertRaises(RuntimeError):
            mq.push_and_cleanup(self.repo, [branch])
        self.assertIn(branch["name"], mq.git(self.repo, "ls-remote", "origin").stdout)

    def test_periodic_hourly_and_idle_full_policy(self):
        queue = self.queue(now=lambda: 10000)
        self.assertTrue(queue.full_due())
        queue.state.update(last_full=9999, batches_since_full=3, idle_since=9500)
        self.assertFalse(queue.full_due())
        self.assertFalse(queue.idle_due())
        queue.state["batches_since_full"] = 4
        self.assertTrue(queue.full_due())
        queue.state.update(batches_since_full=0, last_full=6400, idle_since=9400)
        self.assertTrue(queue.full_due())
        self.assertTrue(queue.idle_due())
        queue.state["last_full_main"] = self.base
        self.assertFalse(queue.idle_due())

    def test_idle_main_gets_full_check_without_push(self):
        queue = self.queue(now=lambda: 10000)
        queue.state.update(idle_since=9400)
        self.assertTrue(self.run_cycle(queue))
        self.assertEqual(self.checks[0][0], "full")
        self.assertEqual(self.records()[0]["branches"], [])
        self.assertEqual(self.posts, [])

    def test_failed_full_after_selected_success_reports_gap(self):
        branch = self.branch("red")
        queue = self.queue()
        queue.state["selected_passed"] = True
        self.assertFalse(queue.attempt([branch]))
        self.assertTrue(any(subject == "possible queue selection gap" for _, subject, _ in self.posts))

    def test_dirty_checkout_is_rejected_without_overwriting_it(self):
        branch = self.branch("a")
        self.write("untracked", "keep\n")
        with self.assertRaisesRegex(RuntimeError, "uncommitted or untracked"):
            mq.build_batch(self.repo, [branch])
        self.assertEqual((self.repo / "untracked").read_text(), "keep\n")

    def test_empty_queue_is_noop_even_in_dirty_checkout(self):
        self.write("untracked", "keep\n")
        self.assertFalse(self.run_cycle(self.queue(dry=True)))
        self.assertEqual(self.checks, [])


class PostTests(unittest.TestCase):
    def test_post_uses_queue_identity_and_treats_exit_three_as_delivered(self):
        result = subprocess.CompletedProcess([], 3, "queued, nobody will wake it")
        with patch.object(mq, "command", return_value=result) as command:
            outcome = mq.post("fixture@example.test", "a subject", "body\n")
        self.assertEqual(command.call_args.args[0],
                         ["post", "send", "--as", "queue", "fixture@example.test", "a subject"])
        self.assertEqual(command.call_args.kwargs["body"], "body\n")
        self.assertTrue(outcome["sent"])

    def test_metadata_json_is_separate_from_diagnostics(self):
        result = mq.command([sys.executable, "-I", "-B", "-c",
                             'import sys; print("{}"); print("diagnostic", file=sys.stderr)'],
                            ROOT, separate_stderr=True)
        self.assertEqual(json.loads(result.stdout), {})
        self.assertIn("diagnostic", result.stderr)


if __name__ == "__main__":
    unittest.main()
