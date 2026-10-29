"""The old-forms gate counts tracked Rust and lets each count only fall."""

from pathlib import Path
import runpy
import subprocess
import sys
import tempfile
import unittest


SCRIPT = Path(__file__).with_name("old_forms.py").resolve()


class OldForms(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory(prefix="test-old-forms-")
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        (self.root / "scripts").mkdir()
        self.baseline = self.root / "scripts/old-forms.txt"
        subprocess.run(["git", "init", "-q"], cwd=self.root, check=True,
                       capture_output=True)

    def source(self, text, path="src/lib.rs"):
        target = self.root / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(text, encoding="utf-8")
        subprocess.run(["git", "add", "--", path], cwd=self.root, check=True,
                       capture_output=True)

    def run_script(self, *arguments, expected=0):
        result = subprocess.run([sys.executable, "-I", "-B", str(SCRIPT), *arguments],
                                cwd=self.root, text=True, capture_output=True,
                                timeout=10)
        self.assertEqual(result.returncode, expected, result.stderr)
        return result

    def counts(self):
        return {name: int(count) for line in self.baseline.read_text(encoding="utf-8").splitlines()
                if line and not line.startswith("#") for name, count in [line.split()]}

    def test_comments_do_not_count_and_strings_do(self):
        self.source('// worst_case\n/* max_out /* nested */ */\n'
                    'const TEXT: &str = "worst_case";\nfn worst_case() {}\n')
        self.run_script("--write")
        self.assertEqual(self.counts(), {"max_out": 0, "JournalRoom": 0, "takes": 0, "worst_case": 2})
        self.run_script()

    def test_a_raise_fails_and_states_the_rule(self):
        self.source("fn worst_case() {}\n")
        self.run_script("--write")
        baseline = self.baseline.read_text(encoding="utf-8")
        self.source("fn worst_case() { worst_case(); }\n")
        for arguments in ((), ("--write",)):
            with self.subTest(arguments=arguments):
                result = self.run_script(*arguments, expected=1)
                self.assertIn("worst_case: count 2, baseline 1", result.stderr)
                self.assertIn("may only fall", result.stderr)
                self.assertEqual(self.baseline.read_text(encoding="utf-8"), baseline)

    def test_a_fall_passes_and_write_lowers_it_keeping_raises(self):
        self.source("fn worst_case() { worst_case(); }\n")
        self.run_script("--write")
        raised = "# Raised: worst_case by director for the fixture\n"
        baseline = self.baseline.read_text(encoding="utf-8") + raised
        self.baseline.write_text(baseline, encoding="utf-8")
        self.source("fn worst_case() {}\n")
        result = self.run_script()
        self.assertIn("worst_case: count 1, baseline 2", result.stderr)
        self.assertIn("--write", result.stderr)
        self.assertEqual(self.baseline.read_text(encoding="utf-8"), baseline)
        self.run_script("--write")
        self.assertEqual(self.counts()["worst_case"], 1)
        self.assertIn(raised, self.baseline.read_text(encoding="utf-8"))
        self.assertEqual(self.run_script().stderr, "")

    def test_generated_components_are_not_counted(self):
        forms = "fn max_out() { JournalRoom; queue.takes(&input); worst_case(); }\n"
        self.source(forms, "generated/top.rs")
        self.source(forms, "src/generated/nested.rs")
        self.source(forms, "src/generated_more.rs")
        self.run_script("--write")
        self.assertEqual(self.counts(), {"max_out": 1, "JournalRoom": 1, "takes": 1, "worst_case": 1})

    def test_edge_identifiers_are_exempt_only_as_whole_identifiers(self):
        self.source('MAX_OUT_DOWN MAX_OUT_UP max_output_tokens max_output_bytes '
                    'my_max_output_tokens\n')
        self.run_script("--write")
        self.assertEqual(self.counts()["max_out"], 1)

    def test_escapes_separate_words_in_ordinary_literals(self):
        for literal, expected in (
                (r'"\nmax_out"', 1),
                (r'"\nmax_output_bytes"', 0),
                (r'"\x41max_out"', 1),
                (r'"\u{41}max_out"', 1),
                (r"'\n'max_out", 1),
        ):
            with self.subTest(literal=literal):
                self.source(literal)
                self.baseline.unlink(missing_ok=True)
                self.run_script("--write")
                self.assertEqual(self.counts()["max_out"], expected)

    def test_raw_strings_keep_escape_text_as_written(self):
        self.source(r'r"\nmax_out"')
        self.run_script("--write")
        self.assertEqual(self.counts()["max_out"], 1)
        counter = runpy.run_path(str(SCRIPT))
        candidates = counter["without_comments"](r'r"\nmax_out"')
        self.assertEqual(counter["FORMS"]["max_out"].findall(candidates), ["nmax_out"])

    def test_a_missing_baseline_needs_write(self):
        self.source("fn ordinary() {}\n")
        result = self.run_script(expected=2)
        self.assertIn("scripts/old-forms.txt is missing", result.stderr)
        self.assertIn("--write", result.stderr)
        self.assertFalse(self.baseline.exists())
        self.run_script("--write")
        self.assertEqual(self.counts(), {"max_out": 0, "JournalRoom": 0, "takes": 0, "worst_case": 0})
        self.run_script()


if __name__ == "__main__":
    unittest.main()
