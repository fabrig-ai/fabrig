+++
id = "copy-facts"
part = "copy-facts"
kind = "feature"
needs = ["migrate-request", "throwaway-migrations"]
files = ["skein/tests/forge/conformance/src/migrate.rs", "skein/tests/forge/conformance/src/lib.rs", "skein/tests/forge/conformance/src/main.rs", "skein/tests/forge/conformance/src/mask.rs", "skein/tests/forge/conformance/README.md", "skein/tests/forge/conformance/facts.txt", "skein/tests/forge/conformance/transcripts/setup-migrate-*/**", "skein/tests/forge/conformance/transcripts/setup-repository-delete*/**", "skein/crates/skein-forge/tests/suite/forgejo_migrate.rs", "skein/crates/skein-forge/tests/suite/main.rs", "skein/crates/skein-forge/tests/common/forgejo.rs"]
effort = "high"
minutes = 75
+++

# The facts a copy relies on

## Goal

The conformance records, on the pinned Forgejo, every fact `Copy` relies on (the list and names of
`skein/docs/design/forge-setup.md`, section 8, "A copy"), by a focused mode that touches nothing else.
What it records may overturn what 4.10 assumes: **that is the point of this task**, so say so in the
report rather than fitting the fact to the design.

Acceptance:

- `forgejo-conformance --migrate` runs on its own instances, writes transcripts for
  `setup-migrate-*` and `setup-repository-delete*` and only their fact lines, as `--create-repository`
  does, and each of section 8's "A copy" facts is `holds`, or `fails` with what Forgejo does instead.
- The scopes: `Migrate` and `DeleteRepository` each measured as the `setup-scope-*` facts are, and the
  table's `write:repository` confirmed or reported.
- `skein/crates/skein-forge/tests/suite/forgejo_migrate.rs`, on `forgejo_create_repository.rs`'s
  pattern: the client's bytes match the recording, the server routes them, and the `201` decodes as
  `Document::CreatedRepository` in one, seven and whole-byte chunks.
- `heavy cargo nextest run -p forgejo-conformance -p skein-forge -p skein-forge-machine-world` pass.
- No request reaches a real host: the source is a second throwaway on loopback.

## Inputs

- `temper/docs/plan/repository-copies-1/notes/common.md`; `notes/contracts.md`, S1 and S5.
- `skein/docs/design/forge-setup.md`, 4.10 and section 8, "A copy" and the scope table.
- `skein/docs/design/fake-forge.md`, 6 and 7.
- `temper/docs/design/draft/repository-copies.md`, 4, 6 and 8.

## Owns and may change

- The files of the header. `skein/tests/forge/conformance/src/facts.rs`, `fixture.rs`: helpers.
- **Departures from the implementer's text:** this task records with the pinned binary, which the
  gate never does. The binary's fetch into `target/pins` is pre-approved, as
  `skein/tests/forge/conformance/README.md` says; if a builder has no way to get the pin, report
  `Infra: pin`, with what you tried.
- **The pin:** record at the pin your base holds (`skein/crates/skein-forge/src/forgejo/pin.rs`) and name
  it in the report. `macos-cheap` moves it to 17.0.1; if your base is still at 17.0.0, record there,
  report it, and the mode reruns unchanged once the pin lands.
- **`facts.txt` and the transcripts of others are `macos-cheap`'s:** add your lines and directories and
  rewrite nothing. Run the focused mode alone, never the whole recording.
- **Reader of transcripts:** `skein/crates/skein-forge/tests/common/forgejo.rs`: `setup_cases` and the
  `probe` list in `json_cases`, which panics on a 2xx transcript the server cannot route.
- **The cases, in one new `src/migrate.rs`:**
  - The source: a second throwaway, with `Init::Readme` and a branch that is not its default, one
    repository of `Init::Empty`, on its own port.
  - The stall: a loopback proxy of the crate's own, on `deliveries.rs`'s `Listener`'s pattern, between
    the two; it holds a source's bytes until told. Masking takes the proxy's port as `listener`.
  - The first throwaway has `local_migrations`; the source keeps the default, for
    `setup-migrate-of-a-local-address-is-refused-by-default`.
  - `setup-migrate-cut-short-leaves-a-repository-with-no-branch`: `POST` on a thread through the stall,
    read the repository mid-flight, `kill`, `restart`, read it, delete it, copy again.
  - `setup-migrate-goes-on-when-its-client-goes` and `setup-migrate-deleted-while-it-runs`: the same
    stall with Forgejo up: the client drops, the stall lifts or the repository is deleted, and what
    Forgejo then shows is recorded as it is. Say plainly in the report whether a `DELETE` under a
    live migration is safe.
  - `setup-actions-reads-forgejo-workflows-alone-when-both-are-there`: a source holding both
    `.forgejo/workflows/` and `.github/workflows/`; the runner group is `unchecked` without `--runner`.
  - `setup-migrate-runs-with-offline-mode`: the first throwaway's `app.ini` has `OFFLINE_MODE = true`, as
    temper's owned Forgejo's has.
- Pre-approved:
  - Run the mode on its own instances: `Settings::small()` and the shared fixture's ids stay.
  - A fact Forgejo does not hold is `fails` with its reason: commit it, and report it first.
  - If `Mask` needs a second port, add it.

## Steps

1. `--migrate`, on `create_repository.rs`'s pattern. 2. The source and the stall. 3. The cases, then the
facts. 4. Record, copy in the new directories and lines, check the secrets scan passes. 5.
`forgejo_migrate.rs`. 6. The implementer's checks; commit and push.

## Report

As the implementer's text asks; **first, each fact that did not hold, and what 4.10 should say instead**;
the pin recorded; how long a run takes.
