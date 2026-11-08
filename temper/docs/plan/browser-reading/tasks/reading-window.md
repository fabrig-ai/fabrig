+++
id = "reading-window"
part = "reading-window"
kind = "feature"
needs = []
files = ["temper/crates/temper-client-shell-web/src/limits.rs", "temper/crates/temper-client-shell-web/src/host_tests.rs", "temper/tests/client-browser/src/reading.rs", "temper/docs/design/client/browser.status.md"]
effort = "medium"
minutes = 90
+++

# The tier's window at 32,768 bytes, and reading's replies sized to it

## Goal

The browser tier's reading story costs too much because of its fixture: eight long replies whose 9,600-word code lines fill a 131,072-byte window three times over. Shrink the tier's window to 32,768 bytes, a quarter of today's. The window holds no less than `long_text` (`temper/docs/design/client/README.md`, 5.15; `temper_client_domain::limits::valid`), so the same `small-window` feature also sets the bundle's `long_text` and `turn_text` to 32,768. Then scale reading's code lines by the same quarter, so that each history keeps its share of the window. The replies' counts and paragraphs stay, and so does every step and assertion of the story.

Acceptance:

1. `temper/crates/temper-client-shell-web/src/limits.rs`: under `small-window`, `WINDOW_BYTES` is `Some(32_768)` and `LONG_TEXT` is `32_768`, so `client()` gives `window_bytes`, `long_text` and `turn_text` all at 32,768. Without the feature, nothing changes: `None` and 131,072. Each of the four constants has a comment saying why it's that size (Steps, 1).
2. `host_tests.rs`'s `the_small_window_bundle_names_a_valid_narrower_window` asserts the window, `long_text` and `turn_text` at 32,768, and `limits::valid`, and passes with `--features small-window`. `the_ordinary_bundle_keeps_the_whole_document_bound` also asserts `long_text` and `turn_text` at 131,072, and passes without the feature.
3. `temper/tests/client-browser/src/reading.rs`: `READING_WINDOW_BYTES` is `32_768`, `LONG_CODE_WORDS` is `2_020`, `POSITION_CODE_WORDS` is `1_520`. Every other constant keeps its value. Every size constant has a comment saying why it's that size (Steps, 2). The file's three focused tests pass with their assertions as they are.
4. `temper/tests/client-browser/tests/browser_all/browser_pages.rs` is unchanged: `git diff origin/main -- temper/tests/client-browser/tests/browser_all/browser_pages.rs` is empty. No assertion is removed or loosened anywhere, no wait is lengthened and no poll interval is changed (the request, 3).
5. The story `browser_pages::a_chat_opens_at_its_end_and_reads_back_keeping_its_place` passes, run by name, alone (Steps, 4). The report gives each run's `date -u` time, its outcome and how many tests it selected.
6. `temper/docs/design/client/browser.status.md` no longer has the **Not yet** line on 4.3.

## Inputs

- The request: `temper/docs/plan/browser-reading/inputs/request.md`, read whole (it is short). Its `For every task of this plan` binds this task and stands in for a `notes/common.md`. Where it and this spec differ, it wins.
- `temper/docs/design/client/browser.md`, 4.3, the bullet *The tier's window* (this cut wrote it), and 4.6, item 14: the story.
- `temper/docs/design/client/README.md`, 5.4 (`long_text`, and what a line past it does: its page is refused) and 5.15 (the window, at least `long_text`).
- `jig/crates/jig-client-domain/src/contract/documents.rs`, `LINE_ROOM` and `DecodedActivityLine::weight`: what a line weighs in the window. That is its bytes plus 64 for the line and for each message and block it holds.
- `temper/crates/temper-docs/src/lib.rs`, `PAGE_BYTES` (32,768): the largest page `read_docs` answers.

What the cut found, for the comments and the numbers. The lead read this and ran nothing.

