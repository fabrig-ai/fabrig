+++
id = "only-region"
part = "region"
kind = "feature"
needs = ["only-read", "accounts-expiring"]
files = ["temper/crates/temper-client-view/src/pages/only_you.rs", "temper/crates/temper-client-view/src/pages.rs", "temper/crates/temper-client-view/src/pages/needs.rs", "temper/crates/temper-client-view/src/words.rs", "temper/crates/temper-client-view/src/look.rs", "temper/crates/temper-client-view/src/tests/only_you.rs", "temper/crates/temper-client-view/src/tests.rs", "temper/crates/temper-client-shell-web/temper.css", "temper/docs/design/client/journeys/**"]
effort = "high"
minutes = 140
+++

## Goal

Draw Only you on Needs you: its head "Only you · N", "work goes on
around it", and an item a card, from line, question and its act alone,
"Sign in again" to Accounts, after the cards' *Answered* or Done's
"Home" and before Decided for you, from `pages::only_you`'s read and
nothing else (`temper/docs/design/client/README.md`, 3.5, **Only you**,
*The region*). The browser's alone: the terminal's inbox stays as it is.

Acceptance:

- `pages/only_you.rs` holds `main` as `notes/contracts.md`, section 3
  gives it: a region named by its heading, the level-2 heading "Only you
  · N" ("N+" where the read is `cut`), at its right "work goes on around
  it", muted; then an item each in the read's order: its from line, the
  chip "Deployment" and "an LLM account"; its question, signed out "Your
  ChatGPT sign-in is signed out. Sign in again?", expiring "Your Claude
  Code login claude-alice is expiring. Sign in again?", SOURCE as the
  contract names it; its act, the link "Sign in again" bound to
  `Act::Go { address: Address::Accounts }`, in the recommended choice's
  look (`look::RECOMMENDED` inside `look::ASK_CHOICES`, as `needs::choices`
  draws it) with no "recommended". An item whose row or source the
  document no longer holds is skipped.
- `pages/needs.rs`'s `main` calls it once, `if parts.only {
  super::only_you::main(b, root, w, bindings); }`, as the line before `if
  parts.decided {`, and changes nothing else. `pages.rs` gains `mod
  only_you;`.
- Its words in `words.rs`, each its own `Word` with a stable identifier,
  after the Decided for you words; the chip's words are
  `Word::RailDeployment`'s. Its styles: `look.rs` tokens for the region,
  an item and its question (`temper-only`, `temper-only-item`,
  `temper-only-q`), their rules in `temper.css` under `/* ==== ui: needs
  ==== */` (`notes/common.md`): the head as Decided for you's head, an
  item the mock's `.card` with gap 12, its question 19 px, 500; at 760 px
  and under, its act full width and at least 44 px tall.
- `src/tests/only_you.rs`, one table test, its rows a domain state each,
  built on `src/tests/needs.rs`'s fixtures (`decided_fixture`, `fixture`)
  and an `accounts` document delivered as `src/tests/accounts.rs:591`'s
  `deliver` does, with the tree's words, order and acts expected: no
  item, no region; an item with the inbox not answered, no region; a
  ChatGPT sign-in signed out, with a card open, its region after the list
  and before Decided for you, its words, and "Sign in again" going to
  `Accounts`; a Claude Code login expiring; two items of a `cut` document,
  "Only you · 2+"; no card open, "Nothing needs you." and the region after
  "Home". Its existing Needs you tests keep their expectations.
- The rule bites: draw the region with the inbox not answered once, see
  its row fail, restore; the report names it.
- If a journey's print now differs, regenerate it with
  `temper-client-journeys again` for that journey (generation, not a walk:
  `inputs/request.md`).
- Delete `temper/docs/design/client/README.status.md`'s **Not yet** entry
  `3.5, **Only you**, whole, and *A page's parts*' row for it`.

## Inputs

- `temper/docs/plan/ui-n3a-only/notes/common.md`;
  `temper/docs/plan/ui-n3a-only/notes/contracts.md`, sections 1 to 3.
- `temper/docs/design/client/README.md`, 3.5, **Only you** (*It shows*,
  *The region*), *Needs you*'s *Only you* line, *Done*; 3.5, **Decided
  for you**, *The region* (the head's look); 5.5, *An account* and *Each
  source* words.
- The look: the whole story's
  `temper/docs/design/client/web/whole-story/needs.html` (its "Only you"
  section) and `whole-story.css` (`.shead`, `.card`, `.q`, `.choice.rec`).
- Patterns: `pages/needs.rs:19` (`main`), `:83` (`decided`), `:428`
  (`choices`); `pages/decided.rs` (a region and its head);
  `pages/accounts/rows.rs:186` (a source's words);
  `src/tests/needs.rs:891` (`decided_fixture`).

## Owns and may change

- **Owns:** the files in this spec's header, and the status entry above.
- **May change:** `pages/accounts.rs`'s `named` and
  `pages/accounts/rows.rs`'s use of it, to `pub(crate)` only, if the
  source's words reuse it; the gallery's `needs-*` prints if a state of
  theirs holds an Only you item, by their tool; the journeys' prints and
  recordings your change makes stale, by their tool alone.
- **Not yours:** `pages/inbox.rs` (the terminal's inbox), the TUI,
  `temper-client-domain`, and `temper/tools/gallery`'s sources, which
  `only-gallery` writes.
- **Options, pre-approved:** if a region in the view takes its name from
  its heading by a helper (`pages/decided.rs`), use it. If the chip's
  style is `look::CHIP` or the from line's is `needs`' own, use those. If
  the source's provider has no plan word, SOURCE is its provider's name
  with " sign-in", as Accounts' row says it.

## Steps

1. The region and its call; words and styles.
2. The table; break the rule once.
3. The journeys, if stale.
4. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo
   clippy -p temper-client-view -p temper-client-shell-web --all-targets
   -- -D warnings`; `heavy cargo nextest run -p temper-client-view
   --lib`; `heavy cargo nextest run -p temper-client-shell-web --lib`;
   `heavy cargo check -p temper-client-gallery --tests`; `heavy cargo
   check -p temper-client-world --tests`. Commit and push.

## Report

As the implementer's text asks; name the journeys regenerated, if any.
