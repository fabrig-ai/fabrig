"""Queue policy and effects against disposable local repositories; no real posts."""

from contextlib import redirect_stderr, redirect_stdout
import importlib.machinery
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

    def test_only_heavy_exit_two_with_disk_floor_text_is_a_refusal(self):
        for code, output, refused in ((2, "builder: 39 GB below the disk floor", True),
                                      (1, "below the disk floor", False),
                                      (2, "test failed", False)):
            with self.subTest(code=code, output=output):
                def runner(argv, **kwargs):
                    heavy = argv[0] == "heavy"
                    if heavy:
                        kwargs["stdout"].write(output)
                    return subprocess.CompletedProcess(argv, code if heavy else 0)

                with tempfile.TemporaryDirectory(dir=ROOT, prefix=".mq-test-") as directory:
                    steps = mq.run_checks(ROOT, "full", [], Path(directory) / "checks.log", runner=runner)
                self.assertEqual("error" in steps[-1], refused)
                self.assertEqual(len(steps), 2)


class CleanupTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(dir=ROOT, prefix=".mq-test-")
        self.addCleanup(self.temporary.cleanup)
        self.repo = Path(self.temporary.name)
        self.deps = self.repo / "target/debug/deps"
        self.deps.mkdir(parents=True)
        self.batch_start = 10_000_000_000

    def artifact(self, name, *, mtime=1_000_000_000, executable=True):
        path = self.deps / name
        path.write_bytes(b"test binary")
        path.chmod(0o700 if executable else 0o600)
        os.utime(path, ns=(mtime, mtime))
        return path

    def test_prunes_oldest_first_until_floor_plus_forty_is_free(self):
        oldest = self.artifact("oldest", mtime=1_000_000_000)
        middle = self.artifact("middle", mtime=2_000_000_000)
        newest = self.artifact("newest", mtime=3_000_000_000)
        with patch.object(mq, "free_gb", side_effect=[84, 84, 85]):
            report = mq.prune_test_executables(self.repo, 45, self.batch_start)
        self.assertFalse(oldest.exists())
        self.assertTrue(middle.exists() and newest.exists())
        self.assertEqual(report, {"count": 1, "gb": 11 / mq.GB})

    def test_protects_libraries_nonexecutables_and_batch_outputs(self):
        protected = [self.artifact("fresh", mtime=self.batch_start),
                     self.artifact("newer", mtime=self.batch_start + 1),
                     self.artifact("ordinary", executable=False)]
        protected += [self.artifact("library" + suffix) for suffix in (".rlib", ".rmeta", ".so", ".d")]
        old = self.artifact("old")
        (self.deps / "symlink").symlink_to(old)
        (self.deps / "directory").mkdir()
        with patch.object(mq, "free_gb", return_value=0):
            report = mq.prune_test_executables(self.repo, 45, self.batch_start)
        self.assertEqual(report["count"], 1)
        self.assertTrue(all(path.exists() for path in protected))
        self.assertTrue((self.deps / "symlink").is_symlink())
        self.assertTrue((self.deps / "directory").is_dir())

    def test_no_cleanup_at_space_goal(self):
        old = self.artifact("old")
        with patch.object(mq, "free_gb", return_value=85):
            self.assertEqual(mq.prune_test_executables(self.repo, 45, self.batch_start),
                             {"count": 0, "gb": 0.0})
        self.assertTrue(old.exists())

    def test_rewritten_candidate_is_preserved(self):
        old = self.artifact("old")
        calls = []

        def space(repo):
            calls.append(repo)
            if len(calls) == 2:
                os.utime(old, ns=(self.batch_start, self.batch_start))
            return 0

        with patch.object(mq, "free_gb", side_effect=space):
            self.assertEqual(mq.prune_test_executables(self.repo, 45, self.batch_start)["count"], 0)
        self.assertTrue(old.exists())

    def test_symlinked_deps_directory_is_preserved(self):
        old = self.artifact("old")
        moved = self.deps.with_name("other")
        self.deps.rename(moved)
        self.deps.symlink_to(moved, target_is_directory=True)
        with patch.object(mq, "free_gb", return_value=0):
            self.assertEqual(mq.prune_test_executables(self.repo, 45, self.batch_start)["count"], 0)
        self.assertTrue(old.exists())


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
        kwargs.setdefault("disk_floor", 0)
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

    def ratchet_base(self, trees=("skein",)):
        for tree in trees:
            self.write(f"{tree}/scripts/old-forms.txt", "3\n")
            self.write(f"{tree}/src/a.rs", "old old\n")
            self.write(f"{tree}/src/b.rs", "old\n")
            self.write(f"{tree}/scripts/old_forms.py", r"""from pathlib import Path
import sys
baseline = Path('scripts/old-forms.txt')
count = sum(p.read_text().count('old') for p in Path('src').glob('*.rs'))
if count > int(baseline.read_text()):
    print('count rose above HEAD baseline')
    sys.exit(1)
baseline.write_text(str(count) + '\n')
""")
        self.commit("ratchet fixture")
        self.base = self.tip()
        mq.git(self.repo, "push", "origin", "HEAD:main")
        mq.git(self.repo, "fetch", "origin")

    def ratchet_branch(self, task, source, baseline, trees=("skein",), *, publish=False):
        mq.git(self.repo, "switch", "--detach", self.base)
        for tree in trees:
            self.write(f"{tree}/src/{source}.rs", "")
            self.write(f"{tree}/scripts/old-forms.txt", f"{baseline}\n")
        self.commit(task)
        branch = {"name": "ready/coord/" + task, "tip": self.tip(), "timestamp": 1}
        if publish:
            mq.git(self.repo, "push", "origin", f"HEAD:refs/heads/{branch['name']}")
        return branch

    def test_ratchet_only_conflicts_recount_and_amend_each_tree(self):
        trees = ("skein", "smith")
        self.ratchet_base(trees)
        first = self.ratchet_branch("a", "a", 1, trees)
        second = self.ratchet_branch("b", "b", 2, trees)
        merged, conflicts = mq.build_batch(self.repo, [first, second])
        self.assertEqual(conflicts, [])
        self.assertEqual([branch["name"] for branch in merged], [first["name"], second["name"]])
        for tree in trees:
            self.assertEqual((self.repo / tree / "scripts/old-forms.txt").read_text(), "0\n")
            self.assertEqual(mq.git(self.repo, "show", f"HEAD:{tree}/scripts/old-forms.txt").stdout, "0\n")
        self.assertEqual(len(mq.git(self.repo, "show", "-s", "--format=%P").stdout.split()), 2)
        self.assertEqual(mq.git(self.repo, "show", "-s", "--format=%s").stdout.strip(),
                         "Merge ready/coord/b")
        self.assertEqual(mq.git(self.repo, "status", "--porcelain").stdout, "")

    def test_ratchet_rise_rejects_branch_and_restores_batch_so_far(self):
        self.ratchet_base()
        first = self.ratchet_branch("a", "a", 1, publish=True)
        mq.git(self.repo, "switch", "--detach", self.base)
        self.write("skein/scripts/old-forms.txt", "2\n")
        self.write("skein/src/b.rs", "old old\n")
        self.commit("raise combined count")
        second = {"name": "ready/coord/b", "tip": self.tip(), "timestamp": 1}
        mq.git(self.repo, "push", "origin", f"HEAD:refs/heads/{second['name']}")
        queue = self.queue()
        self.run_cycle(queue)
        self.assertEqual((self.repo / "skein/src/a.rs").read_text(), "")
        self.assertEqual((self.repo / "skein/src/b.rs").read_text(), "old\n")
        record = self.records()[0]
        self.assertEqual(record["conflicts"][0]["step"], "ratchet:skein")
        self.assertIn("count rose above HEAD baseline", record["conflicts"][0]["error"])
        self.assertTrue(any("failing step: ratchet:skein" in body for _, _, body in self.posts))
        self.assertEqual(queue.state["tried"][second["name"]], second["tip"])
        self.assertEqual(record["branches"][0]["tip"], first["tip"])

    def test_mixed_ratchet_and_source_conflict_is_aborted(self):
        self.ratchet_base()
        first = self.ratchet_branch("a", "a", 1)
        second = self.ratchet_branch("b", "b", 2)
        self.write("skein/src/a.rs", "other\n")
        self.commit("source conflict")
        second["tip"] = self.tip()
        with patch.object(mq, "merge_ratchets", side_effect=AssertionError("mixed conflict")):
            merged, conflicts = mq.build_batch(self.repo, [first, second])
        self.assertEqual(len(merged), 1)
        self.assertNotIn("step", conflicts[0])
        self.assertEqual((self.repo / "skein/scripts/old-forms.txt").read_text(), "1\n")

    def test_merge_commits_name_the_pinned_branch(self):
        branches = [self.branch("first"), self.branch("last")]
        merged, _ = mq.build_batch(self.repo, branches)
        self.assertEqual([mq.git(self.repo, "show", "-s", "--format=%s", branch["commit"]).stdout.strip()
                          for branch in merged], ["Merge ready/coord/first", "Merge ready/coord/last"])

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

    def run_cycles(self, queue, count):
        stop = self.directory / "stop"
        sleeps = []

        def sleeper(seconds):
            sleeps.append(seconds)
            if len(sleeps) == count:
                stop.touch()

        with redirect_stdout(io.StringIO()):
            queue.run(stop, sleeper=sleeper)
        return sleeps

    def test_run_recovers_errors_and_reports_each_distinct_text_once(self):
        branch = self.branch("a", publish=True)
        queue = self.queue()
        failures = iter(["gate guard", "gate guard", "git failed"])

        def fail(repo, selected, paths, log):
            raise RuntimeError(next(failures))

        queue.checker = fail
        self.assertEqual(self.run_cycles(queue, 3), [60, 60, 60])
        self.assertEqual(self.tip(), self.base)
        self.assertEqual(mq.git(self.repo, "branch", "--show-current").stdout.strip(), "")
        self.assertEqual(queue.state["tried"], {})
        self.assertEqual([body.splitlines()[0] for _, _, body in self.posts], ["gate guard", "git failed"])
        batches = [record for record in self.records() if "event" not in record]
        self.assertEqual([record["outcome"] for record in batches], ["error"] * 3)
        self.assertTrue(all(record["branches"][0]["tip"] == branch["tip"] for record in batches))
        self.assertTrue(self.run_cycle(self.queue()))
        self.assertTrue((self.repo / "a").exists())

    def test_run_lands_batch_on_next_cycle_after_error(self):
        self.branch("a", publish=True)
        queue = self.queue()
        calls = []
        stop = self.directory / "stop"

        def checker(*args):
            calls.append(args)
            if len(calls) == 1:
                raise RuntimeError("temporary git failure")
            stop.touch()
            return self.checker(*args)

        queue.checker = checker
        with redirect_stdout(io.StringIO()):
            queue.run(stop, sleeper=lambda seconds: None)
        self.assertEqual(len(calls), 2)
        self.assertTrue((self.repo / "a").exists())
        self.assertEqual(self.records()[-1]["outcome"], "green")
        self.assertEqual(len([subject for _, subject, _ in self.posts if subject == "queue error"]), 1)

    def test_run_survives_recovery_log_failure(self):
        queue = self.queue()
        queue.cycle = lambda: (_ for _ in ()).throw(RuntimeError("git failed"))
        output = io.StringIO()
        with patch.object(mq, "log_batch", side_effect=OSError("disk full")), redirect_stderr(output):
            self.assertEqual(self.run_cycles(queue, 2), [60, 60])
        self.assertIn("recovery failed: disk full", output.getvalue())
        self.assertEqual(len(self.posts), 1)

    def test_run_aborts_unfinished_merge_and_preserves_ignored_target(self):
        self.write(".gitignore", "/target/\n")
        self.commit("ignore build output")
        self.base = self.tip()
        mq.git(self.repo, "push", "origin", "HEAD:main")
        mq.git(self.repo, "fetch", "origin")
        first = self.branch("first", "shared", "first\n")
        second = self.branch("second", "shared", "second\n")
        mq.build_batch(self.repo, [first])
        mq.git(self.repo, "merge", "--no-ff", "--no-edit", second["tip"], check=False)
        self.write("target/kept", "build output\n")
        queue = self.queue()
        queue.cycle = lambda: (_ for _ in ()).throw(RuntimeError("unfinished merge"))
        self.run_cycles(queue, 1)
        self.assertEqual(self.tip(), self.base)
        self.assertNotEqual(mq.git(self.repo, "rev-parse", "--verify", "--quiet", "MERGE_HEAD",
                                   check=False).returncode, 0)
        self.assertEqual((self.repo / "target/kept").read_text(), "build output\n")

    def test_error_does_not_reject_conflicted_branch_in_same_batch(self):
        for task, text in (("a", "first\n"), ("b", "second\n")):
            self.branch(task, "shared", text, publish=True)
        queue = self.queue()
        queue.checker = lambda *args: (_ for _ in ()).throw(RuntimeError("gate guard"))
        self.run_cycles(queue, 1)
        self.assertEqual(queue.state["tried"], {})
        self.assertFalse(any("rejected" in subject for _, subject, _ in self.posts))

    def test_root_and_tree_targets_are_ignored(self):
        paths = ["target/fixture"] + [f"{tree}/target/fixture" for tree in mq.TREES]
        self.assertEqual(mq.git(ROOT, "check-ignore", *paths).stdout.splitlines(), paths)

    def test_heavy_disk_refusal_errors_without_rejection_or_bisection(self):
        for task in ("a", "b"):
            self.branch(task, publish=True)
        queue = self.queue()
        attempts = []

        def refuse(*args):
            attempts.append(args)
            return [{"name": "clippy", "green": False, "returncode": 2,
                     "error": "builder: 39 GB free, below the disk floor"}]

        queue.checker = refuse
        self.run_cycles(queue, 2)
        batches = [record for record in self.records() if "event" not in record]
        self.assertEqual([record["outcome"] for record in batches], ["error", "error"])
        self.assertEqual(len(attempts), 2)
        self.assertEqual(queue.state["tried"], {})
        self.assertEqual(len(self.posts), 1)
        self.assertIn("builder: 39 GB", self.posts[0][2])
        self.assertEqual(self.tip(), self.base)

    def test_low_disk_waits_without_starting_a_batch_and_posts_once(self):
        self.branch("a", publish=True)
        queue = self.queue(disk_floor=45)
        with patch.object(mq, "free_gb", side_effect=[44, 43]):
            self.assertEqual(self.run_cycles(queue, 2), [60, 60])
        self.assertEqual(self.checks, [])
        self.assertEqual(queue.state["tried"], {})
        self.assertEqual(len(self.posts), 1)
        self.assertIn("44.0 GB free", self.posts[0][2])
        self.assertTrue(all(record["branches"] == [] for record in self.records()))

    def test_disk_floor_allows_a_batch_at_the_boundary(self):
        self.branch("a", publish=True)
        with patch.object(mq, "free_gb", return_value=45):
            self.assertTrue(self.run_cycle(self.queue(disk_floor=45)))
        self.assertEqual(self.records()[0]["outcome"], "green")

    def test_every_batch_logs_cleanup_for_green_red_and_error(self):
        for task, outcome in (("a", "green"), ("red", "red"), ("error", "error")):
            with self.subTest(outcome=outcome):
                branch = self.branch(task)
                queue = self.queue()
                if outcome == "error":
                    queue.checker = lambda *args: (_ for _ in ()).throw(RuntimeError("gate guard"))
                with patch.object(mq, "prune_test_executables", return_value={"count": 2, "gb": 3}) as prune:
                    if outcome == "error":
                        with self.assertRaisesRegex(RuntimeError, "gate guard"):
                            queue.attempt([branch])
                    else:
                        queue.attempt([branch])
                self.assertEqual(prune.call_count, 1)
                self.assertEqual(prune.call_args.args[:2], (self.repo, 0))
                self.assertGreater(prune.call_args.args[2], 0)
                record = self.records()[-1]
                self.assertEqual(record["outcome"], outcome)
                self.assertEqual(record["target_cleanup"], {"count": 2, "gb": 3})

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
