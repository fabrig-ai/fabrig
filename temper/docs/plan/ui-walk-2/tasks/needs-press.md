+++
id = "needs-press"
part = "needs-press"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-domain/src/pages/inbox.rs",
  "temper/crates/temper-client-domain/src/pages/needs.rs",
  "temper/crates/temper-client-domain/src/tests/pages/inbox.rs",
  "temper/crates/temper-client-domain/src/tests/pages/needs.rs",
  "temper/crates/temper-client-view/src/frame.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/tests/needs.rs",
  "temper/crates/temper-client-view/src/tests/toast.rs",
  "temper/docs/design/client/README.status.md",
]
effort = "high"
minutes = 90
+++

## Goal

Finding A4: on Needs you, a choice pressed straight after Undo didn't answer, and nothing said so. **A person's answer that silently doesn't send is the worst thing this page can do.** Find it, red first at the lowest tier, and fix it.

The cut read the code and holds one cause, unproved by a run; you prove or refute it. `Inbox::refresh` (`temper-client-domain`, `pages/inbox.rs`) runs on every change of the inbox's document, and A4's box had a goal running and a proposal arriving. It rebuilds each card with `page: None`, and at a filtered address, which is what Needs you's usual one is (`Filter { project, .. }`), it empties `cards` and adds them back only as two reads answer, then reads each card: two round trips. In that window:

- the view draws no choices (`parsed` comes from `card.page`), and then draws them again as new elements, so a press between its mouse down and up does nothing (`temper/docs/design/client/browser.md`'s *A press a render can lose*; the walk's driver sends mouse down and up as two calls);
- a press that lands returns from `needs::answer` at `answerable` (`card.page.as_ref()?`) with no line, no toast and no send;
- a card read that fails leaves `page: None` until the next change.

Acceptance:

1. A domain test is red first on the unchanged code and green after, driving a real `Told::Changed` for the entries, at a filtered and at an unfiltered address: answer, Undo, the change, a choice before any read answers. The choice makes a held line; at 5 seconds exactly one `Answer` goes, with the choice's words.
2. `temper/docs/design/client/README.md`, 3.5, *A press stays under the finger*, holds, each sentence by a test.
3. A view test holds Needs you's tree unchanged by a refresh of the same rows, before any read answers.
4. A replay on a fake box: the steps below, at least five times at 1440 and once at 420, all answered. A fail is a cause this task hasn't found: follow it.

If the first test can't be made red, say so and what was ruled out. Acceptance 2's toast and every other line stay yours, and the finding is reported as not reproduced.

## Inputs

- `temper/docs/plan/ui-walk-2/notes/common.md`, and the request it names.
- `temper/docs/design/client/README.md`, 3.5: *The held send*, *A press stays under the finger*, *The toast*.
- `temper/docs/design/client/browser.md`, its open question *A press a render can lose*, and `git show 61c1b7af70` for how Sign out kept its element.
- The finding: `temper/docs/plan/ui-final-walk/findings.md`, A4, its row and its reproduction at the end of the file. Evidence (`1440/05-needs-count-1440-*`, `needs-confirm/1440/05-needs-count-1440-*`, the ctl JSON) is under `~/src/fabrig/ui-final-walk/` on hetzner11, if your builder is that machine; it isn't needed.

## Owns and may change

Owns the files in the header. Also:

- **`temper-client-view/src/pages/needs.rs`, `pages/ask.rs`, `pages/home.rs`:** only if a draw needs a line for acceptance 3, and named in your report.
- **`temper/crates/temper-client-domain/src/tests/fixtures*`:** a helper your tests need.
- **Arms for the new `Toasted` variant** wherever the compiler asks.
- **`temper/docs/design/client/browser.md`:** a phrase in *A press a render can lose*, if Needs you's case is worth naming there.

Pre-approved options:

