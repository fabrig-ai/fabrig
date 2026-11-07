+++
id = "card-and-latest"
part = "card"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-domain/src/forge/ways.rs",
  "temper/crates/temper-client-domain/src/tests/forge_ways.rs",
  "temper/crates/temper-client-domain/src/pages/calls.rs",
  "temper/crates/temper-client-domain/src/tests/pages/calls.rs",
  "temper/crates/temper-client-view/src/cards/proposal.rs",
  "temper/crates/temper-client-view/src/pages/chat_line.rs",
  "temper/crates/temper-client-view/src/pages/goals.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/tests/cards.rs",
  "temper/crates/temper-client-view/src/tests/chat_line.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/crates/temper-client-shell-web/tests/suite/**",
  "temper/tests/client/tests/suite/configuring_chat.rs",
  "temper/tools/gallery/**",
  "temper/docs/design/client/README.status.md",
]
effort = "high"
minutes = 75
+++

# The card's rows and words, and Latest off a phone's controls

## Goal

Three things a real model's walk left in the client, none of which needs another task:

- A Goals-only proposal's card has no setting row: draw application items as rows, and repair the read that loses their values.
- The activity row says `propose_settings`, and the card ends with a monospace "template 1": a person's words for both, in the card's type.
- At a phone's width Latest floats over the log's foot and takes a press meant for a control beneath it: give it a row of its own.

Acceptance:

- **Rows.** A live card whose heads are empty and whose `settings.review` has an `Application { kind: 7, .. }` item draws "Goals: shown" and its neighbours once the forge's read delivers, and names it as `temper ctl` does until then. Another application's item and a project's policy draw their named rows. A jig item isn't drawn twice. `settings_lines` no longer returns early on empty heads, and draws no empty list when no row exists.
- **The read.** `forge/ways.rs` asks `ProposalItems` under `review.key`, not the proposal's number, once per proposal, and `Side::proposed` stays looked up by the proposal's number, so `proposal.rs`'s call doesn't change. A decided card asks nothing and keeps what it read.
- **The trace, in the real engine.** `a_configuring_proposal_of_a_projects_ways_reaches_its_needs_and_its_rail` in `temper/tests/client/tests/suite/configuring_chat.rs`, whose item is kind 7, sees "Goals: shown" on the chat's card before it is accepted. A test on hand-made documents alone is what let this be lost.
- **The activity row.** A chat's `propose_settings` reads "Proposed a change" alone, "proposed a change N×" among others or repeated, and "Proposing a change" under way, never naming its input (the first member of its input would be the reason). The row of calls inside the disclosure may keep the tool's name, as every tool's does.
- **The template.** A template's row says its name, by the rule `pages::goals::ways` names a way ("goal" as `GoalTaskByTask`, "goal for a draft" as `GoalLandWhole`, any other as written), with " removed" or " reset" as today, or "template 4" where `me` names none.
- **A refused Accept says so.** `settings_state` in `cards/proposal.rs` reads a card whose decision was answered `c::Answer::Refused { refusal: c::Refusal::Unknown, .. }` as `SettingsState::Withdrawn`: jig refuses a task's withdrawn proposal so (`notes/contracts.md`), and this person had just read it. So the card draws no Accept, and says "Replaced by the proposal below" with a later card, or "Withdrawn · nothing changes". A test in `tests/cards.rs`, in `decided_settings`' style, holds both.
- **The type.** `.temper-settings-proposal-items small` in `temper.css` takes the card's sans type, small and faint, where it stands. Every row of the list follows it. `--gf-mono` is gone from that rule.
- **Latest.** Under `/* ==== ui: phone ==== */`, in its `@media (max-width: 760px)` block, both `#app > .temper-session .temper-thread > .temper-latest` and `.temper-agent-page .temper-thread > .temper-latest` are in flow: `position: static`, no `transform`, no negative margin, centred, `flex: none`. So in the thread's flex column it takes a row below the log, and nothing lies beneath it. Over 760 px nothing changes: the pill floats as the mock has it.
- **A test of Latest's geometry.** No focused world lays out a page, so this is the lowest that can hold it: a test in `temper/crates/temper-client-shell-web/tests/suite/`, in the style of `phone_session_hides_the_keyboard_hint`, reading both rules out of the phone block and holding each to the declarations above, and holding that the thread is a flex column (`temper.css`, *A chat's log is the scroller*). Say in the report that it holds the rules, not a measured press: temper/ui's look after landing presses it.
- `temper/docs/design/client/README.status.md`: the **Not yet** of 5.13 on a live proposal's items, and the **Not yet** of 3.5 on Latest and `propose_settings`, are gone. The **Design departs** on the card's type stays.

## Inputs

