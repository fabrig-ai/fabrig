+++
id = "box-slots"
part = "box-slots"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-init/template/temper.json",
  "temper/crates/temper-init/template/bare.json",
  "temper/docs/examples/owned.json",
  "temper/crates/temper-init/src/tests/documents.rs",
  "temper/crates/temper/tests/suite/configuration.rs",
  "temper/crates/temper/src/tests.rs",
  "temper/crates/temper/tests/process.rs",
  "temper/docs/help/deployment.md",
  "temper/docs/help/reference/configuration-engine.md",
  "temper/docs/design/protocol/configuration.status.md",
  "temper/docs/design/protocol/setup.status.md",
]
effort = "medium"
minutes = 40
+++

# A box defaults to 4 slots

## Goal

With a configuring chat writing, a person started a second chat on their box and saw it wait, because `temper init` writes one slot. A box now runs 4 agents at once, so two chats and a goal's agents run side by side. Limits for a coding run don't change: only how many run at once.

Acceptance:

- `temper init`'s two documents and the owned example write `workers[0].slots` 4.
- They write no `hub.slots` or `fleet.slots` override, in any `overrides` block. jig's `standard` holds both at 4, and a document's overrides say only what differs from the profile (`temper/docs/design/protocol/configuration.md`, 13.4).
- They write `checkout.workspaces` 4 and `git_component.operations` 4 in `application.limits.overrides`. temper's `standard` leaves both at 3.
- Every other override stays as it is, `fleet.workers` and `fleet.configured_workers` among them.
- A start takes all three documents, on the focused tests that already start them, in `temper/crates/temper/tests/suite/startup.rs`: `serve_prepares_once_on_the_owned_example`, `serve_prepares_once_on_a_bare_init` and `a_box_on_a_fresh_init_reaches_serves_link_with_forge_3_dormant`.
- `temper help deployment` says 4 slots, and `docs/help/reference/configuration-engine.md` is regenerated.
- A focused test holds these values for all three documents.

## Inputs

- `temper/docs/design/domain/worker.md`, 6, the bullet `What rises with its slots`.
- `temper/docs/design/protocol/configuration.md`, 13.4, the sentence beginning `The document's overrides say only what differs`.
- `temper/docs/design/protocol/setup.md`, 8.1, the bullets `Its engine.limits`, `Its worker's limits` and `Its application.limits`.
- `temper/docs/plan/ui-box-slots/notes/common.md`.

## Owns and may change

Owns the files in the header, for these edits and no others.

**This changes what a shared document means, so its readers' tests run:** the focused tests of `temper-init`, `temper`, `temper-service` and `temper-fake`, which read these documents (step 5).

**Readers, surveyed on the trunk:**

- `temper/crates/temper/src/tests.rs`, `fixed_sign_in_ports_are_checked_against_owned_and_fake_listeners` (near line 205), and `temper/crates/temper/tests/process.rs` (near line 1931) set `checkout.workspaces` and `git_component.operations` to 1 on these documents. A start would now refuse the slots past them, before what each test reaches for. Each also sets `workers[0].slots` to 1, the value it ran at.
- `temper/crates/temper/tests/suite/configuration.rs`, `assert_owned_limits` (near line 103), pins the engine's overrides: `"fleet.slots":2` and `"hub.slots":2` leave it.
- `temper/crates/temper/tests/suite/startup.rs`, `eight_slots` and `serve_names_what_to_raise_for_eight_slots`, set their own 8 and 10, and leave out one at a time. Each left-out limit is now 4 or absent, still short of 8, so they need no edit.
- `temper/crates/temper-service/src/tests/fixtures/startup.rs`, `owned_worker_alone_document`, and `temper/crates/temper/tests/process.rs` near line 922 copy `workers` and `application.limits` together, so they stay consistent.
- `temper/crates/temper/tests/support/configuration.rs` takes its workers from jig's kit, not these documents. The cluster's and GitHub's fixtures, `temper/tests/real-loop/src/landing.rs` and `temper-playground` set their own slots.
- `docs/help/server.md`'s `To run more at once` and `reference/configuration.md`'s `More slots` stay true, and `reference/limits*.md` print the profiles, which don't change. Leave them.

