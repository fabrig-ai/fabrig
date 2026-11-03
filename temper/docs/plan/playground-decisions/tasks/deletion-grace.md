+++
id = "deletion-grace"
part = "deletion-grace"
kind = "feature"
needs = ["remove-project-grace"]
files = [
  "jig/crates/jig-domain-core/src/admission/deletion.rs",
  "jig/crates/jig-domain-core/src/routing/party/people.rs",
  "jig/crates/jig-domain-core/src/routing/party/deployment_acts.rs",
  "jig/crates/jig-domain-core/src/tests/admission/deletion.rs",
  "jig/tests/core/tests/suite/deletion.rs",
  "jig/docs/design/domain/people.status.md",
  "jig/docs/design/STATUS.md",
]
effort = "high"
minutes = 120
+++

# A deleted project's conversations are closed with the deletion's grace

## Goal

The user decided (`inputs/request.md`, decision 3) that a deleted project's conversations are
cancelled with their close's grace, as a person's close cancels one. `remove-project-grace` made
`RemoveProject` (the party's ask and the deployment's act) and the kept deletion carry a grace;
the core still ignores it. This task gives it its meaning in jig-domain-core: the grace is
checked at admission, kept with the deletion, and each live conversation of the project is
closed as the deployment's `Close` closes one.

Acceptance:

- A `RemoveProject`, a party's or the deployment's act, whose grace is past the tasks child's
  bound (`env.limits.tasks.grace`) is refused `Refusal::Limit { which: LimitKind::Grace }`
  before anything of the deletion is made: the project stays open, no deletion is kept.
- The deletion keeps its grace (`Deleting.grace`, saved with each step through
  `people::Event::Deleting`, restored from `people.deletions()`).
- Its cancel step composes, within `env.limits.people.operations` per compound: for each live
  root flagged conversation, each of its live delegates cancelled with no grace, then the
  conversation cancelled with the deletion's grace and a fresh message number, all in one
  compound, never split; every other live root with no grace; no reason; a root closing, or
  whose row holds a grace (`row.grace.is_some()`), is not asked again. A grace kept past the
  bound as a later start configures it is given as that bound.
- With no grace, the deletion behaves exactly as today.
- The tests below pass; the status entries below are gone.

## Inputs

- `temper/docs/plan/playground-decisions/notes/common.md` and `notes/contracts.md`, C2.
- `jig/docs/design/domain/people.md`, 5.2: the paragraph on the deletion's grace, "Its tasks
  ended, then erased" and "Kept until whole"; 5.3: the `RemoveProject` act row and the refusals
  below the table; 7: "Closed by the deployment" and "Closed with its project"; 11: the
  deletion's story.
- `jig/docs/design/domain/tasks.md`, section 7: "A project deleted", "Cancelling with a grace"
  and "Several operations, one decision" (read only).

## Owns and may change

