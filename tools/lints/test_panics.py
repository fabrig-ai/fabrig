"""Regression checks for tools/lints/design.md, 7, on repositories built per test."""
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

HERE = Path(__file__).resolve().parent
LIST = "tools/lints/panics.txt"
SUMMARY = "0 findings (skein/docs/foundation/programming-model.md, 10.4)\n"


class Panics(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory(prefix="panics-")
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        shutil.copytree(HERE, self.root / "tools/lints")
        self.write(LIST, "# fixture list\n")
        self.git("init", "-q")
        self.git("add", ".")

    def write(self, path, text):
        path = self.root / path
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")

    def track(self, path, text):
        self.write(path, text)
        self.git("add", path)

    def git(self, *args):
        return subprocess.run(("git", *args), cwd=self.root, capture_output=True, text=True, check=True)

    def run_check(self, *args, status=0):
        result = subprocess.run((sys.executable, "-I", "-B", "tools/lints/panics.py", *args),
                                cwd=self.root, capture_output=True, text=True)
        self.assertEqual(result.returncode, status, result.stdout + result.stderr)
        self.assertEqual(result.stderr, "")
        return result.stdout

    def test_a_use_and_a_call_are_refused(self):
        self.track("a.rs", "use std::panic::catch_unwind;\nlet _ = std::panic::catch_unwind(f);\n")
        output = self.run_check(status=1)
        self.assertEqual(output, "a.rs: catch_unwind count 2, refused\n"
                         "1 findings (skein/docs/foundation/programming-model.md, 10.4)\n")

    def test_unwind_names_count_lines_once_including_comments(self):
        self.track("a.rs", "catch_unwind(f); catch_unwind(g); resume_unwind(x); resume_unwind(y);\n"
                   "// catch_unwind and resume_unwind\n")
        self.assertEqual(self.run_check(status=1), "a.rs: catch_unwind count 2, refused\n"
                         "a.rs: resume_unwind count 2, refused\n"
                         "2 findings (skein/docs/foundation/programming-model.md, 10.4)\n")

    def test_containing_names_and_expectations_count_none(self):
        self.track("a.rs", "catch_unwind_silent(f); my_resume_unwind(x);\n"
                   '#[expect(clippy::should_panic_without_expect, reason = "fixture")]\n')
        self.assertEqual(self.run_check(), SUMMARY)

    def test_attributes_count_each_but_not_comment_lines(self):
        self.track("a.rs", '  #[should_panic(expected = "x")]\n#[should_panic]\n'
                   '#[ should_panic] #[should_panic]\n'
                   '  /// quoting #[should_panic(expected = "x")]\n'
                   '\t// #[should_panic]\n//! #[should_panic]\n')
        self.write(LIST, "a.rs should_panic 4\n")
        self.assertEqual(self.run_check(), SUMMARY)

    def test_only_tracked_rust_files_count(self):
        patterns = "catch_unwind resume_unwind #[should_panic]\n"
        self.write("untracked.rs", patterns)
        self.track("tracked.txt", patterns)
        self.assertEqual(self.run_check(), SUMMARY)

    def test_non_utf8_rust_still_counts(self):
        self.track("a.rs", "")
        (self.root / "a.rs").write_bytes(b"// \xff catch_unwind\n")
        output = self.run_check(status=1)
        self.assertIn("a.rs: catch_unwind count 1, refused\n", output)
        self.assertNotIn("Traceback", output)

    def test_a_count_above_its_line_is_a_finding(self):
        self.track("a.rs", "#[should_panic]\n#[should_panic]\n")
        self.write(LIST, "a.rs should_panic 1\n")
        self.assertIn("a.rs: should_panic count 2 exceeds list 1\n", self.run_check(status=1))

    def test_a_pattern_with_no_line_cannot_return(self):
        self.track("a.rs", "#[should_panic]\n")
        self.assertIn("a.rs: should_panic count 1 exceeds list 0\n", self.run_check(status=1))

    def test_a_refused_count_is_a_finding_without_a_list_line(self):
        for pattern in ("catch_unwind", "resume_unwind"):
            with self.subTest(pattern=pattern):
                self.track("a.rs", f"{pattern}(f);\n")
                self.assertEqual(self.run_check(status=1), f"a.rs: {pattern} count 1, refused\n"
                                 "1 findings (skein/docs/foundation/programming-model.md, 10.4)\n")

    def test_a_refused_list_line_sets_no_limit(self):
        for pattern in ("catch_unwind", "resume_unwind"):
            for source in (f"{pattern}(x);\n", ""):
                with self.subTest(pattern=pattern, source=source):
                    self.track("a.rs", source)
                    self.write(LIST, f"a.rs {pattern} 1\n")
                    counted = f"a.rs: {pattern} count 1, refused\n" if source else ""
                    total = 2 if source else 1
                    self.assertEqual(self.run_check(status=1),
                                     f"{LIST}: line 1: {pattern} is refused; no line names it\n"
                                     f"{counted}{total} findings "
                                     "(skein/docs/foundation/programming-model.md, 10.4)\n")

    def test_a_refused_line_is_not_an_ordering_error(self):
        self.track("a.rs", "#[should_panic]\nresume_unwind(x);\n")
        self.write(LIST, "a.rs should_panic 1\na.rs resume_unwind 1\n")
        self.assertEqual(self.run_check(status=1),
                         f"{LIST}: line 2: resume_unwind is refused; no line names it\n"
                         "a.rs: resume_unwind count 1, refused\n"
                         "2 findings (skein/docs/foundation/programming-model.md, 10.4)\n")

    def test_malformed_lines_are_findings(self):
        for line in ("\n", "a.rs should_panic\n", "a.rs other 1\n", "a.rs should_panic 0\n",
                     "a.rs should_panic -1\n", "a.rs should_panic one\n",
                     "a.rs  should_panic 1\n", "a.rs should_panic  1\n",
                     "a.rs should_panic 1 \n", "a.rs\tshould_panic 1\n"):
            with self.subTest(line=line):
                self.write(LIST, line)
                output = self.run_check(status=1)
                self.assertIn(f"{LIST}: line 1: expected PATH PATTERN COUNT", output)
                self.assertIn("1 findings (skein/docs/foundation/programming-model.md, 10.4)", output)

    def test_lines_must_be_in_path_order(self):
        self.write(LIST, "b.rs should_panic 1\na.rs should_panic 1\n")
        self.assertIn(f"{LIST}: line 2: a.rs should_panic is out of order\n",
                      self.run_check(status=1))
        self.write(LIST, "a.rs should_panic 1\nb.rs should_panic 1\n")
        self.run_check()

    def test_a_repeated_path_and_pattern_is_a_finding(self):
        self.write(LIST, "a.rs should_panic 1\na.rs should_panic 1\n")
        self.assertIn(f"{LIST}: line 2: repeated a.rs should_panic\n", self.run_check(status=1))

    def test_no_list_is_a_finding(self):
        (self.root / LIST).unlink()
        output = self.run_check(status=1)
        self.assertIn(f"{LIST}: no list; initialize with --write\n", output)
        self.assertIn("1 findings (skein/docs/foundation/programming-model.md, 10.4)", output)

    def test_lower_counts_and_gone_files_remind_without_failing(self):
        self.track("a.rs", "#[should_panic]\n")
        self.track("gone.rs", "#[should_panic]\n")
        (self.root / "gone.rs").unlink()
        lines = "a.rs should_panic 2\ngone.rs should_panic 1\n"
        self.write(LIST, lines)
        output = self.run_check()
        self.assertIn("a.rs: should_panic count 1, list 2; lower with --write\n", output)
        self.assertIn("gone.rs: should_panic count 0, list 1; lower with --write\n", output)
        self.assertTrue(output.endswith(SUMMARY))
        self.assertEqual((self.root / LIST).read_text(), lines)

    def test_write_lowers_drops_zero_and_keeps_comments_in_place(self):
        self.track("a.rs", "#[should_panic]\n")
        self.track("b.rs", "")
        self.write(LIST, "# first\na.rs should_panic 2\n# between\nb.rs should_panic 1\n"
                   "# last file\ngone.rs should_panic 1\n# end\n")
        self.assertEqual(self.run_check("--write"), SUMMARY)
        self.assertEqual((self.root / LIST).read_text(),
                         "# first\na.rs should_panic 1\n# between\n# last file\n# end\n")
        self.track("b.rs", "#[should_panic]\n")
        self.assertIn("b.rs: should_panic count 1 exceeds list 0\n", self.run_check(status=1))

    def test_write_changes_nothing_with_any_finding(self):
        self.track("a.rs", "#[should_panic]\n")
        for lines, source in (("a.rs should_panic 2\nz.rs should_panic 1\n", "#[should_panic]\n" * 2),
                              ("a.rs should_panic 2\n", "#[should_panic]\n"),
                              ("a.rs should_panic 2\nmalformed\n", "")):
            with self.subTest(lines=lines):
                self.track("z.rs", source)
                self.write(LIST, lines)
                self.run_check("--write", status=1)
                self.assertEqual((self.root / LIST).read_text(), lines)

    def test_write_refuses_counts_with_a_list(self):
        self.track("b.rs", "#[should_panic]\n")
        for pattern in ("catch_unwind", "resume_unwind"):
            for refused_line in ("", f"a.rs {pattern} 1\n"):
                with self.subTest(pattern=pattern, refused_line=refused_line):
                    self.track("a.rs", f"{pattern}(f);\n")
                    lines = f"# keep\n{refused_line}b.rs should_panic 2\n"
                    self.write(LIST, lines)
                    self.assertIn(f"a.rs: {pattern} count 1, refused\n",
                                  self.run_check("--write", status=1))
                    self.assertEqual((self.root / LIST).read_text(), lines)

    def test_write_refuses_counts_without_a_list(self):
        self.track("b.rs", "#[should_panic]\n")
        for pattern in ("catch_unwind", "resume_unwind"):
            with self.subTest(pattern=pattern):
                (self.root / LIST).unlink(missing_ok=True)
                self.track("a.rs", f"{pattern}(f);\n")
                self.assertEqual(self.run_check("--write", status=1),
                                 f"a.rs: {pattern} count 1, refused\n"
                                 "1 findings (skein/docs/foundation/programming-model.md, 10.4)\n")
                self.assertFalse((self.root / LIST).exists())

    def test_write_initializes_should_panic_counts_in_path_order(self):
        (self.root / LIST).unlink()
        self.track("z.rs", "#[should_panic]\n" * 2)
        self.track("a space.rs", "#[should_panic]\n")
        self.assertEqual(self.run_check("--write"), SUMMARY)
        lines = (self.root / LIST).read_text().splitlines()
        self.assertEqual(lines, ["# tools/lints/design.md, 7: the count of catches.",
                                 "# --write lowers this list; nothing raises it.",
                                 "# Refused, so no line names them: catch_unwind, resume_unwind.",
                                 "a space.rs should_panic 1", "z.rs should_panic 2"])
        self.assertEqual(self.run_check(), SUMMARY)


if __name__ == "__main__":
    unittest.main()