- **The window is at its floor today.** `small-window`'s window is 131,072, and so is the bundle's `long_text`. `temper_client_domain::limits::valid` refuses a window under `long_text`, since a line holds up to `long_text` (the thread's `text` and `live`, `temper_client_domain::pages::chat::thread_limits`). So a smaller window needs a smaller `long_text` in the same bundle.
- **32,768 is the tier's `long_text` floor.** It is the largest page `read_docs` answers (`PAGE_BYTES`). A help page's kind bounds it lower still (`temper_docs::Kind::bound`: a concept 12,288). Only the history story calls `read_docs` (hold, 5,412 bytes, and chat, 8,274), in one turn of about 14 KB. No tier story types or shows text near 32,768. Only the reading story uses `reading.rs` (`Tier::reading`, `temper/tests/client-browser/src/lib.rs`).
- **A reply's bytes and its height are apart.** `reading_reply` puts the bytes on one horizontally scrollable code line (`"data " × code_words`, one line tall at any length) and the height on its numbered paragraphs (64 bytes each). The story's keys (6 PageUps to the first read, 4 a read after it, 6 then 4 PageDowns) and the reply each step names (`reply_end(2)`, `reply_end(3)`, `read_to_end` from 4, `messages - 2`) rest on the heights and on how many replies the window holds, about two and a half. So the paragraphs and counts stay, and only the code lines scale.
- **The numbers.** A long reply is `54 + 5 × code_words + 64 × paragraphs` bytes. A position reply is the same.
  - Long, today: 7 × 50,614 + 49,974 = 404,272 bytes, 3.084 windows of 131,072. At `LONG_CODE_WORDS = 2_020`: 7 × 12,714 + 12,074 = 101,072 bytes, 3.085 windows of 32,768. The test's floor of three windows (98,304) needs at least 1,951 words.
  - Position, today: 35,694 + 2 × 36,974 = 109,642 bytes, 0.837 of a window. At `POSITION_CODE_WORDS = 1_520`: 8,294 + 2 × 9,574 = 27,442 bytes, 0.837 of 32,768.
- **The window's own room per line** (`LINE_ROOM`: the cue's line, the reply's line, message and block, and the run's lines, a few hundred bytes an exchange) weighs more beside 12 KB replies than beside 50 KB ones. Each long exchange then takes about 0.40 of the window rather than 0.39. The window still holds two whole replies and part of a third. The position history, with that room, stays near 0.9 of its window.
- **The fake's allowance** (`temper_fake::SCRIPT_BYTES`, 524,288) held about 515 KB of replies today, and will hold about 130 KB. `the_fake_admits_every_historys_replies` checks it, unchanged.

Departures from the implementer's text, and why:

- **You run one browser story, by name and alone,** as the request allows: the tier is over its budget and this repair changes that story's fixture. Run only it, never the tier or another story. The command is in Steps, 4.
- **You remove one status line** in `temper/docs/design/client/browser.status.md` (acceptance 6). With the fallback below, you also change 4.3's numbers in `browser.md`.

## Owns and may change

Owns:

- `temper/crates/temper-client-shell-web/src/limits.rs`: `WINDOW_BYTES` and `LONG_TEXT` under `small-window`, and their comments. Nothing else in it changes.
- `temper/crates/temper-client-shell-web/src/host_tests.rs`: the two window tests (acceptance 2).
- `temper/tests/client-browser/src/reading.rs`: the constants, their comments, and the module's opening comment where it names the eighth. `replies`, `reading_reply` and the tests keep their logic.
- `temper/docs/design/client/browser.status.md`: the **Not yet** line on 4.3 alone.

Reader and neighbour files:

- Another `temper-client-shell-web` test that fails under `--features small-window` because it assumes 131,072 (Steps, 3): read `limits::LONG_TEXT` there instead of the literal. Change nothing else in it, and name it in the report.

Pre-approved options:

