+++
id = "cl-play-domain"
part = "client"
kind = "feature"
needs = ["cl-surface"]
files = [
  "temper/crates/temper-client-domain/src/forge/demo.rs",
  "temper/crates/temper-client-domain/src/forge/vocabulary.rs",
  "temper/crates/temper-client-domain/src/domain.rs",
  "temper/crates/temper-client-domain/src/route.rs",
  "temper/crates/temper-client-domain/src/pages.rs",
  "temper/crates/temper-client-domain/src/pages/{connect_model,accounts,chats,chat,task,inbox,choice}.rs",
  "temper/crates/temper-client-domain/src/pages/accounts/**",
  "temper/crates/temper-client-domain/src/limits.rs",
  "temper/crates/temper-client-domain/src/tests/{demo,playground}.rs",
  "temper/crates/temper-client-domain/src/tests/pages/{connect_model,accounts}.rs",
  "temper/tests/client/tests/**",
]
effort = "high"
minutes = 85
+++

## Goal

The client's playground on the reset (draft 6.1-6.3): its read is
running, resetting or none; "set up temper" sends `EndPlayground` once
and the client reads *leaving* until the reset answers, re-reading, then
opens the welcome's Forge step, `/welcome?at=forge`; a reload during the
reset reads leaving and ends the same way. What goes on the client goes
from the domain (6.3): the retirement on another forge's connect, the
model's and forge's halves judged apart, the client's own removal of the
scripted account, the playground provider kept after its end, the
connect-model page's "stepped aside" latch. The view still compiles
through shims (below) that `cl-play-view` stops reading.

Acceptance (`temper-client-domain`, `src/tests/playground.rs`, beside
surface's tests):

- `the_end_is_sent_once_under_its_name_and_reads_leaving`
  (`playground()` is `Leaving` from the send; a second `EndPlayground`
  sends nothing).
- `leaving_reads_again_until_no_playground_then_opens_the_forge_step`
  (a resetting read, then another after `READ_AGAIN`, then none: one
  `Request::Show { address: Welcome { step: Forge { forge: None } },
  replace: true }`).
- `a_resetting_read_after_a_reload_reads_leaving_and_ends_the_same_way`.
- `a_refused_end_stops_leaving_and_reads_again`.
- `a_real_forge_connecting_neither_retires_the_playground_nor_removes_its_account`
  (no `AccountChange::Remove` is sent; `retired_demo()` is none).
- `the_playground_provider_is_none_once_the_playground_is_none`.
- `src/tests/demo.rs`: the start's tests pass as today; the tests of the
  retirement, the halves and the account removal are deleted (listed in
  the report).

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md` and
  `temper/docs/plan/playground-reset-r2/notes/scope.md`.
- `temper/docs/plan/playground-reset-r2/notes/contracts.md`, 5 (the read
  `Playground`, `EndPlayground`'s answers).
- `temper/docs/plan/playground-reset-r2/tasks/cl-wire.md` (the wire
  answer) and `cl-surface.md`, step 4 (the API this fills).
- `temper/docs/design/draft/playground-reset.md`, 6.1, 6.2, 6.3.
- `temper/docs/design/client/README.md`, 4.8 (the halves: what goes).

## Owns and may change

Owns the files in the header. Not yours now (others hold them):
`D/forge.rs`, `D/forge/{family,forges,connect,listed,remove,form}.rs`,
`D/pages/{welcome,project}.rs`, and every view file. So:

- The wire answer is decoded in `D/forge/demo.rs` itself (a private fn
  over `temper_client_protocol_wire::decode_playground_answer`), not by
  `family::demo`, which stays, unread, for `cl-tidy`.
- Shims kept for readers outside your files, each commented
  `// read by the view until cl-play-view`: `Domain::demo()` (running:
  `(demo, Halves { model: true, forge: true })`, else none),
  `forge::demo::{Halves, halves, current, other_model_usable,
  owner_deployment, deployment_model, usable_account}` (bodies as today
  where they read only the account documents), `Side::retired_demo()`
  (always none), `Side::demo_provider()` (the running playground's,
  else none), `forge::listed::setup` (not yours: it stays as it is).
  Delete an old item only when no file outside the header reads it.
- `temper/tests/client/tests/**`: a story of the retirement, the halves
  or the client's account removal (`suite/demo_start.rs`'s model-half
  and forge-end cases, `qa_all/qa_welcome_m4.rs`, `qa_first_run_hooks.rs`
  and their kin) is deleted; any other is changed only by forced edits.
- Forced edits (scope.md) anywhere else, the smallest.

## Steps

D = `temper/crates/temper-client-domain/src`.

1. **The read (D/forge/demo.rs).** The held answer is `Option<Demo>`
   (running) and `resetting: bool`; `DemoState` goes from the client's
   vocabulary (`D/forge/vocabulary.rs`'s `Demo` loses its `state`, or
   keeps `Running` only; follow the compiler). Read as today (link live,
   after the start's answer), and again while leaving.
2. **The end.** `end(side, limits, out)`: once, with a party and served
   not `OutOfDate`, while the playground runs or reads resetting: send
   `end_ask` as `Request::Send { request, name: Some(end_key()), ask }`,
   `end_key()` = `end-playground` under `crate::key::TOOLS` (ctl's
   `playground end` sends the same name: `cl-tools`). Its answer:
   `Resetting` → leaving, read again; `NotOffered` or `Unknown` → not
   leaving, read again. `pub const READ_AGAIN: Duration` = 1 s: while
   leaving, a read is sent each `READ_AGAIN` after the last answer
   (`pub fn next_deadline(side: &Side) -> Option<Time>` in demo.rs; D/
   domain.rs's `next_deadline` takes it into its minimum and `fire`'s
   `route::demo_progress` sends the read when due).
3. **`playground()`:** `Running(demo)` while the answer runs and no end
   is sent; `Leaving` while the end is sent and not refused, or the read
   says resetting; else none.
4. **Out of leaving (D/route.rs, `demo_progress`):** the first read
   that answers no playground after leaving pushes `Request::Show {
   address: Address::Welcome { step: Step::Forge { forge: None } },
   replace: true }` (draft 6.2), once.
5. **What goes (6.3):** the retirement (`RetiredDemo` kept,
   `observe_end`, `model_ended`, `forge_ended`), `remove` (the client's
   `AccountChange::Remove` of the scripted account; the reset removes
   it), `watch_deployment` unless a shim still needs it, `Pages`'
   `retired_project` and `demo_model` and the project reselection after
   a retirement (D/pages.rs), `ConnectModel.playground`'s latch
   (D/pages/connect_model.rs); the playground provider is excluded from
   the account ways only while the playground runs
   (D/pages/accounts/ways.rs).
6. **Checks:** the tests above; `cargo fmt`; `heavy cargo clippy -p
   temper-client-domain -p temper-client-view -p temper-client-tools -p
   temper-client-shell-web -p temper-client-shell-terminal -p
   temper-client-world -p temper-client-gallery --all-targets -- -D
   warnings`; one `heavy cargo nextest run` over the same packages and
   `temper-client-journeys`.
7. **Journeys:** `welcome-playground` sends the start as today; if the
   journeys test reports it stale, follow `cl-surface.md`'s step 13.
8. Old forms from `temper/`; commit and push.

## Report

- Each shim kept and who reads it (`cl-play-view` removes their readers,
  `cl-tidy` the shims).
- Each test deleted, by package and name.
- `READ_AGAIN` and whether a watch would serve better (for the design:
  the family has no playground watch).
