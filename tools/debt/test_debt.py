"""Debt indexes checked in repositories built per test, plus this repository."""
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
TREES = ("skein", "smith", "jig", "temper", "hearth")
NOTE = "- **The highest ID used is D010.** The branch that adds an item raises it.\n"
ITEM = ("# D002: Title\n\n- **Kind:** dead code\n- **Value:** high (reason)\n"
        "- **Size:** M (reason)\n- **Status:** open (reason)\n\n## What\n")


class Debt(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory(prefix="debt-")
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.write("tools/debt/debt.py", (ROOT / "tools/debt/debt.py").read_text())
        for tree in TREES:
            self.write(f"{tree}/docs/development/debt/README.md", NOTE)

    def write(self, path, text):
        path = self.root / path
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text)

    def item(self, name="D002-item.md", text=ITEM):
        path = f"skein/docs/development/debt/{name}"
        self.write(path, text)
        return path

    def run_check(self, *args, status=0, root=None):
        result = subprocess.run((sys.executable, "-I", "-B", "tools/debt/debt.py", *args),
                                cwd=root or self.root, capture_output=True, text=True)
        self.assertEqual(result.returncode, status, result.stdout + result.stderr)
        if status:
            self.assertEqual(result.stdout, "")
        else:
            self.assertEqual(result.stderr, "")
        return result

    def test_one_tree(self):
        self.item("D010-item.md", ITEM.replace("D002", "D010"))
        self.item()
        self.item("D1-item.md", ITEM.replace("D002", "D1"))
        self.assertEqual(self.run_check("skein").stdout,
                         "## skein\n\n| ID | title | kind | value | size | status |\n"
                         "|---|---|---|---|---|---|\n"
                         "| D1 | Title | dead code | high | M | open |\n"
                         "| D002 | Title | dead code | high | M | open |\n"
                         "| D010 | Title | dead code | high | M | open |\n")

    def test_all_trees(self):
        self.assertEqual([line for line in self.run_check().stdout.splitlines() if line.startswith("## ")],
                         [f"## {tree}" for tree in TREES])

    def test_missing_fields(self):
        for field in ("title", "kind", "value", "size", "status"):
            line = next(line for line in ITEM.splitlines() if
                        line.startswith("# " if field == "title" else f"- **{field.title()}:**"))
            for replacement in ("", line.split(":", 1)[0] + (": " if field == "title" else ":** "),
                                line.split(":", 1)[0] + (": " if field == "title" else ":** (reason)")):
                with self.subTest(field=field, replacement=replacement):
                    broken = ITEM.replace(line, replacement)
                    first = self.item(text=broken)
                    second = self.item("D003-item.md", broken.replace("D002", "D003"))
                    self.assertEqual(self.run_check("skein", status=1).stderr,
                                     f"{first}: no {field}\n{second}: no {field}\n")
        self.item(text=ITEM.replace("# D002:", "# D003:"))
        self.item("D003-item.md", ITEM.replace("D002", "D003"))
        self.assertIn("no title", self.run_check("skein", status=1).stderr)
        self.item(text=ITEM.replace("- **Status:** open (reason)\n", "") + "- **Status:** open\n")
        self.assertIn("no status", self.run_check("skein", status=1).stderr)

    def test_non_items(self):
        self.write("skein/docs/development/debt/README.md", "# Not an item\n" + NOTE)
        self.item("reviews/2026-11-04-x.md", "not an item")
        self.assertNotIn("| D", self.run_check("skein").stdout)

    def test_highest_note(self):
        path = "skein/docs/development/debt/README.md"
        self.item()
        for note, error in (("", "no highest ID"), (NOTE.replace("D010", "D001"),
                                                   "highest ID D001 is below D002")):
            with self.subTest(note=note):
                self.write(path, note)
                self.assertEqual(self.run_check("skein", status=1).stderr, f"{path}: {error}\n")

    def test_usage(self):
        for args in (("unknown",), ("skein", "smith")):
            self.assertIn("usage:", self.run_check(*args, status=2).stderr)
        shutil.rmtree(self.root / "skein")
        self.assertIn("usage:", self.run_check("skein", status=2).stderr)

    def test_repository(self):
        output = self.run_check(root=ROOT).stdout
        self.assertEqual(sum(line.startswith("| D") for line in output.splitlines()),
                         sum(len(list((ROOT / tree / "docs/development/debt").glob("D[0-9]*-*.md")))
                             for tree in TREES))
        for tree in TREES:
            section = output.split(f"## {tree}\n", 1)[1].split("\n## ", 1)[0]
            for path in (ROOT / tree / "docs/development/debt").glob("D[0-9]*-*.md"):
                self.assertIn(f"| {path.name.split('-', 1)[0]} |", section)


if __name__ == "__main__":
    unittest.main()
