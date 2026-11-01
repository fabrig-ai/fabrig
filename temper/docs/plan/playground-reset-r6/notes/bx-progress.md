# bx: the box, the binary, init and the process tier (progress)

Designer bx of playground-reset-r6. Crates: temper-box, temper-forge-setup,
temper-forge-setup-shell, temper-fake, temper-init, temper (the binary, its src and tests).

## Specs

| id | status | needs | crates (files) |
|---|---|---|---|
| bx-pipe | final | - | temper-box |
| bx-steps | final | - | temper-forge-setup (setup.rs, step.rs), temper-fake (steps.rs) |
| bx-compose | final | bx-pipe, bx-steps | temper (box_command, forge.rs, fake.rs, suite) |
| bx-doc | final | bx-compose, sv-startup-machine | temper-fake (demo, steps), temper-forge-setup (check.rs), temper (setup.rs, box_command, suite) |
| bx-init | final | bx-doc | temper-init, temper (init, fake_backends), temper-fake (backends) |
| bx-harness | final | bx-init, cl-ctl | temper (tests/support: boxed, ctl, forges) |
| bx-fixtures | final | bx-harness, fe-make, sv-registry-setup, sv-family | temper (tests/support, process.rs, suite) |
| bx-walks | final | bx-fixtures | temper (tests/fake_backends) |
| bx-story-playground | waiting for ids | bx-walks, fe-playground-forge, fe-playground-root, fe-kept-root, sv-removal, sv-family, cl-ctl (+ jig's project deletion, pd) | temper (tests/reset) |
| bx-story-reset-cut | waiting for ids | bx-story-playground | temper (tests/reset) |
| bx-story-setup-cut | waiting for ids | bx-story-reset-cut, fe-remove, fe-make, sv-registry-setup | temper (tests/reset) |
| bx-story-github | waiting for ids | bx-story-setup-cut | temper (tests/fake_backends/walk) |
| bx-unforge | final | bx-story-github, sv-startup-refuse | temper, temper-fake, temper-forge-setup |

"Waiting for ids": the spec is written; it names fe/sv tasks by the ids in `tasks/` today,
none yet marked final in their progress files, and the reset needs jig's project deletion
(J3/J4), whose pd task id is not written yet. The lead adds that id to
`bx-story-playground`'s `needs` (the three later stories follow it) and releases them.

### What each gives

- `bx-pipe`: the box's load seam (`Service::take_load`/`loaded`, `take_removal`/`removed`,
  `Loaded::{Group, Absent, Refused}`), group slots (`box.groups`, 4), `remove N`/`removed N`,
  `failed N no settings`. The box reads no file of a group: its shell (the binary, in
  bx-compose) reads `box/groups/N`, builds the steps and removes the files. Recorded groups
  at a box start are loaded by the shell before construction and given in `Steps.asked`, so
  they join the start chain before serve as today.
- `bx-steps`: `temper_forge_setup::setup::{Entry::group, Entry::body, Entry::body_text,
  group_settings, directory, BODY_BYTES, GROUP_BYTES}`; `temper_forge_setup::step::owned_group`;
  `temper_fake::steps::{playground_group, group}`. For sv: `Entry::body`/`body_text` check
  `application.machine.owned`, `application.demo.forge` and a record's setup text;
  `group_settings(body, N)` is the text serve writes to `box/groups/N`. This answers sv's
  request 3 (`read_one` is `Entry::body_text`).
- `bx-compose`: the binary answers the seam (`box_command/groups.rs`, `Boxed` host), loads
  recorded groups from files at the box's start, and `temper forge ... N`/`temper fake forge N`
  read `box/groups/N` first, the document's setup entry as a fallback until bx-unforge.
- `bx-doc`: the binary and the fakes read T4's document beside today's (the demo's two
  shapes, no `application.setup`, `application.machine.owned`), and the box holds the
  machine's ports (owned, fake forge, fake LLM) from its start.
- `bx-init`: `temper init` writes T4's document; `temper box --fake-backends` imports and
  rewrites it (GitHub into `application.machine.github`; no managed forge 4).
