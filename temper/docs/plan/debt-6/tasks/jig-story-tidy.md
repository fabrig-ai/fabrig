+++
id = "jig-story-tidy"
part = "jig-review"
kind = "feature"
needs = ["jig-story-fixtures", "jig-fake-tidy"]
files = ["jig/tests/client/src/stories/shared/b_config.rs", "jig/tests/client/src/stories/shared/b_inbox.rs", "jig/tests/client/src/stories/shared/b_conversations.rs"]
effort = "medium"
minutes = 35
+++

# jig: `b_config::setup` and the chat stories, and the part's lines

## Goal

The review's Part B, findings 5 and 8, the part's last. `b_config::setup` gained an
`#[expect(clippy::too_many_lines)]`: its scripts and the `InboxOrder` overrides come out, and `CHAT_SIGN_INS`
is the one value for the literal beside it. Three tree reads in `b_conversations.rs` become
`chat_order`, and `a_person_s_chats_are_their_own` asserts its result once.

Acceptance:

- `rg -n 'too_many_lines' jig/tests/client/src/stories/shared/b_config.rs` reads nothing. If the lint still
  fires after `jig-delegate-story`'s arms, keep the expect and say why.
- `rg -n 'proposal_script|result_scripts' jig/tests/client/src` reads nothing.
- `rg -c 'key.kind == 20' jig/tests/client/src/stories/shared/b_conversations.rs` reads 1.
- `a_person_s_chats_are_their_own` has one assertion of the list's contents.
- The part's net line change is negative:
  `git diff --shortstat 361eefcd8c HEAD -- jig/tests/client jig/testing/jig-client-scripted-engine jig/docs/design/client/domain/testing.status.md`.
  If it is not, report the number and what is left; do not cut anything else to reach it.

## Inputs

- `temper/docs/plan/debt-6/notes/common.md`, `AGENTS.md`, `jig/AGENTS.md`.
- `jig/docs/design/client/domain/testing.md`, 4.

## Owns and may change

- Owns: `b_config.rs`, `b_inbox.rs`'s script functions, `b_conversations.rs`.
- Not this task's: `b_goals.rs`, where `goal_order` stays. `chat_order` goes in `b_conversations.rs`.

## Steps

1. `b_config.rs` and `b_inbox.rs`:

   ```rust
   /// The worker's scripts for each story's conversations.
   fn scripts(facts: &Facts) -> Vec<Box<[Script]>>;
   // b_inbox::scripts() -> Vec<Box<[Script]>> replaces proposal_script and result_scripts.
   ```

   The two proposers' scripts and the two results come back in today's order. The `limits.core.tasks`
   overrides for `InboxOrder` move into that case, past the shared arm that sets them; the last write
   stays the one that counts. The shared arm's `sign_ins = 4` is `CHAT_SIGN_INS`.
2. `b_conversations.rs`:

   ```rust
   fn chat_order<F: Front<Test>>(world: &World<Test, F>, client: ClientId) -> Vec<u64>;
   ```

   In `a_person_s_chats_are_their_own`, `Step::Gone` goes and `assert_eq!(chat_order(..), vec![CHAT])`
   stays: it reads the whole list, where `Gone` reads one title.
3. Hand-break: in `jig-client-scripted-engine/src/queries.rs`, make `selects` ignore `query.requester`;
   `scripted::a_person_s_chats_are_their_own` must go red. If it stays green because of `watches.rs`'s own
   check, break that one too, and name it.
4. `cargo fmt`; `heavy cargo clippy -p jig-client-world --all-targets -- -D warnings`;
   `heavy cargo nextest run -p jig-client-world`; the citations check. One commit for each finding. Push.

## Report

The common note's table. The part's net lines, from the acceptance's command.
