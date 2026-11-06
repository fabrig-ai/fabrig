+++
id = "card-reload"
part = "card-reload"
kind = "feature"
needs = ["card-place"]
files = [
  "temper/crates/temper-client-domain/src/pages.rs",
  "temper/crates/temper-client-domain/src/pages/chat.rs",
  "temper/crates/temper-client-domain/src/pages/needs.rs",
  "temper/crates/temper-client-domain/src/tests/pages/chat.rs",
  "temper/crates/temper-client-domain/src/tests/pages/needs.rs",
  "temper/crates/temper-client-view/src/cards/proposal.rs",
  "temper/crates/temper-client-view/src/pages/chat.rs",
  "temper/crates/temper-client-view/src/tests/chat.rs",
  "temper/crates/temper-client-view/src/tests/cards.rs",
  "temper/crates/temper-client-shell-terminal/src/tests/**",
  "temper/tests/client/tests/suite/configuring_chat.rs",
  "temper/tools/gallery/src/worlds.rs",
]
effort = "high"
minutes = 90
+++

# A decided proposal's card stays at its place after a reload

## Goal

Decided proposals vanish from the chat on reload, even paging to the start: only "Decision made" rows are left. A decided proposal's card stays at its place after a reload and paging back, with its state line, read as the pending ones are. A card and a "Decision made" row
never both say it. Once a card is drawn from what jig retains, the session's snapshots have no use, and they go: they are also how a reopened card could say "Waiting for an owner".

Acceptance:

- A making line in the window brings its proposal's card: read by `Read::Proposal`, drawn at its place with its state line, whichever way the chat was reached (a reload, another device, a return in the session).
- Paging back brings it when its line comes into the window; a card stays at one place through this.
- A settings card draws from `SettingsFacts`, a live proposal's or a retained record's, the same lines: title by `projects`, reason's Markdown, items.
- Where a card stands, its proposal's `Decided` lines draw no row. A proposal with no card standing keeps its row: its making line out of the window, or its read answering `Gone`.
- A proposal that isn't of settings, once decided, draws no card and keeps today's row.
- `Chat.settings_proposals`, `remember_settings` and `settings_snapshot` in `pages/chat.rs`, and `Needs`' `settings`, `remember_settings` and `restore_settings`, are gone.
- `cards` stays bounded: when it is full, a decided card whose making line isn't in the window gives way.

## Inputs

- `temper/docs/plan/ui-proposal-card/notes/common.md`.
- `temper/docs/plan/ui-proposal-card/notes/contracts.md`: *The facts jig gives*, *A card's place*, *A settings card's facts*. They bind.
- `temper/docs/design/client/README.md`, 5.13, *A decided card after a reload* and *"Decision made"*.

## Owns and may change

**Owns:** the `files` above.

**Readers, each edited only as said:**

- `pages.rs`, near lines 916 and 1198: the calls to `needs.remember_settings` and `needs.restore_settings` go.
- `pages/chat.rs` in the view: `settings_proposal`'s callers (`proposal_card`, `settings_replaced`) take `settings_facts`; `hidden_line` is given the numbers of the cards that stand, computed with their places.
- `Chat::proposals`: a card its document no longer lists stays if its page is `Ended`, or a live `Proposal` of settings, re-read; or if its page is `None` and its making line is in the window; otherwise it departs as today (`CardLeft`, `decisions`).
- `told`, at its end beside `note_calls`: a new `made_cards(client, out)` scans the window.

**Pre-approved options:**

- **A decided card of another kind** takes a place in `cards` with an `Ended` page and isn't drawn, so its read isn't repeated. Take that; the eviction rule bounds it.
- **A departure then a return:** a batch proposal decided in the open chat departs (`CardLeft`, as today), and `made_cards` then finds its line and reads it, undrawn. Take that.
- **Accept on a card, decided elsewhere meanwhile:** today's refusal words stay.
- **`temper-client-domain`'s tests of the session's memory** go with it, and any that keeps a true rule moves to the reload's terms.
- **A read past jig's bound** answers `Gone`: page `None`, no card, the row stays. That is the design, not a fault.

**Not yours:** jig; the place's rules, as `card-place` left them, but what this needs; `docs/design/`.

## Steps

1. **The domain's failing tests,** in `tests/pages/chat.rs`, on `f::opened()`'s shape: `a_making_line_asks_for_its_proposal`; `a_second_page_with_the_line_asks_once`; `a_decided_card_stays_when_the_document_arrives_after_its_line` (the loop this could make: document, departure, line, read);
   `a_gone_proposal_has_no_card_and_is_not_asked_again`; `a_full_list_lets_a_decided_card_out_of_the_window_go`.
2. **The domain,** as the contract says. Then remove the snapshots, seeing what still compiles against them in the view.
3. **The view's failing tests,** in `tests/cards.rs` and `tests/chat.rs`, in both shells, with no snapshot and `call.objects` empty:
   `a_decided_card_draws_from_what_jig_retains`, each of accepted, rejected and withdrawn, with `## temper enforces` in the reason and `projects` in the title;
   `a_decided_card_stands_at_its_place_after_a_reload`, with a later reply below it;
   `where_a_card_stands_there_is_no_decision_row`; `a_row_stays_where_no_card_stands`, its making line out of the window; `a_decided_proposal_of_another_kind_keeps_its_row_and_draws_no_card`.
4. **The view:** `SettingsFacts`, `settings_facts`, `settings`, `settings_lines`, `settings_title`, `proposal_card`, `hidden_line`.
5. **The world,** `configuring_chat.rs`: `a_decided_proposal_stands_after_a_reload`: propose, reply, reply; reject under Needs you; then a new client of the same party on `Address::Chat { task }`, standing for a reload: it sees the title, "Rejected · nothing changes", its place between the replies, and no "Decision made". Run it first against the unfixed code to see it fail.
6. **Checks, once:** `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-domain -p temper-client-view -p temper-client-gallery -p temper-client-world --all-targets -- -D warnings`;
   `heavy cargo nextest run -p temper-client-domain -p temper-client-view -p temper-client-gallery -p temper-client-shell-terminal -p temper-client-shell-web`; `heavy cargo nextest run -p temper-client-world --test suite -E 'test(configuring) | test(chat) | test(inbox)'`, since this changes what `Needs` keeps. Check the count.
7. Commit in small steps that say why, and push.

## Report

As your text says, and: the rule each new test bit on and the failure seen; what you removed; anything under `## Left`.
