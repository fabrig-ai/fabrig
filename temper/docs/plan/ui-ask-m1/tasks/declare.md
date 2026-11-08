+++
id = "declare"
part = "declare"
kind = "interface"
needs = ["family"]
files = ["temper/crates/temper-engine-smith/src/calls.rs", "temper/crates/temper-engine-smith/src/see.rs", "temper/crates/temper-engine-smith/src/document.rs", "temper/crates/temper-engine-smith/src/lib.rs", "temper/crates/temper-engine-smith/src/room.rs", "temper/crates/temper-engine-smith/src/answers.rs", "temper/crates/temper-engine-smith/src/tests.rs", "temper/crates/temper-engine-smith/src/tests/reference.rs", "temper/crates/temper-engine-smith/src/tests/room.rs", "temper/crates/temper-engine-domain/src/boundary.rs", "temper/crates/temper-engine-domain/src/translate.rs", "temper/crates/temper-engine-domain/src/room.rs", "temper/crates/temper-engine-domain/src/route/tools.rs", "temper/crates/temper-engine-domain/src/route/see.rs", "temper/crates/temper-engine-domain/src/route/document.rs", "temper/crates/temper-engine-domain/src/route.rs", "temper/tests/engine/domain/tests/suite/settings/b2.rs", "temper/docs/help/reference/host-tools.md", "temper/docs/help/charter.md", "temper/docs/design/domain/agent.status.md", "temper/docs/design/protocol/documentation.status.md"]
effort = "high"
minutes = 180
+++

# ui-ask-m1: an asking charter is declared its own tools alone

## Goal

A charter whose `tools` name the asking family is declared `see`, `document` and the
asking form of `propose_settings`, as `notes/contracts.md`, section 5, gives them, and
no other host tool; no other charter is declared the asking tools, and every other
charter's offer is byte for byte today's. `see` and `document` calls decode into the
root's `Tool::See` and `Tool::Document`, are decided by the asking family, and are
answered by stubs until the tasks `see` and `document` land. This is the surface those
tasks, `propose` and `stock` build on: it carries no reading and no writing.

Acceptance:

