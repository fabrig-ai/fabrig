+++
id = "cl-journeys"
part = "pg"
kind = "migration"
needs = ["cl-world-pg", "cl-ctl", "sv-family", "sv-playground", "sv-startup-machine", "bx-init", "bx-compose", "fe-playground-forge", "fe-playground-root"]
files = [
  "temper/crates/temper-client-protocol-wire/**",
  "temper/docs/design/client/journeys/welcome-playground/**",
  "temper/docs/design/client/journeys/welcome-set-up/**",
  "temper/tools/journeys/**",
]
effort = "high"
minutes = 90
+++

## Goal

The client wire moves to version 12 (`notes/contracts.md` T8: "in the client's last task
with the journeys recorded again"), `welcome-playground` is recorded again on a fresh
`temper box --fake-backends` of the run's code, and `welcome-set-up`, whose walk changes
with forges made at runtime, is deleted (contracts G), to be walked again by a person after
the run.

- `family temper-client-wire 12` and `CLIENT_VERSION = 12`; generated code and goldens
  match; the two hand-made goldens `golden/connect_forgejo_admin_{with,without}_first.bin`
  and `src/tests/forgejo_admin.rs`' `peer` head carry `0, 12`.
- `temper/docs/design/client/journeys/welcome-set-up/` is removed whole (recording, pages,
  README).
- `welcome-playground` is recorded again by `again welcome-playground` and its pages
  reprinted; its README still describes it (edit it only if its walk's words changed, and
  say so).
- `heavy cargo nextest run -p temper-client-journeys --lib` passes; so do the codecs
  world's client wire tests and every client package's tests.

## Inputs

- `temper/docs/plan/playground-reset-r6/notes/common.md`; `notes/contracts.md` T8 and G;
  `notes/cl-wire.md` 1 ("The version"); `notes/cl-progress.md`, requests 1, 5 and 6.
- `temper/docs/design/client/journeys.md`, 5.1 (the program), 8 and 8.1 (`again`: a fresh
  fake box, headless Chromium from `SKEIN_TEST_CHROMIUM`).
- `tasks/cl-wire.md` (the files a version move touches).

## Owns and may change

Owned: `temper/crates/temper-client-protocol-wire/`; the two journeys' directories as the
Goal says; `temper/tools/journeys/` only if its tests name `welcome-set-up` (then the
reference goes, nothing else). Every other reader of `CLIENT_VERSION` reads the constant
and needs no edit; a test that wrote the literal 11 for the client wire is edited to the
constant (listed).

Options, pre-approved:

- `SKEIN_TEST_CHROMIUM` unset or not a Chromium on this builder, or `again` failing before
  its walk starts (the bundle's build, the box's start): stop with a question naming the
  failure and its log; never hand-edit or synthesize a recording.
- `again` refusing the walk (`WalkDiffers`, a missing event): the playground's walk
  changed; stop with a question, its refusal quoted.

## Steps

1. Schema line, `CLIENT_VERSION`; `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p
   temper-codecs-world --test suite -E
   'test(=drift::client_wire_schema_matches_generated_code_and_goldens)'`, then without the
   variable; the hand-made goldens' second byte 11 → 12 (check them with
   `forgejo_admin_without_first_round_trips_its_golden` and `..._with_first_...`).
   `heavy cargo nextest run -p temper-client-protocol-wire --lib`, `-p temper-codecs-world
   --test suite -E 'test(/client_wire/)'`.
2. `git rm -r temper/docs/design/client/journeys/welcome-set-up`.
3. `heavy cargo run -p temper-client-journeys -- again welcome-playground` (it builds the
   recording bundle and `temper`, starts the fake box in a scratch directory and walks it
   in headless Chromium; `TEMPER_BUNDLE=none` as the builders set it). Then `heavy cargo
   run -p temper-client-journeys` (reprint) and `heavy cargo nextest run -p
   temper-client-journeys --lib`. Read the new `02.html`: the playground's chats page,
   with the one-line banner.
4. `heavy cargo nextest run -p temper-client-domain --lib`, `-p temper-client-view --lib`,
   `-p temper-client-tools --lib`, `-p temper-client-shell-web --lib`, `-p
   temper-client-shell-terminal --lib`, `-p temper-client-shell-cli --lib`, `-p
   temper-client-gallery --lib`, `-p temper-client-world --test suite`, `-p temper-service
   --lib -E 'test(/tests::family::/)'`. `cargo fmt`; clippy of the wire `--tests` and its
   wasm clippy; commit and push.

## Report

- The new recording's turn count and its pages' diff in short.
- Under `## Left`: `welcome-set-up` to walk anew by a person on a fresh `temper box
  --fake-backends` (`add welcome-set-up FILE`), its README's walk: the welcome, "Set up
  temper", "Run on this machine" on the Forgejo card, the Model step.
