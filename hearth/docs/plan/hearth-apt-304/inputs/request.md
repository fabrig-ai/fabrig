# hearth-apt-304: mmdebstrap's recordings for apt 3.0.4

Debian trixie's apt is now 3.0.4, a security update that stays. Hearth's conformance cases read `hearth/testing/recordings/mmdebstrap/<mmdebstrap's version>-apt<apt's>/` (`hearth/docs/design/domain/testing.md`, 4.5), and main holds only `1.5.7-apt3.0.3`, so on a machine with apt 3.0.4 every case that reads its recording finds none.

The recordings for `1.5.7-apt3.0.4` are captured, never written by hand (4.5): they are this plan's input, `inputs/1.5.7-apt3.0.4/`, recorded from real runs with `HEARTH_RECORD=1` on a machine with mmdebstrap 1.5.7 and apt 3.0.4. Against `1.5.7-apt3.0.3` they differ only in stderr lines no expectation holds: `Fetched 10.2 MB in 1s (… MB/s)` gives a different rate in five cases, and `I: success in … seconds` a different time in `test-min` and `test-boot`. Every other file matches byte for byte.

## What's wanted

1. **The recordings, byte for byte:** `inputs/1.5.7-apt3.0.4/` becomes `hearth/testing/recordings/mmdebstrap/1.5.7-apt3.0.4/`, every case and file as given. Don't edit, regenerate or reformat one.
2. **Their expectations:** `hearth/tests/mmdebstrap/expect/1.5.7-apt3.0.4/<case>.expect` for each of the seven cases, reviewed as 4.5 asks. `recordings::every_recording_has_an_expectation_and_the_reference_agrees` prints each missing one as a draft. As decoded, each should equal its `1.5.7-apt3.0.3` expectation: the expectations keep `fetched` as a byte count, not a rate, and hold no `I: success` line. If a draft differs from 3.0.3's anywhere else, keep the draft and name the difference in the report.
3. **`1.5.7-apt3.0.3` stays,** recordings and expectations, so a machine with either apt reads its own.

## Limits

- Change only those two new directories. No code, doc, fake or test changes are expected. If a focused or fuzzy test trips over a second version, repair the test's assumption and name it in the report, but don't touch the recordings.
- One direct task, low effort, no parts. Its checks are the focused and fuzzy suites of `hearth-mmdebstrap-world`, `hearth-builder-world` and `hearth-conformance-world`, whose default and fuzzy binaries read these recordings.

## Sources

- `hearth/tests/mmdebstrap/src/recordings.rs`
- `hearth/tests/mmdebstrap/tests/suite/recordings.rs`
- `hearth/testing/hearth-recordings/src/lib.rs`

## For every task of this plan

- **Only focused and fuzzy run in a plan (the user, 11-06).** Never `tools/tiers/run`, a browser, process, live or conformance tier, one of their tests, or a tier's CPU figure. This overrides anything below or in a spec.
- **Decide, don't ask.** A `Question:` is only for what truly blocks the task.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
