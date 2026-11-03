# D005: The Forgejo file credentials live on for test fixtures

- **Kind:** dead code
- **Where:** `temper-service/src/settings.rs`, `temper-service/src/forge.rs`, `temper-service/src/setup.rs`, `temper-service/src/forges.rs`, `temper-service/src/forge/records.rs`, `temper-service/src/forge/gone.rs`, `temper/crates/temper/tests/support/owned.rs`
- **Seen:** 2026-11-03 at `41d63da8c3`, in `temper/docs/design/protocol/configuration.status.md` (its Departs entry 6) and `temper/docs/design/protocol/credentials.status.md` (its Departs entry for 2.1 and 5)
- **Seen:** 2026-10-20, in `temper/docs/development/hot-restart-report.md` ("this checkout still reads configured token files")
- **Seen:** 2026-11-03 at `41d63da8c3`, in the code's comments: "Transitional Forgejo files; temper-15 replaces them with records" (`temper-service/src/forge.rs`, above `file_access`), "Connected Forgejo's legacy file input" (`temper-service/src/setup.rs`, `file_api_token`)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** medium (one credential path, the records', in product code; refusals that name only members the document has)
- **Size:** M
- **Status:** open
- **Paid when:** `settings::Forge` has no `api_token` or `webhook_secret`, temper-service has no `file_access` or `file_api`, no refusal names `application.forges[N].api_token` or `app_key`, the fixtures connect their forges by records, and both status entries are gone.
- **Measure:** `rg -c --type rust '\bfile_api\b|\bfile_access\b|\bfile_api_token\b' temper/crates/temper-service/src | awk -F: '{s+=$2} END {print s}'`: 19

## What

The design keeps a forge's access in records (`temper/docs/design/protocol/credentials.md`,
2.1 and 5); the configuration's secrets' files are no longer a forge's
(`temper/docs/design/protocol/configuration.md`, 6). temper-service still
carries a second path, which only in-process test fixtures fill:

- `settings::Forge.api_token` and `webhook_secret`
  (`temper-service/src/settings.rs`; `temper-service/src/startup/application.rs`
  parses `api_token`);
- `forge.rs`'s `file_access`, giving a forge its `Access` from those
  fields, and the `file_access` map that `temper-service/src/forges.rs`,
  `temper-service/src/forge/records.rs` and `temper-service/src/forge/gone.rs` keep;
- `setup.rs`'s `file_api` map and `file_api_token`, which the setup host
  reads for `Item::ApiToken`;
- refusals that point at `application.forges[N].api_token` and
  `app_key` (`forge.rs`, the `Unusable` arms), members the document no
  longer has.

The fixtures that fill them: `api_token` appears in 16 test files, among
them `temper-service/src/tests/fixtures.rs` and `setup.rs`,
`temper/tests/client/src/fixtures.rs`, `temper/tests/system/src/composed/link.rs`
and `temper/crates/temper/tests/suite/startup.rs`. The process harness
keeps the same old form beside it: `owned::old_form`
(`temper/crates/temper/tests/support/owned.rs`) is a function that does
nothing, called from `owned.rs` and `forgejo.rs`, and `owned::adopt` is
"temper-01a's Adopt stand-in", both marked for removal by temper-15.

## Why it matters

Product code holds a credential path that no deployment uses, beside
the one the design has; a reader must find out which one a forge took
(`file_access.contains_key` in `temper-service/src/forge/records.rs`), and an operator can
be told to fix a member that does not exist.

## What pays it back

Move each fixture to connect its forge by records, as the shipped path
does, then delete the fields, maps, refusal arms and the harness's
no-op, and strike both status entries. One plan: the fixtures by
world, then the removal. D006 removes the other half of the old forms
(the reconcile path) and D008 the "temper-15" comments that point here.
