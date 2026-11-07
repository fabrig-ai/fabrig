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
