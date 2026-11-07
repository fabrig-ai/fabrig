+++
id = "recordings"
part = "recordings"
kind = "feature"
needs = []
files = [
  "hearth/testing/recordings/mmdebstrap/1.5.7-apt3.0.4/**",
  "hearth/tests/mmdebstrap/expect/1.5.7-apt3.0.4/*.expect",
]
effort = "medium"
minutes = 30
+++

# mmdebstrap's recordings for apt 3.0.4

## Goal

Debian trixie's apt is now 3.0.4, and hearth's conformance cases read `hearth/testing/recordings/mmdebstrap/<mmdebstrap's version>-apt<apt's>/` (`hearth/docs/design/domain/testing.md`, 4.5), where main holds only `1.5.7-apt3.0.3`. This task adds the captured `1.5.7-apt3.0.4` recordings, byte for byte, and their reviewed expectations, beside 3.0.3's, so a machine with either apt reads its own.

This task departs from the implementer's text in one way: it changes no code, so there is no clippy or `cargo fmt` step, and no new test to make bite (it adds data, and the existing tests are its check).

Acceptance:

1. `hearth/testing/recordings/mmdebstrap/1.5.7-apt3.0.4/` holds the seven cases of `hearth/docs/plan/hearth-apt-304/inputs/1.5.7-apt3.0.4/`, every file byte for byte: `diff -r` of the two directories prints nothing and exits 0, and `git ls-files` lists the same 50 files under each.
2. `hearth/tests/mmdebstrap/expect/1.5.7-apt3.0.4/<case>.expect` exists for each of the seven cases, each the draft that `recordings::every_recording_has_an_expectation_and_the_reference_agrees` printed for it. Each is expected to equal its `1.5.7-apt3.0.3` expectation byte for byte (`fetched` is a byte count, not a rate, and no expectation holds an `I: success` line); a draft that differs is kept, and the difference is named in the report.
3. Nothing else changes: `hearth/docs/plan/hearth-apt-304/inputs/` and `1.5.7-apt3.0.3`'s recordings and expectations stay as they are (`git diff --stat origin/main...HEAD` names only files under the two new directories, unless an option below was taken).
4. The focused and fuzzy suites of `hearth-mmdebstrap-world`, `hearth-builder-world` and `hearth-conformance-world` pass.

## Inputs

