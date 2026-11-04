+++
id = "throwaway-migrations"
part = "copy-facts"
kind = "interface"
needs = []
files = ["skein/testing/skein-throwaway-forgejo/src/lib.rs", "skein/testing/skein-throwaway-forgejo/src/instance.rs", "skein/testing/skein-throwaway-forgejo/src/cli.rs", "skein/testing/skein-throwaway-forgejo/src/http.rs", "skein/testing/skein-throwaway-forgejo/src/http/host.rs", "skein/testing/skein-throwaway-forgejo/src/tests.rs"]
effort = "medium"
minutes = 35
+++

# The throwaway, for a migration

## Goal

The throwaway Forgejo can allow local migrations, take a request that lasts minutes, and be killed and
restarted on its scratch (contracts S5), so that the conformance can cut a migration short.

Acceptance:

- `Settings::local_migrations`, false in `small()`: true appends `[migrations]` and
  `ALLOW_LOCALNETWORKS = true` to the `app.ini`; false writes the bytes the golden holds today. A test
  beside the existing `ini` tests.
- `Instance::exchange_patient(outgoing, patience)` is `exchange` with `patience` for the answer in
  place of 10 s; `exchange` keeps its 10 s.
- `Instance::kill` ends the process and keeps the scratch; `Instance::restart` starts it on the same
  scratch and port and waits for the API; `stop` after either cleans up. A test in
  `skein/testing/skein-throwaway-forgejo`'s ignored tests, which need the pinned binary, shows a
  repository made before the kill reads after the restart.
- `heavy cargo nextest run -p skein-throwaway-forgejo` passes;
  `heavy cargo check --tests -p forgejo-conformance -p temper-git-transcripts` passes.

## Inputs

- `temper/docs/plan/repository-copies-1/notes/common.md`; `notes/contracts.md`, S5.
- `skein/docs/design/fake-forge.md`, 6.

## Owns and may change

- The files of the header.
- `macos-cheap` moved the pin and touched `tests/support/attached.rs`, one line: leave it.
- Pre-approved:
  - If `Instance` holds what `restart` needs nowhere, keep it there (the binary's path, the port, the
    environment `start` built); private fields of yours.
  - An attached instance's `kill` and `restart` refuse, as its `cli` does.
  - The `Settings` literal lives in `small()` alone: no reader changes.

## Steps

1. The setting, 2. the patience, threaded to `http/host.rs`'s `arm`, 3. `kill` and `restart`, 4. the
tests, each biting: write the section unconditionally, and drop the patience, each once, 5. the
implementer's checks; commit and push.

## Report

As the implementer's text asks; whether the ignored test ran, on which binary.
