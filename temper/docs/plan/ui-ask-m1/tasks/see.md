+++
id = "see"
part = "see"
kind = "feature"
needs = ["declare", "stock", "document"]
files = ["temper/crates/temper-engine-smith/src/see.rs", "temper/crates/temper-engine-smith/src/settings.rs", "temper/crates/temper-engine-smith/src/docs.rs", "temper/crates/temper-engine-smith/src/lib.rs", "temper/crates/temper-engine-smith/src/tests.rs", "temper/crates/temper-engine-smith/src/tests/see.rs", "temper/crates/temper-engine-domain/src/route/see.rs", "temper/crates/temper-engine-domain/src/route/see/forge.rs", "temper/crates/temper-engine-domain/src/route/see/proposal.rs", "temper/crates/temper-engine-domain/src/route/host_route.rs", "temper/crates/temper-engine-domain/src/route.rs", "temper/crates/temper-engine-domain/src/lib.rs", "temper/crates/temper-engine-domain/src/decision.rs", "temper/crates/temper-engine-domain/src/tests.rs", "temper/crates/temper-engine-domain/src/tests/see.rs", "temper/crates/temper-service/src/route.rs", "temper/tests/engine/domain/src/host_tool.rs", "temper/tests/engine/domain/src/protocol.rs", "temper/tests/engine/domain/tests/suite/configuring.rs", "temper/tests/engine/domain/tests/suite/configuring/fixture.rs", "temper/tests/engine/domain/tests/suite/settings/b2.rs", "temper/tests/system/src/composed.rs", "temper/docs/design/domain/agent.status.md"]
effort = "high"
minutes = 240
+++

# ui-ask-m1: `see` reads the settings, the history and temper's pages

## Goal

