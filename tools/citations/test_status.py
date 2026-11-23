"""Focused status roll-up checks against small, freshly committed repositories."""
import importlib.util
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

SCRIPT = Path(__file__).with_name("status.py").resolve()
README = SCRIPT.with_name("README.md")
DESIGN = "temper/docs/design"
OPENING = "# a: status\n\n2026-11-20. Where the code lags or departs from [a.md](a.md).\n\n"
ZERO = "0 Not yet (0 in flight), 0 Departs, 0 Design lags, 0 Design departs, 0 Open"


def expected(tree="temper", counts=ZERO, opened="- none", over="- none",
             done="- none", unread="- none"):
    """The complete output shape; rows supply literal expected entries."""
    return (f"# {tree}\n\nOpen entries: {counts}.\n\n"
            f"Status files with an open entry:\n{opened}\n\n"
            f"Over the mark, 49,152 bytes, largest first:\n{over}\n\n"
            f"Drafts done, outside draft/archive/:\n{done}\n\n"
            f"Unread status files:\n{unread}\n")


class Status(unittest.TestCase):
    def repository(self, files):
        temporary = tempfile.TemporaryDirectory(prefix="status-")
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.git("init", "-q", "-b", "main")
        self.git("config", "user.email", "test@example.invalid")
        self.git("config", "user.name", "test")
        for path, text in files.items():
            self.write(path, text)
        self.commit()

    def git(self, *args):
        return subprocess.run(("git", *args), cwd=self.root, check=True,
                              capture_output=True, text=True).stdout.strip()

    def write(self, path, text):
        target = self.root / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(text, encoding="utf-8")

    def commit(self):
        self.git("add", "-A")
        self.git("commit", "--allow-empty", "-qm", "fixture")

    def run_status(self, *args):
        return subprocess.run((sys.executable, "-I", "-B", str(SCRIPT), *args),
                              cwd=self.root, capture_output=True, text=True)

    def test_committed_trees(self):
        rows = [
            ("sections", {f"{DESIGN}/a.status.md": OPENING
                          + "- **Not yet:** 3.2: a. 5: b\n- **Departs:** none.\n"},
             expected(counts="2 Not yet (0 in flight), 0 Departs, 0 Design lags, 0 Design departs, 0 Open",
                      opened=f"- {DESIGN}/a.status.md, 2026-11-20: 2 Not yet")),
            ("nested", {f"{DESIGN}/a.status.md": OPENING
                        + "- **Not yet:**\n  - 3.2: x\n  - 4 and 5: y\n"},
             expected(counts="2 Not yet (0 in flight), 0 Departs, 0 Design lags, 0 Design departs, 0 Open",
                      opened=f"- {DESIGN}/a.status.md, 2026-11-20: 2 Not yet")),
            ("prose and other bold words", {f"{DESIGN}/a.status.md": OPENING
                         + "Ignored preface 8: not an entry.\n"
                         + "- **Not yet:** No section label; **Later:** is text.\n"
                         + "- **Departs:** None\n"},
             expected(counts="1 Not yet (0 in flight), 0 Departs, 0 Design lags, 0 Design departs, 0 Open",
                      opened=f"- {DESIGN}/a.status.md, 2026-11-20: 1 Not yet")),
            ("in flight", {f"{DESIGN}/a.status.md": OPENING
                           + "- **Not yet:** 4: x [in flight: change 12].\n"},
             expected(counts="1 Not yet (1 in flight), 0 Departs, 0 Design lags, 0 Design departs, 0 Open",
                      opened=f"- {DESIGN}/a.status.md, 2026-11-20: 1 Not yet (1 in flight)")),
            ("tag must end entry", {f"{DESIGN}/a.status.md": OPENING
                       + "- **Not yet:** 4: x [in flight: change 12] more\n"
                       + "  - 5: x [in flight: change 12] .\n"
                       + "  - 6: x [in flight: change 12]).\n"
                       + "  - 7: x[in flight: change 12].\n"
                       + "- **Departs:** 1: x [in flight: change 12].\n"},
             expected(counts="4 Not yet (0 in flight), 1 Departs, 0 Design lags, 0 Design departs, 0 Open",
                      opened=f"- {DESIGN}/a.status.md, 2026-11-20: 4 Not yet, 1 Departs")),
            ("nested tags", {f"{DESIGN}/a.status.md": OPENING
                            + "- **Not yet:**\n  - 3.2: x [in flight: change 12].\n"
                            + "  - 4 and 5: y\n  - 6: z [in flight: outside];!?\n"},
             expected(counts="3 Not yet (2 in flight), 0 Departs, 0 Design lags, 0 Design departs, 0 Open",
                      opened=f"- {DESIGN}/a.status.md, 2026-11-20: 3 Not yet (2 in flight)")),
            ("section boundaries and lists", {f"{DESIGN}/a.status.md": OPENING
                       + "- **Not yet:** 3.2 and 4: a. 5, 6: b\n"
                       + "7 or 8: c\n  - 9 to 10: d\n  - 11–12: e\n"
                       + "prose mentions 13: not at a boundary\n"},
             expected(counts="5 Not yet (0 in flight), 0 Departs, 0 Design lags, 0 Design departs, 0 Open",
                      opened=f"- {DESIGN}/a.status.md, 2026-11-20: 5 Not yet")),
            ("done forms", {
                f"{DESIGN}/draft/a.status.md": OPENING + "- **Not yet:** none.\n"
                + "- **Departs:** None\n- **Design lags:** NONE.\n"
                + "- **Design departs:** none\n- **Open:** none.\n",
                f"{DESIGN}/draft/b.status.md": OPENING + "No gap.\n",
                f"{DESIGN}/draft/c.status.md": OPENING,
                f"{DESIGN}/draft/archive/d.status.md": OPENING + "No gap.\n",
                f"{DESIGN}/e.status.md": OPENING + "No gap.\n"},
             expected(done=f"- {DESIGN}/draft/a.md\n- {DESIGN}/draft/b.md\n- {DESIGN}/draft/c.md")),
            ("notes and execution are not drafts", {
                f"{DESIGN}/draft/a.notes.status.md": OPENING + "No gap.\n",
                f"{DESIGN}/draft/a.execution.status.md": OPENING + "No gap.\n",
                f"{DESIGN}/draft/b.status.md": OPENING + "No gap.\n",
                f"{DESIGN}/draft/archive/a.notes.status.md": OPENING + "No gap.\n",
                f"{DESIGN}/draft/archive/a.execution.status.md": OPENING + "No gap.\n"},
             expected(done=f"- {DESIGN}/draft/b.md")),
            ("notes and execution statuses still read", {
                f"{DESIGN}/draft/a.notes.status.md": OPENING + "- **Open:** decision\n",
                f"{DESIGN}/draft/a.execution.status.md": OPENING + "- **Not yet:** 1: x\n",
                f"{DESIGN}/draft/b.notes.status.md": OPENING + "No known gaps.\n",
                f"{DESIGN}/draft/b.execution.status.md": ""},
             expected(counts="1 Not yet (0 in flight), 0 Departs, 0 Design lags, 0 Design departs, 1 Open",
                      opened=f"- {DESIGN}/draft/a.execution.status.md, 2026-11-20: 1 Not yet\n"
                      + f"- {DESIGN}/draft/a.notes.status.md, 2026-11-20: 1 Open",
                      unread=f"- {DESIGN}/draft/b.execution.status.md\n"
                      + f"- {DESIGN}/draft/b.notes.status.md")),
            ("unread", {
                f"{DESIGN}/draft/a.status.md": OPENING + "No known gaps.\n",
                f"{DESIGN}/draft/b.status.md": "# b: status\n\n- **Not yet:** 1: x\n",
                f"{DESIGN}/draft/c.status.md": "2026-11-20. No heading.\n\nNo gap.\n",
                f"{DESIGN}/draft/d.status.md": "",
                f"{DESIGN}/draft/e.status.md": "# e: status\n\nUndated.\n\n- **Open:** x\n"},
             expected(unread="\n".join(f"- {DESIGN}/draft/{name}.status.md" for name in "abcde"))),
            ("kinds and sorted files", {
                f"{DESIGN}/z.status.md": OPENING + "- **Design departs:** 2: x\n",
                f"{DESIGN}/draft/archive/b.status.md": OPENING + "- **Open:** x\n",
                f"{DESIGN}/draft/a.status.md": OPENING
                + "- **Design lags:** 3: x\n- **Design departs:** 4: x\n- **Open:** decision\n"},
             expected(counts="0 Not yet (0 in flight), 0 Departs, 1 Design lags, 2 Design departs, 2 Open",
                      opened=f"- {DESIGN}/draft/a.status.md, 2026-11-20: 1 Design lags, 1 Design departs, 1 Open\n"
                      + f"- {DESIGN}/draft/archive/b.status.md, 2026-11-20: 1 Open\n"
                      + f"- {DESIGN}/z.status.md, 2026-11-20: 1 Design departs")),
            ("indented labels", {f"{DESIGN}/a.status.md": "\n" + OPENING
                        + "before\n  - **Not yet:** 1: x\n  - **Departs:** prose\n"},
             expected(counts="1 Not yet (0 in flight), 1 Departs, 0 Design lags, 0 Design departs, 0 Open",
                      opened=f"- {DESIGN}/a.status.md, 2026-11-20: 1 Not yet, 1 Departs")),
            ("byte mark and ordering", {
                f"{DESIGN}/z.md": "x" * 49_153,
                f"{DESIGN}/a.md": "x" * 49_153,
                f"{DESIGN}/at.md": "x" * 49_152,
                f"{DESIGN}/draft/archive/b.md": "é" * 25_000,
                f"{DESIGN}/big.status.md": "x" * 49_154,
                "temper/docs/elsewhere.md": "x" * 60_000,
                "smith/docs/design/a.md": "x" * 60_000},
             expected(over=f"- {DESIGN}/draft/archive/b.md: 50,000 bytes\n"
                      + f"- {DESIGN}/big.status.md: 49,154 bytes\n"
                      + f"- {DESIGN}/a.md: 49,153 bytes\n- {DESIGN}/z.md: 49,153 bytes",
                      unread=f"- {DESIGN}/big.status.md")),
            ("done paths sort by document", {
                f"{DESIGN}/draft/a.n.status.md": OPENING,
                f"{DESIGN}/draft/a.status.md": OPENING},
             expected(done=f"- {DESIGN}/draft/a.md\n- {DESIGN}/draft/a.n.md")),
            ("tag-only text", {f"{DESIGN}/a.status.md": OPENING
                              + "- **Not yet:** 4: [in flight: change 12].\n"},
             expected(counts="1 Not yet (1 in flight), 0 Departs, 0 Design lags, 0 Design departs, 0 Open",
                      opened=f"- {DESIGN}/a.status.md, 2026-11-20: 1 Not yet (1 in flight)")),
            ("empty", {}, expected()),
        ]
        for name, files, output in rows:
            with self.subTest(name=name):
                self.repository(files)
                result = self.run_status("temper")
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertEqual(result.stderr, "")
                self.assertEqual(result.stdout, output)

    def test_tree_order(self):
        self.repository({})
        for trees, names in [((), ("temper", "smith", "skein", "jig", "hearth")),
                             (("hearth", "temper", "smith"), ("hearth", "temper", "smith"))]:
            with self.subTest(trees=trees):
                result = self.run_status(*trees)
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertEqual(result.stdout, "\n".join(expected(tree=name) for name in names))

    def test_committed_revision_only(self):
        path = f"{DESIGN}/a.status.md"
        self.repository({path: OPENING + "- **Not yet:** 1: x\n"})
        revision = self.git("rev-parse", "HEAD")
        original = expected(counts="1 Not yet (0 in flight), 0 Departs, 0 Design lags, 0 Design departs, 0 Open",
                            opened=f"- {path}, 2026-11-20: 1 Not yet")
        self.write(path, OPENING + "No gap.\n")
        result = self.run_status("temper")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout, original)
        self.commit()
        result = self.run_status("--head", revision, "temper")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout, original)
        result = self.run_status("temper")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout, expected())

    def test_page_without_repository(self):
        temporary = tempfile.TemporaryDirectory(prefix="status-page-")
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        text = README.read_text(encoding="utf-8").split("- **The page,**", 1)[1].split("  ```", 2)[1]
        template = "\n".join(line.removeprefix("  ") for line in text.strip("\n").splitlines()) + "\n"
        spec = importlib.util.spec_from_file_location("status", SCRIPT)
        status = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(status)
        for tree in ("temper", "hearth"):
            with self.subTest(tree=tree):
                wanted = template.replace("TREE", tree)
                self.assertEqual(status.page(tree), wanted)
                result = self.run_status("--page", tree)
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertEqual(result.stdout, wanted)
                self.assertEqual(result.stderr, "")
        with self.assertRaises(ValueError):
            status.page("unknown")

    def test_usage_and_git_failures(self):
        self.repository({})
        for args in [("unknown",), ("--page",), ("--page", "temper", "smith"),
                     ("--unknown",), ("--head",), ("--head", "no-such-revision")]:
            with self.subTest(args=args):
                result = self.run_status(*args)
                self.assertEqual(result.returncode, 2)
                self.assertNotEqual(result.stderr, "")
                self.assertEqual(result.stdout, "")
        temporary = tempfile.TemporaryDirectory(prefix="status-no-git-")
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        result = self.run_status("temper")
        self.assertEqual(result.returncode, 2)
        self.assertNotEqual(result.stderr, "")
        self.assertEqual(result.stdout, "")


if __name__ == "__main__":
    unittest.main()
