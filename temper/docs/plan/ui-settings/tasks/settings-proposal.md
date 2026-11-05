+++
id = "settings-proposal"
part = "settings-proposal"
kind = "feature"
needs = ["how-work-runs"]
files = [
  "temper/crates/temper-client-view/src/cards/proposal.rs",
  "temper/crates/temper-client-view/src/cards.rs",
  "temper/crates/temper-client-view/src/pages/chat.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/tests/cards.rs",
  "temper/crates/temper-client-view/src/tests/chat.rs",
  "temper/crates/temper-client-domain/src/forge/ways.rs",
  "temper/crates/temper-client-domain/src/pages/chat.rs",
  "temper/crates/temper-client-domain/src/tests/forge_ways.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/crates/temper-client-shell-terminal/src/tests/**",
  "temper/tools/gallery/names.txt",
  "temper/tools/gallery/src/samples.rs",
  "temper/tools/gallery/src/worlds.rs",
  "temper/tools/gallery/src/tests.rs",
]
effort = "high"
minutes = 110
+++

# A proposal of settings, in its card

## Goal

`cards/proposal.rs` draws nothing for `ProposalAction::Settings(_)`, so the configuring chat can't show what it proposes. Fill it, as the mock's `proposalCard()`, by `temper/docs/design/client/README.md`, 5.13, *The card in a chat*.

Acceptance:

- `.proposal-title`: "How temper will run PROJECT", from `proposal.project`.
- `ul`: an `li` per line of `proposal.reason`, a `span`, ink. Then an `li` per item of the proposal's `items`, its setting in `small`, 12 px mono, faint:
  - kind 7, from `Side::proposed`: an `li` each of "Drafts: on|off", "Goals: shown|hidden", "Changes page: shown|hidden", "Away switch: shown|hidden", "Way: NAME", NAME by the same words as How work runs' ways, the first way for a `way` of 0;
  - kind 6, from `Side::proposed`: "Questions: answered by the goal", "Questions: asked of me too", "Questions: not asked";
  - kinds 6 and 7 not delivered or refused: "project-ways P", "project-questions P", as `temper ctl` names them;
  - any other: "charter 2", "template 4", "policy 1", "rules", "landing rule 1", "forge charter 1", "project policy 2": a `RevisionItemEdit` of removed or reset adds " removed", " reset".
- **Acts:** "Accept", primary, small: jig's `Decide` with `Running::Stays`, by the offered act (`proposal.acts`, `c::ActKind::Accept`), as the card's existing accept; "Change something", quiet, small, focusing the composer (`Builder::focus`, its count kept in the chat's page), once per press. A person with no offered accept sees "Waiting for an owner" and who it waits for, as the proposal's `waiting` names them; no "Accept".
- **`.proposal-state`:** a later proposal of settings in the same conversation and this one not pending: "Replaced by the proposal below", with `.proposal.superseded`, `opacity: .5`; accepted: `.ok`, "✓ Accepted · new work follows it". Pending and only: the acts.
- **No `.proposal-split`:** it's the review page's.
- **The read:** `w::Read::ProposalItems { key }`, `scope: c::Scope::Deployment`, as `temper-client-tools/src/jig/runner/application.rs` asks it, once per proposal shown in a chat or inbox card that names kind 6 or 7, again when the link comes back; the model is `Side`'s reads of kinds 6 and 7. Bounded: at most `forges::ROWS` proposals kept, the oldest dropping.
- **Needs you's card** is `ui-needs`': it shows the same lines, by the same function in `cards/proposal.rs`, if that card is on the trunk.
- **`.proposal`, `.proposal-title`, `.proposal-state`** are `ui-chats`' (`conversation-turns`). If they are on the trunk at your base, use them; if not, take them from `mock.css` under `/* ==== ui: settings ==== */`, and say so in the report: the later catch-up removes the copy.
- **The gallery:** new `configure-chat-proposal`, `configure-chat-accepted`, `configure-chat-replaced`, each with its `Holds`, `configure-chat` keeping its other parts.
- `README.status.md`: the card goes from *3.5, 5.3, 5.13*.
- The tests below pass and bite.

## Inputs

- `temper/docs/plan/ui-settings/notes/common.md`; `temper/docs/plan/ui-settings/notes/contracts.md`, *A proposal's values*.
- `temper/docs/design/client/README.md`, 5.13, *The card in a chat*; 3.6.
- `temper/docs/design/client/forge.md`, section 9, on a proposal's items. `temper/docs/design/protocol/configuration.md`, 13.2, *Read from the forge*.
- The mock: `temper/docs/design/client/greenfield/app.js`, `proposalCard`; `mock.css`, `.proposal*`.

## Owns and may change

Owns `files`. May also change:

- **`pages/inbox.rs`** only to call the same function from Needs you's card.
- **`pages.rs`,** the composer's focus count, a field and its act.
- **Golden chats** (`temper-client-shell-terminal/src/tests/golden/chat*.txt`), and their prints, regenerated.
- **Design status:** `temper/docs/design/client/README.status.md`.

Pre-approved:

- If `Domain` can't tell a proposal replaced with what the conversation holds: "replaced" needs a later proposal of settings in the chat's turns (`proposal_anchor`) and this one withdrawn or rejected. Read that from the card's existing facts, and name the facts in the report.
- A `RevisionItemEdit` of a kind the words don't know prints as `temper ctl` prints it: `settings_form::render::setting_text`, if the view may name it; otherwise a local table.

## Steps

1. `forge/ways.rs`: `Proposed`, `Side::proposed`, its read.
2. `cards/proposal.rs`: one function drawing the settings card. The acts' act and facts stay jig's.
3. Tests, `tests/cards.rs`: `a_settings_proposal_draws_its_title_lines_and_items`, table of kinds; `kinds_six_and_seven_are_named_until_their_values_are_read`; `accept_decides_with_running_work_staying`; `change_something_focuses_the_composer`; `a_person_who_may_not_accept_sees_who_decides`; `replaced_and_accepted_say_so`.
   Domain, `tests/forge_ways.rs`: `a_proposals_items_are_read_once_and_bounded`; `a_refused_read_leaves_the_names`.
4. Stylesheet; gallery; `cargo fmt`; old forms; clippy and nextest of `temper-client-domain`, `temper-client-view`, `temper-client-shell-web`, `temper-client-shell-terminal`, `temper-client-gallery`, `temper-client-world`; `heavy cargo check -p temper --tests`. Commit and push.

## Report

As the implementer's text says.