- `calls.rs`: `SPECS` gains `see` and `document` (after `read_docs`, in that order)
  with section 5's exact descriptions and schemas; a constant `ASK_PROPOSE_SETTINGS`,
  outside `SPECS`, the asking form, substituted as `PLAN_CHANGE` is for `change`.
  `tools_for(timeout, charter, authority)` offers, for an asking charter (section 5's
  test, `temper_engine_domain::asking(tools: jig_domain_tasks::Tools) -> bool`, which you
  write in `temper-engine-domain`'s `boundary.rs` and the task `workspace` reuses),
  exactly section 5's list in its order, gated as it says, and
  nothing else; for every other charter, `RunPolicy::EVERY_FAMILY` among them, it
  leaves out `see` and `document` and is otherwise unchanged.
  `tools(timeout)`, the full registry, holds `see` and `document` (two more than
  today's 41) and not the asking form.
- Decoding (`call_configured`): `see {address}` into `Tool::See { line, from: 0 }`, the
  line trimmed, empty or past 512 bytes refused at decoding naming the bound;
  `document {action, body}` into `Tool::Document { action, body }`; both in
  `see.rs` and `document.rs`, new modules of `temper-engine-smith` with a module doc
  citing `temper/docs/design/domain/agent.md`, 8.4.1 and 8.4.3. The one
  `propose_settings` decoder (`settings.rs`, `proposal`) takes `more`: absent or
  `false` decodes as today; `true` is answered at decoding with section 5's error line.
- `temper-engine-domain`: section 6's `Tool::See`, `Tool::Document` and
  `DocumentAction` in `boundary.rs` (re-exported as the crate's other boundary types
  are); `translate.rs`'s `tool_kind` maps both to `ToolKind::Ask`, and its other
  classifications treat `See` as a read that keeps no call record (as `Settings`) and
  `Document` as a write; `route/tools.rs` dispatches them to `route/see.rs` and
  `route/document.rs`, new modules whose one function each answers, after the call's
  authority is decided, section 6's stub line as an error. A call from a run whose
  authority lacks the asking family is answered beyond authority, naming the family,
  by the same path every family takes; `answers.rs` names the family "asking" where it
  names families in refusals.
- Host-part accounting: `room.rs` (`temper-engine-smith`, `host_part`) measures the
  asking declarations apart (a field of `temper_engine_domain::room::HostPart`, say
  `asking`), and `HostPart::non_brief` (`temper-engine-domain/src/room.rs`) counts, for
  a policy naming the asking bit, those bytes in place of every other host family's;
  `tests/room.rs`, `the_table_is_smiths_measure`, gains a row for an asking charter
  (smith's measure of exactly the three declarations equals the table's).
- Rows, not new tests:
  - `temper-engine-smith/src/tests.rs`, `tool_cases`: a valid and an invalid row for
    `see` and for `document`; `every_declared_tool_decodes_its_minimum_shape_and_rejects_a_wrong_shape`
    covers them; a `propose_settings` row with `"more": true` answered with the error
    line, and one with `"more": false` decoded as today.
  - The offer: beside the settings and message gate tests (`tests.rs`, near the
    `tools_for` uses at lines 135, 2093 and 3110), one table: `EVERY_FAMILY` →
    today's offer, no `see` or `document`; charter 3,072 + 512 (asking with read) →
    today's offer for those families, no `see` or `document`; charter 3,072 with
    authority holding asking and settings → `see`, `document`, `propose_settings`
    (the asking form's schema, no `note`); 2,048 alone → `see`, `document`; 3,072 with
    an authority lacking asking → `propose_settings` alone; charter 1,023 (every family
    but settings and asking) → today's offer, no `see` or `document`; charter 1,920
    (charter 3's) → today's offer, `propose_settings` with `note`.
  - `tests/reference.rs`: the catalogue's count 41 → 43 and its order.
  - `temper/tests/engine/domain/tests/suite/settings/b2.rs`, beside
    `no_settings_or_message_tools` and its counts (38, 39): a run of a charter with
    tools 3,072, under an authority holding asking and settings, is declared those
    three host tools alone; a `see` call from it is answered `see: not there yet.`, and
    from a run whose authority lacks asking, beyond authority naming the family. Use
    the world's own settings fixture for the charter (stock charter 11 is a later
    task's).
- Generated: `temper/docs/help/reference/host-tools.md` regenerated by
  `temper/crates/temper/src/tests/commands.rs`,
  `the_page_host_tools_is_what_the_declarations_render` (`SKEIN_CODEGEN_WRITE=1`),
  within its bound; if the asking form must show there, as a row of its own, add it as
  the page's generator shows `PLAN_CHANGE`, and say so.
- `temper/docs/help/charter.md`: its sentence that only the settings declarations are
  selected by a charter's mask gains, in a sentence, that a charter naming the asking
  family is declared `see`, `document` and `propose_settings` alone; within its 12,288
  bytes.
- Status (a departure from the implementer's text, which closes `docs/design/`: the
  status must say what is built): delete from `temper/docs/design/domain/agent.status.md`
  the **Not yet** "4.3: a charter whose `tools` name the asking family is declared
  every host tool ..." bullet, and from `temper/docs/design/protocol/documentation.status.md`
  the **Not yet** "5: `read_docs` is declared to every charter, one naming the asking
  family among them ..." bullet, and nothing else.

## Inputs

- `temper/docs/plan/ui-ask-m1/notes/common.md`; `notes/contracts.md`, sections 1, 5, 6
  and 7.
- `temper/docs/design/domain/agent.md`, 4.3 ("A charter that names the asking family
  is declared its own tools alone"; "The host part's budget"; "Declared, decoded and
  rendered in one place") and 8.4 (Ask temper's charter);
  `temper/docs/design/protocol/documentation.md`, section 5;
  `jig/docs/design/domain/authority.md`, section 3.

## Owns and may change

- Owns the files in the header. `see.rs` and `document.rs` in both crates are yours to
  create; after you, the tasks `see` and `document` write them (contracts, section 7):
  keep your code there to the decode and the stub.
- Readers: every exhaustive `match` on `temper_engine_domain::Tool` the compiler names
  (`temper-engine-smith`'s answers and renderers, `temper-service`'s route and tools,
  test fixtures) gains the two arms, each doing for `See` what it does for `Settings`
  and for `Document` what it does for `Note`, and nothing else. `temper-playground`,
  `temper/tests/system` and other users of `temper_engine_smith::tools(..).len()` see
  two more tools: a count they pin moves by two, edited only for that.
- `temper-engine-smith/src/settings.rs`: the `proposal` decoder's `more` alone; the
  rest of that file is the task `propose`'s.
- A change of meaning: none for any existing charter, whose offer stays byte for byte;
  `temper-service`'s stock tests (`src/tests/stock.rs`, `stock_brief.rs`) must pass
  unchanged, which proves it: run them.
- Pre-approved: if `HostPart` is built in more places than `host_part`, give each the
  new field's measure or zero where no asking charter can run there, and say so.

## Steps

1. The rows above; run them; see them fail (or not compile).
2. The root's vocabulary, the specs, the offer, the decoders, the stubs, the
   accounting.
3. Regenerate `host-tools.md`; correct `charter.md`.
4. Narrow tests: `heavy cargo nextest run -p temper-engine-smith --lib`, `-p
   temper-engine-domain --lib`, the b2 rows by name in the engine's domain world
   (`-p temper-engine-domain-world` or the package of `temper/tests/engine/domain`),
   `-p temper --lib -E 'test(=the_page_host_tools_is_what_the_declarations_render)'`
   (the package holding `temper/crates/temper/src/tests/commands.rs`), and
   `heavy cargo nextest run -p temper-service --lib` for the stock's unchanged figures.
   Then `cargo fmt`, `python3 -I -B temper/scripts/old_forms.py`, `heavy cargo clippy
   -p temper-engine-smith -p temper-engine-domain -p temper-service --all-targets -- -D
   warnings`, and `heavy cargo check -p temper-playground --tests` and `-p
   temper-system-world --tests` (or the packages of `temper/crates/temper-playground`
   and `temper/tests/system`).
5. Commit and push.

## Report

As the implementer's text says; give the asking charter's declaration bytes (the three
together) as `room.rs` measures them.