- `temper/docs/plan/ui-proposal-2/inputs/request.md`, items 2, 3 and item 4's second bullet.
- `temper/docs/plan/ui-proposal-2/notes/contracts.md`, *What a card reads for its application's items*: binding.
- `temper/docs/plan/ui-proposal-2/notes/common.md`.
- `temper/docs/design/client/README.md`, 5.13, *The card in a chat*, and 3.5: *Reading back* and the phrases of a run of calls.
- `temper/docs/design/client/greenfield/mock.css`, `.proposal`: the target's card.
- `temper/docs/design/protocol/configuration.md`, 13.2.

## Owns and may change

Owns the files in the header. `temper/docs/design/client/README.status.md` is yours here, a departure from the implementer's text, on purpose.

`model-catalogue-2/configuring-learns` is changing `cards/proposal.rs`: whichever lands second catches up. `chat-revises` adds a story to `configuring_chat.rs`: change only the one function named above there.

Readers, as found by survey:

- `temper/crates/temper-client-domain/src/tests/forge_ways.rs`: `proposal_cards`, `proposal_read`, `a_proposals_items_are_read_once_and_bounded`. Their pages name kind 6 and 7 in heads under one key. Give them a `review` with a `key` different from the proposal's number, and repair what they assert, keeping the read-once, bound and eviction rows.
- `temper/crates/temper-client-view/src/tests/cards.rs`: `kinds_six_and_seven_are_named_until_their_values_are_read`, `a_settings_proposal_s_reason_draws_its_markdown` (its "charter 2" row stays), `draw_settings`. Add the empty-heads row and the template's.
- `temper/crates/temper-client-view/src/tests/chat.rs`: `place_turn` makes a `propose_settings` call; `tests/limits.rs` too. Repair what reads the raw name, only that.
- `temper/tools/gallery`: `configure-chat-proposal` and its rows. Regenerate what's stale. `src/tests.rs` near "Replaced by the proposal below" stays true.
- `temper/tests/client-browser`, `temper/docs/design/client/journeys`, `temper/tools/journeys`: the survey found no word of these there. `temper/tools/journeys/src/tests.rs` and `temper-client-bundle/tests/steps.rs` read `temper.css`: run them.

Options, each decided:

- **`Touched` for `propose_settings`:** `calls::touched` returns no argument for it (`Touched::Other { argument: None }`), and `tool_words` gets its row with new `Word`s, not `numbered`. `under_way` returns the tool's name on an empty argument, for every tool, and `present` takes a name's hole: special-case this tool there, saying `WorkProposingChange` with no hole, and change no other tool's words.
- **New `Word`s:** `WorkProposedChange` ("proposed a change"), `WorkProposedChangeCount` ("proposed a change ", N, "×"), `WorkProposingChange` ("proposing a change"), beside `WorkCheckedSettings`, numbered as that table numbers them.
- **The template's lookup:** extract the name's rule from `pages::goals::ways` into one small function both use, and look in `me.projects` in order for the first template of that number, tracked or not. Don't read the settings document for it.
- **A row for `ProjectPolicy`:** `setting_base`'s words as they stand.
- **If the phone rule makes `data-log-away` flicker when reading it:** by reading it can't, since the log shrinks when the pill shows, so it moves further from its end. If the code says otherwise, report it rather than changing skein.

## Steps

Outer shape:

```rust
// forge/ways.rs
struct ProposalRead { number: u64 /* the proposal's */, key: u64 /* review.key, what the forge keeps it under */, .. }
// proposals(): a Proposal page with review.key and an Application { kind: 6 | 7, .. } review item asks;
// an Ended page asks nothing.

// cards/proposal.rs
pub(crate) struct SettingsFacts<'a> { /* as now */ review: &'a [c::ReviewItem] /* empty once ended */, .. }
```

Tests: in `temper-client-domain` (the key, with `review.key` != the number; bite it by asking under the number), `temper-client-view` (rows from empty heads; the template's name, and unknown; the three phrases; no reason text in the name), the joined world's story, the CSS test above.

Then: `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-domain -p temper-client-view -p temper-client-shell-web -p temper-client-world -p temper-client-gallery --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-client-domain -p temper-client-view -p temper-client-shell-web -p temper-client-gallery -p temper-client-bundle`; `heavy cargo nextest run -p temper-client-world -E 'test(configuring) or test(reading) or test(chat_accept)'`. `heavy cargo nextest run -p temper-client-journeys`. A browser story compiles with `heavy cargo check -p temper-client-browser-world --tests`; none runs. No walk or render check.

`python3 -I -B tools/citations/citations.py --base origin/main`. Commit and push.

## Report

What ran and how many; the rule each new test bit on; what you regenerated; that Latest's test holds rules, not a press; your implementation and check minutes apart.
