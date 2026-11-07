+++
id = "box-words"
part = "box-words"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-view/src/choice.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/forge/connect.rs",
  "temper/crates/temper-client-view/src/tests/chat.rs",
  "temper/crates/temper-engine-smith/src/lib.rs",
  "temper/crates/temper-engine-smith/src/settings.rs",
  "temper/crates/temper-engine-smith/src/calls.rs",
  "temper/crates/temper-engine-smith/src/answers.rs",
  "temper/crates/temper-engine-smith/src/tests/settings.rs",
  "temper/crates/temper-service/src/tools.rs",
  "temper/crates/temper-service/src/tests/tools.rs",
  "temper/docs/design/client/README.status.md",
  "temper/docs/design/domain/agent.status.md",
]
effort = "medium"
minutes = 40
+++

# A box's wait, and a failed settings read, say what to do

## Goal

Two sets of words a person read on their own box. A run waiting for a slot on a box says "none is connected", which isn't true there, and "host", which they don't know. And a configuring agent's `settings` read without its `number` shows "failed: host call member missing: number" in the chat, where the agent could have been told what to do.

Acceptance:

- With the deployment's read saying `owned`, `TaskRunWait::Host` draws "Waiting for other work to finish. It starts by itself." on both surfaces in `choice.rs` (`waiting` and `wait_why`).
- With `owned` false, or the read not yet answered, it draws today's "No host has room for this, or none is connected. It starts when one does."
- No wire field changes.
- `settings` with `{"setting":{"kind":"landing-rule"}}` answers, as an error, exactly `landing-rule needs number; read the inventory and name what it lists`. The same holds for `charter`, `template`, `policy` and `forge-charter`.
- `settings` with `{"setting":{"kind":"project-ways"}}` answers, as an error, exactly `project-ways needs project; read the inventory and name what it lists`. The same holds for `project-policy` and `project-questions`.
- A kind no setting has, with no `number`, is `Problem::Range(b"kind")`, today's answer for one with a `number`.
- `number`'s description in both the `settings` and `propose_settings` schemas is exactly `Required for charter, template, policy, landing-rule and forge-charter; the other kinds take none.`
- Failed calls keep showing: no view of a failed call changes.

## Inputs

