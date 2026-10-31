+++
id = "stage-forge"
part = "stage-sites"
kind = "migration"
needs = ["echo-stage", "kit-shell", "counts", "abort-connection", "abort-setup"]
files = ["skein/testing/skein-throwaway-forgejo/**", "skein/tests/forge/conformance/**", "skein/tests/forge/setup/**", "skein/scripts/old-forms.txt"]
effort = "high"
minutes = 40
+++
# The forge hosts on the shared stage and surfaces

## Goal

Draft, 3.3: the throwaway Forgejo's one-shot host, the forge
conformance's hosted component and the forge-setup world's closing host
write io's stage by hand; they call `skein_io::stage` instead, and their
hosts move onto the provided deadline, by `notes/stage.md`.

Acceptance:

- Moved: `skein/testing/skein-throwaway-forgejo/src/http/host.rs:81`
  (`impl Host for OneShot`, from 74), `skein/tests/forge/conformance/src/component/hosted.rs:254`
  and `:259` (`impl Host for Hosted`, from 247),
  `skein/tests/forge/setup/src/closing.rs:67`, unless a default leaves
  one (named).
- `io_resume` falls by the sites moved (4 expected).
- The hosts' `next_deadline`s (`host.rs:174`, `hosted.rs:307`,
  `tests/forge/setup/src/host.rs:400`) follow `stage.md` item 2.
- `heavy cargo nextest run -p skein-throwaway-forgejo -p
  forgejo-conformance -p skein-forge-setup-world` passes (counts named;
  tests that need a live Forgejo are skipped as they are today, and
  named); clippy clean.

## Inputs

- `skein/docs/plan/component-kit-2/notes/common.md`, `notes/stage.md`.
- `skein_io::stage`, `skein_shell::Host` on your base.

## Owns and may change

- The directories in the header, `skein/scripts/old-forms.txt`.
- `skein/tests/forge/setup/src/memory.rs`'s `Request::Abort` arm is
  `abort-setup`'s, merged on your base: keep it.
- The components (`skein/crates/skein-forge-*`) are not yours.

## Steps

1. Each stage, item 1; each host's deadline, item 2; any referee in
   these files, item 4.
2. `--write` from `skein/`; checks of `common.md`; commit; push.

## Report

Per `common.md`.
