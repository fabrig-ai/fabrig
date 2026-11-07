# ui-ask-m1: the shapes that cross tasks

Binding for every task of the plan. A later batch adds sections; a section, once
pushed, keeps its numbering and its shapes.

## 1. The asking family (jig)

`jig/docs/design/domain/authority.md`, section 3; `jig/docs/design/domain/engine.md`, 7.3.

In `jig/crates/jig-domain-core/src/lib.rs`:

```rust
pub struct ToolFamilies {
    // ... the eleven as they stand ...
    pub settings: u64,
    /// An application's tools for a run that explains to its person (authority.md, 3).
    pub ask: authority::Tools,
}

impl ToolFamilies {
    pub const JIG: Self = Self { /* ... as today ... */ settings: 1024, ask: authority::Tools(2048) };
}
```

- `valid()` counts `ask` among the families: twelve distinct bits.
- In `jig/crates/jig-domain-core/src/vocabulary.rs`, `ToolKind` gains `Ask`, a unit
  variant, decided by `ToolFamilies.ask` in `Core::tool_family`
  (`jig/crates/jig-domain-core/src/routing/mod.rs`) and listed wherever `ToolKind` is
  matched.
- No wire or stored schema changes: authority and charter tools are `u64` masks already.
- In words: "asking", bit 2,048. A mask that holds it is written with it added: charter 11's
  families are asking and settings, `3,072`; the rules' and policy 1's tools become
  `1,927 + 2,048 = 3,975`.

## 2. Ask temper's stock numbers

`temper/docs/design/draft/ask-temper.md`, section 9; `temper/docs/design/protocol/configuration.md`, 13.4.

| Setting | Number | Name |
|---|---|---|
| charter, Ask temper's | 11 | (charters have no name) |
| template, Ask temper | 5 | `ask` |

- Template 5 is a conversation run by charter 11, as template 3 is run by charter 3.
- The client knows it by number: `ASKING_TEMPLATE: u32 = 5` beside
  `CONFIGURING_TEMPLATE` (`temper/crates/temper-client-domain/src/pages/chats.rs`), and
  finds an Ask temper chat by template 5's configured charter, as it finds a configuring
  chat by template 3's.

## 3. The client's address

