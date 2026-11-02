+++
id = "crossing-paths"
part = "crossing"
kind = "feature"
needs = ["claude-tools"]
files = ["smith/tests/end-to-end/live.rs", "smith/tests/tools/tests/suite/simulation.rs"]
effort = "medium"
minutes = 50
+++
# The live stories name files in the working directory, never under the directory's name

## Goal

Rerun alone at main 46e6e70d9, `smith::live a_chat_crosses_from_chatgpt_to_claude_then_back`
failed on its first, ChatGPT leg:

```
panicked at smith/tests/end-to-end/live.rs:531:9:
account=chatgpt invocation=1 step=Exec  failed=checkout file matches its prescribed content
exit=Code(0) end=Completed
session=1 run.completed status=Accepted turns=3
session=1 tool.completed tool=list verdict=Listed / tool=write verdict=Written / tool=finish verdict=Result
file "first.txt": missing
```

The story's words are the cause. Its prompt says "Create first.txt in repository repo ...". A run
on one directory hides that directory's name: the model is told paths start in its working
directory, without a directory-name prefix (smith/docs/design/domain/tools.md, 2; the prompt's
line in `smith/crates/smith-domain-run/src/prompt.rs`). A model that writes `repo/first.txt`
gets `Written` at `<scratch>/repo/repo/first.txt`, as the design resolves it; the `list` it saw
first shows nothing named `repo`. `CHANGE` says "In repository repo" too; its stories pass
because the model reads `original.txt` first. smith/docs/design/testing.md, 2.3 now says it:
"a story names a file in the working directory, never under the directory's name".

Bring the stories' words to the design, make a missing file's report say where it was found
instead, and pin the resolution in the tools world.

Acceptance:

- `CHANGE` and the crossing's prompt name the working directory, never `repo`.
- When a prescribed file is missing from the working directory but present at `repo/<path>`
  beneath it, the failure's report says so in fixed words, with no file bytes.
- The tools world pins that, on one directory `work`, `write` of `work/first.txt` is `Written`
  at the working path `work/first.txt` beneath the directory (the file lands at
  `<work>/work/first.txt`), and `first.txt` lands at `<work>/first.txt`.
- fmt, clippy and the focused tests below are green.

## Inputs

- `smith/docs/plan/live-smith-reds-2/notes/common.md` (read it first). This task follows
  `claude-tools`, which changed `program_contract` in `live.rs`; do not touch it.
- `smith/docs/design/domain/tools.md`, 2 ("One directory hides its name").
- `smith/docs/design/testing.md`, 2.3 ("Paths as the run tells them", "Outcomes, not words",
  "A failure says why, in typed facts").

## Owns and may change

- `smith/tests/end-to-end/live.rs`: the `CHANGE` constant; the prompt and the final file loop
  of `a_chat_crosses_from_chatgpt_to_claude_then_back`; the checkout-file loop at the end of
  `assert_exec`; one new private helper. Nothing else, `program_contract` and the refresh story
  above all (another task edits the refresh story's body).
- `smith/tests/tools/tests/suite/simulation.rs`: one new test.
- Pre-approved:
  - Wording: `CHANGE` starts "In the working directory, read original.txt, ..." and keeps the
    rest; the crossing's prompt is "Create {filename} in the working directory containing
    exactly {word} followed by one newline. Finish with a report."; a near wording is fine if it
    names no directory.
  - The report's added words, e.g. `file "first.txt": found at repo/first.txt beneath the
    working directory instead`, appended after `file_report`'s line; `file_report` itself (in
    `support/live_report.rs`, another task's file) is unchanged.
  - If the tools world answers the prefixed write otherwise than `Written` at the prefixed
    working path, stop: commit the test, push, and ask with what it answered.

## Steps

1. Read the inputs, `live.rs` (`CHANGE`, the crossing story, `assert_exec`) and
   `simulation.rs`'s `a_directory_name_written_as_a_prefix_fails_with_the_path_it_resolved_to`
   and `paths_in_one_directory_and_in_several`.
2. `simulation.rs`: `a_write_under_the_one_directory_s_name_lands_beneath_it`, modelled on the
   prefix test with `ONE`: `write(b"work/first.txt", b"first\n")` and `write(b"first.txt",
   b"first\n")` are both `Written`, and the checkout then holds `work/first.txt` and `first.txt`
   beneath the mount (read them back with `read`, or through the fixture's checkout, as the file
   does elsewhere). It pins what the design says and is green before any change; say so.
   `heavy cargo nextest run -p smith-tools-world --test suite -E 'test(/one_directory_s_name/)'`.
3. `live.rs`: the wording; the helper (e.g. `fn found_beneath(scratch: &Scratch, path: &[u8]) ->
   Option<String>`) reads only whether `<scratch>/repo/repo/<path>` exists, never its bytes; both
   file loops append its words when the file is missing. `heavy cargo check -p smith --tests`;
   never run the `live` binary.
4. `cargo fmt`; `heavy cargo clippy -p smith -p smith-tools-world --all-targets -- -D warnings`.
5. `heavy cargo nextest run -p smith-tools-world`, then
   `heavy cargo nextest run -p smith --test live_configuration`.
6. Commit (the tools pin, then the stories), push.

## Report

The tools pin's result (green before, as expected, or what it answered), the wording you chose,
the files changed, the test runs (selected, passed), and implementation and check minutes apart.
