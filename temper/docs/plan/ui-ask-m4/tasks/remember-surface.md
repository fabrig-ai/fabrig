+++
id = "remember-surface"
part = "remember"
kind = "interface"
needs = []
files = ["temper/crates/temper-engine-domain/src/boundary.rs", "temper/crates/temper-engine-domain/src/store.rs", "temper/crates/temper-engine-domain/src/route/remember.rs", "temper/crates/temper-engine-domain/src/route/tools.rs", "temper/crates/temper-engine-domain/src/route/host_route.rs", "temper/crates/temper-engine-domain/src/translate.rs", "temper/crates/temper-engine-smith/src/calls.rs", "temper/crates/temper-engine-smith/src/remember.rs", "temper/crates/temper-engine-smith/src/answers.rs", "temper/crates/temper-engine-smith/src/room.rs", "temper/crates/temper-engine-smith/src/tests/remember.rs", "temper/crates/temper-engine-smith/src/tests.rs", "temper/crates/temper-service/src/tools.rs", "temper/crates/temper-service/src/tests/ask_bound.rs", "temper/docs/help/reference/host-tools.md"]
effort = "high"
minutes = 150
+++

## Goal

Declare Ask temper's fifth tool, `remember`, to an asking charter beside
`see` and `document`, decode its calls, carry its answer type and render
every answer and refusal in 8.4.4's words
(`temper/docs/design/domain/agent.md`, 8.4.4); the root's route answers
every call "could not be read" until task `remember-route` writes it.

Acceptance:

- `RememberAction`, `Tool::Remember`, `Remembered`, `RememberDid` and
  `RememberRefusal` exist as `temper/docs/plan/ui-ask-m4/notes/contracts.md`,
  section 4 gives them, with their doc comments.
- `temper_engine_smith::calls::REMEMBER`: name `remember`, its description
  and members exactly 8.4.4's *Its declaration*, effect `Write` as
  `DOCUMENT`'s; `tools_for` declares it to an asking charter, after
  `document`, wherever it declares `document`, and `offered_tools` keeps it
  from every other charter, as it keeps `see` and `document`.
- `call_in` decodes it in `remember.rs`'s `decode`, as `document.rs`
  decodes `document`: `action` required, one of `add`, `change`, `drop`;
  `line` an integer from 1, required for `change` and `drop` and refused for
  `add`; `text` a string, required for `add` and `change` and refused for
  `drop`; any other member refused; each refusal in the words the shared
  decoder gives `document`'s.
- `translate.rs`'s `tool_kind` maps it to `ToolKind::Ask`, so the asking
  family decides it (`agent.md`, 4.3); `route/tools.rs` sends it to
  `route/remember.rs`'s `call`, which answers
  `Remembered::Refused(RememberRefusal::Unread)` and writes nothing.
- `remember.rs`'s `render` gives each answer and refusal 8.4.4 lists,
  exactly, with `answers.rs` dispatching to it as it dispatches to
  `document::render`; one table test, `src/tests/remember.rs`
  (`remember_answers_say_what_changed_or_why_not`), a row each: the three
  done answers with lists of 0, 1 and 12 lines, and every refusal, a
  `Note(jig_domain_notes::Refusal::Full)` among them.
- `every_declared_tool_decodes_its_minimum_shape_and_rejects_a_wrong_shape`
  (`src/tests.rs`) gains `remember`'s rows; the asking room (`room.rs`,
  `host_part`) counts its declaration with `see`'s and `document`'s.
- `temper-service`'s `asks_first_request_fits_both_shapes`
  (`src/tests/ask_bound.rs`) declares it among the offer and still passes
  within 16,384 bytes; `temper/docs/help/reference/host-tools.md` is
  regenerated with it.
- The engine domain world's `asking_declarations`
  (`temper/tests/engine/domain/tests/suite/settings/b2.rs:370`) and any other
  list of an asking charter's declarations name it.
- `temper/docs/design/domain/agent.status.md`'s **Not yet** entry `8.4,
  8.4.4: charter 11's runs are declared no remember` is deleted.

## Inputs

- `temper/docs/plan/ui-ask-m4/notes/common.md`; `notes/contracts.md`,
  section 4.
- `temper/docs/design/domain/agent.md`, 8.4.4 (*Its declaration*, *Its
  answer*, *Its refusals*); 8.4.3 for the model it follows; 4.3.
- The model to follow, `document` end to end (this cut's survey):
  `temper-engine-smith/src/calls.rs:162` (`DOCUMENT`), `:248` (`tools_for`),
  `:271` (`offered_tools`), `:323` (`call_in`); `src/document.rs` (`decode`,
  `render`, `refusal`); `src/answers.rs:820`;
  `temper-engine-domain/src/boundary.rs:158`; `src/store.rs:162`
  (`Documented`); `src/route/tools.rs:15` (`relay_payload`, line 121);
  `src/route/host_route.rs:271`; `src/translate.rs:469`;
  `temper-service/src/tools.rs:15` and `:125`; `src/tests/ask_bound.rs:210`.
- The host-tools page's generator:
  `TEMPER_BUNDLE=none SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p temper
  --lib -E 'test(=tests::commands::the_page_host_tools_is_what_the_declarations_render)'`.

## Owns and may change

- **Owns:** the files in this spec's header, for `remember` alone; the new
  types go right after `Documented` and `Tool::Document`, the new arms right
  after `document`'s, so that the jig tasks' forced arms in `store.rs` fall
  elsewhere.
- **May change:** `temper/tests/engine/domain/tests/suite/settings/b2.rs`
  and any other test naming an asking charter's declarations, a row each;
  any exhaustive match on `Tool` or `Documented`-like answers the compiler
  names, an arm each (`temper-service/src/route.rs`, `render_host`, among
  them).
- **Options, pre-approved:** if the first request passes 16,384 bytes with
  `remember` declared, shorten the `text` member's description first, then
  the tool's, keeping their facts, and say so with the parts the test
  prints; if it still does not fit, report `Question:` with those parts. If
  the shared decoder cannot refuse a member by action, refuse it in
  `decode` with the words `document`'s action check uses, and say so.

## Steps

1. The types; the declaration and `tools_for`; the decode.
2. The route's stub; the tool kind; the render and its table.
3. The declarations' tests, the first request's bound and the host-tools
   page; break one rule (declare `remember` to charter 1) to see a row
   fail, and restore.
4. `cargo fmt`; `heavy cargo clippy -p temper-engine-domain -p
   temper-engine-smith -p temper-service -p temper --all-targets -- -D
   warnings`; `heavy cargo nextest run -p temper-engine-smith --lib`;
   `heavy cargo nextest run -p temper-engine-domain --lib`; `heavy cargo
   nextest run -p temper-service --lib -E 'test(/ask_bound|stock/)'`;
   `heavy cargo nextest run -p temper-engine-domain-world -E
   'test(/asking|settings::b2/)'`; the host-tools page test. Commit and push.

## Report

As the implementer's text asks; give the first request's parts on both
shapes as the test prints them.
