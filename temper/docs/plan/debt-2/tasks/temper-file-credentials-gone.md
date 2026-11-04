+++
id = "temper-file-credentials-gone"
part = "temper-forge-records"
kind = "feature"
needs = ["temper-fixtures-records"]
files = [
  "temper/crates/temper-service/src/settings.rs",
  "temper/crates/temper-service/src/forge.rs",
  "temper/crates/temper-service/src/forges.rs",
  "temper/crates/temper-service/src/forge/records.rs",
  "temper/crates/temper-service/src/forge/gone.rs",
  "temper/crates/temper-service/src/setup.rs",
  "temper/crates/temper-service/src/startup/application.rs",
  "temper/crates/temper/tests/support/owned.rs",
  "temper/crates/temper/tests/support/forgejo.rs",
  "temper/docs/design/protocol/credentials.status.md",
  "temper/docs/design/protocol/configuration.status.md",
  "temper/docs/development/debt/D005-forgejo-file-credentials.md",
  "temper/docs/development/debt/README.md",
]
effort = "high"
minutes = 80
+++

# temper D005, step 2: the Forgejo file-credential path goes

## Goal

With every fixture connecting its forges by records (`temper-fixtures-records`), delete
temper-service's second credential path: `settings::Forge`'s `api_token` and
`webhook_secret`, `file_access` and `file_api`, and every refusal naming a forge's credential
members, which the document does not have (`temper/docs/design/protocol/credentials.md`, 2.1
and 5; `temper/docs/design/protocol/configuration.md`, 1, 3 and 6); then the harness's no-op
and both status entries. This pays `temper/docs/development/debt/D005-forgejo-file-credentials.md`.

Acceptance:

