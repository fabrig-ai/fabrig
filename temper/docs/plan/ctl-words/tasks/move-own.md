+++
id = "move-own"
part = "move-own"
kind = "feature"
needs = []
files = [
  "jig/crates/jig-domain-core/src/admission/roles.rs",
  "jig/crates/jig-domain-core/src/tests/acts.rs",
  "jig/docs/design/domain/people.status.md",
]
effort = "medium"
minutes = 25
+++

# Moving your own task isn't offered

## Goal

A walk read `move` refusing `not_waiting` on a task waiting for an account, and took it for part of that wait. It isn't. `tasks::Domain::takeover_state` is false when the party is already the requester, and `person_task_refusal` turns that `State` into `NotWaiting`, whatever the task waits on. Say what's true: on a live task the party already requests, a move is not offered.

Acceptance:

- `Move` by a party of a live task whose requester they are, whether by its offer (`offers::admit`) or by `core.move_admit`: `Refused(people::Refusal::NotOffered)`.
- A move of a task another party requests is unchanged: `Allowed` where it was.
- A closing or ended task keeps the refusal that names its ending, the requester's own included.
- No wire or stored form changes: `NotOffered` exists on both.

## Inputs

- `jig/docs/design/domain/people.md`, 5.1, the `Move` row.
- `temper/docs/design/client/tools.md`, 6.1, the bullet beginning `A move`.
- `temper/docs/plan/ctl-words/notes/common.md`, but for its first sentence on walks: this cause is from reading, and `jig-domain-core/src/tests/acts.rs` already pins it.

## Owns and may change

Owns the files in the header. Only `admission/roles.rs`'s `move_admit` changes in production.

Readers, none needing an edit but the first:

- `jig-domain-core/src/tests/acts.rs`, about line 757, expects `NotWaiting` on the requester's own task, in both `offers::admit` and `move_admit`. It now expects `NotOffered`.
- `jig/tests/core/tests/stories/offers.rs`, line 159, offers a move on another's task: `Allowed`, unchanged.
- `jig-domain-core/src/routing/party/controls.rs`'s `move_request` calls `move_admit` and answers what it returns.
- `jig/tests/core/tests/suite/refusals.rs`, line 50, is an escalation's `NotWaiting`, not a move's. `routing/refusals.rs`'s mapping stays as it is, for the other acts that read it.
- `temper-client-tools`' `render/acts.rs` and `render/outcome.rs`, `jig-client`'s `words`, and `temper-client-view` all handle `NotOffered` already, and read it from the document.

**This changes a refusal's value,** so its readers' tests run: `jig-domain-core`, `jig-core-world`, and step 4's temper packages.

Departs from the implementer's text: in `jig/docs/design/domain/people.status.md` delete the bullet in step 3, nothing else.

Options, each pre-approved:

- `Staged`'s `requester` is public (`jig-domain-tasks/src/compound/staged.rs`). Default: read it in `move_admit`, leaving `takeover_state` as it is.
- Default shape, in `move_admit`'s `!context.takeover_state(person)` block: take `why` as it does now, and return `NotOffered` where `why` is `NotWaiting` and `context.requester == tasks::Party::Person(person)`. So every other state refusal keeps its words.

## Steps

1. **Reproduce.** Change the expectation at `tests/acts.rs`, about line 757, to `NotOffered`, in both asserts. See it fail with `NotWaiting`.
2. **The fix** in `move_admit`, as above. Add a story at that test's side: the same task closing or ended, its requester moving it, keeps the ending's refusal.
3. **Status.** In `jig/docs/design/domain/people.status.md`, delete the bullet beginning `**Not yet:** 5.1: a party's `Move` of a live task`.
4. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy -p jig-domain-core --all-targets -- -D warnings`; `heavy cargo nextest run -p jig-domain-core`; `heavy cargo nextest run -p jig-core-world`; `heavy cargo nextest run -p temper-client-tools --lib`; `heavy cargo nextest run -p temper-engine-domain-world`. Commit and push.

## Report

As the implementer's text has it. Name any test that read the old refusal and isn't listed here.