- `hearth/docs/plan/hearth-apt-304/notes/common.md`: binds.
- `hearth/docs/plan/hearth-apt-304/inputs/request.md`: the request, items 1 to 3 and its limits.
- `hearth/docs/design/domain/testing.md`, section 4.5: recordings are captured, never written by hand; mmdebstrap's sit under `<its version>-apt<apt's>/`; a new version is a diff to review.
- What reads the recordings, surveyed on the trunk:
  - `hearth/testing/hearth-recordings/src/lib.rs`, `all`: every version directory, sorted, then every case, so `1.5.7-apt3.0.3`'s seven cases come before `1.5.7-apt3.0.4`'s.
  - `hearth/tests/mmdebstrap/src/recordings.rs`, `all`: pairs each with `hearth/tests/mmdebstrap/expect/<version>/<case>.expect`.
  - `hearth/tests/mmdebstrap/tests/suite/recordings.rs`: eight tests over every recording; `the_recorded_failures_have_the_causes_5_5_gives_them` finds each case's first recording (3.0.3's), and 3.0.4's causes are held by its expectations through the first test. `hearth/tests/mmdebstrap/tests/suite/fuzzy/mmdebstrap.rs`, `recordings_cut_and_mutated_read_as_the_reference_reads_them`: its 128 seeds now spread over 14 cases. Neither needs a change.
  - `hearth/tests/builder/src/fixtures/resolution.rs`, `recorded_closure`, and `hearth/tests/conformance/tests/recorded_resolve.rs` and `hearth/tests/conformance/tests/conformance_resolve.rs`: read only by `conformance_*` and `recorded_*` binaries, which the focused and fuzzy profiles leave out (`.config/nextest.toml`). Not run here; after-landing runs them.
- Checked by the cut: against `1.5.7-apt3.0.3`, the inputs differ only in `Fetched 10.2 MB in 1s (… /s)` in five cases and `I: success in … seconds` in `test-min` and `test-boot`.

## Owns and may change

**Owned:** the two new directories in `files`.

**Readers and neighbours, edits allowed:** none expected. If a focused or fuzzy test trips over the second version directory (it assumes one version, one recording per case name, or a count), repair that test's assumption in its own file under `hearth/tests/mmdebstrap/`, `hearth/tests/builder/`, `hearth/tests/conformance/` or `hearth/testing/hearth-recordings/`, keeping what it checks for each version, and name it in the report. Never edit a recording.

**Options, each with its default:**

- Copy, don't move: `inputs/1.5.7-apt3.0.4/` stays as the plan's input.
- If a draft differs from its 3.0.3 expectation, keep the draft, commit it, and name each differing line (case, both sides) in the report.
- If the drafting test panics with *the recording's pipes decode* for a 3.0.4 case, the decoder refuses a captured line: don't change the parser or the recording; push what you have and report `Question:` with the case and the line.
- If a test fails that reads no recording, run it once on the pristine base (`notes/common.md`); passing there and failing here, it's yours.

## Steps

1. The recordings, from the repository root:

   ```sh
   cp -a hearth/docs/plan/hearth-apt-304/inputs/1.5.7-apt3.0.4 hearth/testing/recordings/mmdebstrap/
   diff -r hearth/docs/plan/hearth-apt-304/inputs/1.5.7-apt3.0.4 hearth/testing/recordings/mmdebstrap/1.5.7-apt3.0.4
   diff -r hearth/testing/recordings/mmdebstrap/1.5.7-apt3.0.3 hearth/testing/recordings/mmdebstrap/1.5.7-apt3.0.4
   ```

   The first diff prints nothing; the second prints only the rate and time lines above (the review 4.5 asks for: name in the report anything more). `git add` the directory, including the empty `stdout` files, and commit: "hearth: mmdebstrap 1.5.7 / apt 3.0.4's recordings, so a machine with trixie's apt 3.0.4 reads its own".
2. The drafts: run

   ```sh
   heavy cargo nextest run -p hearth-mmdebstrap-world --test suite -E 'test(=recordings::every_recording_has_an_expectation_and_the_reference_agrees)'
   ```

   It fails on the first case with no expectation, printing `<case>: missing expectation:` and then the draft. Write the draft verbatim, from its `# Captured mmdebstrap output` line to its last line, each line ending in a newline, as `hearth/tests/mmdebstrap/expect/1.5.7-apt3.0.4/<case>.expect`, and rerun. The cases come in order: `absent-snapshot`, `no-candidate`, `test-boot`, `test-min`, `unknown-package`, `unreachable-mirror`, `unsatisfiable`; the eighth run passes, with one test selected.
3. The review: `diff -r hearth/tests/mmdebstrap/expect/1.5.7-apt3.0.3 hearth/tests/mmdebstrap/expect/1.5.7-apt3.0.4` is expected to print nothing; a difference takes the option above. Commit: "hearth: apt 3.0.4's mmdebstrap expectations, reviewed against 3.0.3's".
4. The checks, one at a time: `python3 -I -B hearth/scripts/old_forms.py` (no count moves); `heavy cargo nextest run -p hearth-mmdebstrap-world -p hearth-builder-world -p hearth-conformance-world`; `heavy cargo nextest run --profile fuzzy -p hearth-mmdebstrap-world -p hearth-builder-world -p hearth-conformance-world`. Check each run's count is not zero.
5. Push.

## Report

The first line as the implementer's text says. Then: the two diffs of steps 1 and 3 (what each printed, or that it printed nothing); each draft that differs from 3.0.3's, with its lines; any test assumption repaired, with its file; the counts and times of the focused and fuzzy runs, and the times of the `recordings::` tests; the branch and commits pushed; implementation and check minutes apart.