`temper/docs/design/client/README.md`, 3.5 (*Ask temper's page*) and 5.13 (*Ask temper*).

- `Address::Ask { project: u64 }` beside `Address::Configure { project }`
  (`temper/crates/temper-client-domain/src/address.rs`), printed and parsed as
  `/projects/<project>/settings/ask`.
- Words, exactly: the row's label "Ask temper"; its help "Ask how temper works, how it's
  set up here, or how your work ran. It changes nothing; it may propose settings for you
  to accept."; the page's heading "Ask temper", its question "What would you like to
  know?", its lede "Ask how temper works, how PROJECT is set up, or how a piece of work
  ran. It changes nothing; where you want temper to work differently, it proposes
  settings for you to accept.", its composer "Ask temper", its text area "Ask about
  temper or this project", and with no template 5 "Ask temper isn't set up in this
  project."

## 4. Claude Code with no workspace (smith)

`smith/docs/design/protocol/programs.md`, 5.1, 5.3 and section 8.

- Decided inside `smith-protocol-program`'s backend by `Settings.workspace` being `None`;
  no public type changes.
- The working directory is `.runs/<run>/work` under the programs directory: made empty,
  private, with the run's other files, and removed with them.
- Its arguments: `--system-prompt-file INSTRUCTIONS` in place of
  `--append-system-prompt-file INSTRUCTIONS`, `--tools ""` (one empty argument), and
  `--disable-slash-commands`, after `--strict-mcp-config`; every other argument as for a
  run with a workspace. Its summary session takes the same three.

## 5. The asking charter's declarations

`temper/docs/design/domain/agent.md`, 4.3 and 8.4.

An **asking charter** is one whose `tools` hold the asking bit (2,048) and no bit but
it and settings (1,024): `tools & 2048 != 0 && tools & !(2048 | 1024) == 0`. A mask of
every family (`RunPolicy::EVERY_FAMILY`, `u64::MAX`, as a record with none decodes) is
not one. The test is `temper_engine_domain::asking(tools: jig_domain_tasks::Tools) -> bool`
(`declare` writes it in `boundary.rs`); the same test decides its runs' workspace
(`workspace`). A run whose charter
is an asking one is offered, in this order, and nothing else of `SPECS`
(`temper/crates/temper-engine-smith/src/calls.rs`):

1. `see`, where its task's authority holds the asking family;
2. `document`, where its task's authority holds the asking family;
3. `propose_settings` in its asking form, where its charter names the settings family
   and its task's authority holds it.

No other charter is offered `see`, `document` or the asking form; charter 3's offer, and
every other charter's, is byte for byte today's. Each with the charter's call timeout.

`see`, effect `Read`:

- description: `Read temper by address, in your project: settings, revisions, proposals, temper's pages, a repository's file. Each answer ends with what to read next.`
- schema: `{"type":"object","additionalProperties":false,"required":["address"],"properties":{"address":{"type":"string","description":"One line, as your instructions list addresses: settings charter 3, help, more TOKEN."}}}`

`document`, effect `Write`:

- description: `Keep this chat's one document: write it whole first, its first line # TITLE; then append a section, or replace the one under the same ## heading.`
- schema: `{"type":"object","additionalProperties":false,"required":["action","body"],"properties":{"action":{"enum":["write","append","replace"],"description":"write: the whole document; append: a section at its end; replace: the section under body's first ## heading."},"body":{"type":"string","description":"Markdown: the whole document, or one section beginning with its ## heading."}}}`

`propose_settings`, its asking form, effect `Write` (a second `Spec` of the same name,
`ASK_PROPOSE_SETTINGS`; the decoder is one, and takes either form's members):

- description: `Propose whole settings against the newest revision you read; an owner decides, and nothing changes until then. A large set goes in parts.`
- schema: today's `propose_settings` schema with `note` removed, `items` required beside
  `against` and `reason`, `items`' description `The changes, at most one per setting, in this call's part.`,
  `set`'s description `Its whole value as a settings file holds it; forge-charter takes a number; a document's repository is OWNER/NAME.`,
  and one member more after `items`:
  `"more":{"type":"boolean","description":"true: hold these items for the next call; the call without it proposes them all."}`.
  Until the task `propose` lands, a call with `"more": true` is answered at decoding,
  an error: `propose_settings: more is not taken yet; give the whole set in one call.`

## 6. The root's vocabulary

In `temper/crates/temper-engine-domain/src/boundary.rs`, `Tool` gains:

```rust
/// Read by one address line, from a byte offset of its answer (domain/agent.md, 8.4.1).
See { line: Box<[u8]>, from: u32 },
/// Write the chat's one document whole, or one section of it (domain/agent.md, 8.4.3).
Document { action: DocumentAction, body: Box<[u8]> },
```

```rust
/// What a `document` call does with its body.
pub enum DocumentAction { Write, Append, Replace }
```

- `line` is the address as the call gave it, at most 512 bytes, trimmed, and `from` 0;
  `declare` decodes every `see` call so. The task `see` then decodes `more TOKEN` into the
  `See` its token names (section 8), in `see.rs`.
- `translate.rs`'s `tool_kind` maps both to `jig_domain_core::ToolKind::Ask`; every
  other exhaustive match on `Tool` gains the arms the compiler names (a See is a read:
  it decides nothing and keeps no call record, as `Settings` does; a Document is a
  write, decided and named once).
- Until `see` and `document` land, `route/see.rs` and `route/document.rs` answer every
  call, after its authority is decided, with the error line `see: not there yet.` and
  `document: not there yet.`

## 7. Who writes which file

So that the tasks after `declare` work in parallel on files of their own:

| File | Created by | Then written by |
|---|---|---|
| `temper/crates/temper-engine-smith/src/calls.rs` | (exists) | `declare` alone; later tasks add nothing to `SPECS` |
| `temper/crates/temper-engine-smith/src/see.rs` | `declare` (decode of `see`) | `see`, then `see-forge`, `see-proposal` in their own functions |
| `temper/crates/temper-engine-smith/src/document.rs` | `declare` (decode of `document`) | `document` |
| `temper/crates/temper-engine-domain/src/route/see.rs` | `declare` (the stub) | `see`; `see-forge` and `see-proposal` add their arms' bodies in `route/see/forge.rs` and `route/see/proposal.rs` |
| `temper/crates/temper-engine-domain/src/route/document.rs` | `declare` (the stub) | `document` |
| `temper/crates/temper-engine-domain/src/route/forge_route.rs`, `run_workspace` | (exists) | `workspace` |
| `temper/crates/temper-engine-smith/src/settings.rs` | (exists) | `propose` |
| `temper/crates/temper-service/src/stock.rs`, `stock/ask.txt` | `stock` | |
| `temper/crates/temper-service/src/tests/stock_brief.rs` | (exists) | `stock` (charter 11's row), then `bound` |

## 8. `see`'s answers, their outer form

`temper/docs/design/domain/agent.md`, 8.4.1 gives each address's answer; every answer
keeps this form, rendered as text by `temper/crates/temper-engine-smith/src/see.rs`:

- **Bounded:** the body at most 6,144 bytes, cut at a line's end, then a line
  `more: TOKEN`. TOKEN is the address's words joined by `+`, then `@` and the decimal
  byte offset where the next part starts, as `settings+charter+3@6144`; a word holding
  `+`, `@` or `%` writes them `%2B`, `%40` and `%25`. `see {"address": "more TOKEN"}`
  reads that address again from that offset; a token that names no address is
  answered as an unknown address.
- **Then what to read next:** a last line `see: ADDRESS · ADDRESS`, the addresses one
  level down, but on a refusal, a not-yet and an unknown address.
- **Not yet:** `see ADDRESS: not there yet.`, an error, for `me`, `project`, `goals`,
  `goals ended`, `tasks`, `task N`, `task N tree`, `task N activity`, `task N turn A.T`,
  `change N`, `changes` and `inbox`.
- **Unknown:** `see: no address ADDRESS; addresses: ` and the list of section 8's
  instructions, `me · project · goals [ended] · ...`, an error.
- **A refusal** says in a line what the read did not reach, an error: the existing
  reads' refusals in their words, prefixed `see ADDRESS: `.

## 9. `see`'s reads: the root's answer, its delivery, its renderer, jig's proposal read

`temper/docs/design/domain/agent.md`, 8.4.1; `jig/docs/design/domain/engine.md`, 7.3.

`see` writes the whole skeleton, every type, variant, dispatch arm and branch below, with
stubs; `see-forge` and `see-proposal` fill their stubs' bodies in their own modules and
functions, and touch neither `route/host_route.rs` nor `temper-engine-smith/src/settings.rs`.

### 9.1 What the root read, and its delivery (the task `see`)

In `temper/crates/temper-engine-domain/src/route/see.rs`, exported from the crate root as
the host route's types are (`route.rs`'s `pub use`, then `lib.rs`'s `pub use route::{..}`):

```rust
/// What a `see` call read, for its renderer (domain/agent.md, 8.4.1).
#[derive(PartialEq, Eq, Hash, Debug)]
pub enum SeeAnswer {
    /// A `settings` family read's answer, with an inventory's context.
    Settings(Box<SeeSettings>),
    /// A read's own answer as the tool `tool` renders it: the settings family's refusal
    /// (`b"settings"`); a branch or file read's answer or refusal (`b"read_branch"`,
    /// `b"read_file"`); a repository the project has none of (`b"read_file"`).
    Call { tool: &'static [u8], answer: Box<crate::CallAnswer> },
    /// `help` (the page `index`) and `help NAME`.
    Page { name: Box<[u8]> },
    /// `help search WORDS`, the words joined by single spaces.
    Search { words: Box<[u8]> },
    /// `settings landing` (filled by `see-forge`).
    Landing(Box<SeeLanding>),
    /// `settings gates OWNER/NAME` (filled by `see-forge`).
    Gates(Box<SeeRepository>),
    /// `proposal tN/K`, `proposal pN/K` (filled by `see-proposal`).
    Proposal(Box<SeeProposal>),
    /// An address of `ask-temper.md`, 5.1 this milestone does not read, or does not read
    /// yet in this plan (the stubs).
    NotYet,
    /// No address.
    Unknown,
}

/// What `HostDelivery::Settings` carries beside its reply and offset.
#[derive(PartialEq, Eq, Hash, Debug)]
pub struct SeeSettings {
    pub answer: jig_domain_core::SettingsAnswer,
    pub repositories: Box<[RepositoryIdentity]>,
    pub people: Box<[ProjectPerson]>,
    pub forge_people: Box<[ForgePerson]>,
    pub offer: Box<[OfferedModel]>,
    pub application: Option<temper_engine_domain_forge::settings::SizedValue>,
    pub requested: Option<u64>,
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct SeeLanding {
    pub by: temper_engine_domain_forge_change::Way,
    pub merge_job: Option<Box<[u8]>>,
    /// The project's after-landing settings, where its configuration gives them.
    pub after: Option<temper_engine_domain_forge::AfterLanding>,
    /// Each of the project's readable repositories, in the inventory's order.
    pub repositories: Box<[SeeRepository]>,
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct SeeRepository {
    /// `OWNER/NAME`, as the inventory names it.
    pub name: Box<[u8]>,
    /// Its gates and tiers; both empty where the configuration gives it none.
    pub checks: temper_engine_domain_forge::RepositoryGates,
}

/// A proposal of settings as jig's run read answers it (9.4).
#[derive(PartialEq, Eq, Hash, Debug)]
pub enum SeeProposal {
    Waiting(jig_domain_core::documents::Proposal),
    Decided(jig_domain_core::reads::Ended),
    Unknown,
}
```

`HostDelivery` (`route/host_route.rs`) gains, after `Settings`:

```rust
/// A `see` read's answer, for its outer form (domain/agent.md, 8.4.1).
See { to: ReplyTo, line: Box<[u8]>, from: u32, answer: Box<SeeAnswer> },
```

- `line` is the address as read: the call's line, trimmed, or the words `more`'s token
  names, joined by single spaces. `from` is the byte offset the part starts at.
- No codec: `HostDelivery` has no stored or wire form.

The flight of every `see` read that waits for an answer, crate-private, in `route/see.rs`:

```rust
/// A `see` read in flight: its line, and which read answers next (domain/agent.md, 8.4.1).
pub(crate) struct SeeFlight {
    pub line: Box<[u8]>,
    pub reading: SeeReading,
}
pub(crate) enum SeeReading {
    /// A `settings` family read, `revisions`, `revision N`.
    Settings,
    /// `file`'s branch read, then its file read.
    Head,
    File,
    /// jig's proposal read.
    Proposal,
}
```

- `FlightRender::Settings` gains `see: Option<Box<SeeFlight>>` (`None` where `decoded`
  builds it); a `see` read's flight is `Settings { see: Some(..), from, requested, asked
  }`, `requested` and `asked` none but for a settings read.
- `host_route::settings` emits `HostDelivery::See` for it; `host_route::render` emits it
  with `SeeAnswer::Call { tool: b"settings", .. }` for the core's refusal of a
  `Settings` or `Proposal` read; `host_route::read` passes a forge answer for it to
  `route/see/forge.rs`'s `answered`, unchanged otherwise.

### 9.2 The renderer (the task `see`)

In `temper/crates/temper-engine-smith/src/see.rs`, exported from the crate root:

```rust
/// The most an answer's body holds (domain/agent.md, 8.4.1).
pub const SEE_PART: u32 = 6_144;

/// The line of addresses an unknown address answers with, as `ask.txt` gives it.
pub const ADDRESSES: &str = "me · project · goals [ended] · tasks · task N [tree | activity | turn A.T] · change N · changes · inbox · settings [KIND N | rules | landing-rule N [project] | landing | gates OWNER/NAME | project-ways | project-questions] · revisions · revision N · proposal tN/K or pN/K · help [NAME | search WORDS] · file OWNER/NAME PATH [BRANCH] · more TOKEN";

/// Render a `see` read's answer in its outer form, within `max_bytes`: the call's
/// answer allowance less the tool's name, and the receiving agent's.
#[must_use]
pub fn see_answer(
    line: &[u8],
    from: u32,
    answer: &temper_engine_domain::SeeAnswer,
    catalogue: &[jig_domain_core::settings::CatalogueModel],
    max_bytes: u32,
) -> smith_domain_run::HostAnswer;
```

- `ADDRESSES` is, byte for byte, the line after "see reads by address, in your project:"
  in `ask-temper.md`, section 8, which `stock/ask.txt` holds (` · `, U+00B7 between
  spaces).
- Inside `see.rs`, each answer's whole text comes from a function of its own, which the
  form then cuts: `see` writes the settings, help, `Call` and error ones, and three
  stubs, `landing_text`, `gates_text` (`see-forge`'s) and `proposal_text`
  (`see-proposal`'s), each answering the not-yet line until filled.
- Called by `temper-service`'s `route.rs` (`host_delivery`) with
  `call_answer_bytes − 3`, at most `host_reply_bytes`, and by the engine's domain world
  (`temper/tests/engine/domain/src/host_tool.rs`) and the system world
  (`temper/tests/system/src/composed.rs`) with `call_answer_bytes − 3`.

### 9.3 The forge's addresses (the task `see-forge`)

In `temper/crates/temper-engine-domain-forge/src/landing.rs`, `impl Domain`, beside
`merge_job` and `after_landing`:

```rust
/// A project's landing way: queue where its configuration gives none (domain/landing.md, 13.1).
#[must_use]
pub fn landing_way(&self, project: u32) -> temper_engine_domain_forge_change::Way;
/// A repository's configured gates and tiers in its project, if it has any.
#[must_use]
pub fn repository_gates(&self, project: u32, repository: skein_forge::api::Repository) -> Option<&RepositoryGates>;
```

- `file` answers with `SeeAnswer::Call { tool: b"read_file", .. }` (the file or the
  directory, or the file read's refusal) or `{ tool: b"read_branch", .. }` (the head's
  read refused or missing).

### 9.4 jig's proposal read (the task `proposal-read`)

A core action and answer of its own, beside the settings reads, so that temper's
exhaustive matches on `SettingsRead` and `SettingsAnswer` stay as they are. In
`jig/crates/jig-domain-core/src/vocabulary.rs` (or where `NamedAction` and `Now` are):

```rust
// NamedAction gains:
/// A run's read of one proposal of settings, by the `settings` family (engine.md, 7.3).
ReadProposal { proposer: jig_domain_people::Proposer, number: u64 },

// Now gains:
/// A run's proposal read answered (engine.md, 7.3).
ProposalRead { to: ReplyTo, key: CallKey, answer: Box<ProposalRead> },
```

In `jig/crates/jig-domain-core/src/reads/settings.rs`, exported beside `SettingsAnswer`:

```rust
/// One proposal of settings, as a run reads it (engine.md, 7.3).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ProposalRead {
    /// Waiting, with its review, as a party's live proposal read gives it, with no acts.
    Waiting(crate::documents::Proposal),
    /// Decided, withdrawn or replaced, as its history row keeps it.
    Decided(crate::reads::Ended),
    /// None, not settings, or a task's of another project.
    Unknown,
}
```

- `number` is `Read::Proposal`'s `proposal`: the deployment-wide number a `tN/K` or
  `pN/K` names as K; N is the task or the party.
- Like `NamedAction::Settings`, it takes the unrecorded path
  (`routing/run/calls.rs`): decided by the `settings` family, no call record; a refusal
  answers `Now::EffectAnswer`, as a settings read's does.
- jig's own `settings` tool (`jig-protocol-tools`) decodes `{"kind":"proposal","task":N,"number":K}`
  and `{"kind":"proposal","party":N,"number":K}` into it, and renders the answer in one
  line: `Proposal waiting`, `Proposal decided`, `Unknown proposal`.
- temper's one forced arm: `temper/crates/temper-engine-domain/src/route/now.rs`,
  `core::Now::ProposalRead { .. }`, unreachable until `see-proposal` sends the action
  and replaces it with a call into `route/see/proposal.rs`.
- No wire or stored schema changes: neither is encoded.

### 9.5 Files beyond section 7

| File | Written by | What |
|---|---|---|
| `temper/crates/temper-engine-domain/src/route/see.rs` | `see` | the parser, the routing, 9.1's types and every arm |
| `temper/crates/temper-engine-domain/src/route/see/forge.rs` | `see` (stubs answering `NotYet`: `landing`, `gates`, `file`, `answered`) | `see-forge` |
| `temper/crates/temper-engine-domain/src/route/see/proposal.rs` | `see` (stubs answering `NotYet`: `read`, `answered`) | `see-proposal` |
| `temper/crates/temper-engine-domain/src/route/host_route.rs` | `see` | `HostDelivery::See`; `FlightRender::Settings`' `see` and its uses in `settings`, `render` and `read` (9.1) |
| `temper/crates/temper-engine-domain/src/route/now.rs` | `proposal-read` (the unreachable arm) | `see-proposal` (that arm's body) |
| `temper/crates/temper-engine-domain/src/route/forge_route/reads.rs` | (exists) | `see-forge`: the file read's admission from `domain.limits` |
| `temper/crates/temper-engine-domain-forge/src/landing.rs` | (exists) | `see-forge`: 9.3's accessors |
| `temper/crates/temper-engine-smith/src/see.rs` | `declare` (decode) | `see` (the token, the form, every arm, the settings, help and error texts, the three stubs); `see-forge` (`landing_text`, `gates_text`, the file's `see:` line); `see-proposal` (`proposal_text`) |
| `temper/crates/temper-engine-smith/src/settings.rs` | (exists) | `propose` (its decoders); `see`: `settings_page`'s text before its page as one `pub(crate)` function |
| `temper/crates/temper-settings-form/src/render.rs` (or a module beside it) | (exists) | `see-proposal`: a proposal's and a decided proposal's text |
| `jig/crates/jig-domain-core/src/vocabulary.rs`, `reads/settings.rs`, `reads.rs`, `reads/rows.rs`, `documents/objects.rs`, `routing/run/calls.rs` | (exist) | `proposal-read` |

## 10. `document` and `propose_settings`' asking form

`temper/docs/design/domain/agent.md`, 8.4.2 and 8.4.3. Section 9 is `see`'s.

### 10.1 `document`'s answer

In `temper/crates/temper-engine-domain/src/store.rs`, beside `CallAnswer`, exported from
the crate's `lib.rs` as `CallAnswer` is (the task `document`):

```rust
/// A `document` call's answer, joined at its host flight (domain/agent.md, 8.4.3).
CallAnswer::Document(Box<Documented>),

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Documented {
    pub action: DocumentAction,
    /// The title for a write, the section's heading for an append or a replace, as the
    /// call gave it; empty where the body names none.
    pub heading: Box<[u8]>,
    pub outcome: DocumentOutcome,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum DocumentOutcome {
    /// jig's note write, as `CallAnswer::NoteWritten` carries it.
    Written { name: u64, revision: u32, lines: u32, added: u32, removed: u32, first: u32 },
    /// Nothing changed.
    Refused(DocumentRefusal),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum DocumentRefusal {
    NoTitle,
    TitlePastBound { bytes: u32, bound: u32 },
    NotOneSection,
    NoDocument,
    /// The document's headings in order within 1,024 bytes; `more` when some are left out.
    NoSection { headings: Box<[Box<[u8]>]>, more: bool },
    SectionThere,
    PastBound { bytes: u32, bound: u32 },
    Moved,
    Busy,
    Unread,
    Denied { family: NoteFamily, answer: jig_domain_authority::Answer, findings: Box<[jig_domain_authority::Finding]> },
    Note(jig_domain_notes::Refusal),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum NoteFamily { Note, Recall }
```

- **Rendered** by `temper/crates/temper-engine-smith/src/document.rs` alone, in 8.4.3's
  words. A `Written` answer's first line is, byte for byte, what `note`'s answer says
  of the same write (`Note N revision R written · ...`); the client reads that line
  (`temper-client-domain/src/pages/chat/document.rs`, `tool_write(b"document", ..)`).
- **No codec:** `CallAnswer` has no stored or wire form; nothing migrates.

### 10.2 Files beyond section 7

| File | Written by | What |
|---|---|---|
| `temper/crates/temper-engine-domain/src/store.rs` | `document` | 10.1's types, `call_answer_bytes`' arm |
| `temper/crates/temper-engine-domain/src/route/host_route.rs` | `document` | `FlightRender::Document`, its arms in `decoded` and `render` |
| `temper/crates/temper-engine-domain/src/boundary.rs` | `document` | `Read::Document` |
| `temper/crates/temper-engine-domain/src/archive.rs`, `restart.rs` | `document` | the `Waiting` arm routing its pages |
| `temper/crates/temper-engine-smith/src/answers.rs` | `document` | one arm, `CallAnswer::Document`, beside `NoteRefused`; `Text`, `note_written` `pub(crate)` |
| `temper/crates/temper-client-domain/src/pages/chat.rs`, `pages/chat/document.rs`; `temper/crates/temper-client-view/src/pages/document.rs` | `document` | `note` read as `note` or `document` |
| `temper/crates/temper-engine-domain/src/store.rs` | `propose` | 10.3's `PartsAnswer`, `CallAnswer::Parts`, its `call_answer_bytes` arm |
| `temper/crates/temper-engine-domain/src/boundary.rs` | `propose` | `Tool::ProposeSettings`' `more` |
| `temper/crates/temper-engine-domain/src/domain.rs` | `propose` | `repository_identities`; the held parts and the restart guard's set |
| `temper/crates/temper-engine-domain/src/route/tools.rs`, `route/kinds.rs`; `route/settings_parts.rs` (new) | `propose` | `propose_settings` through the parts; `call_approval_refusal` |
| `temper/crates/temper-engine-smith/src/calls.rs` | `propose` | `call_in` beside `call_configured`; no `Spec` |
| `temper/crates/temper-engine-smith/src/answers.rs` | `propose` | `proposed`'s settings words; one arm, `CallAnswer::Parts` |
| `temper/crates/temper-settings-form/src/file.rs`, `settings_file_values.rs` | `propose` | `Repository`, `proposal_items_in` |
| `temper/crates/temper-service/src/tools.rs`, `route.rs`; `temper/tests/engine/domain/src/host_tool.rs` | `propose` | the decoder given the task's repositories |

### 10.3 `propose_settings`' asking form (the task `propose`)

In `temper/crates/temper-engine-domain/src/boundary.rs`:

```rust
ProposeSettings {
    against: u32,
    items: Box<[jig_domain_core::settings::Item]>,
    reason: Box<[u8]>,
    kinds: Box<[KindValue]>,
    /// A part to hold: `"more": true` (domain/agent.md, 8.4.2).
    more: bool,
},
```

In `store.rs`, beside `CallAnswer`, exported from `lib.rs`; rendered by
`temper-engine-smith/src/settings.rs` alone, in 8.4.2's words:

```rust
/// A `propose_settings` call answered by its parts, or refused before jig decides it
/// (domain/agent.md, 8.4.2). It has no record.
CallAnswer::Parts(Box<PartsAnswer>),

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum PartsAnswer {
    /// A part held: `count` its own items, `total` all held, for revision `against`;
    /// `dropped`, the count and revision of the parts it replaced.
    Holding { count: u32, total: u32, against: u32, dropped: Option<(u32, u32)> },
    /// A setting held already.
    Held { setting: jig_domain_core::settings::Setting },
    /// Past one proposal's bound.
    PastBound { most: u32, would: u32 },
    /// A proposing call against another revision than the parts held, which it dropped.
    Revision { held: u32, against: u32, named: u32 },
    /// The first proposing call of a run that ran across the engine's restart.
    Restarted,
    /// `more` from a run whose charter names no asking family.
    NotOffered,
    /// Acceptance's approval refusal, judged at the call for the chat's project.
    Approvals { why: Box<[u8]> },
}
```

- `temper_engine_domain::Domain::repository_identities(&self, task: u64) ->
  Box<[RepositoryIdentity]>`: what `settings_repositories` gives an inventory of the
  task's project.
- `temper_engine_smith::call_in(name, tool, input, models, catalogue, repositories:
  &[temper_engine_domain::RepositoryIdentity]) -> Result<Decoded, Problem>`, beside
  `call_configured`, which passes none.
- `temper_settings_form::file::Repository<'a> { host, owner, name }` and
  `proposal_items_in(bytes, limits, repositories: Option<&[Repository<'_>]>)`;
  `proposal_items` is it with `None`.
- **No codec:** nothing of these is stored or crosses a wire.