Ask temper's `see` reads by address: `settings` and its members, `revisions`, `revision
N`, `help` and its forms, and `more TOKEN`, through the run reads that exist, each answer
in 8.4.1's outer form; the draft's other addresses answer not yet, and any other line
unknown. It writes the whole skeleton the later tasks fill (`notes/contracts.md`, 9):
`settings landing`, `settings gates`, `file` and `proposal` are parsed and routed here to
stubs that answer not yet, so that `see-forge` and `see-proposal` touch neither
`route/host_route.rs` nor `temper-engine-smith/src/settings.rs`.

Acceptance:

- **Decode** (`temper-engine-smith/src/see.rs`, `declare`'s decoder): `see {"address":
  "more TOKEN"}` decodes into `Tool::See { line, from }`, `line` the token's words joined
  by single spaces and `from` its offset (contracts, section 8); a token that does not
  decode (no `@`, no digits after it, an offset past `u32`, a `%` not one of `%2B`,
  `%40`, `%25`, an empty word) decodes into `Tool::See { line: the call's line, from: 0
  }`, which the root answers as unknown. Every other line as `declare` decodes it.
- **Parse and route** (`temper-engine-domain/src/route/see.rs`, in the function `declare`
  made there): the line parses into an `Address` (sketch below) in the chat's project,
  `domain.core.tasks.delegation(key.task)`'s; the settings addresses, `revisions` and
  `revision N` are jig's run settings read, `NamedAction::Settings`, exactly as
  `route/tools.rs` sends `Tool::Settings` (a setting at revision 0 read at the newest);
  `help`, `help NAME`, `help search WORDS`, the not-yet addresses and an unknown line are
  answered at once with `HostDelivery::See`; `Landing`, `Gates` and `File` go to
  `route/see/forge.rs` (`landing`, `gates`, `file`, and `answered` for a forge read's
  answer), `Proposal` to `route/see/proposal.rs` (`read`, and `answered` for jig's
  answer), new modules whose functions each answer `SeeAnswer::NotYet` (contracts, 9.5:
  the later tasks fill them).
- **The flight** (`route/host_route.rs`, contracts 9.1): `FlightRender::Settings` gains
  `see: Option<Box<SeeFlight>>` (`None` where `decoded` builds it for `Tool::Settings`);
  `SeeFlight` and `SeeReading` are crate-private in `route/see.rs`. `route/see.rs` sets
  the call's flight to `Settings { see: Some(..), from, requested, asked }`, reading
  `Settings`, before it sends the read. `host_route::settings` keeps its body and, for
  `see: Some(..)`, emits `HostDelivery::See { to, line, from, answer:
  SeeAnswer::Settings(..) }` with the members it gives `HostDelivery::Settings` today;
  `host_route::render`, for such a flight reading `Settings` or `Proposal`, emits
  `HostDelivery::See { .., answer: SeeAnswer::Call { tool: b"settings", answer } }` in
  place of `Render`, so that the settings family's refusal and its `Unavailable` reach
  the renderer; `host_route::read`, for such a flight, passes the answer to
  `route/see/forge.rs`'s `answered` and nothing else (no `see` read of this task reaches
  it; `see-forge`'s does). A refusal of the asking family itself (`declare`'s path,
  before `route/see.rs` runs) stays as it is.
- **The types** of contracts 9.1, `HostDelivery::See`, `SeeAnswer` (all its variants),
  `SeeSettings`, `SeeLanding`, `SeeRepository` and `SeeProposal`, exported from the crate
  root; their readers gain an arm (below).
- **The renderer** (`temper-engine-smith/src/see.rs`): contracts 9.2's `SEE_PART`,
  `ADDRESSES` and `see_answer`, exported from the crate root, writing 8.4.1's form: the
  whole text from `from`, cut at the end of the last line within the room, `more: TOKEN`
  where the text goes on, then `see: ` and the address's line of the table; the room the
  smaller of `SEE_PART` and `max_bytes` less what follows the body; the not-yet, unknown,
  refusal, nothing-from and no-such-setting lines in 8.4.1's words, each an error with no
  `see:` line.
  - `SeeAnswer::Settings`: the text `settings` gives for the same answer and context,
    whole (`settings_page`'s before its page), but `SettingsAnswer::Unknown`, which
    answers 8.4.1's no-such-setting line.
  - `SeeAnswer::Call`: `answers.rs`' text for that tool (`answer_for_tool`); an error is
    prefixed `see LINE: ` and has no `see:` line; any other is the form's body, its
    `see:` line `file`'s.
  - `SeeAnswer::Landing`, `Gates` and `Proposal`: through `landing_text`, `gates_text`
    and `proposal_text`, functions of their own in `see.rs` that answer the not-yet line
    until `see-forge` and `see-proposal` fill them (contracts, 9.2).
  - `SeeAnswer::Page`: `temper_docs::find`; the whole text `NAME: TITLE`, a newline, the
    page; a name no page has, `no page NAME; the sections holding its words:` and the
    search of its words (`-` read as a space), not an error.
  - `SeeAnswer::Search`: `temper_docs::search` within eight hits, a line each `PAGE:
    HEADING: LINE`; `nothing matches`; past eight, a last line `more sections match;
    narrow the words`; a refused search (`Refused::says`) an error.
  - The `see:` line by the table of 8.4.1: by the settings answer's kind, and for a
    setting by its kind (kinds 1 and 2 the landing rule's line).
- **Its callers:** `temper-service/src/route.rs`, `host_delivery`, renders
  `HostDelivery::See` with `see_answer(.., &engine.domain.catalogue(),
  call_answer_bytes − 3, at most host_reply_bytes)` into `RenderedHostCall`, as it does
  `Settings`; the engine's domain world's `host_tool.rs` (its render loop stops at a
  `See` delivery as at a `Settings` one, and renders it with `call_answer_bytes − 3`),
  its `protocol.rs` (passes `See` on as `Settings`), and the system world's
  `composed.rs` (renders `See` as it renders `Settings`); `decision.rs`, `host_within`:
  `See` within as `Settings` is, but a `Call` answer, held to `transcript_bytes` as
  `Render`'s answer is.
- **Rows, failing first** (common.md):
  - `temper-engine-domain/src/tests/see.rs`, new, `mod see;` in `src/tests.rs`: one table
    of the parser, line and project to `Address`: each of 8.4.1's forms; `settings
    landing-rule 3 project` in project 7 to kind 2, scope 7, number 3; `settings
    project-ways` to kind 7, scope the project, number 1; `settings forge-charter 1` to
    kind 3, scope 0; `proposal t4/12` and `p2/9`; `help` to the page `index`; each
    not-yet form, `task 41 turn 1.2` among them; and unknown: `Settings`, `settings
    charter`, `settings charter 0`, `settings charter x`, `revision`, `help Bad_Name`,
    `help search`, `file qa/pilot`, `file qapilot README.md`, `proposal x4/12`, `more
    a@1`, an empty word list.
  - `temper-engine-smith/src/tests/see.rs`, new, `#[path]`ed from `src/tests.rs` as its
    siblings are: the token both ways (`settings policy 1` at 6,120 is
    `settings+policy+1@6120`; a word `a+b@c%d` is `a%2Bb%40c%25d`) and each undecodable
    token above; the cut: a text of short lines past `SEE_PART` cut at a line's end and
    its next part from there, a line longer than `SEE_PART` cut at a character's end with
    a multi-byte character at the edge, `from` inside a character, `from` at the end, a
    `max_bytes` that shrinks the body, one that holds no part; each error line exact;
    the `see:` line of each answer kind; a page whole and in parts, an unknown page, a
    search, a refused search.
  - `temper/tests/engine/domain/tests/suite/configuring.rs`, beside
    `the_configuring_agent_reads_each_kind`, one new table test,
    `an_asking_chat_reads_each_address`: the stock world, revision 2 made as that test
    makes it, a chat from template 5 (charter 11); its instructions hold
    `temper_engine_smith::ADDRESSES`; then a row an address, `(address, first line,
    a part of the text, error)`: `settings`, `settings charter 1`, `settings template
    5`, `settings model 1`, `settings rules`, `settings landing-rule 1`, `settings
    forge-charter 1`, `settings project-ways`, `settings project-questions`,
    `revisions`, `revision 2`, `help`, `help settings`, `help no-such-page`, `help search
    landing rule`, `settings landing`, `settings gates OWNER/NAME`, `file OWNER/NAME
    README.md`, `proposal t1/1`, `me`, `task 4 tree`, `goals ended`, `bogus`, `settings
    charter 99`; each answer within the host reply bound, each body (before its `more:`
    and `see:` lines) within `SEE_PART`, each non-error ending with a `see: ` line;
    `settings policy 1` (over 7,000 bytes) read whole through its `more` tokens, its
    parts joined parsing as the policy's JSON after its heading; the store's rows
    unchanged by every read. Then a chat of charter 11 whose authority lacks the
    settings family (as `a_chat_is_offered_no_settings_tools` narrows one): `see
    settings` answers an error beginning `see settings: ` and naming `tool family
    settings`.
  - `temper/tests/engine/domain/tests/suite/settings/b2.rs`: `declare`'s row expecting
    `see: not there yet.` now reads `me` and expects `see me: not there yet.`; nothing
    else of it changes.
