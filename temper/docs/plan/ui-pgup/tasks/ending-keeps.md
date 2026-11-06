+++
id = "ending-keeps"
part = "ending-keeps"
kind = "feature"
needs = ["live-kept"]
files = [
  "temper/crates/temper-client-view/src/tests/chat.rs",
  "temper/tests/client/tests/suite/live_text.rs",
]
effort = "high"
minutes = 35
+++

# What a turn's ending keeps on the page

## Goal

A turn's ending redraws it as committed, and the words being read must stay on the page through that redraw, in the same nodes, so that the page part can hold them where they are on the screen. Reading says the keys are already right (`a_provisional_turn_and_its_committed_one_share_a_key`), but no test holds what the whole redraw keeps, or that no step of a real ending lacks the words. This task adds those tests. It changes production code only if they, or `reproduced.md`, show a cause.

Acceptance:

- The two tests below pass, and each bites.
- The view's patches from a turn being written to its committed turn remove and insert no node of its words.
- A world's whole turn's ending, a step at a time, never shows a tree without the words.

## Inputs

- `temper/docs/plan/ui-pgup/notes/common.md`, then `temper/docs/plan/ui-pgup/reproduced.md`.
- `temper/docs/design/draft/chat-reading.md`, section 3, *Committed without moving*, and section 5, *New words while reading back*.
- `jig/docs/design/client/domain/client.md`, section 10, *Live, then committed*.
- `skein/docs/design/ui-web.md`, section 7, *The view stays*: why a node's staying matters. The page part holds the first entries still there, and a removed node holds nothing.

## Owns and may change

- **Owns:** the files in this spec's header.
- **May change, how:**
  - `temper/crates/temper-client-view/src/pages/chat.rs`, `provisional`, `committed` and the keys they use, only if a test here or `reproduced.md` shows the redraw replacing the words' nodes. Keep the change to keys or to the shape the two share, and keep the hunk small: other plans are changing this file.
  - `temper/crates/temper-client-domain/src/pages/chat.rs`, `told`, if `reproduced.md` or the world's test shows `end_run` running before the committed page has been read. One condition, with its test in `temper/crates/temper-client-domain/src/tests/pages/chat.rs`.
- **Options:**
  - If the redraw does replace nodes only where a turn's first drawn block isn't its text (a call before its words), that is rare. Say so in your report and change nothing, unless `reproduced.md` saw it.
  - If the world's order never brings a run state with no attempt before the page, the world's test passes with or without `live-kept`. Say so: `jig-client`'s own tests hold that order, and yours holds the whole path.

## Steps

1. **The view,** in `temper/crates/temper-client-view/src/tests/chat.rs`, right after `the_end_goes_when_the_turn_commits`, using this file's `conversation_turn`, `styled` and `subtree`, and `provisional_with_text` and `tree` from its fixtures:

   `a_committed_redraw_keeps_the_nodes_of_the_words_being_read`: a turn being written with several closed blocks, `provisional_with_text(b"First.\n\nSecond.\n\nThird.")`, and the same words committed. Diff the live tree to the committed one with `skein_ui::diff`, as this file already does elsewhere, and assert:
   - no `Patch::Remove` of a node under the turn's `look::SAYS` but the end's mark (`look::WRITING_END`) and what it holds;
   - no `Patch::Insert` under `look::SAYS`;
   - the turn's slot keeps its node.

   Between the two trees, take the thread through a run state with `attempt: None`, and assert the tree is the live one, unchanged: a run's end alone redraws nothing.

2. **The world,** in `temper/tests/client/tests/suite/live_text.rs`, beside `full_live_text_stays_provisional_until_the_turn_commits`, whose shape it takes (`reply_in_pieces`, `release_reply`, `Find::text`, `TreeFace`):

   `the_words_stay_on_the_page_at_every_step_of_a_turns_end`: write the reply in pieces and release them, until the whole is seen with the writing mark. Release the turn's commit. Then run `Until::Steps(1)` at a time, up to a bound well past the ending's steps, and at each step assert that `Find::text(WHOLE)` is found in `TreeFace::new(world.tree(client))`, until the writing mark has gone. Assert that it did go within the bound.

3. **Each must bite.** For the world's test, revert `live-kept`'s early return in `Thread::run` once, run, and report what you saw, under this spec's option if it passes. For the view's test, change `line_key`'s provisional arm once to another kind.

4. If `reproduced.md` names a cause in the redraw, fix it now, keeping the tests above.

5. Once: `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-view -p temper-client-world --all-targets -- -D warnings`, adding `-p temper-client-domain` if you changed it; `heavy cargo nextest run -p temper-client-view -p temper-client-world -p temper-client-shell-terminal`, adding `-p temper-client-domain` if you changed it. Commit and push.

## Report

What `reproduced.md` said of a turn's ending. Whether production code changed, and why. The rule you broke for each test and the failure seen, or the option you took. The tests, how many, and your minutes.