- `bx-harness`: ctl stories kill and restart the whole box (`Step::KillBox`, `StartBox`);
  `support::forges` wraps `cl-ctl`'s `forge make`/`remove`, `playground start`/`reset`.
- `bx-fixtures`: the process tier's and suite's document builders name no forge; a story's
  forge is made (and, for a throwaway Forgejo, connected as admin) through ctl before its
  steps.
- `bx-walks`: the fake backends' walks make GitHub, the owned and the fake Forgejo at runtime.
- The four stories are draft section 8's process tier, each with its process-profile command.
- `bx-unforge`: after the refusal, the binary's and fakes' fallbacks to the document's
  forges and today's demo shape go.

## Coordination

- **sv-startup-refuse needs bx-init, bx-fixtures and bx-walks:** they move every reader of
  the document's forges in bx's crates and tests (but the suite tests that test the old
  members' own checks, which stay while they are read: `sv-startup-refuse` turns those in
  `temper/crates/temper/tests/suite/{startup,forge,fake}.rs` into refusal tests or deletes
  them, as its own compiler- and test-forced edits). bx-fixtures and bx-walks list under
  `## Left` what they could not move.
- **Crate `temper` has one writer at a time.** bx's chain on it is bx-compose, bx-doc,
  bx-init, bx-harness, bx-fixtures, bx-walks, the four stories, bx-unforge. None of them
  edits `temper/crates/temper/src/host.rs` or `src/lib.rs` (bx-compose and bx-doc keep off
  them), which `sv-resolve` and `sv-removal` edit: the lead orders those two against bx's
  chain, or lets them run beside it since the files are disjoint.

## Critical path

bx-pipe (120) → bx-compose (110) → bx-doc (100, and sv-startup-machine) → bx-init (120) →
bx-harness (90, and cl-ctl) → bx-fixtures (120, and fe-make, sv-registry-setup,
sv-family) → bx-walks (90) → bx-story-playground (75, and the playground's engine tasks) →
bx-story-reset-cut (60) → bx-story-setup-cut (110) → bx-story-github (75) → bx-unforge (75,
and sv-startup-refuse). About 19 hours of implementer time in series, serialised by crate
`temper`; bx-steps (75) runs beside bx-pipe. The stories are the longest stretch: request 3
would let them run at once.

## Contract requests

1. **A failed removal on the pipe (T3).** T3 gives `remove N` only its success,
   `removed N`. bx-pipe answers a removal the shell could not finish `failed N not removed`
   (the group stays, dormant; a later `remove N` tries again), and a `remove N` while another
   group starts or stops `failed N busy`, as `stop N` is answered. **Request:** sv's box link
   (`sv-removal`) takes a `failed N ...` while its `remove N` is in flight as "not removed":
   the kept act stays and is sent again (at the next start at the latest). No change to
   T3's lines.
2. **The fakes' git program (T1, T4).** Under `temper box --fake-backends` every forge but the
   owned one is a fake whose git goes through temper's fake git program
   (`<data>/demo/bin/git`): today the rewrite sets `git_program` on forge 2 (GitHub) and
   forge 4 (the managed fake Forgejo). T4's `application.machine.github` has no
   `git_program`, and a Forgejo elsewhere is made from the request's addresses alone, so a
   record made at runtime has `git_program: None` and the walks cannot push.
   **Recommendation:** `application.machine.git_program`, `null` or a data path, `null` in
   `temper init`'s document: the connector copies it into every GitHub and Forgejo-elsewhere
   record it makes (the playground's fake keeps its setup entry's `git`). The fake backends'
   rewrite sets it. Parsed by `sv-startup-machine`, copied by `fe-make`. Until answered,
   bx-init sets nothing and bx-walks marks the pushing walks ignored, naming this request.
3. **The four story tasks write only new files** (`temper/crates/temper/tests/reset/*.rs`,
   one `mod` line each in `tests/reset/mod.rs`, and `bx-story-github` its own walk in
   `tests/fake_backends/walk/github.rs`). bx orders them in series by the one-writer rule.
   **Request:** the lead may let them run at once (after bx-story-playground, which makes
   `tests/reset/mod.rs`), which takes about four hours off the path.