- **Status** (a departure from the implementer's text, which closes `docs/design/`: the
  status must say what is built): delete from `temper/docs/design/domain/agent.status.md`
  the **Not yet** "8.4.1: `see` reads no address: ..." bullet, and nothing else.

## Inputs

- `temper/docs/plan/ui-ask-m1/notes/common.md`; `notes/contracts.md`, sections 6, 7, 8
  and 9 (9.1, 9.2, 9.5).
- `temper/docs/design/domain/agent.md`, 8.4.1 (all of it: the table is the spec of each
  answer), 8.1 (the inventory's context), 4.3 (the asking family's decision);
  `temper/docs/design/protocol/documentation.md`, section 5 (the pages, `read_docs`'
  search); `jig/docs/design/domain/engine.md`, 7.3 ("A run's reads are not recorded").
- The code it follows: `route/tools.rs` (`relay_payload`, the `Tool::Settings` branch);
  `route/host_route.rs` (`decoded`, `settings`, `render`, `rendered`); `temper-service`'s
  `route.rs` (`host_delivery`); `temper-engine-smith/src/settings.rs` (`settings_page`,
  `repository_text`) and `docs.rs`.

## Owns and may change

- Owns the files in the header. `route/see.rs` and `see.rs` are `declare`'s, then yours
  (contracts, section 7): keep `declare`'s decode of a plain line and its stub's
  signature, or widen the signature with `env` or `decision` where `route/tools.rs`'s
  call site has them, editing that call site alone.
- `temper-engine-smith/src/settings.rs`: `settings_page`'s text before its page becomes
  one `pub(crate)` function, which `settings_page` calls; nothing else of the file (the
  task `propose` writes its decoders).
- `docs.rs`: a function made `pub(crate)` for the search's hits, if you reuse it.
- `route/host_route.rs`: only what the acceptance names (the `see` member and its uses in
  `settings`, `render` and `read`, `HostDelivery::See`); the task `document`, before you,
  added its own variant and arms there. The tasks after you write nothing here.
