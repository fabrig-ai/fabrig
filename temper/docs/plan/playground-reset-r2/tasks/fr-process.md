+++
id = "fr-process"
part = "forge-records"
kind = "migration"
needs = ["fr-qa", "fr-fake-backends", "bx-serve"]
files = [
  "temper/crates/temper/tests/process.rs",
  "temper/crates/temper/tests/fake_backends/**",
  "temper/crates/temper/tests/support/**",
]
effort = "high"
minutes = 90
+++

## Goal

The process tier (`process`, the `process-temper` profile) runs on
forges as records (`temper/docs/design/draft/playground-reset.md, 3`,
and 8's process tier for this part): a first run's `temper init` names
no forge, the welcome's SetUp makes the owned Forgejo's record and
serve writes its group file (bx-serve), a restart builds it from its
record, and `temper box --fake-backends` starts empty, its GitHub
choice making a record that talks to the fake GitHub
(`notes/answers.md`, "A fresh box is empty"). The playground stays
today's demo (D4's `pg-process` changes it). fr-'s last task;
`bx-process` follows it.

Acceptance (`heavy cargo nextest run -p temper --test process
--profile process-temper -E 'test(=NAME)'`, one at a time, then the
whole target once):

- New: `a_first_run_names_no_forge_and_sets_up_owned_forgejo_as_forge_two`
  (init, serve under the box, ctl SetUp: the forge is numbered after
  the playground's seed, its group file is written, it connects and
  its first repository is adopted); `a_restart_before_the_first_repository_makes_it`
  (the box killed after reconcile, before Create: the next start makes
  the first repository with its names, `setup.md` 6.7 now kept).
- Today's cases that assumed configured forges (`process.rs:1783`
  playground then owned set up, `:2100`, `:2192`, `:2224`, `:4800`, and
  those `rg -n 'forge 3|forges\[|setup\[' tests` names) pass on records:
  the owned Forgejo's number is the one its SetUp answered.
- `tests/fake_backends/` (`mod.rs:114`, `walk.rs:128,145,150,155`):
  a fresh sandbox lists no forge, the welcome offers the playground,
  and the GitHub choice makes a GitHub record against the fake; a case
  that needs the managed Forgejo writes its seed
  (`temper_fake::backends::managed_seed`).
- The tier's wall within its budget (report the time).

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md`,
  `temper/docs/plan/playground-reset-r2/notes/scope.md` and
  `temper/docs/plan/playground-reset-r2/notes/answers.md`.
- `temper/docs/design/draft/playground-reset.md`, sections 3, 6.3 and
  8.
- `temper/docs/design/protocol/setup.md`, 6.6 and 6.7, 7.1.
- `skein/docs/foundation/testing-strategy.md`, 8.1 (budgets).
- `tasks/fr-service.md`, `tasks/fr-setup.md`, `tasks/fr-fake-backends.md`,
  `tasks/bx-serve.md`.

## Owns and may change

Owns `tests/process.rs`, `tests/fake_backends/**` and
`tests/support/**` (after `fr-qa`, whose `configuration.rs` and
`mod.rs` it may now change too), for this task; `bx-process`, then
D4's `rm-process` and `pg-process` own them next. Production code is
not this task's: a failure that needs one is a question, unless it is a
call site `notes/scope.md` allows.

## Steps

1. Support (`boxed.rs`, `owned.rs`, `forge_land.rs`,
   `first_repository.rs:116,220`, `github_land.rs:805`, `playground.rs`,
   the others that read the init template's forges): read the forge's
   number from the SetUp's or Connect's answer, or from the forges'
   rows, never from the template.
2. The new cases (Goal), on the shipped binaries, the box killed with
   the harness's own kill, never by a sleep.
3. Run each touched case alone, then the target once; read each
   failure's log before any rerun; one rerun only after exit 137.
   `cargo fmt`; `heavy cargo clippy -p temper --all-targets -- -D
   warnings`; old forms.
4. Commit and push.

## Report

The standard report, plus: each case whose assertions changed and why;
the tier's wall time and CPU against its budget; anything `bx-process`
must know.
