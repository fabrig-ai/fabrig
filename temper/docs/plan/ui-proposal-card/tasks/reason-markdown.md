+++
id = "reason-markdown"
part = "card-reason"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-view/src/cards/proposal.rs",
  "temper/crates/temper-client-view/src/pages/ask.rs",
  "temper/crates/temper-client-view/src/tests/cards.rs",
  "temper/crates/temper-client-view/src/tests/needs.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/tools/gallery/src/worlds.rs",
  "temper/tools/gallery/src/tests.rs",
]
effort = "medium"
minutes = 35
+++

# A proposal's reason reads as Markdown in its card

## Goal

A real model writes its reason in Markdown, and the card prints "## temper enforces" where the Accept sheet renders a heading.
Draw the reason in the card by the path the sheet uses.

Acceptance:

- A settings card, in a chat and in *Needs you*, draws its reason through `crate::cards::document`, as `confirm.rs` does through
  `proposal_reason`: a `## temper enforces` reason is a heading named "temper enforces", its bullets are items, and no `##` is in
  the card's text. The settings' own item lines stay as they are, in their small mono type, after the reason.
- The Needs you question line, `pages/ask.rs`' `first_sentence`, leaves off a heading's marks: "temper enforces", not
  "## temper enforces".
- The gallery's `configure-chat-proposal` states show such a reason and `Holds` names its heading.

## Inputs

- `temper/docs/plan/ui-proposal-card/notes/common.md`.
- `temper/docs/design/client/README.md`, 5.13, *The card in a chat*, and 3.5, *Needs you*, *By its kind*.

## Owns and may change

**Owns:** the `files` above.

**Readers, each edited only as said:**

- `reason_line` goes; `settings_lines` is its one caller and `settings` that function's one caller.
- `.temper-settings-proposal-items .temper-settings-proposal-reason` in `temper.css` is replaced where it stands, since the reason
  is no longer inside the items' list. New rules for the reason's headings and paragraphs go under `/* ==== ui: settings ==== */`.
- `temper-client-shell-terminal/src/tests/**`: a golden that prints a settings card moves, regenerated.

**Pre-approved options:**

- **Single newlines:** Markdown joins them, so the reason's lines may run together where the old card kept them apart. Take that:
  the ruling is the sheet's path. Change `a_settings_proposal_draws_its_title_lines_and_items` if it reads the lines as items.
- **An empty item list:** don't open the items' `Role::List` when `settings.items` is empty, so the card draws no empty list.
- **A reason's first line that is only marks** (`##`): `first_sentence` skips to the next line with words; if none has words, it says
  the card's title as other empty words do.

**Not yours:** the card's place, its state lines and how it is read, later tasks of this plan; `confirm.rs`; `docs/design/`.

## Steps

1. **The failing tests,** in `temper-client-view/src/tests/cards.rs`, beside `a_settings_proposal_draws_its_title_lines_and_items`
   and on its helpers (`settings_root`, `draw_settings`):
   `a_settings_proposal_s_reason_draws_its_markdown`: a reason of `## temper enforces\nNo direct pushes.\n\n- one\n- two`; a
   `Role::Heading` named "temper enforces", two `Role::Item`s saying "one" and "two" beside the setting's item, and no `##` in
   `fixtures::text`. In `tests/needs.rs`, `a_proposal_s_question_line_leaves_off_a_heading_s_marks`, by a neighbour's shape.
2. **The card:** in `settings_lines`, before the items' list, a `Role::Slot` styled `look::SETTINGS_PROPOSAL_REASON` holding
   `super::document(b, &proposal.reason)`. The heading level is `document`'s, as on the sheet.
3. **The question line:** `first_sentence` as the acceptance says.
4. **The look:** `temper.css`, the reason in `--gf-ink`, its first and last child with no outer margin, its headings no larger than the
   card's body, so a heading doesn't outweigh the card's title. The sheet's rules are the guide. The plan's `card-proof` render checks
   this at 1440 and 420.
5. **The gallery:** `settings_proposal`'s reason in `worlds.rs` gains a `## temper enforces` heading; `Holds` for
   `configure-chat-proposal` names it with `Part::Named { role: Role::Heading, .. }`.
6. **Checks, once:** `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-view
   -p temper-client-gallery --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-client-view -p temper-client-gallery
   -p temper-client-shell-terminal -p temper-client-shell-web`.
7. Commit in small steps that say why, and push.

## Report

As your text says, and: the rule each new test bit on and the failure seen; any golden regenerated; whether `Holds` needed more than the
one row; anything under `## Left`.
