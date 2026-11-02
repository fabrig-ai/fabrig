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

    def test_a_path_two_directories_up_is_read(self):
        self.write(f"{DESIGN}/x/y/c.md",
                   "# c\n\n## 1. One\n\nSee `../../a.md`, 2.1, and `../../a.md`, 7.\n")
        self.commit("head")
        result = self.since_base()
        self.assertEqual(result.returncode, 1)
        self.assertIn(f"section {DESIGN}/a.md 7", result.stdout)
        self.assertNotIn(f"{DESIGN}/a.md 2.1", result.stdout)

    def test_a_path_more_than_two_directories_up_is_read(self):
        self.write(f"{DESIGN}/x/y/z/c.md",
                   "# c\n\nSee `../../../a.md`, 2, and `../../../a.md`, 7.\n")
        self.commit("head")
        result = self.since_base()
        self.assertEqual(result.returncode, 1)
        self.assertIn(f"section {DESIGN}/a.md 7", result.stdout)
        self.assertNotIn(f"{DESIGN}/a.md 2 (", result.stdout)

    def test_a_possessive_or_parenthesised_section_is_the_cited_files(self):
        forms = ("`a.md`'s, section {}", "`a.md`’s, {}", "`a.md`'s section {}",
                 "`a.md` (section {} holds it)", "a.md's, section {}", "a.md's, {}",
                 "a.md's section {}", "a.md (section {} holds it)")
        citations = "\n\n".join(form.format(n) for form in forms for n in ("2.1", "9"))
        self.write(f"{DESIGN}/c.md", "# c\n\n" + citations
                   + "\n\nRead a.md's 32 bytes and `a.md`'s 32 bytes.\n")
        self.commit("head")
        result = self.since_base()
        self.assertEqual(result.returncode, 1)
        findings = result.stdout.splitlines()
        self.assertEqual(len(findings), len(forms), result.stdout)
        self.assertTrue(all(f"section {DESIGN}/a.md 9 (" in line for line in findings), result.stdout)

    def test_its_with_a_capital_is_the_cited_files(self):
        sentences = [f"{case(word)} sections 7 and 8 are stale."
                     for word in ("its", "their", "that", "this")
                     for case in (str.lower, str.title, str.upper)]
        self.write(f"{DESIGN}/c.md", "# c\n\nSee [a](a.md). " + " ".join(sentences)
                   + "\n\nSee section 9 here.\n")
        self.commit("head")
        result = self.since_base()
        self.assertEqual(result.returncode, 1)
        self.assertEqual(len(result.stdout.splitlines()), 1, result.stdout)
        self.assertIn(f"section-self {DESIGN}/c.md 9", result.stdout)

    def test_its_after_another_trees_path_is_that_trees(self):
        self.write("jig/docs/design/domain/e.md", "# e\n\n## 1. One\n")
        self.write("jig/docs/design/domain/t.md", "# t\n\n## 4. Four\n\n## 9. Nine\n")
        self.write("temper/docs/design/domain/t.md", "# t\n\n## 4. Four\n\n## 7. Seven\n\n## 9. Nine\n")
        self.write("temper/docs/design/c.md",
                   "# c\n\njig's `domain/e.md`, section 1, and its `domain/t.md`, sections 4 and 9.\n\n"
                   "jig's `domain/e.md`, section 1, and its `domain/t.md`, sections 4 and 7.\n")
        self.commit("head")
        result = self.since_base()
        self.assertEqual(result.returncode, 1)
        self.assertEqual(len(result.stdout.splitlines()), 1, result.stdout)
        self.assertIn("section jig/docs/design/domain/t.md 7", result.stdout)

    def test_its_uses_the_nearest_explicit_owner_in_the_same_sentence(self):
        self.write("jig/docs/design/e.md", DOC)
        self.write("smith/docs/design/e.md", DOC)
        self.write("smith/docs/design/t.md", DOC)
        self.write("jig/docs/design/t.md", "# t\n")
        self.write("temper/docs/design/t.md", "# t\n\n## 7. Seven\n")
        self.write("temper/docs/design/c.md",
                   "# c\n\njig's `e.md`, smith's `e.md`, and its `t.md`, section 2.1.\n\n"
                   "smith's `e.md`. Its `t.md`, section 7.\n\n"
                   "smith's `e.md`, but `t.md`, section 7.\n")
        self.commit("head")
        result = self.since_base()
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_an_rfcs_parenthesised_section_is_the_rfcs(self):
        self.write(f"{DESIGN}/c.md",
                   "# c\n\nRFC 9500's `testRSA2048` (section 2.1) supplies the key.\n\n"
                   "RFC 9500’s test key (section 2.1) and RFC 9500's key (section 2.1).\n\n"
                   "RFC 9500's (section 2.1) and RFC 6749, section 5.1.\n\n"
                   "RFC 9500's second test key (section 9).\n\n"
                   "Our `testRSA2048` (section 9) supplies the key.\n")
        self.commit("head")
        result = self.since_base()
        self.assertEqual(result.returncode, 1)
        findings = result.stdout.splitlines()
        self.assertEqual(len(findings), 2, result.stdout)
        self.assertTrue(all(f"section-self {DESIGN}/c.md 9 (" in line for line in findings), result.stdout)

    def test_a_count_after_a_path_is_not_a_section(self):
        self.write(f"{DESIGN}/c.md",
                   "# c\n\nskein's `a.md`, 21 places; `a.md`, 2 places.\n\n"
                   "See `a.md`, 9, and `gone.md`, 21 places.\n\n"
                   "See `a.md`, section 8 places the rule here.\n")
        self.commit("head")
        result = self.since_base()
        self.assertEqual(result.returncode, 1)
        self.assertEqual(len(result.stdout.splitlines()), 3, result.stdout)
        self.assertIn(f"section {DESIGN}/a.md 9", result.stdout)
        self.assertIn(f"section {DESIGN}/a.md 8", result.stdout)
        self.assertIn("path gone.md", result.stdout)

    def test_a_status_file_by_step_labels_steps(self):
        self.write(f"{DESIGN}/a.status.md", "# Status\n\nBy step:\n\n- **14, 22:** waiting.\n")
        self.write(f"{DESIGN}/b.status.md", "# Status\n\n**Not yet:** 14: waiting.\n")
        self.commit("head")
        result = self.since_base()
        self.assertEqual(result.returncode, 1)
        self.assertEqual(len(result.stdout.splitlines()), 1, result.stdout)
        self.assertIn(f"status-label {DESIGN}/b.md 14", result.stdout)

    def test_by_step_must_precede_the_first_label_and_only_skips_labels(self):
        self.write(f"{DESIGN}/a.status.md",
                   "# Status\n\n- **2:** waiting. By step:\n\n- **14:** waiting.\n")
        self.write(f"{DESIGN}/b.status.md",
                   "# Status\n\nBy step:\n\n- **14:** waiting; see section 9.\n")
        self.commit("head")
        result = self.since_base()
        self.assertEqual(result.returncode, 1)
        self.assertEqual(len(result.stdout.splitlines()), 2, result.stdout)
        self.assertIn(f"status-label {DESIGN}/a.md 14", result.stdout)
        self.assertIn(f"section-self {DESIGN}/b.md 9", result.stdout)

    def test_a_list_items_heading_path_owns_its_bare_sections(self):
        self.write(f"{DESIGN}/c.md",
                   "# c\n\n- **`a.md`:** 1, the first; section 2, the second.\n"
                   "  Another sentence with `b.md`; section 2.1 still belongs to the heading.\n"
                   "  See section 5 for the missing rule.\n"
                   "6. **a.md:** A different item. Its state (section 2.1) is defined.\n"
                   "  Section references include section 5 too.\n")
        self.commit("head")
        result = self.since_base()
        self.assertEqual(result.returncode, 1)
        findings = result.stdout.splitlines()
        self.assertEqual(len(findings), 2, result.stdout)
        self.assertTrue(all(f"section-self {DESIGN}/c.md|{DESIGN}/a.md 5 (" in line
                            for line in findings), result.stdout)

    def test_a_heading_path_does_not_own_the_next_list_item(self):
        self.write(f"{DESIGN}/c.md",
                   "# c\n\n- **`a.md`:** See section 2.1.\n"
                   "- A different item; section 2.1 is local.\n\n"
                   "Outside the list, section 2.1 is local.\n\n"
                   "- **`gone.md`:** A missing heading path.\n")
        self.commit("head")
        result = self.since_base()
        self.assertEqual(result.returncode, 1)
        self.assertEqual(len(result.stdout.splitlines()), 3, result.stdout)
        self.assertEqual(result.stdout.count(f"section-self {DESIGN}/c.md 2.1 ("), 2, result.stdout)
        self.assertIn("path gone.md", result.stdout)

    def test_a_link_to_a_tracked_page_resolves(self):
        self.write(f"{DESIGN}/page.html", "<p>A tracked page.</p>\n")
        self.write(f"{DESIGN}/c.md",
                   "# c\n\n[p](page.html) and [q](page.html?world=x&step=5#part) exist.\n\n"
                   "[g](gone.html) does not.\n")
        self.commit("head")
        self.write(f"{DESIGN}/gone.html", "<p>Untracked.</p>\n")
        result = self.since_base()
        self.assertEqual(result.returncode, 1)
        self.assertEqual(len(result.stdout.splitlines()), 1, result.stdout)
        self.assertIn("path gone.html", result.stdout)

    def test_a_page_must_be_tracked_at_the_checked_commit(self):
        self.write(f"{DESIGN}/page.html", "<p>A tracked page.</p>\n")
        self.write(f"{DESIGN}/c.md", "# c\n\n[p](page.html?world=x#part) exists.\n")
        self.commit("linked page")
        self.git("rm", f"{DESIGN}/page.html")
        self.commit("remove page")
        result = self.since_base()
        self.assertEqual(result.returncode, 1)
        self.assertIn("path page.html (", result.stdout)
        self.assertEqual(self.run_check("--all", "--head", "HEAD~1").returncode, 0)

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