- `jig-domain-core/src/admission/deletion.rs`:
  - `begin(core, env, work, out, request, project, grace: Option<skein_lib::Duration>)`: refuses
    unknown as today, then the grace past `env.limits.tasks.grace` as above, then keeps the
    deletion with its grace.
  - `Deleting` gains `grace: Option<skein_lib::Duration>`; `keep` sends it in
    `people::Event::Deleting`; `restored` and `restore_project(core, project, step, grace)` take
    it from `KeptDeletion`.
  - `cancel` composes as the acceptance says. A conversation's group is
    `row.delegates.len() + 1` operations; a group that does not fit what is left of this
    compound waits for the next (`advance` sends it). Take the conversation's message number
    with `crate::fresh(&mut core.counters, crate::Family::Message)`, as
    `routing/party/deployment_acts.rs` `close` does; if none is given, cancel it with no grace.
    You may share the delegates-then-conversation composition with `close` through one private
    helper; `close`'s behaviour stays.
  - Defaults, pre-approved: a conversation whose group exceeds `operations` even in an empty
    compound (possible only below configuration's floor, as in a unit test) is cancelled alone
    with the grace, its delegates ending with its tree; a root requested by anyone, if flagged
    conversation, takes the grace.
- `routing/party/people.rs:275` and `routing/party/deployment_acts.rs:57` pass the request's
  grace to `begin`; the pattern at `people.rs:261` stays as `remove-project-grace` left it.
- Tests, `jig-domain-core/src/tests/admission/deletion.rs` (the file's `begin` helper and the
  `restore_project` calls take a grace, `None` where the story is unchanged). New, in its
  fixtures (`make_tree`'s root flagged conversation for a chat with three live delegates):

  ```rust
  #[test]
  fn a_grace_past_the_bound_is_refused_and_nothing_is_deleted() {
      // the act and an owner's ask, grace = limits.tasks.grace + 1 ns: Refused(Limit Grace),
      // core.open_project(1) still, no people::Event::Deleting sent
  }
  #[test]
  fn a_deleted_projects_conversation_is_asked_to_finish_within_the_grace() {
      // begin with Some(grace): delegates 2, 3, 4 closing; the chat (1) not closing, its row's
      // grace set; a plain root of the project closing with no grace
  }
  #[test]
  fn a_root_finishing_under_a_grace_is_not_asked_again() {
      // after begin, resume_fleet: no further compound, the message counter unchanged
  }
  #[test]
  fn a_kept_deletion_resumes_with_its_grace() {
      // restore_project(.., Tasks, Some(grace)); after Open the chat is asked to finish
  }
  #[test]
  fn a_kept_grace_past_a_lowered_bound_is_given_as_that_bound() {
      // restored grace above limits.tasks.grace: the chat's grace ends at the bound
  }
  ```

- The core's world, `jig/tests/core/tests/suite/deletion.rs`: a new story beside
  `a_deleted_project_leaves_only_its_spend`, on `deleted_world` (its live chat and delegates):

  ```rust
  #[test]
  fn a_deleted_projects_chat_finishes_within_the_deletions_grace() {
      // remove with a grace (the act, its own key): answered removed; the chat's delegates
      // end cancelled at once; the chat is asked to finish (FinishRequested in its inbox);
      // the world's clock past the grace: the chat ends cancelled, then the deletion ends
      // whole as a_deleted_project_leaves_only_its_spend reads it (final_rows)
  }
  ```

  Keep `remove` ungraced for the existing stories; add a graced helper beside it. Do not add a
  graced cut sweep: the existing sweep keeps its budget.
- Status, as `notes/common.md` says (date line today):
  - `jig/docs/design/domain/people.status.md`, **Not yet**: remove the entry `5.2: a deleted
    project's conversations asked to finish within the grace its \`RemoveProject\` gives (5.1,
    5.3 and 7): …` whole, through its closing `(\`jig-domain-core/src/admission/deletion.rs\`).`
  - `jig/docs/design/STATUS.md`: remove `, nor are its conversations asked to finish within the
    grace its \`RemoveProject\` gives` from the summary's deletion sentence, and nothing else
    (rewrap the lines).

## Steps

1. `begin`'s check and the kept grace; the refusal and resume tests:
   `heavy cargo nextest run -p jig-domain-core --lib -E 'test(deletion)'`.
2. `cancel`'s composition and its tests, same command.
3. The world story: `heavy cargo nextest run -p jig-core-world --test suite -E 'test(deletion::)'`.
4. Status edits; the citations check.
5. `cargo fmt`; old forms for jig; `heavy cargo clippy -p jig-domain-core -p jig-core-world
   --all-targets -- -D warnings`. This changes what a deletion does, so run its readers' tests:
   `heavy cargo nextest run -p jig-domain-core -p jig-core-world -p jig-conformance-world -p
   jig-people-world`, then `heavy cargo nextest run -p temper-engine-domain-world -E
   'test(reset) or test(playground) or test(removal)'` (temper's reset deletes a project by this
   act). Commit the code and tests, then the status text; push.

## Report

As the implementer's text says; whether `close` and `cancel` now share a helper; the world
story's clock steps and its measured time.