- Readers of `HostDelivery` and `FlightRender::Settings`: the arms and the member the
  compiler names, each edited only for that.
- A change of meaning: none for `settings` (`HostDelivery::Settings` and its text are
  unchanged); run `temper/tests/engine/domain`'s `configuring::` tests whole, which
  prove it, and `temper-service`'s lib tests.
- Pre-approved: if `HostDelivery::See` cannot cross the journal's door where
  `host_route::settings` sends its delivery, emit it as `render` emits `Render`, and say
  so. If the stock world has no repository for the `file` and `gates` rows, use
  `OWNER/NAME` literally (they answer not yet). If the chat's first line or a stock
  number differs from the stock's on the trunk, take the trunk's and say so.

## Steps

1. The rows; run them; see them fail (or not compile).
2. The root: `Address`, its parser, the routing, the flight's `see`, the delivery; the two
   stub modules.
3. smith: the token's decode, the renderer, the factored settings text; the callers.
4. Narrow tests: `heavy cargo nextest run -p temper-engine-smith --lib`; `-p
   temper-engine-domain --lib`; `-p temper-engine-domain-world --test suite -E
   'test(/configuring::/) | test(/settings::b2::/)'`; `-p temper-service --lib`. Then
   `cargo fmt`, `python3 -I -B temper/scripts/old_forms.py`, `heavy cargo clippy -p
   temper-engine-smith -p temper-engine-domain -p temper-service -p
   temper-engine-domain-world --all-targets -- -D warnings`, and `heavy cargo check -p
   temper-system-world --tests`.
5. Commit and push.

## Sketch

```rust
// temper-engine-domain/src/route/see.rs
//! `see`: Ask temper's read by address (temper/docs/design/domain/agent.md, 8.4.1).
mod forge;
mod proposal;

// contracts 9.1: SeeAnswer, SeeSettings (pub).

/// One address, as its line parses in the chat's project (agent.md, 8.4.1).
pub(crate) enum Address {
    Inventory,
    /// `settings KIND N | rules | landing-rule N [project] | project-ways | project-questions`.
    Setting(jig_domain_core::settings::Setting),
    Landing,
    Gates { repository: Box<[u8]> },
    Revisions,
    Revision { number: u32 },
    Proposal { proposer: jig_domain_people::Proposer, number: u64 },
    Page { name: Box<[u8]> },
    Search { words: Box<[u8]> },
    File { repository: Box<[u8]>, path: Box<[u8]>, branch: Option<Box<[u8]>> },
    NotYet,
    Unknown,
}

/// Words split on ASCII whitespace; a number decimal, from 1, within u32 (a task's, u64);
/// a page's name 1 to 32 bytes of a-z, 0-9 and -; OWNER/NAME one `/` between two words.
pub(crate) fn address(line: &[u8], project: u32) -> Address;
```

- The `Setting`s: `charter|template|policy|model N` as `Setting::Charter(N)` and the
  rest; `rules`; `forge-charter N` kind 3, scope 0; `landing-rule N` kind 1, scope 0, and
  with `project` kind 2, scope the project; `project-ways` kind 7 and
  `project-questions` kind 6, scope the project, number 1 (as `temper-settings-form`'s
  `kinds.rs` maps them).
- `requested` and `asked` as `decoded` sets them for `Tool::Settings`: a revision read's
  number, a setting read's setting and revision; `host_route::settings`' stock fallback
  for kinds 6 and 7 then serves `see` too.

## Report

As the implementer's text says; give the bytes of `see settings`' first part in the
stock world, and say which of the pre-approved options you took.

## Rebase (2026-11-07, step merge, foreman ui-ask-m1/see/foreman-1)

resolve agent.status.md by preserving both branches’ gap removals
The merge conflicts only in `temper/docs/design/domain/agent.status.md`.
Commit `7bbc4e4ab368bdf23a765dc95acdb997ed34fffe` on the integration trunk removes the 8.4 “no step test holds … 16,384 bytes” bullet.
This task’s `1bf62c5020cfcbdd21e1df4231d7512dd1d9af46` removes the adjacent 8.4.1 “`see` reads no address” bullet.
Resolve the conflict with both bullets absent.
Keep the remaining forge-read, proposal-read and 8.4.2 gaps unchanged.
The log reports clean automatic merges for `temper-engine-smith/src/settings.rs` and `src/tests.rs`; retain those combined changes.
