+++
id = "bx-story-github"
part = "pg"
kind = "feature"
needs = ["bx-story-playground", "fe-make", "sv-registry-setup"]
files = ["temper/crates/temper/tests/fake_backends/walk/**", "temper/crates/temper/tests/fake_backends/walk.rs"]
effort = "high"
minutes = 75
+++

## Goal

Draft section 8's fourth process story: GitHub gone on after a restart, on `temper box
--fake-backends`' fake GitHub (draft 6.4: "GitHub goes on by default: its App's records are
kept as they come, so the step offers Go on, to the App's installation"). The App made by
the manifest flow is kept across the box's restart, and the set up goes on to the
installation and a first chat without making a second App.

Acceptance:
- In `tests/fake_backends/walk/github.rs`, `go_on_after_a_restart()`, called by
  `#[test] fn github_goes_on_after_a_box_restart_to_its_installation_and_a_first_chat()`
  in `walk.rs` beside the other walks: `forge make github` → N; the manifest flow's
  first hops (the App made and converted: `/forges/N/github/manifest`, the fake's
  `/settings/apps/new`, `/forges/N/github/redirect`), stopping before the installation;
  `KillBox`; `StartBox` on the same sandbox; `forge list --json` reads forge N kept,
  not connected, its App present (its installations empty or stray); `forge install N`,
  the visit to the installation and back to `/forges/N/github/setup`; then as the walk's
  `finish` (adopt `widget`, a key, a first chat), with no second App made (the fake
  GitHub's App count, or the manifest route not visited again).
- Run alone:
  `heavy cargo nextest run -p temper --profile process-temper --test process -E 'test(=fake_backends::walk::github_goes_on_after_a_box_restart_to_its_installation_and_a_first_chat)'`
  (read the exact path with `--list` once; check the selected count).

## Inputs

- Draft `temper/docs/design/draft/playground-reset.md`, sections 6.4 and 8.
- `temper/docs/design/draft/fake-backends.md`; `temper/docs/design/protocol/setup.md`, 17.2
  (the manifest flow).
- `temper/docs/plan/playground-reset-r6/notes/common.md`.
- `tests/fake_backends/walk.rs` and `walk/github.rs` as bx-walks left them; bx-harness's
  report (`KillBox`, `StartBox` on a fake-backends box).

## Owns and may change

- Owns: `tests/fake_backends/walk/github.rs` and the test's lines in `walk.rs`.
- May change: `temper/crates/temper/tests/support/boxed.rs` only if `StartBox` does not yet
  restart a `--fake-backends` box (its flags kept), and say so.
- Options, pre-approved:
  - If the fake GitHub's state (its Apps) does not survive the box's restart, because its
    peer keeps it in memory, the story checks that temper's records of the App survive and
    that `forge install N` reaches the installation; say what the fake kept.

## Steps

1. The story; `heavy cargo check -p temper --tests`.
2. Run it alone; read a failure's log before anything reruns.
3. `cargo fmt`; `heavy cargo clippy -p temper --all-targets`. Commit and push.

## Report

The hops seen before and after the restart, and what forge N read after it.

## Answers

2026-11-01. Change forge admission so Install runs from kept App records while disconnected?
No: forge admission stays. After the restart temper retried its kept App key and the fake
refused it (`KeyRefused`), because the fake GitHub's peer memory lasts one launch
(`temper/docs/design/draft/fake-backends.md`, 3) and the manifest's converted App went with
it. An Install of an App whose key is refused cannot reach an installation, on the fake or on
GitHub, so the refusal is right; on GitHub the App outlives a restart and the forge reads
connected, as it did here before the kill. The story therefore walks the App the fake keeps
across launches, its seeded own App (the walk's `own_app` connect, `--app` with the printed id
and key): `forge make github` → N; connect N with the seeded App, stopping before any
installation is visited; `KillBox`; `StartBox`; `forge list --json` reads N kept and connected
(its installations empty or the seeded one); `forge install N`, the visit to the installation
and back to `/forges/N/github/setup`; then `finish`, with no second connect and the manifest
route never visited. Keep the test's name; rename nothing else. The manifest-made App's restart
is not walked: list under `## Left` "a manifest-made App across a box restart needs the fake
GitHub to keep a converted App across launches (skein's fake forge, fake-backends.md 3)". If N
does not read connected after the restart with the seeded App, stop with a question quoting
the row. `fake::check::tests::verb` is not this task's: leave it under `## Left` as reported.