- If a test reading `Inbox` assumes a blank card during a refresh, the design line wins: change only that assertion, and name it. `invalid_or_unoffered_cards_cannot_be_chosen` keeps its `toast() == None`: those cards draw no choice, so no toast goes there.
- If the filtered path's buffering is much harder than the sketch, keep the cards per section instead (a section's cards replaced as its read answers), provided room stays right: new rows are not lost to another section's old cards. Say so.
- If a further silent path turns up (a stale key, the shell's timer, a dropped `Send`), fix it here with its own red test, and name it.

## Steps

**Prove the cause.** Write acceptance 1's test first and run it: `heavy cargo nextest run -p temper-client-domain --lib -E 'test(=NAME)'`. Build from `fixtures::inbox_home(filter)` as `tests/pages/inbox.rs` does (`reply_counts_and_source_cut_survive_both_inbox_documents`), not from `f::pages`, which sets `cards` directly and never refreshes.

**Rule out the rest,** with a test where it costs little:

- The hold: `Later`, `OpenReply`, `SendReply`, `Undo`, `Choose` in a row. The first hold's deadline passes and sends nothing; the second's sends once, with the choice's words, not the reply's. Name it `later_reply_undo_choose_sends_the_choice_once`.
- The timer: read `next_deadline`'s path into `temper-client-shell-web/src/host.rs` (`deadline(..)`): does the deadline Undo removed leave a timer that hides the next? Report what you read; a test only if it's a fault.
- `Session::told`: can a late `Told::Answered` take the new line? A held line has no key; report it.

**The fix, in outline** (inside it the choices are yours):

```rust
// pages/inbox.rs
pub struct Inbox {
    ...
    /// The filtered reads' answers, applied together when both have answered.
    arrived: [Option<...>; 2],
}
```

- `refresh`, unfiltered: the rows are local, so `cards` is whole at once. `previous` keeps its pages; `add` carries `page` with `words`, `key` and `outcome`, and still reads the card.
- `refresh`, filtered: `cards` stays as it is and the two reads go. In `told`, a section's answer is kept in `arrived`; when no query read is pending, swap and add needs' rows then updates', as the unfiltered path does. A section whose read failed keeps its cards. Two overlapping refreshes reuse the same handles: the newest answer has to win, so a later answer applies alone over the kept section.
- `told`: a read that isn't a `Value` leaves the card's page as it was.
- `delivered` keeps its meaning: nothing pending. Check every reader of `pending`.
- A card that leaves the rows leaves `cards` when the swap happens, so a stale press can't answer it; and a stale page's press that the engine refuses is the existing refusal on the card.

**The toast:** `Toasted::NotSent`, `Word::ToastNotSent`, "That answer wasn't sent. Press it again.", with no act, as `Toasted::Later` is drawn. `needs::answer` says it where a drawn choice or a sent reply fails `Text::new` or `make_room`, and nowhere else.

**Tests, each must bite** (implementer's text):

In `tests/pages/needs.rs`:
- `a_choice_pressed_while_the_inbox_reads_again_is_held_and_sent` (acceptance 1; filtered and unfiltered)
- `later_reply_undo_choose_sends_the_choice_once`
- `a_refused_bound_says_the_answer_was_not_sent` (cue bound, full capacity, a reply past its bound; no line, no send, the toast). Add these to the existing tests' cases or beside them; keep each assertion they have.

In `tests/pages/inbox.rs`:
- `a_refresh_keeps_each_cards_page_until_its_read_answers`
- `a_failed_read_keeps_the_page_it_had`
- `a_filtered_refresh_keeps_the_cards_until_both_reads_answer`
- `a_failed_section_read_keeps_that_sections_cards`
- `a_card_gone_from_the_rows_leaves_at_the_swap`
- `overlapping_filtered_refreshes_show_the_newest_rows`

In the view's `tests/needs.rs`: `a_refresh_of_the_same_rows_leaves_the_tree_unchanged`. In `tests/toast.rs`: the new toast's words.

**The replay.** A throwaway script, untracked, on `temper/docs/plan/ui-final-walk/driver/cdp.py` if it's on your base (`walk.py`, near `Use the small implementation`, holds the steps), run directly: a fresh fake box, enable the features, send Start a local away goal; on Needs you choose Later, open Reply, send Use the small implementation, press Undo, press the offered Use the small implementation choice; six seconds later the answered card shows, the count falls and `temper ctl` (with `</dev/null`) has no unanswered question. If you can, make an inbox change land between Undo and the press. Gone before the push.

**Checks.** `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; clippy on `temper-client-domain` and `temper-client-view`. This changes what `Inbox.cards` means during a refresh, so run its readers' tests: `heavy cargo nextest run -p temper-client-domain -p temper-client-view -p temper-client-shell-terminal -p temper-client-world`, each through `heavy`, one command at a time. Golden or print changes you can't explain are red. Then remove the **Not yet** entry `3.5, *A press stays under the finger*` from `temper/docs/design/client/README.status.md`, run `python3 -I -B tools/citations/citations.py --base origin/main`, commit and push.

## Report

The cause, proved or refuted: the red test's name and its failure on the unchanged code. What was ruled out, and how. The replay's count at each width and what it did to make the inbox change land. Tests that read `Inbox` you changed, by name. What you left of *A press a render can lose* elsewhere on the page.
