+++
id = "jig-chat-from"
part = "chat-template"
kind = "interface"
needs = []
files = ["jig/crates/jig-client/src/shapes.rs", "jig/crates/jig-client/src/tests/shapes.rs", "jig/docs/design/client/domain/client.status.md"]
effort = "medium"
minutes = 20
+++

# jig: `shapes::chat_from`, a chat from a template named by its number

## Goal

An application offering more than one chat's template (temper's `chat start --template`)
fills any of them into a `Create` the way `shapes::chat` fills template 1
(`jig/docs/design/client/domain/client.md`, 19, the `shapes` bullet). temper's ctl builds on
it in `ctl-chat-template`.

Acceptance:

- `pub fn chat_from(me: &Me, project: u32, number: u32, words: Text) -> Result<Ask, NotOffered>`
  in `jig/crates/jig-client/src/shapes.rs`, beside `chat`: the template `number` that `me`
  offers the project (`template(me, project, number)`), filled by the same `fill` as `chat`,
  into `Ask::Create { project, tasks }` of one task.
- It gives `Err(NotOffered::Project)` or `Err(NotOffered::Creating)` as `chat` does for the
  project and its creating bounds; `Err(NotOffered::Template)` for a number `me` does not
  offer the project, for a template whose `flags.conversation` is false (a goal's, for one),
  and for every template `fill` cannot fill.
- `chat(me, project, words)` keeps its behaviour and its signature (its callers in
  `jig-client-view`, `temper-client-domain` and `temper-client-tools` are unchanged): it may
  call `chat_from(me, project, CHAT, words)` only if no test of `chat` changes; otherwise both
  share a private helper and only `chat_from` checks the conversation flag.
- Tests in `jig/crates/jig-client/src/tests/shapes.rs`, using today's `me()`,
  `shape_project` and `shape_task` fixtures (template 1 is a conversation's, template 2 is
  not):
  - `chat_from_fills_a_named_conversation_template`: a `me` whose project offers a third
    template, number 3, a conversation's, run by charter 3 (charter 3 added to the creating
    bounds' charters); the task's executor is `Agent { charter: 3 }`, its flags keep
    `conversation`, its words are the words given;
  - `chat_from_template_1_is_chat`: `chat_from(.., CHAT, ..)` equals `chat(..)` for the same
    inputs;
  - `chat_from_refuses_a_template_that_is_no_conversation`: template 2 gives
    `NotOffered::Template`;
  - `chat_from_refuses_a_template_not_offered`: number 9 gives `NotOffered::Template`.
  `heavy cargo nextest run -p jig-client --lib -E 'test(/shapes::/)'` passes, its count in
  the report.
- The status entry this builds leaves `jig/docs/design/client/domain/client.status.md`: delete
  the sentence starting "19: `shapes::chat_from`" from its **Not yet** line, and nothing else.

## Inputs

- `temper/docs/plan/configuring-p1/notes/common.md`.
- `jig/docs/design/client/domain/client.md`, 19 (the `shapes` bullet); contract.md, 3.2 (a
  task's flags, *conversation*) and 3.6 (templates).
- Old code, for reference: the old jig commit "Add named chat templates for temper's
  configuring chat" did this in 9 lines: `chat` became `chat_from(me, project, CHAT, words)`,
  and `chat_from` looked the template up by number, returned `NotOffered::Template` when
  `!template.flags.conversation`, then `fill(template, creating, words, None)`.

## Owns and may change

- Owns the files in the header.
- Readers of `shapes::chat` (`jig/crates/jig-client-view/src/state/page/conversations.rs`,
  `temper/crates/temper-client-domain/src/pages/chats.rs`,
  `temper/crates/temper-client-tools/src/jig/runner/acts.rs`): read only; nothing of theirs
  changes.
- Pre-approved: if a fixture needs a third template, build it in the test from `me()`'s
  template 1 (`Template { number: 3, executor: TemplateExecutor::Charter(3), ..base }`), not
  in `fixtures.rs`; if `fixtures.rs` must change, keep it to one helper and list it.

## Steps

1. Add `chat_from` with a doc comment citing `jig/docs/design/client/domain/client.md, 19`.
2. Write the four tests; run them alone, then the crate's lib tests
   (`heavy cargo nextest run -p jig-client --lib`).
3. Edit the status line as above.
4. `cargo fmt`; `heavy cargo clippy -p jig-client --all-targets -- -D warnings`;
   `heavy cargo check -p temper-client-tools --tests`; the citations check.
5. Commit (why: an application starts a chat from any of its chat templates); push your
   branch.

## Report

As the implementer's text says, and also: whether `chat` delegates to `chat_from` or both
share a helper, and the tests' count.
