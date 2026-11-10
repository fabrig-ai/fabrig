"""Regression checks for document moves, on small repositories built per test."""
from pathlib import Path
import os
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

    def test_split_citation_is_rewritten(self):
        self.write(B, "# b\n\nSee `a.md`, sections 1 and 2.1.\n")
        self.commit("split")
        result = self.apply(f"{A} 2 -> {DESIGN}/new.md 4\ntitle {DESIGN}/new.md: New\n")
        self.assertNotIn("split citation", result.stdout)
        self.assertEqual(self.read(B), "# b\n\nSee `a.md`, section 1, and `new.md`, section 4.1.\n")
        self.clean_check()

    def test_ambiguous_bare_section_is_left_and_listed(self):
        self.write(B, "# b\n\n## 1. Own\n\nSee [a](a.md); section 1.\n")
        self.commit("ambiguous")
        result = self.move(f"{A} 1 -> {DESIGN}/new.md 4\ntitle {DESIGN}/new.md: New\n")
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn("ambiguous bare section", result.stdout)
        self.assertNotIn("Wrong:", result.stdout)
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
        (self.root / "names.bin").write_bytes(b"\x00\xff\n" + A.encode() + b"\n")
        self.write("skein/crates/k/src/lib.rs", f'const PATH: &str = "{A}"; // See {A}, 2.1.\n'
                   'const DOC: &str = include_str!("../../../../docs/design/a.md");\n')
        self.write("skein/docs/plan/p.md", f"See {A}.\n")
        self.write("skein/docs/development/2026-01-01.md", f"See docs/design/a.md.\n")
        self.commit("advisories")
        result = self.apply(f"{A} -> {DESIGN}/sub/a.md\n")
        self.assertIn("glob matches a moved path", result.stdout)
        self.assertIn(".github/workflows/check.yml:1: name in code", result.stdout)
        self.assertIn("names.bin:2: name in code", result.stdout)
        self.assertIn("skein/crates/k/src/lib.rs:1: name in code", result.stdout)
        self.assertIn("skein/crates/k/src/lib.rs:2: name in code", result.stdout)
        self.assertNotIn("skein/docs/plan/p.md", result.stdout)
        self.assertNotIn("skein/docs/development/2026-01-01.md", result.stdout)
        self.assertEqual(self.read("tools/citations/allow.txt"), f"{DESIGN}/sub/a.md {DESIGN}/sub/a.md 2.1 # a label\n{DESIGN}/*.md {DESIGN}/sub/a.md # a glob\n")
        self.clean_check()

    def test_cross_document_status_entries_follow_labels(self):
        self.write(f"{DESIGN}/a.status.md", "# status\n\n**Not yet:** 2.1: waiting.\n")
        self.commit("status")
        result = self.apply(f"{A} 2 -> {DESIGN}/new.md 4\ntitle {DESIGN}/new.md: New\n")
        self.assertEqual(result.stdout.count("moved status entry"), 1)
        self.assertEqual(self.read(f"{DESIGN}/a.status.md"), "# status\n\nNo gap.\n")
        self.assertTrue(self.read(f"{DESIGN}/new.status.md").endswith("**Not yet:** 4.1: waiting.\n"))
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
        (self.root / A).touch()
        index = (self.root / ".git/index").read_bytes()
        result = self.move(mapping, "--dry-run")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual((self.root / ".git/index").read_bytes(), index)
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

    def test_anchor_slug_uses_visible_heading_text(self):
        self.write(A, DOC.replace("2.1 Part", "2.1 `Value<T>` and _other_"))
        self.write(B, "# b\n\n[part](a.md#21-valuet-and-other).\n")
        self.commit("heading markup")
        self.apply(f"{A} 2.1 -> {DESIGN}/new.md 4\ntitle {DESIGN}/new.md: New\n")
        self.assertEqual(self.read(B), "# b\n\n[part](new.md#4-valuet-and-other).\n")
        self.clean_check()

    def injected_move(self, mapping, override, *args):
        self.write("move.map", mapping)
        script = '''import runpy, sys
state = runpy.run_path(sys.argv[1])['main'].__globals__
exec(sys.argv[2], state)
sys.exit(state['main'](['move.map', *sys.argv[3:]]))
'''
        return subprocess.run((sys.executable, "-I", "-B", "-c", script, str(MOVER), override, *args),
                              cwd=self.root, capture_output=True, text=True)

    def test_section_becomes_document_preamble_and_promoted_subsections(self):
        self.write(A, "# a\n\n## 1. One\n\nKeep.\n\n## 2. Two\n\nOpening text.\n\n### 2.1 Part\n\n#### 2.1.1 Detail\n\nBody.\n")
        self.write(B, "# b\n\n`a.md`, 2; `a.md`, 2.1; `a.md`, 2.1.1.\n")
        self.commit("preamble")
        self.apply(f"{A} 2 -> {DESIGN}/sub/new.md\ntitle {DESIGN}/sub/new.md: The C# client\n")
        self.assertEqual(self.read(f"{DESIGN}/sub/new.md"), "# The C# client\n\nOpening text.\n\n## 1. Part\n\n### 1.1 Detail\n\nBody.\n")
        self.assertEqual(self.read(B), "# b\n\n`sub/new.md`; `sub/new.md`, 1; `sub/new.md`, 1.1.\n")
        self.assertTrue(self.read(f"{DESIGN}/sub/new.status.md").endswith("\n\nNo gap.\n"))
        self.clean_check()

    def test_bare_dotted_numbers_and_exclusions(self):
        self.write(A, "# a\n\n## 1. One\n\nEligible (2.1), as 2.1's rule says.\n"
                   "Literal `2.1`, [label](https://example.invalid/2.1), https://example.invalid/2.1, "
                   "v2.1, 2.1-rc, 2.1 ms, 2.1%, 2026-02-01, and (2).\n\n"
                   "```md\n(2.1)\n```\n\n## 2. Two\n\n### 2.1 Part\n\nOwn (2.1).\n")
        self.commit("own dotted references")
        result = self.apply(f"{A} 2 -> {DESIGN}/new.md\ntitle {DESIGN}/new.md: New\n")
        self.assertIn("3 bare numbers", result.stdout)
        self.assertIn("Eligible (`new.md`, 1), as `new.md`, 1's rule says.", self.read(A))
        self.assertIn("Literal `2.1`, [label](https://example.invalid/2.1), https://example.invalid/2.1, v2.1, 2.1-rc, 2.1 ms, 2.1%, 2026-02-01, and (2).", self.read(A))
        self.assertIn("```md\n(2.1)\n```", self.read(A))
        self.assertTrue(self.read(f"{DESIGN}/new.md").endswith("Own (1).\n"))
        self.clean_check()

    def test_bare_dotted_range_and_list_split(self):
        self.write(A, "# a\n\n## 1. One\n\nThe live trial (2.1 to 2.3), as 2.1 and 2.3 say.\n\n"
                   "## 2. Two\n\n### 2.1 First\n\n### 2.2 Middle\n\n### 2.3 Last\n")
        self.commit("dotted ranges")
        self.apply(f"{A} 2 -> {DESIGN}/trial.md\n{A} 2.2 -> {DESIGN}/middle.md 1\n"
                   f"title {DESIGN}/trial.md: Trial\ntitle {DESIGN}/middle.md: Middle\n")
        self.assertIn("The live trial (`trial.md`, 1, and `middle.md`, 1, and `trial.md`, 3), as `trial.md`, 1 and 3 say.", self.read(A))
        self.clean_check()

    def test_ranges_expand_all_middle_headings_and_keep_joiners(self):
        self.write(A, "# a\n\n## 2. Two\n\n### 2.1 First\n\n### 2.2 Middle\n\n### 2.3 Last\n")
        self.write(B, "# b\n\n`a.md`, 2.1 to 2.3; `a.md`, 2.1–2.3; `a.md`, 2.1-2.3.\n")
        self.commit("range middle")
        mapping = f"{A} 2.2 -> {DESIGN}/new.md 1\ntitle {DESIGN}/new.md: New\n"
        broken = self.injected_move(mapping, "Plan.numbers_to_rewrite = lambda self, cite, tree: list(cite.numbers)", "--dry-run")
        self.assertEqual(broken.returncode, 1, broken.stdout + broken.stderr)
        self.assertIn("Wrong:", broken.stdout)
        self.assertIn(f"{DESIGN}/new.md 1", broken.stdout)
        self.apply(mapping)
        part = "`a.md`, 2.1, and `new.md`, 1, and `a.md`, 2.3"
        self.assertEqual(self.read(B), f"# b\n\n{part}; {part}; {part}.\n")
        self.clean_check()

    def test_split_range_drops_whole_document_number(self):
        self.write(A, "# a\n\n## 5. Packages\n\nText.\n\n## 6. Six\n\n## 7. Seven\n")
        self.write(B, "# b\n\n`a.md`, 5 to 7.\n")
        self.commit("whole range member")
        self.apply(f"{A} 5 -> {DESIGN}/build/packages.md\ntitle {DESIGN}/build/packages.md: Packages\n"
                   f"{A} -> {DESIGN}/build/README.md\n")
        self.assertEqual(self.read(B), "# b\n\n`build/packages.md` and `build/README.md`, 6 to 7.\n")
        self.clean_check()

    def test_inserted_path_does_not_steal_later_antecedent(self):
        self.write(A, "# a\n\n`c.md` keeps the store, as section 3 says, and as section 7 says.\n\n## 3. Three\n")
        self.write(B, "# b\n\n## 7. Seven in b\n")
        self.write(f"{DESIGN}/c.md", "# c\n\n## 7. Seven in c\n")
        self.commit("antecedents")
        mapping = f"{A} 3 -> {B} 2\n"
        broken = self.injected_move(mapping, "Plan.protect_antecedent = staticmethod(lambda cite, introduced: False)", "--dry-run")
        self.assertEqual(broken.returncode, 1, broken.stdout + broken.stderr)
        self.assertIn("Wrong:", broken.stdout)
        self.assertIn(f"should read: {DESIGN}/c.md 7", broken.stdout)
        self.apply(mapping)
        self.assertIn("`c.md` keeps the store, as `b.md`, section 2 says, and as `c.md`, section 7 says.", self.read(A))
        self.clean_check()

    def test_suffix_shadow_preserves_name_and_reports_resolution(self):
        self.write("jig/docs/design/client.md", "# client\n")
        self.write("temper/docs/design/a.md", "# a\n\njig has two `client.md`.\n")
        self.write("skein/docs/design/source.md", "# source\n")
        self.commit("suffix names")
        mapping = "skein/docs/design/source.md -> temper/docs/design/client.md\n"
        broken = self.injected_move(mapping, "Plan.preserve_suffix_name = lambda self, cite, shadowed, resolved: False", "--dry-run")
        self.assertEqual(broken.returncode, 1, broken.stdout + broken.stderr)
        self.assertIn("preserved bare name", broken.stdout)
        result = self.move(mapping)
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        self.assertIn("the bare name now resolves to temper/docs/design/client.md", result.stdout)
        self.assertEqual(self.read("temper/docs/design/a.md"), "# a\n\njig has two `client.md`.\n")

    def test_shortest_relative_fallback_and_cross_tree_root(self):
        self.write(f"{DESIGN}/other/a.md", "# other a\n")
        self.write(f"{DESIGN}/sub/read.md", "# read\n\nSee `a.md`, 2.1.\n")
        self.write("jig/docs/design/read.md", "# read\n\nSee `b.md`, 3.\n")
        self.commit("fallbacks")
        self.apply(f"{A} -> {DESIGN}/sub/deep/a.md\n{B} -> {DESIGN}/b2.md\n")
        self.assertEqual(self.read(f"{DESIGN}/sub/read.md"), "# read\n\nSee `deep/a.md`, 2.1.\n")
        self.assertEqual(self.read("jig/docs/design/read.md"), f"# read\n\nSee `{DESIGN}/b2.md`, 3.\n")
        self.clean_check()

    def test_document_sections_and_remaining_renumber_together(self):
        self.write(A, "# a\n\n## 2. Two\n\nSee (4.1).\n\n## 3. Three\n\n### 3.1 Part\n\n## 4. Four\n\n### 4.1 End\n")
        self.write(B, "# b\n\n`a.md`, 2; `a.md`, 3.1; `a.md`, 4.1.\n")
        self.write(f"{DESIGN}/a.status.md", "# status\n\n- **Not yet:** 4.1: waiting.\n")
        self.commit("one map")
        self.apply(f"{A} 3 -> {DESIGN}/part.md\ntitle {DESIGN}/part.md: Part\n"
                   f"{A} -> {DESIGN}/sub/a.md\nrenumber {A}\n")
        self.assertEqual(self.read(f"{DESIGN}/sub/a.md"), "# a\n\n## 1. Two\n\nSee (2.1).\n\n## 2. Four\n\n### 2.1 End\n")
        self.assertEqual(self.read(B), "# b\n\n`sub/a.md`, 1; `part.md`, 1; `sub/a.md`, 2.1.\n")
        self.assertIn("**Not yet:** 2.1:", self.read(f"{DESIGN}/sub/a.status.md"))
        self.clean_check()

    def test_status_entry_split_copies_continuations_and_reports_once(self):
        self.write(f"{DESIGN}/a.status.md", "# status\n\n- **Departs:** 1 and 2.1: shared text.\n  Continued.\n")
        self.commit("split status")
        result = self.apply(f"{A} 2 -> {DESIGN}/new.md\ntitle {DESIGN}/new.md: New\n")
        self.assertEqual(result.stdout.count("split status entry"), 1)
        self.assertIn("- **Departs:** 1: shared text.\n  Continued.\n", self.read(f"{DESIGN}/new.status.md"))
        self.assertEqual(self.read(f"{DESIGN}/a.status.md"), "# status\n\n- **Departs:** 1: shared text.\n  Continued.\n")
        self.clean_check()

    def test_quiet_ambiguity_and_ambiguous_path_token_move(self):
        self.write(f"{DESIGN}/sub/a.md", DOC)
        self.write(B, "# b\n\n## 1. Own\n\nSee `a.md`, 2.1; section 1.\n")
        self.commit("harmless ambiguity")
        result = self.apply(f"{A} -> {DESIGN}/new/a.md\n{DESIGN}/sub/a.md -> {DESIGN}/else/a.md\n")
        self.assertNotIn("ambiguous", result.stdout)
        self.assertIn("`new/a.md`, 2.1", self.read(B))
        self.clean_check()

    def test_unread_section_names_and_final_report_coordinates(self):
        self.write(A, DOC + "\n```md\n(a.md, 2.1)\n```\n")
        self.write("skein/crates/k/src/lib.rs", 'const DOC: &str = "(a.md, 2.1)";\n')
        self.write(".config/docs.txt", "a.md, 2.1\n")
        self.commit("unread section names")
        result = self.apply(f"{A} 2 -> {DESIGN}/new.md 4\ntitle {DESIGN}/new.md: New\n")
        self.assertIn(f"{DESIGN}/new.md:10: name in code or other text: a.md", result.stdout)
        self.assertLess(result.stdout.index(".config/docs.txt"), result.stdout.index(f"{DESIGN}/new.md:10"))
        self.assertIn("skein/crates/k/src/lib.rs:1: name", result.stdout)
        self.clean_check()

    def test_backticked_link_labels_and_tails(self):
        self.write(B, "# b\n\n[`a.md`](a.md) and [`a.md#21-part`](a.md#21-part).\n")
        self.commit("backticked labels")
        self.apply(f"{A} -> {DESIGN}/sub/a.md\n")
        self.assertEqual(self.read(B), "# b\n\n[`sub/a.md`](sub/a.md) and [`sub/a.md#21-part`](sub/a.md#21-part).\n")
        self.clean_check()

    def test_keyboard_interrupt_rolls_back(self):
        mapping = f"{A} -> {DESIGN}/sub/a.md\n"
        before, index = self.snapshot(), self.git("write-tree")
        result = self.injected_move(mapping, '''real_git = git
def fail_add(*args):
    if args[0] == 'add':
        raise KeyboardInterrupt('injected interrupt')
    return real_git(*args)
git = fail_add
''')
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("KeyboardInterrupt", result.stderr)
        self.assertEqual(self.snapshot(), before)
        self.assertEqual(self.git("write-tree"), index)

    def test_ranges_keep_disjoint_members_and_same_document_reordering(self):
        self.write(A, "# a\n\n## 2. Two\n\n### 2.1 First\n\n### 2.2 Middle\n\n### 2.3 Last\n\n### 2.4 Fourth\n\n### 2.5 Fifth\n")
        self.write(B, "# b\n\n`a.md`, 2.1 to 2.3 and 2.5.\n")
        self.commit("disjoint ranges")
        self.apply(f"{A} 2 -> {DESIGN}/new.md\ntitle {DESIGN}/new.md: New\n")
        self.assertEqual(self.read(B), "# b\n\n`new.md`, 1 to 3 and 5.\n")
        self.clean_check()
        self.git("rm", "move.map")
        self.commit("remove map fixture")
        result = self.apply(f"{DESIGN}/new.md 2 -> {DESIGN}/new.md 6\n")
        self.assertNotIn("Wrong:", result.stdout)
        self.assertEqual(self.read(B), "# b\n\n`new.md`, 1 and 6 and 3 and 5.\n")
        self.clean_check()

    def test_copied_status_citations_use_each_destination_directory(self):
        self.write(f"{DESIGN}/a.status.md", "# status\n\n- **Open:** 1 and 2.1: shared.\n  See `b.md`, 3.\n")
        self.commit("copy status references")
        self.apply(f"{A} 1 -> {DESIGN}/one/new.md 4\n{A} 2 -> {DESIGN}/two/new.md\n"
                   f"title {DESIGN}/one/new.md: One\ntitle {DESIGN}/two/new.md: Two\n")
        for directory, label in (("one", "4"), ("two", "1")):
            self.assertTrue(self.read(f"{DESIGN}/{directory}/new.status.md").endswith(
                f"- **Open:** {label}: shared.\n  See `../b.md`, 3.\n"))
        self.clean_check()

    def test_status_ranges_and_wrapped_continuation_labels_follow_sections(self):
        self.write(A, "# a\n\n## 1. One\n\n## 2. Two\n\n## 3. Three\n\n## 4. Four\n")
        self.write(B, "# b\n")
        self.write(f"{DESIGN}/a.status.md", "# status\n\n- **Not yet:** 1 to 4: shared.\n  More. 2: another clause.\n  3: third clause.\n")
        self.commit("status ranges")
        result = self.apply(f"{A} 2 -> {DESIGN}/new.md\ntitle {DESIGN}/new.md: New\n")
        self.assertEqual(result.stdout.count("split status entry"), 1)
        self.assertIn("- **Not yet:** 1 to 4: shared.", self.read(f"{DESIGN}/a.status.md"))
        self.assertIn("More.  another clause.\n  3: third clause.", self.read(f"{DESIGN}/a.status.md"))
        self.assertIn("- **Not yet:** the whole document: shared.", self.read(f"{DESIGN}/new.status.md"))
        self.assertIn("More. the whole document: another clause.\n   third clause.", self.read(f"{DESIGN}/new.status.md"))
        self.clean_check()

    def test_whole_document_status_follows_extracted_documents(self):
        self.write(f"{DESIGN}/a.status.md", "# status\n\n- **Not yet:** the whole document, waiting.\n")
        self.commit("whole status")
        result = self.apply(f"{A} 2 -> {DESIGN}/new.md\ntitle {DESIGN}/new.md: New\n")
        self.assertEqual(result.stdout.count("split status entry"), 1)
        self.assertTrue(self.read(f"{DESIGN}/new.status.md").endswith("- **Not yet:** the whole document, waiting.\n"))
        self.assertEqual(self.read(f"{DESIGN}/a.status.md"), "# status\n\n- **Not yet:** the whole document, waiting.\n")
        self.clean_check()

    def test_status_index_backticked_document_label_and_tail(self):
        self.write(f"{DESIGN}/a.status.md", "# status\n\nNo gap.\n")
        self.write(B, "# b\n\n[`a.md`](a.status.md) and [`a.md` status](a.status.md).\n")
        self.commit("index labels")
        result = self.apply(f"{A} -> {DESIGN}/sub/a.md\n")
        self.assertEqual(self.read(B), "# b\n\n[`sub/a.md`](sub/a.status.md) and [`sub/a.md` status](sub/a.status.md).\n")
        self.assertNotIn("name in code", result.stdout)
        self.clean_check()

    def test_whole_section_link_anchor_and_plain_path(self):
        self.write(B, "# b\n\n[a](a.md#2-two), a.md, 2; [two](a.md), section 2.\n")
        self.commit("whole link")
        self.apply(f"{A} 2 -> {DESIGN}/new.md\ntitle {DESIGN}/new.md: New\n")
        self.assertEqual(self.read(B), "# b\n\n[a](new.md), new.md; [two](new.md).\n")
        self.clean_check()

    def test_excludes_versions_quantities_and_tilde_fences(self):
        self.write(A, DOC + "\nLinux 2.1 or later, overlayfs since 2.1; version 2.1; 2.1 hours; 2.1 MB/s.\n\n"
                   "~~~md\n2.1\n~~~\n\nOwn (2.1).\n")
        self.commit("numeric exclusions")
        self.apply(f"{A} 2 -> {DESIGN}/new.md\ntitle {DESIGN}/new.md: New\n")
        self.assertTrue(self.read(f"{DESIGN}/new.md").endswith(
            "Linux 2.1 or later, overlayfs since 2.1; version 2.1; 2.1 hours; 2.1 MB/s.\n\n~~~md\n2.1\n~~~\n\nOwn (1).\n"))
        self.clean_check()

    def test_sections_before_path_are_reported_with_final_coordinates(self):
        self.write(B, "# b\n\nSee sections 1 and 2.1 of `a.md`.\n")
        self.commit("dependent grammar")
        result = self.apply(f"{A} 2 -> {DESIGN}/new.md 4\ntitle {DESIGN}/new.md: New\n{B} -> {DESIGN}/sub/b.md\n")
        self.assertIn(f"{DESIGN}/sub/b.md:3: sections before the path need grammatical repair", result.stdout)
        self.assertEqual(self.read(f"{DESIGN}/sub/b.md"), "# b\n\nSee sections 1 and 2.1 of `../a.md`.\n")

    def test_suffix_shadow_can_keep_a_permitted_original_candidate(self):
        self.write("jig/docs/design/domain/client.md", "# client\n")
        self.write("jig/docs/design/protocol/client.md", "# client\n")
        self.write("temper/docs/design/a.md", "# a\n\njig has two `client.md`.\n")
        self.commit("generic suffix name")
        mapping = "jig/docs/design/protocol/client.md -> temper/docs/design/client.md\n"
        broken = self.injected_move(mapping, "Plan.preserve_suffix_name = lambda self, cite, shadowed, resolved: False", "--dry-run")
        self.assertEqual(broken.returncode, 1, broken.stdout + broken.stderr)
        self.assertIn("Wrong:", broken.stdout)
        self.assertIn("preserved bare name", broken.stdout)
        result = self.apply(mapping)
        self.assertNotIn("Wrong:", result.stdout)
        self.assertIn("the bare name now resolves to temper/docs/design/client.md", result.stdout)
        self.assertEqual(self.read("temper/docs/design/a.md"), "# a\n\njig has two `client.md`.\n")
        self.clean_check()

    def test_split_markdown_link_and_or_list(self):
        self.write(B, "# b\n\n[`a.md`](a.md), sections 1 or 2.1.\n")
        self.commit("split links")
        self.apply(f"{A} 2 -> {DESIGN}/new.md 4\ntitle {DESIGN}/new.md: New\n")
        self.assertEqual(self.read(B), "# b\n\n[`a.md`](a.md), section 1, or [`new.md`](new.md), section 4.1.\n")
        self.clean_check()

    def test_bare_number_count_excludes_unchanged_members(self):
        self.write(A, DOC + "\n### 2.2 Other\n\nAs 2.1 and 2.2 say.\n")
        self.commit("count changed members")
        result = self.apply(f"{A} 2.1 -> {A} 2.3\n")
        self.assertIn("1 bare numbers", result.stdout)
        self.assertIn("As 2.3 and 2.2 say.", self.read(A))
        self.clean_check()

    def test_section_only_range_in_string_and_extensionless_name(self):
        self.write(A, DOC + "\n### 2.2 Middle\n\n### 2.3 Last\n")
        self.write(".config/range.txt", '"(a.md, 2.1 to 2.3)"\n')
        self.write(f"{DESIGN}/assets/NOTICE", "Notice.\n")
        self.write(".config/assets.txt", "skein/docs/design/assets/NOTICE\n")
        self.commit("unread names")
        result = self.apply(f"{A} 2.2 -> {DESIGN}/new.md 1\ntitle {DESIGN}/new.md: New\n"
                            f"{DESIGN}/assets/ -> {DESIGN}/web/\n")
        self.assertIn(".config/range.txt:1: name in code or other text: a.md", result.stdout)
        self.assertIn(".config/assets.txt:1: name in code or other text: skein/docs/design/assets/NOTICE", result.stdout)
        self.clean_check()

    def test_ambiguous_candidates_with_identical_number_rewrites_are_quiet(self):
        self.write(A, DOC + "\nSee [b](b.md); section 2.\n")
        self.write(B, "# b\n\n## 2. Two\n")
        self.commit("same candidate rewrites")
        result = self.apply(f"{A} 2 -> {A} 4\n{B} 2 -> {B} 4\n")
        self.assertNotIn("ambiguous", result.stdout)
        self.assertTrue(self.read(A).endswith("See [b](b.md); section 4.\n"))
        self.clean_check()

    def test_companion_link_labels_are_consumed_even_when_the_name_stays(self):
        self.write(f"{DESIGN}/a.status.md", "# status\n\nSee [a.md](a.md).\n\nNo gap.\n")
        self.commit("companion link label")
        result = self.apply(f"{A} -> {DESIGN}/sub/a.md\n")
        self.assertNotIn("name in code", result.stdout)
        self.assertEqual(self.read(f"{DESIGN}/sub/a.status.md"), "# status\n\nSee [a.md](a.md).\n\nNo gap.\n")
        self.clean_check()

    def test_discarded_section_heading_has_no_surviving_citation(self):
        self.write(A, "# a\n\n## 2. See `b.md`\n\nOpening.\n\n### 2.1 Part\n\nBody.\n")
        self.commit("discarded heading")
        result = self.apply(f"{A} 2 -> {DESIGN}/new.md\ntitle {DESIGN}/new.md: New\n")
        self.assertNotIn("Wrong:", result.stdout)
        self.assertEqual(self.read(f"{DESIGN}/new.md"), "# New\n\nOpening.\n\n## 1. Part\n\nBody.\n")
        self.clean_check()

    def test_newly_read_citations_need_an_original_reading(self):
        self.write("skein/docs/plan/old.md", "# old\n\nSee `a.md`, 2.1.\n")
        self.commit("unread source")
        result = self.move(f"skein/docs/plan/old.md -> {DESIGN}/new.md\n")
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        self.assertIn("Wrong:", result.stdout)
        self.assertIn("should read: an original reading", result.stdout)

    def test_missing_original_heading_cannot_gain_a_different_heading(self):
        self.write(B, "# b\n\nSee `a.md`, 9.\n")
        self.commit("missing original heading")
        result = self.move(f"{A} 2 -> {A} 9\n")
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        self.assertIn("Wrong:", result.stdout)
        self.assertIn("should read: unresolved", result.stdout)

    def test_whole_destination_subsumes_its_numbered_list_members(self):
        self.write(B, "# b\n\n`a.md`, 2 and 2.1; `a.md`, 2.1 and 2; `a.md`, 1 and 2 and 2.1.\n")
        self.commit("whole list members")
        self.apply(f"{A} 2 -> {DESIGN}/new.md\ntitle {DESIGN}/new.md: New\n")
        self.assertEqual(self.read(B), "# b\n\n`new.md`; `new.md`; `a.md`, 1 and `new.md`.\n")
        self.clean_check()

    def test_whole_status_citation_keeps_the_status_path(self):
        self.write(f"{DESIGN}/a.status.md", "# status\n\n- **Not yet:** 2.1: waiting.\n")
        self.write(B, "# b\n\nSee `a.status.md`, 2 and 2.1.\n")
        self.commit("whole status citation")
        self.apply(f"{A} 2 -> {DESIGN}/new.md\ntitle {DESIGN}/new.md: New\n")
        self.assertEqual(self.read(B), "# b\n\nSee `new.status.md`.\n")
        self.clean_check()

    def test_split_status_report_is_identical_across_hash_seeds(self):
        self.write(f"{DESIGN}/a.status.md", "# status\n\n- **Not yet:** 1 and 2.1: shared.\n")
        self.commit("split status report")
        mapping = f"{A} 2 -> {DESIGN}/new.md\ntitle {DESIGN}/new.md: New\n"
        dry = self.move(mapping, "--dry-run", env={"PATH": os.defpath, "PYTHONHASHSEED": "1"})
        applied = self.move(mapping, env={"PATH": os.defpath, "PYTHONHASHSEED": "2"})
        self.assertEqual(dry.returncode, 0, dry.stdout + dry.stderr)
        self.assertEqual(applied.returncode, 0, applied.stdout + applied.stderr)
        self.assertEqual(dry.stdout, applied.stdout)
        self.clean_check()

    def test_status_body_own_numbers_and_capitalised_sections_follow_referents(self):
        self.write(A, DOC + "\n## 9. Open\n")
        self.write(f"{DESIGN}/a.status.md", "# status\n\n- **Departs:** 2.1: 2.1's rule; Section 9 keeps the question.\n")
        self.commit("status body references")
        result = self.apply(f"{A} 2 -> {DESIGN}/new.md\ntitle {DESIGN}/new.md: New\n")
        self.assertIn("1 bare numbers", result.stdout)
        self.assertTrue(self.read(f"{DESIGN}/new.status.md").endswith(
            "- **Departs:** 1: 1's rule; `a.md`, section 9 keeps the question.\n"))
        self.clean_check()

    def test_capitalised_path_sections_keep_the_explicit_document(self):
        self.write(A, DOC + "\nSee `b.md`, Section 3.\n")
        self.commit("capital path section")
        self.apply(f"{B} 3 -> {B} 4\n")
        self.assertTrue(self.read(A).endswith("See `b.md`, Section 4.\n"))
        self.clean_check()

    def test_own_dotted_reader_keeps_attributed_sections_and_protocol_versions(self):
        self.write(A, DOC + "\nRFC 6749, section 2.1; RFC 9500's `testRSA` (section 2.1).\n"
                   "Its section 2.1; HTTP/2.1; TLS 2.1. Own (2.1).\n")
        self.commit("attributed numbers")
        self.apply(f"{A} 2 -> {DESIGN}/new.md\ntitle {DESIGN}/new.md: New\n")
        self.assertTrue(self.read(f"{DESIGN}/new.md").endswith(
            "RFC 6749, section 2.1; RFC 9500's `testRSA` (section 2.1).\n"
            "Its section 2.1; HTTP/2.1; TLS 2.1. Own (1).\n"))
        self.clean_check()

    def test_capitalised_ranges_keep_plural_and_local_word_case(self):
        self.write(A, "# a\n\n(Sections 2.1 to 2.2).\n\n## 2. Two\n\n(Sections 2.1 to 2.2).\n\n### 2.1 First\n\n### 2.2 Second\n")
        self.commit("capital ranges")
        self.apply(f"{A} 2 -> {DESIGN}/new.md\ntitle {DESIGN}/new.md: New\n")
        self.assertEqual(self.read(A), "# a\n\n(`new.md`, sections 1 to 2).\n\n")
        self.assertIn("(Sections 1 to 2).", self.read(f"{DESIGN}/new.md"))
        self.clean_check()

    def test_wrapped_splits_keep_paragraphs_and_comment_prefixes(self):
        self.write(B, "# b\n\nSee `a.md`, sections\n  1 and\n  2.1 for detail.\n\nNext `a.md`, sections\n  1 and\n  2.1\ncontinues.\n")
        self.write("skein/crates/k/src/lib.rs", "//! See `docs/design/a.md`, sections\n//!   1 and\n//!   2.1 for detail.\n")
        self.commit("wrapped split")
        self.apply(f"{A} 2 -> {DESIGN}/new.md 4\ntitle {DESIGN}/new.md: New\n")
        self.assertEqual(self.read(B), "# b\n\nSee `a.md`, section 1, and `new.md`, section 4.1\n   for detail.\n\nNext `a.md`, section 1, and `new.md`, section 4.1\ncontinues.\n")
        self.assertEqual(self.read("skein/crates/k/src/lib.rs"), "//! See `docs/design/a.md`, section 1, and `docs/design/new.md`, section 4.1\n//!    for detail.\n")
        self.clean_check()

    def test_citation_coordinates_follow_a_promoted_heading_prefix(self):
        self.write(A, DOC.replace("### 2.1 Part", "### 2.1 Part (`b.md`, 3)"))
        self.commit("heading citation")
        self.apply(f"{A} 2.1 -> {DESIGN}/new.md 4\ntitle {DESIGN}/new.md: New\n")
        self.assertIn("## 4. Part (`b.md`, 3)", self.read(f"{DESIGN}/new.md"))
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
            (f"{A} -> {DESIGN}/sub/new", "end in .md"),
            (f"{A} -> skein/crates/new.md", "five trees"),
            (f"{A} -> {DESIGN}/../new.md", "without . or .."),
            (f"{A} 2 -> {B} 3", "destination section"),
            (f"{A} 2.1 -> {B} 8.1", "parent section"),
            (f"{A} 2 -> {B}", "created document"),
            (f"{A} 2 -> {DESIGN}/new.md", "title"),
            (f"{A} 1 -> {DESIGN}/new.md\n{A} 2 -> {DESIGN}/new.md\ntitle {DESIGN}/new.md: New", "whole-document destination"),
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
