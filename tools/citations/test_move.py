"""Regression checks for document moves, on small repositories built per test."""
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

MOVER = Path(__file__).with_name("move.py").resolve()
CHECKER = Path(__file__).with_name("citations.py").resolve()
DESIGN = "skein/docs/design"
A = f"{DESIGN}/a.md"
B = f"{DESIGN}/b.md"
DOC = "# a\n\n## 1. One\n\nText.\n\n## 2. Two\n\n### 2.1 Part\n\nText.\n"


class Moves(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory(prefix="citations-move-test-")
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.git("init", "-q", "-b", "main")
        # Commit identity is local test data, never read from the environment.
        self.git("config", "user.email", "test@example.invalid")
        self.git("config", "user.name", "test")
        self.write(A, DOC)
        self.write(B, "# b\n\n## 3. Three\n\nAs `a.md`, 2.1 says.\n")
        self.commit("base")

    def git(self, *args):
        return subprocess.run(("git", *args), cwd=self.root, check=True, capture_output=True, text=True).stdout

    def write(self, rel, text):
        path = self.root / rel
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")

    def read(self, rel):
        return (self.root / rel).read_text(encoding="utf-8")

    def commit(self, message):
        self.git("add", "-A")
        self.git("commit", "-qm", message)

    def move(self, mapping, *args, env=None):
        self.write("move.map", mapping)
        return subprocess.run((sys.executable, "-I", "-B", str(MOVER), "move.map", *args),
                              cwd=self.root, capture_output=True, text=True, env=env)

    def apply(self, mapping):
        result = self.move(mapping)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertEqual(self.git("diff", "--name-only"), "", "all changes must be staged")
        return result

    def clean_check(self):
        self.commit("move")
        result = subprocess.run((sys.executable, "-I", "-B", str(CHECKER), "--all"),
                                cwd=self.root, capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn("0 broken", result.stderr)

    def snapshot(self):
        return {p.relative_to(self.root).as_posix(): p.read_bytes() for p in self.root.rglob("*")
                if p.is_file() and ".git" not in p.parts and p.name != "move.map"}

    def test_document_styles_and_companions(self):
        self.write(f"{DESIGN}/a.status.md", "# status\n\n**Not yet:** 2.1: waiting.\n")
        self.write(f"{DESIGN}/a.execution.md", "# execution\n\nSee `a.md`, 1.\n")
        self.write(B, "# b\n\nSee `./a.md`, 2.1; `skein/docs/design/a.md`, 1; `/skein/docs/design/a.md`, 2.\n")
        self.write("jig/docs/design/b.md", "# b\n\nskein's `a.md`, 2.1.\n")
        self.write("skein/crates/k/src/lib.rs", "//! See docs/design/a.md, 2.1.\n")
        self.commit("sources")
        result = self.apply(f"{A} -> {DESIGN}/sub/a.md\n")
        self.assertIn("Rewrote", result.stdout)
        self.assertEqual(self.read(B), "# b\n\nSee `./sub/a.md`, 2.1; `skein/docs/design/sub/a.md`, 1; `/skein/docs/design/sub/a.md`, 2.\n")
        self.assertEqual(self.read("jig/docs/design/b.md"), "# b\n\nskein's `sub/a.md`, 2.1.\n")
        self.assertEqual(self.read("skein/crates/k/src/lib.rs"), "//! See docs/design/sub/a.md, 2.1.\n")
        self.assertTrue((self.root / f"{DESIGN}/sub/a.status.md").exists())
        self.assertTrue((self.root / f"{DESIGN}/sub/a.execution.md").exists())
        self.clean_check()

    def test_directory_moves_binary_and_links(self):
        self.write(f"{DESIGN}/assets/page.html", "<p>page</p>\n")
        (self.root / f"{DESIGN}/assets/data.bin").write_bytes(b"\x00\xff\x01")
        self.write(B, "# b\n\n[page](assets/page.html?x=1#part)\n")
        self.commit("assets")
        self.apply(f"{DESIGN}/assets/ -> {DESIGN}/web/assets/\n")
        self.assertEqual(self.read(B), "# b\n\n[page](web/assets/page.html?x=1#part)\n")
        self.assertEqual((self.root / f"{DESIGN}/web/assets/data.bin").read_bytes(), b"\x00\xff\x01")
        self.clean_check()

    def test_section_subsections_level_and_order(self):
        self.write(A, "# a\n\n## 1. One\n\nKeep.\n\n## 2. Two\n\n### 2.1 Part\n\n#### Aside\n\nBody.\n\n```md\n## 9. Not a heading\n```\n\n### 2.2 Other\n\nEnd.\n")
        self.commit("sections")
        self.apply(f"{A} 2.1 -> {DESIGN}/web/new.md 1\ntitle {DESIGN}/web/new.md: New\n")
        self.assertEqual(self.read(f"{DESIGN}/web/new.md"), "# New\n\n## 1. Part\n\n### Aside\n\nBody.\n\n```md\n## 9. Not a heading\n```\n\n")
        self.assertEqual(self.read(A), "# a\n\n## 1. One\n\nKeep.\n\n## 2. Two\n\n### 2.2 Other\n\nEnd.\n")
        self.assertEqual(self.read(B), "# b\n\n## 3. Three\n\nAs `web/new.md`, 1 says.\n")
        self.clean_check()

    def test_renumber_and_status_forms(self):
        self.write(f"{DESIGN}/a.status.md", "# status\n\n**Not yet:** 2.1: waiting. 1: another.\n\n- **2:** waiting.\n")
        self.commit("status")
        self.apply(f"{A} 2 -> {A} 4\n")
        self.assertEqual(self.read(A), DOC.replace("## 2. Two", "## 4. Two").replace("### 2.1 Part", "### 4.1 Part"))
        self.assertEqual(self.read(f"{DESIGN}/a.status.md"), "# status\n\n**Not yet:** 4.1: waiting. 1: another.\n\n- **4:** waiting.\n")
        self.assertIn("`a.md`, 4.1", self.read(B))
        self.clean_check()

    def test_bare_sections_both_directions(self):
        self.write(A, "# a\n\n## 1. One\n\nSee section 2.1.\n\n## 2. Two\n\nSee section 1.\n\n### 2.1 Part\n\nSee section 2.\n")
        self.commit("bare")
        self.apply(f"{A} 2 -> {DESIGN}/sub/new.md 4\ntitle {DESIGN}/sub/new.md: New\n")
        self.assertEqual(self.read(A), "# a\n\n## 1. One\n\nSee `sub/new.md`, section 4.1.\n\n")
        self.assertEqual(self.read(f"{DESIGN}/sub/new.md"), "# New\n\n## 4. Two\n\nSee `../a.md`, section 1.\n\n### 4.1 Part\n\nSee section 4.\n")
        self.clean_check()

    def test_relative_citation_in_moved_text(self):
        self.write(A, DOC + "\nAs `b.md`, 3 says.\n")
        self.commit("relative")
        self.apply(f"{A} 2 -> {DESIGN}/sub/new.md 4\ntitle {DESIGN}/sub/new.md: New\n")
        self.assertIn("`../b.md`, 3", self.read(f"{DESIGN}/sub/new.md"))
        self.clean_check()

    def test_most_specific_section_wins(self):
        self.write(A, DOC + "\n### 2.2 Extra\n\nRest.\n")
        self.commit("nested")
        self.apply(f"{A} 2 -> {DESIGN}/new.md 4\n{A} 2.1 -> {DESIGN}/other.md 1\ntitle {DESIGN}/new.md: New\ntitle {DESIGN}/other.md: Other\n")
        self.assertEqual(self.read(f"{DESIGN}/new.md"), "# New\n\n## 4. Two\n\n### 4.2 Extra\n\nRest.\n")
        self.assertEqual(self.read(f"{DESIGN}/other.md"), "# Other\n\n## 1. Part\n\nText.\n\n")
        self.assertIn("`other.md`, 1", self.read(B))
        self.clean_check()

    def test_insert_among_existing_siblings(self):
        self.write(B, "# b\n\n## 3. Three\n\n### 3.1 First\n\nFirst.\n\n### 3.4 Last\n\nLast.\n\n## 5. Five\n")
        self.commit("siblings")
        self.apply(f"{A} 2.1 -> {B} 3.2\n")
        self.assertEqual(self.read(B), "# b\n\n## 3. Three\n\n### 3.1 First\n\nFirst.\n\n### 3.2 Part\n\nText.\n### 3.4 Last\n\nLast.\n\n## 5. Five\n")
        self.clean_check()

    def test_split_citation_is_left_and_listed(self):
        self.write(B, "# b\n\nSee `a.md`, sections 1 and 2.1.\n")
        self.commit("split")
        result = self.apply(f"{A} 2 -> {DESIGN}/new.md 4\ntitle {DESIGN}/new.md: New\n")
        self.assertIn("split citation", result.stdout)
        self.assertEqual(self.read(B), "# b\n\nSee `a.md`, sections 1 and 2.1.\n")
        # Apply the reported manual repair before checking the committed tree.
        self.write(B, "# b\n\nSee `a.md`, section 1, and `new.md`, section 4.1.\n")
        self.clean_check()

    def test_ambiguous_bare_section_is_left_and_listed(self):
        self.write(B, "# b\n\n## 1. Own\n\nSee [a](a.md); section 1.\n")
        self.commit("ambiguous")
        result = self.apply(f"{A} 1 -> {DESIGN}/new.md 4\ntitle {DESIGN}/new.md: New\n")
        self.assertIn("ambiguous bare section", result.stdout)
        self.assertEqual(self.read(B), "# b\n\n## 1. Own\n\nSee [a](a.md); section 1.\n")
        self.clean_check()

    def test_link_anchor_and_equal_label(self):
        self.write(B, "# b\n\n[a.md](a.md#21-part) and [part](a.md?view=1#21-part).\n")
        self.commit("anchors")
        self.apply(f"{A} 2.1 -> {DESIGN}/new.md 1\ntitle {DESIGN}/new.md: New\n")
        self.assertEqual(self.read(B), "# b\n\n[new.md](new.md#1-part) and [part](new.md?view=1#1-part).\n")
        self.clean_check()

    def test_wrapped_prose_and_comment_coordinates(self):
        self.write(B, "# b\n\n  See `a.md`, sections\n    2.1 and\n    2.\n")
        self.write("skein/crates/k/src/lib.rs", "fn f() {} // See docs/design/a.md, 2.1.\n//! See `docs/design/a.md`, sections\n//!   2.1 and 2.\n")
        self.commit("wrapped")
        self.apply(f"{A} 2 -> {A} 4\n")
        self.assertEqual(self.read(B), "# b\n\n  See `a.md`, sections\n    4.1 and\n    4.\n")
        self.assertEqual(self.read("skein/crates/k/src/lib.rs"), "fn f() {} // See docs/design/a.md, 4.1.\n//! See `docs/design/a.md`, sections\n//!   4.1 and 4.\n")
        self.clean_check()

    def test_allow_entries_globs_and_code_names(self):
        self.write("tools/citations/allow.txt", f"{A} {A} 2.1 # a label\n{DESIGN}/*.md {A} # a glob\n")
        self.write(".github/workflows/check.yml", f"path: {A}\n")
        self.write("skein/crates/k/src/lib.rs", f'const PATH: &str = "{A}"; // See {A}, 2.1.\n')
        self.write("skein/docs/plan/p.md", f"See {A}.\n")
        self.write("skein/docs/development/2026-01-01.md", f"See docs/design/a.md.\n")
        self.commit("advisories")
        result = self.apply(f"{A} -> {DESIGN}/sub/a.md\n")
        self.assertIn("glob matches a moved path", result.stdout)
        self.assertIn(".github/workflows/check.yml:1: name in code", result.stdout)
        self.assertIn("skein/crates/k/src/lib.rs:1: name in code", result.stdout)
        self.assertIn("skein/docs/plan/p.md:1: name in code", result.stdout)
        self.assertIn("skein/docs/development/2026-01-01.md:1: name in code", result.stdout)
        self.assertEqual(self.read("tools/citations/allow.txt"), f"{DESIGN}/sub/a.md {DESIGN}/sub/a.md 2.1 # a label\n{DESIGN}/*.md {DESIGN}/sub/a.md # a glob\n")
        self.clean_check()

    def test_cross_document_status_labels_are_listed(self):
        self.write(f"{DESIGN}/a.status.md", "# status\n\n**Not yet:** 2.1: waiting.\n")
        self.commit("status")
        result = self.apply(f"{A} 2 -> {DESIGN}/new.md 4\ntitle {DESIGN}/new.md: New\n")
        self.assertIn(f"status label to move to {DESIGN}/new.md 4.1", result.stdout)
        self.assertEqual(self.read(f"{DESIGN}/a.status.md"), "# status\n\n**Not yet:** 2.1: waiting.\n")
        self.write(f"{DESIGN}/a.status.md", "# status\n\nNo gaps.\n")
        self.write(f"{DESIGN}/new.status.md", "# status\n\n**Not yet:** 4.1: waiting.\n")
        self.clean_check()

    def test_simultaneous_document_swap(self):
        self.apply(f"{A} -> {B}\n{B} -> {A}\n")
        self.assertEqual(self.read(B), DOC)
        self.assertIn("`b.md`, 2.1", self.read(A))
        self.clean_check()

    def test_simultaneous_section_swap(self):
        self.apply(f"{A} 1 -> {A} 2\n{A} 2 -> {A} 1\n")
        self.assertEqual(self.read(A), "# a\n\n## 1. Two\n\n### 1.1 Part\n\nText.\n## 2. One\n\nText.\n\n")
        self.assertIn("`a.md`, 1.1", self.read(B))
        self.clean_check()

    def test_dry_run_matches_apply_and_changes_nothing(self):
        mapping = f"{A} -> {DESIGN}/sub/a.md\n"
        before = self.snapshot()
        result = self.move(mapping, "--dry-run")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.snapshot(), before)
        self.assertEqual(self.git("status", "--porcelain", "--untracked-files=no"), "")
        applied = self.apply(mapping)
        self.assertEqual(applied.stdout, result.stdout)
        self.clean_check()

    def test_local_anchors_follow_both_text_and_heading(self):
        self.write(A, "# a\n\n## 1. One\n\n[part](#21-part)\n\n## 2. Two\n\n### 2.1 Part\n\n[one](#1-one) and [two](#2-two).\n")
        self.commit("local anchors")
        self.apply(f"{A} 2 -> {DESIGN}/sub/new.md 4\ntitle {DESIGN}/sub/new.md: New\n")
        self.assertEqual(self.read(A), "# a\n\n## 1. One\n\n[part](sub/new.md#41-part)\n\n")
        self.assertEqual(self.read(f"{DESIGN}/sub/new.md"), "# New\n\n## 4. Two\n\n### 4.1 Part\n\n[one](../a.md#1-one) and [two](#4-two).\n")
        self.clean_check()

    def test_repeated_comment_text_has_physical_coordinates(self):
        self.write("skein/crates/k/src/lib.rs",
                   'const S: &str = " See docs/design/a.md, 2.1."; // See docs/design/a.md, 2.1.\n')
        self.commit("comment")
        self.apply(f"{A} 2 -> {A} 4\n")
        self.assertEqual(self.read("skein/crates/k/src/lib.rs"),
                         'const S: &str = " See docs/design/a.md, 2.1."; // See docs/design/a.md, 4.1.\n')
        self.clean_check()

    def test_cross_tree_owner_and_root_fallback(self):
        self.write("jig/docs/design/t.md", DOC)
        self.write(f"{DESIGN}/sub/owner.md", "# b\n\njig's `t.md`, 2.1, and its `t.md`, 1.\n")
        self.commit("owners")
        self.apply(f"jig/docs/design/t.md -> {DESIGN}/new.md\n")
        self.assertEqual(self.read(f"{DESIGN}/sub/owner.md"), f"# b\n\njig's `{DESIGN}/new.md`, 2.1, and its `{DESIGN}/new.md`, 1.\n")
        self.clean_check()

    def test_directory_and_more_specific_file_override(self):
        self.write(f"{DESIGN}/sub/t.md", DOC)
        self.write(B, "# b\n\nSee `sub/t.md`, 1.\n")
        self.commit("directory")
        self.apply(f"{DESIGN}/sub/ -> {DESIGN}/web/\n{DESIGN}/sub/t.md -> {DESIGN}/other/t.md\n")
        self.assertEqual(self.read(B), "# b\n\nSee `other/t.md`, 1.\n")
        self.clean_check()

    def test_missing_terminal_newline_still_inserts_a_heading(self):
        self.write(B, "# b\n\n## 3. Three\n\nBody.")
        self.commit("newline")
        self.apply(f"{A} 2.1 -> {B} 4\n")
        self.assertEqual(self.read(B), "# b\n\n## 3. Three\n\nBody.\n## 4. Part\n\nText.\n")
        self.clean_check()

    def test_by_step_status_labels_are_not_renumbered(self):
        self.write(f"{DESIGN}/a.status.md", "# status\n\nBy step:\n\n- **2:** waiting.\n\nSee section 2.1.\n")
        self.commit("step labels")
        self.apply(f"{A} 2 -> {A} 4\n")
        self.assertEqual(self.read(f"{DESIGN}/a.status.md"), "# status\n\nBy step:\n\n- **2:** waiting.\n\nSee section 4.1.\n")
        self.clean_check()

    def test_companion_conflicts_in_either_map_order(self):
        status = f"{DESIGN}/a.status.md"
        self.write(status, "# status\n")
        self.commit("status")
        moves = [f"{A} -> {DESIGN}/sub/a.md", f"{status} -> {DESIGN}/other.status.md"]
        before = self.snapshot()
        for mapping in (moves, list(reversed(moves))):
            result = self.move("\n".join(mapping))
            self.assertEqual(result.returncode, 2)
            self.assertIn("conflicting companion move", result.stderr)
            self.assertEqual(self.snapshot(), before)

    def test_late_git_failure_restores_contents_and_index(self):
        mapping = f"{A} -> {DESIGN}/sub/a.md\n{B} 3 -> {DESIGN}/new.md 4\ntitle {DESIGN}/new.md: New\n"
        self.write("move.map", mapping)
        before, index = self.snapshot(), self.git("write-tree")
        script = """import runpy, subprocess, sys
module = runpy.run_path(sys.argv[1])
state = module['main'].__globals__
real_git = state['git']
def fail_add(*args):
    if args[0] == 'add':
        raise subprocess.CalledProcessError(1, ['git', *args], stderr='injected add failure')
    return real_git(*args)
state['git'] = fail_add
sys.exit(module['main'](['move.map']))
"""
        result = subprocess.run((sys.executable, "-I", "-B", "-c", script, str(MOVER)),
                                cwd=self.root, capture_output=True, text=True)
        self.assertEqual(result.returncode, 2, result.stdout + result.stderr)
        self.assertIn("injected add failure", result.stderr)
        self.assertEqual(self.snapshot(), before)
        self.assertEqual(self.git("write-tree"), index)
        self.assertFalse((self.root / f"{DESIGN}/sub").exists())

    def test_directory_destinations_cannot_merge(self):
        self.write(f"{DESIGN}/x/first.md", DOC)
        self.write(f"{DESIGN}/y/second.md", DOC)
        self.commit("directories")
        before = self.snapshot()
        for mapping, error in (
            (f"{DESIGN}/x/ -> {DESIGN}/y/", "destination exists"),
            (f"{DESIGN}/x/ -> {DESIGN}/new/\n{DESIGN}/y/ -> {DESIGN}/new/", "two directory sources"),
        ):
            result = self.move(mapping)
            self.assertEqual(result.returncode, 2, result.stdout + result.stderr)
            self.assertIn(error, result.stderr)
            self.assertEqual(self.snapshot(), before)
        self.clean_check()

    def test_directory_swap_applies_together(self):
        self.write(f"{DESIGN}/x/first.md", DOC)
        self.write(f"{DESIGN}/y/second.md", DOC)
        self.write(B, "# b\n\nSee `x/first.md`, 2, and [second](y/second.md).\n")
        self.commit("directories")
        self.apply(f"{DESIGN}/x/ -> {DESIGN}/y/\n{DESIGN}/y/ -> {DESIGN}/x/\n")
        self.assertEqual(self.read(B), "# b\n\nSee `y/first.md`, 2, and [second](x/second.md).\n")
        self.clean_check()

    def test_line_endings_and_non_utf8_bytes_are_preserved(self):
        raw = DOC.replace("\n", "\r\n").encode() + b"\r\nOpaque: \xff.\r\n"
        (self.root / A).write_bytes(raw)
        self.commit("bytes")
        self.apply(f"{A} -> {DESIGN}/sub/a.md\n")
        self.assertEqual((self.root / f"{DESIGN}/sub/a.md").read_bytes(), raw)
        self.clean_check()

    def test_unaffected_readings_are_left_alone(self):
        self.write(f"{DESIGN}/keep.md", DOC)
        self.write(f"{DESIGN}/other.md", "# other\n\nSee [keep](keep.md); section 2.1.\n")
        self.commit("unaffected")
        self.apply(f"{A} -> {DESIGN}/sub/a.md\n")
        self.assertEqual(self.read(f"{DESIGN}/other.md"), "# other\n\nSee [keep](keep.md); section 2.1.\n")
        self.clean_check()

    def test_status_citation_keeps_status_target(self):
        self.write(f"{DESIGN}/a.status.md", "# status\n\nNo gaps.\n")
        self.write(B, "# b\n\nSee `a.status.md`, section 2.1.\n")
        self.commit("status target")
        self.apply(f"{A} -> {DESIGN}/sub/a.md\n")
        self.assertEqual(self.read(B), "# b\n\nSee `sub/a.status.md`, section 2.1.\n")
        self.clean_check()

    def test_untracked_markdown_is_not_a_resolution_candidate(self):
        self.write(f"{DESIGN}/tracked/t.md", DOC)
        self.write(B, "# b\n\nSee `t.md`, 2.1.\n")
        self.commit("tracked")
        self.write(f"{DESIGN}/untracked/t.md", "# untracked\n\n## 9. Nine\n")
        self.apply(f"{DESIGN}/tracked/t.md -> {DESIGN}/new.md\n")
        self.assertEqual(self.read(B), "# b\n\nSee `new.md`, 2.1.\n")
        (self.root / f"{DESIGN}/untracked/t.md").unlink()
        self.clean_check()

    def test_list_target_and_sentence_antecedent_bare_sections(self):
        self.write(B, "# b\n\n- **`a.md`:** See section 2.1.\n\nSee [a](a.md); section 2.1.\n")
        self.commit("antecedents")
        self.apply(f"{A} 2 -> {DESIGN}/new.md 4\ntitle {DESIGN}/new.md: New\n")
        self.assertEqual(self.read(B), "# b\n\n- **`a.md`:** See `new.md`, section 4.1.\n\nSee [a](a.md); `new.md`, section 4.1.\n")
        self.clean_check()

    def test_encoded_anchor_and_html_heading_text(self):
        self.write(A, DOC.replace("2.1 Part", "2.1 Café &amp; cake"))
        self.write(B, "# b\n\n[part](a.md#21-caf%C3%A9--cake).\n")
        self.commit("encoded anchor")
        self.apply(f"{A} 2.1 -> {DESIGN}/new.md 4\ntitle {DESIGN}/new.md: New\n")
        self.assertEqual(self.read(B), "# b\n\n[part](new.md#4-caf%C3%A9--cake).\n")
        self.clean_check()

    def test_allow_targets_with_multiple_candidate_documents(self):
        self.write("tools/citations/allow.txt", f"{B} {A}|{B} 2.1 # shared target\n")
        self.commit("allow candidates")
        self.apply(f"{A} -> {DESIGN}/sub/a.md\n")
        self.assertEqual(self.read("tools/citations/allow.txt"), f"{B} {DESIGN}/sub/a.md|{B} 2.1 # shared target\n")
        self.clean_check()

    def test_all_map_errors_leave_tree_and_index_unchanged(self):
        cases = [
            (f"{DESIGN}/gone.md -> {DESIGN}/new.md", "source does not exist"),
            (f"{A} 9 -> {B} 4", "source section"),
            (f"{A} -> {DESIGN}/new.md\n{B} -> {DESIGN}/new.md", "two sources"),
            (f"{A} -> {B}", "two sources"),
            (f"{A} -> {DESIGN}/new.md\n{B} -> {DESIGN}/new.md/b.md", "parent is a file"),
            (f"{A} 2 -> {DESIGN}/new.md 4", "title"),
            (f"{A} -> elsewhere/new.md", "five trees"),
            (f"{A} -> skein/crates/new.md", "five trees"),
            (f"{A} -> {DESIGN}/../new.md", "without . or .."),
            (f"{A} -> {DESIGN}/new.md\n{A} 2 -> {B} 4", "document and section"),
            (f"{A} 2 -> {B} 3", "destination section"),
            (f"{A} 2.1 -> {B} 8.1", "parent section"),
            (f"{A} 2 -> {B}", "same kind"),
            (f"{A} -> {DESIGN}/new.md\n{A} -> {DESIGN}/other.md", "duplicate source"),
            (f"{A} 2 -> {B} 3.1.1.1.1.1", "level 6"),
            ("not a map", "SOURCE -> DESTINATION"),
            ("# empty", "no moves"),
            (f"title {DESIGN}/new.md: New", "no moves"),
        ]
        before = self.snapshot()
        index = self.git("write-tree")
        for mapping, error in cases:
            with self.subTest(mapping=mapping):
                result = self.move(mapping + "\n")
                self.assertEqual(result.returncode, 2, result.stdout + result.stderr)
                self.assertIn(error, result.stderr)
                self.assertEqual(self.snapshot(), before)
                self.assertEqual(self.git("write-tree"), index)
                self.assertEqual(self.git("status", "--porcelain", "--untracked-files=no"), "")

    def test_reports_multiple_errors(self):
        result = self.move(f"{DESIGN}/gone.md -> {B}\n{A} 2 -> {DESIGN}/new.md 4\n{A} -> elsewhere/x.md\n")
        self.assertEqual(result.returncode, 2)
        for error in ("source does not exist", "title", "five trees"):
            self.assertIn(error, result.stderr)

    def test_untracked_destination_is_not_overwritten(self):
        self.write(f"{DESIGN}/new.md", "untracked\n")
        before = self.snapshot()
        result = self.move(f"{A} -> {DESIGN}/new.md\n")
        self.assertEqual(result.returncode, 2)
        self.assertIn("destination exists", result.stderr)
        self.assertEqual(self.snapshot(), before)

    def test_dirty_tree_refused(self):
        self.write(A, DOC + "changed\n")
        before = self.snapshot()
        result = self.move(f"{A} -> {DESIGN}/new.md\n")
        self.assertEqual(result.returncode, 2)
        self.assertIn("tracked changes", result.stderr)
        self.assertEqual(self.snapshot(), before)

    def test_git_failure_rolls_back(self):
        # An index lock permits reads and makes the first git mv fail.
        (self.root / ".git/index.lock").write_text("locked\n")
        before = self.snapshot()
        result = self.move(f"{A} -> {DESIGN}/new.md\n")
        self.assertEqual(result.returncode, 2)
        self.assertIn("index.lock", result.stderr)
        self.assertEqual(self.snapshot(), before)
        (self.root / ".git/index.lock").unlink()
        self.assertEqual(self.git("status", "--porcelain", "--untracked-files=no"), "")


if __name__ == "__main__":
    unittest.main()
