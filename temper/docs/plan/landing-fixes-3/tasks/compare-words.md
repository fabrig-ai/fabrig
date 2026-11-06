+++
id = "compare-words"
part = "compare-words"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-engine-smith/src/answers.rs",
  "temper/crates/temper-engine-smith/src/tests.rs",
  "temper/docs/design/domain/agent.status.md",
]
effort = "medium"
minutes = 45
+++

# A compare's refusal says what to read instead

## Goal

`read_compare` said "too large at forge.client.rows or forge.client.answer_bytes (measured size unknown)" on a small real change, and the run retried with a hash it made up. Reproduce the refusal at a real change's size, then make it say first that the ids were read and no other id helps, then what to read instead.

The plan's call: **the refusal changes and the compare's bounds don't.** A compare holds both whole sides of every changed file, so raising `forge.client.rows` doesn't help a long file. A compare of paths alone would change what a settled `Answer::Compare` means, and that's a stored format. The pull's files already answer the same need a page at a time.

Acceptance:

- **Reproduced:** a test shows that a compare within reach of a small change is past the bounds at the stock's values: nine files, nine commits, and one file whose two sides total more than `forge.client.answer_bytes`.
- **The words:** a compare the forge client refused (`Err(api::Error::TooLarge)`) answers, in this order:
  1. That this comparison is too large to return whole, that its commit ids were read, and that another id won't help, so never retry with a changed or guessed one.
  2. What a comparison holds: no pages, both whole sides of every changed file within `forge.client.answer_bytes`, and its files and its commits each within `forge.client.rows`. Which of these it passed is unknown.
  3. The recovery as it stands: `read_pull`, then `read_pull_files` from page 1, then `read_file` at both endpoints within `max_bytes`.
- **No bound to raise:** the refused answer no longer holds the `answer_configuration` paragraph or the "host answer limit" sentence. The forge client refused this, and raising `core.call_answer_bytes` wouldn't help.
- **Unchanged:**
  - The refusal stays marked an error and fits `cap` at every cap the test tries (0, 32, 512, 2048).
  - A successful compare past the host's bound keeps its measured-size feedback, its recovery and `answer_configuration`.
  - `read_file` and `read_pull_files` refusals keep their words.
  - `read_pull`'s `Forge read failed: too large` and `read_compare`'s `Forge read failed: missing` stay as they are.

## Inputs

- `temper/docs/design/domain/agent.md`, 4.3: "A compare is whole, with no pages".
- `skein/docs/design/forge.md`, 5.2: `Read::Compare`, what a compare holds.
- `skein/docs/design/forge-client.md`, 2.3: answers past `rows` or `answer_bytes`.
- `temper/docs/plan/landing-fixes-3/notes/common.md`.

## Owns and may change

- **Owns:** `compare_bounded`, `compare_recovery` and what you add beside them in `answers.rs`, and their tests in `tests.rs`.
- **May not change:** `read_size`, `file_recovery`, `pull_recovery`, `file_read_refusal`, and `answer_configuration`'s own words. Other reads share them.
- **`temper/docs/design/domain/agent.status.md`:** remove this task's **Not yet** entry for section 4.3, the one naming `landing-fixes-3/compare-words`, and nothing else. This departs from the implementer's text, which closes `docs/design/`, because the status says what's built.
- **Not here:** `calls.rs`' tool description and `temper/docs/help/`. Neither changes.
- **Options:**
  - If `temper-engine-smith` can't reach the stock's values, name them as constants in the test, with a comment citing `temper/docs/help/reference/limits.md`.
  - An `Err` other than `TooLarge` that doesn't fit a very small `cap` keeps today's `read_size` words. Only `TooLarge` takes the new words.
  - `forge_read`'s `compare_recovery` branch keeps working. If it has become unreachable, leave it and say so in the report.

## Steps

Failing tests first.

1. **Reproduce**, in `tests.rs`, beside `read_compare_too_large_names_bounds_and_paged_fallback`:

   ```rust
   #[test]
   fn a_real_changes_compare_is_past_the_stocks_bounds() {
       // forge.client.rows 8, forge.client.answer_bytes 65,536.
       // skein_forge::measure::answer_rows and ::answer are what the forge client checks.
   }
   ```

   Build `api::Answer::Compare` values for nine one-line files, nine commits, and one 33 KB file with a line changed. If this finds the request's cause wrong in some way, say how in the report: it's asked for.
2. **Words:** extend `read_compare_too_large_names_bounds_and_paged_fallback` with the new phrases and the order of the three parts. Assert that `core.call_answer_bytes` and `host answer limit` are absent from the refusal. Keep the guidance that test already holds, changing a phrase only where the new words replace it (`decoded` may become `whole sides`), and say so in the report.
3. **`compare_bounded`:** on `Err(api::Error::TooLarge)`, write the new first sentences, then `compare_recovery`, with no `read_size` and no `answer_configuration`. Keep what's said first inside the smallest useful cap, since `clipped_refusal` cuts from the end.
4. **Tests that bite:** break each rule once by hand and see its test fail, as the implementer's text says.
5. **Narrow run:** `heavy cargo nextest run -p temper-engine-smith --lib`.
6. **Before the push:** fmt, `old_forms.py` and clippy as your text says. Run the focused tests of `temper-engine-smith` and its direct dependents, and check `temper_engine_domain_world`'s suites that render a compare, if any. Commit and push.

## Report

As your text says, and also:

- the refusal's whole text
- what the reproduction found: which bound each case passed, and whether the request's cause holds
- anything you left, such as a `read_pull_files` page refused whole with no paths