- The item's Measure, `rg -c --type rust '\bfile_api\b|\bfile_access\b|\bfile_api_token\b'
  temper/crates/temper-service/src | awk -F: '{s+=$2} END {print s}'`, reads 0 (19 at base).
- `settings::Forge` and `startup::Forge` hold no `api_token` or `webhook_secret`;
  `rg -n 'b"api_token"|b"app_key"|b"webhook_secret"|member\(b"user"\)'
  temper/crates/temper-service/src` prints nothing: no refusal names a forge's credential.
- `rg -n '\bold_form\b' temper/crates/temper/tests/support` prints nothing.
- Every test in Steps passes unchanged but for the lines that named removed items, each
  count above zero; a test whose subject was the removed path (a refusal naming a credential
  member, `HEAD_PROBLEMS`' first line) goes or loses that line, as listed in the report.
- `credentials.status.md` loses its Departs entry; `configuration.status.md`'s Departs 6 is
  replaced as below; D005's file and its README row are gone; `tools/citations/citations.py
  --all` reads 0; clippy is clean.

## Inputs

- `temper/docs/plan/debt-2/notes/common.md`.
- `temper/AGENTS.md`.
- `temper/docs/development/debt/D005-forgejo-file-credentials.md`, whole.
- `temper/docs/design/protocol/credentials.md`, 2.1 and 5; `temper/docs/design/protocol/configuration.md`,
  1 ("No forge is configured", "No secret in the document"), 3 ("No forge here") and 6.
- `temper/docs/design/protocol/credentials.status.md` and `configuration.status.md`, their
  Departs entries.

## Owns and may change

- `temper-service/src/settings.rs`: `Forge.api_token`, `Forge.webhook_secret`.
- `temper-service/src/startup/application.rs` (~115-129): `startup::Forge`'s `api_token` and
  `webhook_secret` (nothing constructs it with them: `Named.forges` is always empty; nothing
  parses them: the document refuses `application.forges` as an unknown member). Its other
  fields stay.
- `temper-service/src/forge.rs`: `file_access` and its comment; `held` if nothing else calls
  it (`HELD_ADDRESS` stays); `Layer.file_access`, `Endpoints.file_access`; a forge's
  `access` from `forge::settings` is `None`. `startup_access`, if nothing inserts into it
  (the survey found none), goes with it, said in the report. In `problems`:
  - `Secret` pushes its limit name alone, no `webhook_secret` path;
  - `Request` pushes its four limit names alone, no `api_token` path;
  - `Token | Credential | Key | User` become one arm,
    `unreachable!("temper's prepared settings carry no forge access: a forge's access is its records' (credentials.md, 5)")`
    (only an `access` builds them, and no settings temper prepares carry one);
  - the other arms stay as they are.
- `temper-service/src/forges.rs` (~133-134, ~193-194), `forge/records.rs` (~161: the
  `records_only` half stays), `forge/gone.rs` (~35-36): the map's uses and the two `None`s.
- `temper-service/src/setup.rs`: `file_api` (~37, 190, 243, 367), `file_api_token` and its
  comment (~773-779), and `read_item`'s fallback for `Item::ApiToken` (~837-841), which
  reads the record's value alone.
- Compiler-forced lines in readers, each only the removed field or item: the `None`s in
  `temper-service/src/tests/**` literals (`tests/limits.rs` ~431 among them: its GitHub
  forge's `api_token` line goes), `temper/tests/client/src/fixtures.rs` (~38-39),
  `temper/tests/system/src/composed/link.rs` (~43-44). Tests whose subject was the removed
  path: `tests/forge.rs`'s `HEAD_PROBLEMS` (~525-552) loses its `api_token` line; a test of
  a credential member's refusal, or of `file_access` itself, is deleted.
- The harness: `owned::old_form` (`temper/crates/temper/tests/support/owned.rs` ~37-40) and
  its two calls (`owned.rs` ~47, `forgejo.rs` ~393). `owned::adopt` stays (it adopts the
  repository through the bot; it is not this item's), and so does
  `configuration.rs`'s `old_form_check_document`.
- `temper/docs/design/protocol/credentials.status.md`: its **Departs** line goes whole; the
  **Not yet** stays; date it today.
- `temper/docs/design/protocol/configuration.status.md`: in the **Departs** entry, the
  sentence from "6: `temper-service` keeps a Forgejo file-credential path" to "a code task
  moves them to records." is replaced by one sentence saying what still names the
  document's forges, from what `rg -n 'member\(b"forges"\)|forge_path'
  temper/crates/temper-service/src` finds after your change, in this form (adjust the list to
  the facts): "6: `forge::settings` takes a list of the document's forges
  (`temper-service/src/forge.rs`), which the product passes empty and only in-process test
  fixtures fill, to pair each forge with its endpoint; its refusals name
  `application.forges[N].api`, `log_origins` and `number`, and the startup checks'
  `application.forges` (`temper-service/src/startup/checks.rs`), members the document does
  not have, because those fixtures give their forges' endpoints that way." Touch no other
  words of the file: other drafts are editing its other entries. Date it today.
- `temper/docs/development/debt/D005-forgejo-file-credentials.md` (delete) and its row in
  `temper/docs/development/debt/README.md`. D008 cites D005 by ID only: leave it.

Exemptions (the same words for something else; they stay): `Item::ApiToken` and the
records' rows; the owned forge's handover-directory file names (`api_token` in
`temper/crates/temper/tests/**`, `temper-forge-setup`); the client tokens file's
`webhook_secret` item (`temper/docs/help/`, `temper-client-tools`' goldens);
`temper/crates/temper/src/host.rs`'s stopped lines ("forge N: api_token: invalid"), which
name a connection's failure, not a document member; `temper-service/src/tests/startup.rs`
~500 and ~948-954.

Pre-approved:

- If no caller passes `forge::settings` a non-empty forge list after
  `temper-fixtures-records`, the list and what hangs on it are dead: leave them (their removal
  is not this item's), write the configuration.status sentence to say the list is always
  empty, and name them under `## Left` in the report.
- If a `problems` arm in the `unreachable!` group is reached by a test through a fixture's
  forge, that fixture still builds an access: fix the fixture (records), not the arm; if it
  cannot, stop and ask.

## Steps

1. Delete in this order, building between: the setup host's `file_api`; `forge.rs`'s
   `file_access` and the maps; the `problems` paths; the fields; the harness. Sketch of the
   arm:

   ```rust
   Unusable::Token { .. } | Unusable::Credential { .. } | Unusable::Key { .. } | Unusable::User { .. } => {
       unreachable!("temper's prepared settings carry no forge access: a forge's access is its records' (credentials.md, 5)")
   }
   ```
2. Narrow tests, one at a time, each count above zero:
   - `heavy cargo nextest run -p temper-service --lib`
   - `heavy cargo nextest run -p temper-client-world --test suite`
   - `heavy cargo nextest run -p temper-system-world --test suite`
   - `heavy cargo nextest run -p temper --test suite -E 'test(/startup/)'`
   - `heavy cargo check -p temper --tests` (the process binaries need a Forgejo to run)
3. `cargo fmt`; `python3 -I -B scripts/old_forms.py` from `temper/` (with `--write` if a count
   fell); `heavy cargo clippy -p temper-service -p temper-client-world -p temper-system-world
   -p temper --all-targets -- -D warnings`.
4. Status and debt text as in "Owns"; `python3 -I -B tools/citations/citations.py --base
   origin/main` and `--all`; the Measure.
5. Two commits ("temper-service keeps one forge credential path, the records' (credentials.md,
   2.1 and 5)"; "pay temper D005: both status entries and the item"), pushed.

## Report

As the implementer's text says, and also: the Measure before and after; the acceptance
searches' output; the tests deleted or edited, each with why; the new configuration.status
sentence.
