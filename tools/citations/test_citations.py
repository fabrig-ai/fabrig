"""Regression checks for the citation checker, on small repositories built per test."""
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

SCRIPT = Path(__file__).with_name("citations.py").resolve()
DESIGN = "skein/docs/design"
DOC = "# a\n\n## 1. One\n\nText.\n\n## 2. Two\n\n### 2.1 Part\n\nText.\n"


class Citations(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory(prefix="citations-")
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.git("init", "-q", "-b", "main")
        self.git("config", "user.email", "test@example.invalid")
        self.git("config", "user.name", "test")
        self.write(f"{DESIGN}/a.md", DOC)
        self.write(f"{DESIGN}/b.md", "# b\n\n## 1. One\n\nAs `a.md`, 2.1 says.\n")
        self.commit("base")

    def git(self, *args):
        return subprocess.run(("git", *args), cwd=self.root, check=True, capture_output=True, text=True).stdout

    def write(self, rel, text):
        path = self.root / rel
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text)

    def commit(self, message):
        self.git("add", "-A")
        self.git("commit", "-qm", message)

    def run_check(self, *args):
        return subprocess.run((sys.executable, "-I", "-B", str(SCRIPT), *args), cwd=self.root,
                              capture_output=True, text=True)

    def since_base(self):
        return self.run_check("--base", "HEAD~1")

    def test_a_clean_change_passes(self):
        self.write(f"{DESIGN}/c.md", "# c\n\n## 1. One\n\nSee `a.md`, section 2, and [b](b.md).\n")
        self.commit("head")
        result = self.since_base()
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_a_dead_path_fails(self):
        self.write(f"{DESIGN}/c.md", "# c\n\n## 1. One\n\nSee `gone.md`.\n")
        self.commit("head")
        result = self.since_base()
        self.assertEqual(result.returncode, 1)
        self.assertIn(f"{DESIGN}/c.md:5: path gone.md", result.stdout)

    def test_removing_a_cited_section_fails_where_it_is_cited(self):
        self.write(f"{DESIGN}/a.md", DOC.replace("### 2.1 Part", "### 2.2 Part"))
        self.commit("head")
        result = self.since_base()
        self.assertEqual(result.returncode, 1)
        self.assertIn(f"{DESIGN}/b.md:5: section {DESIGN}/a.md 2.1", result.stdout)

    def test_a_broken_citation_already_on_the_base_passes(self):
        self.write(f"{DESIGN}/b.md", "# b\n\n## 1. One\n\nAs `a.md`, 9 says.\n")
        self.commit("broken base")
        self.write(f"{DESIGN}/c.md", "# c\n\n## 1. One\n\nText.\n")
        self.commit("head")
        self.assertEqual(self.since_base().returncode, 0)
        result = self.run_check("--all")
        self.assertEqual(result.returncode, 1)
        self.assertIn(f"{DESIGN}/a.md 9", result.stdout)

    def test_a_second_copy_of_a_broken_citation_fails(self):
        self.write(f"{DESIGN}/b.md", "# b\n\n## 1. One\n\nAs `a.md`, 9 says.\n")
        self.commit("broken base")
        self.write(f"{DESIGN}/b.md", "# b\n\n## 1. One\n\nAs `a.md`, 9 says.\n\nAgain `a.md`, 9.\n")
        self.commit("head")
        self.assertEqual(self.since_base().returncode, 1)

    def test_a_renamed_file_keeps_its_old_findings(self):
        self.write(f"{DESIGN}/b.md", "# b\n\n## 1. One\n\nAs `a.md`, 9 says, at length.\n")
        self.commit("broken base")
        self.git("mv", f"{DESIGN}/b.md", f"{DESIGN}/c.md")
        self.commit("rename")
        result = self.since_base()
        self.assertEqual(result.returncode, 0, result.stdout)

    def test_code_is_read_in_comments_only(self):
        self.write("skein/crates/k/src/lib.rs",
                   '//! The kit (skein/docs/design/a.md, 2.1).\n'
                   'const FIXTURE: &str = "Missing changes/40.md";\n'
                   'const URL: &str = r#"{"url":"http://host/raw/notes.md"}"#;\n'
                   'fn f() {} // the rule is gone.md\'s\n')
        self.commit("head")
        result = self.since_base()
        self.assertEqual(result.returncode, 1)
        self.assertNotIn("changes/40.md", result.stdout)
        self.assertNotIn("notes.md", result.stdout)
        self.assertIn("skein/crates/k/src/lib.rs:4: path gone.md", result.stdout)

    def test_other_repositories_dates_and_rfcs_are_not_citations(self):
        self.write(f"{DESIGN}/c.md",
                   "# c\n\n## 1. One\n\nAs hearth's `domain/store.md`, 5.2, and agents' `docs/design/gone.md`, 4.\n\n"
                   "Decided with `a.md`, 2026-10-05; RFC 6749, section 5.1, allows it.\n")
        self.commit("head")
        result = self.since_base()
        self.assertEqual(result.returncode, 0, result.stdout)

    def test_plans_are_not_read(self):
        self.write("skein/docs/plan/p/tasks/t.md", "# t\n\nDelete `../../../design/a.md`, 2.1.\n")
        self.commit("plan")
        self.write(f"{DESIGN}/a.md", DOC.replace("### 2.1 Part", "### 2.2 Part"))
        self.write(f"{DESIGN}/b.md", "# b\n\n## 1. One\n")
        self.commit("head")
        self.assertEqual(self.since_base().returncode, 0)

    def test_allow_txt_excuses_a_listed_citation(self):
        self.write(f"{DESIGN}/c.md", "# c\n\n## 1. One\n\nThe probe's `CONTRIBUTING.md`.\n")
        self.write("tools/citations/allow.txt", f"{DESIGN}/c.* CONTRIBUTING.md # the probe repository's file\n")
        self.commit("head")
        self.assertEqual(self.since_base().returncode, 0)

    def test_an_allow_line_without_a_reason_is_refused(self):
        self.write("tools/citations/allow.txt", f"{DESIGN}/c.md gone.md\n")
        self.commit("head")
        result = self.since_base()
        self.assertEqual(result.returncode, 2)
        self.assertIn("allow.txt:1", result.stderr)

    def test_usage(self):
        self.assertEqual(self.run_check().returncode, 2)
        self.assertEqual(self.run_check("--all", "--base", "HEAD").returncode, 2)


if __name__ == "__main__":
    unittest.main()
