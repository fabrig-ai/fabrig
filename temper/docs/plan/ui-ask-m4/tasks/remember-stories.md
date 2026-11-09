+++
id = "remember-stories"
part = "stories"
kind = "feature"
needs = ["party-runs", "party-reads", "party-brief", "withheld", "remember-route", "see-withheld", "ask-remembering", "page-domain"]
files = ["temper/tests/engine/domain/tests/suite/configuring/remembering.rs", "temper/tests/engine/domain/tests/suite/configuring.rs", "temper/tests/engine/domain/tests/suite/configuring/fixture.rs", "temper/tests/client/tests/suite/remembered_page.rs", "temper/tests/client/tests/suite/main.rs"]
effort = "high"
minutes = 210
+++

## Goal

The meeting point: jig's party scope, its brief section and its withheld
turns, temper's `remember` route and `see`, and the person's page, run
together in the request's three stories (`inputs/request.md`, *Its tests*;
`temper/docs/design/domain/agent.md`, 8.4.4, *Its tests*;
`temper/docs/design/client/README.md`, 5.17): a line kept in one Ask
temper chat, read in the next one's brief and removed from the page;
nobody reads another's lines; a turn's preferences hidden from another
role of the project and whole to its person.

Acceptance:

- In the engine's domain world, `suite/configuring/remembering.rs` (its
  `mod` line in `configuring.rs` after `asking_proposal`), on the asking
  chat's fixture (`configuring/fixture.rs`, `member_config` for a project
  of an owner P and a member Q), three stories, each a scripted run:
  - `a_kept_line_reaches_the_next_chat_and_leaves_with_its_person`: P's Ask
    temper chat calls `remember` `add` "I want short answers." and hears
    `remember: line 1 kept; 1 of 12 lines.` and the list; a second Ask
    temper chat of P's is assigned a brief whose "Kept about this person"
    section is `1. I want short answers.`; P's client `DeleteNote` of it,
    project 0, is done; a third chat's section is `None kept.`.
  - `nobody_reads_another_persons_lines`: with P's line kept, Q's client
    read of P's party scope is unknown and Q's `EditNote` and `DeleteNote`
    of it are refused, P's line unchanged; Q's own Ask temper chat's
    section is `None kept.`; Q's chat's `remember` `change` of line 1 hears
    `remember: no line 1; the list is empty. Nothing changed.`.
  - `a_turn_shows_its_persons_list_to_them_alone`: P's committed turn that
    called `remember`, read by P's client whole (the section's line and the
    call's input), by Q's client withheld (the section's line
    `Withheld: shown to its person alone.` and the call's input `{}`), on
    its own and on P's chat's activity page; and P's own `see task N turn
    1.1` from a later Ask temper chat renders it withheld.
- In the joined client world, `suite/remembered_page.rs` (its `mod` line in
  `main.rs`), beside `asking_chat.rs`'s fixture (`f::asking_world`), one
  story, `the_page_reads_and_deletes_what_ask_temper_kept`: a scripted Ask
  temper run keeps a line; P's client opens `Remembered`, reads it as one
  row naming the chat, deletes it, and reads the page with none kept.
- Each story bites: break one rule once (the brief's section left out, the
  withheld mark dropped, the delete's scope another person's) and see its
  story fail for that reason, and restore.

## Inputs

- `temper/docs/plan/ui-ask-m4/notes/common.md`; `notes/contracts.md`,
  sections 1 to 6.
- `temper/docs/plan/ui-ask-m4/inputs/request.md`, *Its tests*;
  `temper/docs/design/domain/agent.md`, 8.4.4; `temper/docs/design/client/README.md`,
  5.17; `jig/docs/design/domain/engine.md`, 10.1.
- Patterns: `temper/tests/engine/domain/tests/suite/configuring/asking_person.rs`
  (an Ask temper run's scripted calls and a person's reads),
  `configuring/fixture.rs` (`member_config`, `see_whole`, `assert_heard`);
  `temper/tests/engine/domain/tests/suite/root_routes.rs:1654` (a
  `document` call's rows); `temper/tests/client/tests/suite/asking_chat.rs`.

## Owns and may change

- **Owns:** the files in this spec's header; helpers it adds to
  `configuring/fixture.rs`, after its last function.
- **Options, pre-approved:** if the engine world's fixture cannot make the
  member's own Ask temper chat, read Q's brief from a chat the owner's
  fixture makes for Q, and say so. If the joined client world cannot drive
  the page's delete, make the story's delete through the client's jig ask
  and the page's read after it, and say so. A failure these stories find in
  another task's code is reported under `## Left` with the story's log, not
  repaired here, unless it is a fixture's.

## Steps

1. The engine world's stories; the client world's story; the breaks.
2. `cargo fmt`; `heavy cargo clippy -p temper-engine-domain-world -p
   temper-client-world --all-targets -- -D warnings`; `heavy cargo nextest
   run -p temper-engine-domain-world -E 'test(/remembering|asking/)'`;
   `heavy cargo nextest run -p temper-client-world -E
   'test(/remembered_page|asking_chat/)'`. Commit and push.

## Report

As the implementer's text asks; give each story's CPU seconds.