- **`LONG_TEXT` under the feature:** write it as two `cfg`'d definitions beside `WINDOW_BYTES`'s, as `WINDOW_BYTES` is. If a reader outside `limits.rs` needs it as one item, keep `pub const LONG_TEXT` and say so. The module's `#![deny(...)]` stays: no `as`, so each constant is its own literal.
- **If the story fails** at a step where an eviction or a read comes one reply early or late (the window's room per line, above): move `LONG_CODE_WORDS` within 1,960 to 2,200, and `POSITION_CODE_WORDS` within 1,400 to 1,600, keeping both of the file's size tests passing. Never change the paragraphs, the counts or a step. Give the failing step and the value that passed in the report and in the constant's comment.
- **If no code words in those bounds pass,** or `limits::valid` refuses 32,768: take the fallback. Set the window, `long_text` and `turn_text` at 65,536, a sixteenth of `document_bytes`. Set the code words at half of today's (4,800 and 3,500). Change `temper/docs/design/client/browser.md` 4.3's "32,768 bytes, a thirty-second" to "65,536 bytes, a sixteenth", and its reason to two pages at `read_docs`' largest. Say why in the report.
- **If a `temper-client-shell-web` test fails under `--features small-window` on the pristine base too,** it isn't yours: name it under `## Left`.

## Steps

1. **`limits.rs`.** Under `small-window`:

   ```rust
   /// The browser tier's conversation window, at its floor: no less than `long_text`
   /// (client/README.md, 5.15), a thirty-second of `document_bytes` (client/browser.md, 4.3).
   #[cfg(feature = "small-window")]
   const WINDOW_BYTES: Option<u64> = Some(32_768);
   /// Whole document bodies, goal words and committed or live turns (client/README.md, 5.4);
   /// the browser tier's at its window, the largest page `read_docs` answers
   /// (`temper_docs::PAGE_BYTES`; client/browser.md, 4.3).
   #[cfg(feature = "small-window")]
   pub const LONG_TEXT: u32 = 32_768;
   ```

   Without the feature, `WINDOW_BYTES` stays `None` and `LONG_TEXT` stays `131_072`, with their comments saying why: no narrower window, and the person's `long_text`. Word the comments in your own way: say what is true now, with no history.

2. **`reading.rs`.** Set the three constants of acceptance 3. Write a comment on each size constant saying why it's that size, from what the story does with it. What the cut read, for you to check against the story and word:
   - `READING_WINDOW_BYTES`: `small-window`'s window (`temper-client-shell-web`'s `limits.rs`), the tier's `long_text`, a thirty-second of a person's.
   - `LONG_CODE_WORDS`: puts each long reply's bytes on its one code line, so that the eight replies span just past three windows and the window holds about two and a half of them.
   - `POSITION_CODE_WORDS`: keeps the three position replies inside one window, with room for the window's lines.
   - `LONG_MESSAGES` (8): enough replies, at that size, for three windows. The long story's reads name replies by their place in them.
   - `POSITION_MESSAGES` (3): the anchor is the second-last message's words, and the read above it reaches the first reply.
   - `LATEST_MESSAGES` (6): enough lines that two PageUps from the composer leave the log's foot, so that Latest shows, while the chat still opens whole.
   - `WINDOW_PARAGRAPHS` (40), `POSITION_PARAGRAPHS` (30), `MIDDLE_PARAGRAPHS` (10): a reply's height, which the story's PageUp and PageDown counts cross. The last reply is shorter, so opening reads the one before it too.

   Replace the comment over `READING_WINDOW_BYTES` that names "an eighth" with the new size's reason.

3. **The narrowest tests first.**
   - `heavy cargo nextest run -p temper-client-browser-world --lib`: the three tests of `reading.rs`, by name, each passing.
   - `heavy cargo nextest run -p temper-client-shell-web --lib --features small-window -E 'test(=host::tests::the_small_window_bundle_names_a_valid_narrower_window)'`, and `heavy cargo nextest run -p temper-client-shell-web --lib --features small-window`. The second is the whole lib under the feature, which no gate runs. A failure there follows the options.
   - `heavy cargo nextest run -p temper-client-shell-web --lib` without the feature.
   - Check the count every time: a run that selected nothing proves nothing.

4. **The story, by name and alone,** through heavy:

   ```sh
   TIERS_WRAPPER=heavy tools/tiers/run temper browser -- -E 'test(=browser_pages::a_chat_opens_at_its_end_and_reads_back_keeping_its_place)'
   ```

   The runner builds the tier's bundle (with `small-window`, `tools/tiers/bundles.toml`), the playground's and the `temper` binary. Then it runs nextest's `browser-temper` profile with the filter (`tools/tiers/design.md`, 1). Check that it selected exactly one test, and record `date -u` before each run. Read a failure's log around its step with `grep` and `tail`, never whole, before anything reruns. It reads and compares no CPU figure (the request's Limits).

5. **Check.**
   - `cargo fmt`.
   - `python3 -I -B temper/scripts/old_forms.py`.
   - `heavy cargo clippy -p temper-client-shell-web -p temper-client-browser-world --all-targets -- -D warnings`, and once more for `temper-client-shell-web` with `--features small-window`.
   - No dependent's tests: without `small-window` the shell's code doesn't change, and only the tier's bundle turns it on.
   - Remove the **Not yet** line on 4.3 from `temper/docs/design/client/browser.status.md`.
   - Commit, then `python3 -I -B tools/citations/citations.py --base origin/main` (it reads `HEAD`).

6. **Commit and push.** Use the message `browser-reading: the tier's window at 32,768 bytes, reading's replies a quarter`. Its body says why: the window's floor is `long_text`, and the code lines carry the bytes while the paragraphs carry the height.

## Report

Follow the implementer's text. Also give:

- the values set, and any you tuned under the options, with the failing step that moved them;
- each run of the story: its `date -u` time, its outcome, how many tests it selected;
- the three focused tests of `reading.rs` and the two window tests, each by name, with their outcomes;
- for each test you changed, the rule you broke by hand and the failure you saw;
- every file you changed outside `files`, and why.
