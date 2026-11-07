+++
id = "family"
part = "family"
kind = "interface"
needs = []
files = ["jig/crates/jig-domain-core/src/lib.rs", "jig/crates/jig-domain-core/src/vocabulary.rs", "jig/crates/jig-domain-core/src/routing/mod.rs", "jig/crates/jig-domain-core/src/tests/**", "jig/docs/design/domain/authority.status.md"]
effort = "medium"
minutes = 45
+++

# ui-ask-m1: jig's asking family

## Goal

jig's engine tool families gain a twelfth, asking, bit 2,048, and a call of kind
`ToolKind::Ask` is decided by it, as every other family's call is by its bit. Nothing
yet makes such a call: temper's root does, in a later task.

Acceptance:

- `ToolFamilies` gains `ask: authority::Tools`, `ToolFamilies::JIG` gives it
  `authority::Tools(2048)`, and `valid()` counts it among twelve distinct bits, as
  `notes/contracts.md`, section 1, gives them, with a one-line doc comment citing
  `authority.md`, section 3.
- `ToolKind::Ask`, a unit variant, documented in a line; `Core::tool_family` maps it to
  `self.settings.tools.ask`; the call classification above it in
  `routing/mod.rs` lists it with the families whose call is `authority::Call::Tool`.
- One row in an existing table test of jig-domain-core's tool authority (find it with
  `rg 'tool_family|authorize_tool' jig/crates/jig-domain-core/src/tests`): a task whose
  authority holds bit 2,048 has an `Ask` call admitted; one whose authority lacks it has
  it answered beyond authority, naming the tool family (`Finding::Tool`). If no table
  holds family rows, add the pair as one table test beside the nearest authority test.
  `ToolFamilies::JIG.valid()` with `ask` equal to another family's bit is false: a row
  in the test that checks `valid` if there is one.
- `jig/docs/design/domain/authority.status.md`: delete the **Not yet** sentence "3: the
  asking family: ... so no call is decided by it." and nothing else (a departure from
  the implementer's text: the status must say what is built).

## Inputs

- `temper/docs/plan/ui-ask-m1/notes/common.md`; `notes/contracts.md`, section 1.
- `jig/docs/design/domain/authority.md`, section 3 (*Tools*: asking);
  `jig/docs/design/domain/engine.md`, 7.3 (the asking tools' row);
  `jig/docs/design/protocol/configuration.md`, section 3 ("the engine's tool families'
  bits").

## Owns and may change

- Owns the files in the header.
- Readers: every `ToolFamilies { .. }` literal and every exhaustive `match` on
  `ToolKind`, in any tree, gains the field or the arm and nothing else (the survey
  found `ToolFamilies::JIG` used by value everywhere, built in
  `jig/crates/jig-protocol-config/src/core.rs`, and `ToolKind` matched only in
  `jig/crates/jig-domain-core/src/routing/mod.rs`; temper's
  `temper/crates/temper-engine-domain/src/translate.rs` and `route/host_route.rs`
  construct it and need no edit).
- No wire or stored schema changes (authority and charter tools are `u64` masks). If a
  jig golden or a generated codec moves anyway, regenerate it by its own test and say so.
- A change of meaning for no existing value: no deployment's mask holds bit 2,048 yet,
  so no reader's test changes; run `heavy cargo check -p temper-engine-domain --tests`
  and `-p temper-service --tests` as the cross-tree check.

## Steps

1. The rows; run; see them fail (or fail to compile, for the new variant).
2. The field, the bit, the variant, the arms.
3. `heavy cargo nextest run -p jig-domain-core --lib`, then `-p jig-protocol-config`;
   `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy -p
   jig-domain-core --all-targets -- -D warnings`; the two cross-tree checks above.
4. Commit and push.

## Report

As the implementer's text says.
