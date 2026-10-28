"""The gate ratchets old forms in tracked Rust without raising a baseline."""

from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


SCRIPT = Path(__file__).with_name("old_forms.py").resolve()
FORMS = ("max_out", "JournalRoom", "takes", "worst_case")
HEADER = (
    "# Uses of the old forms (skein's docs/design/draft/dynamic-memory.md, 7): each count may only fall.\n"
    "# Lower them with `python3 -I -B scripts/old_forms.py --write`; "
    "only the director raises one, recorded in a `# Raised:` line.\n"
)
RULE = (
    "Uses of the old forms may only fall (skein's docs/design/draft/dynamic-memory.md, 7): "
    "new code takes the new forms (Queue::uncapped, Journal::uncapped, decide, backlog), "
    "or removes as many old uses as it adds."
)


class OldForms(unittest.TestCase):
    def repository(self, files, counts=(0, 0, 0, 0), untracked=None):
        temporary = tempfile.TemporaryDirectory(prefix="test-old-forms-")
        self.addCleanup(temporary.cleanup)
        root = Path(temporary.name)
        subprocess.run(["git", "init", "--quiet", str(root)], check=True,
                       capture_output=True, timeout=10)
        for name, source in files.items():
            path = root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(source, encoding="utf-8")
        subprocess.run(["git", "add", "--", *files], cwd=root, check=True,
                       capture_output=True, timeout=10)
        for name, source in (untracked or {}).items():
            path = root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(source, encoding="utf-8")
        (root / "scripts").mkdir(exist_ok=True)
        (root / "scripts/old-forms.txt").write_text(
            HEADER + "".join(f"{form} {count}\n" for form, count in zip(FORMS, counts)),
            encoding="utf-8")
        return root

    def run_script(self, root, *args, exit=0, lines=()):
        result = subprocess.run([sys.executable, "-I", "-B", str(SCRIPT), *args],
                                cwd=root, text=True, capture_output=True, timeout=10)
        self.assertEqual(result.returncode, exit, result.stderr)
        self.assertEqual(result.stdout, "")
        self.assertEqual(result.stderr.splitlines(), list(lines))

    def test_a_count_above_its_baseline_fails_naming_the_form_and_the_rule(self):
        root = self.repository({"src/lib.rs": """
const max_out_events: usize = 1;
let room: JournalRoom = journal.room();
journal.takes(&room);
let size = payload.worst_case();
"""})
        self.run_script(root, exit=1, lines=[
            *(f"{form}: count 1, baseline 0" for form in FORMS), RULE])

    def test_a_use_in_a_comment_or_a_doc_comment_does_not_count(self):
        root = self.repository({"src/lib.rs": """
// max_out JournalRoom journal.takes(&room) worst_case
/// max_out JournalRoom journal.takes(&room) worst_case
//! max_out JournalRoom journal.takes(&room) worst_case
/* max_out /* JournalRoom */ journal.takes(&room) worst_case */
fn ordinary() {}
"""})
        self.run_script(root)

    def test_a_comment_marker_inside_a_string_does_not_hide_the_use_after_it(self):
        root = self.repository({"src/lib.rs": '''
let url = "https://example.test"; let max_out = 1;
let marker = r#"/*"#; let room: JournalRoom = journal.room();
let marker = "/*"; journal.takes(&room); payload.worst_case();
'''}, counts=(1, 1, 1, 1))
        self.run_script(root)

    def test_io_edge_constants_do_not_count_as_max_out(self):
        root = self.repository({"src/lib.rs": """
let edges = (MAX_OUT_DOWN, MAX_OUT_UP);
let internal = (MAX_OUT, max_out_events, MAX_OUT_DOWNSTREAM);
"""}, counts=(3, 0, 0, 0))
        self.run_script(root)

    def test_generated_directories_and_untracked_files_do_not_count(self):
        root = self.repository({
            "src/lib.rs": "fn ordinary() {}\n",
            "generated/top.rs": "max_out JournalRoom journal.takes(&room) worst_case\n",
            "src/generated/codec.rs": "max_out JournalRoom journal.takes(&room) worst_case\n",
            "src/generated_helpers.rs": "let size = payload.worst_case();\n",
        }, counts=(0, 0, 0, 1), untracked={
            "src/untracked.rs": "max_out JournalRoom journal.takes(&room) worst_case\n",
        })
        self.run_script(root)

    def test_a_count_below_its_baseline_passes_with_a_note(self):
        root = self.repository({"src/lib.rs": "let size = payload.worst_case();\n"},
                               counts=(0, 0, 0, 2))
        self.run_script(root, lines=[
            "worst_case: count 1, baseline 2; lower with "
            "`python3 -I -B scripts/old_forms.py --write`"])

    def test_write_lowers_a_count_and_keeps_raised_lines(self):
        root = self.repository({"src/lib.rs": "let size = payload.worst_case();\n"},
                               counts=(0, 0, 0, 2))
        baseline = root / "scripts/old-forms.txt"
        before = baseline.read_text(encoding="utf-8").replace(
            "max_out 0\n", "# Raised: max_out +1, director's recorded exception\nmax_out 0\n"
        ).replace("worst_case 2\n",
                  "# Raised: worst_case +1, director's recorded exception\nworst_case 2\n")
        baseline.write_text(before, encoding="utf-8")
        self.run_script(root, "--write")
        self.assertEqual(baseline.read_bytes(),
                         before.replace("worst_case 2\n", "worst_case 1\n").encode("utf-8"))

    def test_write_refuses_to_raise_and_leaves_the_file_unchanged(self):
        root = self.repository({"src/lib.rs": "let size = payload.worst_case();\n"},
                               counts=(1, 0, 0, 0))
        baseline = root / "scripts/old-forms.txt"
        before = baseline.read_bytes()
        self.run_script(root, "--write", exit=1,
                        lines=["worst_case: count 1, baseline 0", RULE])
        self.assertEqual(baseline.read_bytes(), before)


if __name__ == "__main__":
    unittest.main()