**No process, browser or live story was found that reads one slot by word or value** beyond `process.rs` above. A story that relied on one run at a time without saying so is after-landing's to flag: don't look for it by running one.

**Beside this plan:** `model-catalogue-2/stock-models` is on all three documents and `temper-init`'s tests, and `model-catalogue-2/process-fixtures` on `temper/crates/temper/tests/support/**` and `fixtures/**`. `model-catalogue-2/models-help` is on `docs/help/deployment.md`. Whichever lands second catches up, so keep each hunk to its lines.

**Departs from the implementer's text:** `temper/docs/help/deployment.md` is open for the one line in step 3, and the two status files for the bullets in step 4.

Options, each pre-approved:

- If another fixture lowers one of these limits on these documents, or puts a `small` profile under them, and a start refuses its slots: set that fixture's `workers[0].slots` to 1, the value it ran at, and name it in your report. This includes a process-tier fixture you find by reading: compile it, don't run it.
- If a focused test goes red because two runs now really run at once, and the behaviour is right: repair the test's oracle, keeping what it checks, and name it. If the behaviour looks wrong, report it under `## Left` with the test's log.
- If 4 fails a start's check on any document for a reason this spec doesn't name, that's the request's "pick the nearest sound number": take the highest value that passes on all three, with `checkout.workspaces` and `git_component.operations` at it. State the check and the number on your report's first line.

## Steps

1. **The documents.** In `temper.json`, `bare.json` and `owned.json`:
   - `workers[0].slots` becomes 4.
   - Delete `"fleet.slots": 2,` from `engine.limits.overrides`, and `"hub.slots": 2,` from both `engine.limits.overrides` and `workers[0].limits.overrides`.
   - In `application.limits.overrides`, add `"checkout.workspaces": 4,` after `checkout.message_bytes`, and `"git_component.operations": 4,` after `forge_component.body_bytes`, keeping the block's order.
2. **The readers.** Make the three test edits above: `tests.rs`, `process.rs` (compile it with `heavy cargo check -p temper --tests`; don't run it) and `suite/configuration.rs`.
3. **Help.** In `temper/docs/help/deployment.md`, `engine's process, with one slot.` becomes `engine's process, with 4 slots.` Regenerate `docs/help/reference/configuration-engine.md` with `SKEIN_CODEGEN_WRITE=1` and the test `the_pages_configuration_are_what_the_tables_render`, in `temper/crates/temper/src/tests/configuration.rs`. Its diff is the two `init writes` lines for the overrides and `slots`, and whichever `application` line prints its overrides. Anything else in that diff is a red, not stale.
4. **Status.** Delete the bullet beginning `**Not yet:** 13.4: `temper init`'s documents` in `temper/docs/design/protocol/configuration.status.md`, and the one beginning `**Not yet:** 8.1, 8.3: `temper init`'s documents` in `temper/docs/design/protocol/setup.status.md`.
5. **The test: one existing table, no new test.** In `temper/crates/temper-init/src/tests/documents.rs`, `the_box_keeps_eight_streams_and_four_calls_beside_them` walks `[fixtures::OWNED, template::TEMPER]`. Add `template::BARE`, rename it `the_box_runs_four_agents_and_keeps_eight_streams_and_four_calls_beside_them`, and for each document assert:
   - `workers[0].slots` is 4;
   - `checkout.workspaces` and `git_component.operations` in `application.limits.overrides` are 4;
   - `fleet.slots` is absent from the engine's overrides, and `hub.slots` from the engine's and the worker's.

   Break it once by setting a document's `slots` back to 1, and once by restoring a `hub.slots` override, and see each fail.
6. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-init -p temper --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-init -p temper -p temper-service -p temper-fake`, the focused profile, which leaves out the tiers' binaries. Commit and push.

## Report

As the implementer's text has it. Say:

- that the start took all three documents, and by which tests;
- that `process.rs` was compiled and not run;
- any fixture you set back to one slot, and any test oracle you repaired.