- `temper/docs/design/client/README.md`, 4.6 (*The deployment's read*), and 5.5, the bullet `*A host:*`.
- `temper/docs/design/domain/agent.md`, 8.1, the bullet `The kinds it reads and proposes`.
- `temper/docs/plan/ui-box-slots/notes/common.md`.

## Owns and may change

Owns the files in the header, for these edits and no others. The two families in it don't touch each other; do them in either order.

**Readers, surveyed on the trunk:**

- `Problem` is matched exhaustively only in `temper-engine-smith/src/answers.rs` (the `Err(...)` arm near line 433: add the new variant there, nothing else) and `temper-service/src/tools.rs`, `invalid`.
- No story or fixture reads `host call member missing: number` or `: project`. `temper/tests/system/src/composed/review.rs` builds `host call out of range`, which doesn't change.
- `Word::WaitCapacity`'s words appear in `temper-client-view/src/tests/chat.rs`, and in no gallery print, journey or browser story. Its word stays for `temper serve`.
- `temper-client-tools/src/jig/render/document/llm.rs` prints `host` for this wait. That's the terminal's own word: leave it.

**Beside this plan:** `model-catalogue-2/configuring-learns` is on `settings.rs` and `calls.rs`, `ui-proposal-2/chat-revises` on `answers.rs`, `limits-fixtures/view-words` on `words.rs`. Keep hunks small and don't reflow what's around them.

**Departs from the implementer's text:** `temper/docs/design/client/README.status.md` and `temper/docs/design/domain/agent.status.md` are open for the one bullet each in step 6.

Options, each pre-approved:

- `owned` in `forge/connect.rs` is private. Default: make it `pub(crate)` and call it from `choice.rs`. If that reaches awkwardly across modules, read `root.forge().deployment()` in a small function in `choice.rs` and say so.
- The `propose_settings` form reader (`temper-settings-form/src/file.rs`) answers its own words for a setting short of a member. Leave it, and name what it prints in your report. `configuring-learns` is on that crate.
- If `reference/host-tools.md` or another generated page goes stale, regenerate it with its own tool and say so.

## Steps

1. **The word.** In `temper-client-view/src/words.rs`, beside `WaitCapacity` in each place it's listed (the enum, the list near line 2311, the code near line 3604, the text near line 5133), add `WaitCapacityBox`, with the next free code and the text `Waiting for other work to finish. It starts by itself.`
2. **The view.** In `choice.rs`, one small function chooses between them, used by both `waiting` (the `Some(c::TaskRunWait::Host)` arm) and `wait_why` (the `c::TaskRunWait::Host` arm):

   ```rust
   /// A wait for a host, in a box's words on a box (temper/docs/design/client/README.md, 5.5).
   fn host_wait(root: &Domain) -> Word
   ```

3. **The answer.** In `temper-engine-smith/src/lib.rs`, add to `Problem`:

   ```rust
   /// A setting's kind that needs a member its read left out (domain/agent.md, 8.1).
   Needs { kind: &'static [u8], member: &'static [u8] },
   ```

   In `settings.rs`, `read` finds the kind's static name in two lists, numbered and by project, and returns `Problem::Needs { kind, member: b"number" }` or `Problem::Needs { kind, member: b"project" }` where it returns `Missing` today. Keep every other refusal's order: `Range(b"project")`, `Range(b"number")`. A kind in neither list and not `rules` falls through to `read_setting`'s `Range(b"kind")`, with no number asked of it. Step code: no closures where the file uses loops.
4. **Its words.** In `temper-service/src/tools.rs`, `invalid` renders `Problem::Needs { kind, member }` as `<kind> needs <member>; read the inventory and name what it lists`, with `error: true`, measured as the other arms are. It says what to do beside the `full commit identifier` arm's precedent.
5. **The schemas.** In `calls.rs`, replace `Only numbered kinds take number.` with the sentence in the acceptance, in both specs. It fits `MEMBER_BYTES` (120).
6. **Status.** Delete the bullet beginning `**Not yet:** 5.5, *A host*` in `temper/docs/design/client/README.status.md`, and the one beginning `**Not yet:** 8.1: a `settings` read short of` in `temper/docs/design/domain/agent.status.md`.
7. **Tests: rows of existing tables, no new test.**
   - `temper-client-view/src/tests/chat.rs`, `every_chats_run_wait_says_why_above_the_composer_without_working_dots`: **the fixtures' default deployment already answers `owned: true`** (`tests/fixtures.rs` near line 660), so today's `Host` row flips to `Waiting for other work to finish. It starts by itself.` Add the serve row beside it: the table gains a third column, `owned`, `true` on every row but `(c::TaskRunWait::Host, false, b"No host has room for this, or none is connected. It starts when one does.")`. To draw that row, split `deployment_root` in `tests/connect_forge.rs` (near line 102) so that its re-read after a gap takes `(&mut root, me, owned)` and is `pub(super)`, and give `configuring_task_root` in `chat.rs` a sibling taking `owned`. `tests/fixtures.rs` stays as it is: `limits-fixtures/view-words` is on it. The test keeps asserting on both `page_view` and `browser_page_view`.
   - `temper-engine-smith/src/tests/settings.rs`, `settings_bad_fields_are_refused_by_name`: the `{"kind":"charter"}` and `{"kind":"project-policy"}` rows become `Problem::Needs`, with rows for `landing-rule`, `forge-charter` and `project-ways`, and a row for a kind no setting has. The same test gains a two-line loop over `settings` and `propose_settings` in `SPECS`, finding the exact `number` sentence in each schema. In `project_settings_reads_require_project_and_refuse_number` (near line 690), `("", Problem::Missing(b"project"))` becomes `("", Problem::Needs { kind: kind.as_bytes(), member: b"project" })`.
   - `temper-service/src/tests/tools.rs`, `a_flat_forge_read_error_names_its_missing_member`: rows for `settings` with `{"setting":{"kind":"landing-rule"}}` and `{"setting":{"kind":"project-ways"}}`, each with its whole answer.
   - Break each rule once, as the implementer's text says: the box word's branch, one `Needs` return, the rendered sentence.
8. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-view -p temper-engine-smith -p temper-service --all-targets -- -D warnings`; the focused tests of those three and of their direct dependents through `heavy cargo nextest run -p ...`. Commit and push.

## Report

As the implementer's text has it. Say what `propose_settings` answers for a setting short of a member, and which rules you broke and what failed.
