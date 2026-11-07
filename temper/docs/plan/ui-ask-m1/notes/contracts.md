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
