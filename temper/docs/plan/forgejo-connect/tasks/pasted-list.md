+++
id = "pasted-list"
part = "bundle"
kind = "interface"
needs = []
files = [
  "temper/crates/temper-protocol-records/**",
  "temper/tests/codecs/tests/suite/records.rs",
  "temper/crates/temper-service/src/setup/public.rs",
  "temper/crates/temper-service/src/tests/setup_admin.rs",
  "temper/crates/temper-service/src/tests/setup_lifecycle.rs",
  "temper/crates/temper-client-domain/src/forge/forges.rs",
  "temper/crates/temper-client-domain/src/tests/admin.rs",
  "temper/crates/temper-client-domain/src/tests/github_app.rs",
  "temper/crates/temper-client-shell-cli/src/arguments.rs",
  "temper/crates/temper-client-shell-cli/src/host/input.rs",
  "temper/crates/temper-client-shell-cli/src/tests/keys.rs",
  "temper/crates/temper-client-tools/src/forges.rs",
  "temper/crates/temper-client-tools/src/forges/connection.rs",
  "temper/crates/temper-client-tools/src/tests/forge_connection.rs",
  "temper/crates/temper-client-tools/src/tests/golden/help_forge_connect.txt",
  "temper/docs/help/reference/ctl-forge.md",
  "temper/docs/design/protocol/credentials.status.md",
]
effort = "high"
minutes = 90
+++

# The carried bundle holds a list of items, in the codec's own form

## Goal

A connected Forgejo's connect carries its API token, its git token and its hooks' secret in one
value, but temper's bundle, `temper-protocol-records`' `Pasted`, holds one item. Make it a list of
at most three items in the codec's own form, in place at `temper-secrets` 1, as
`notes/contracts.md`, C1 sketches; move its readers to the new API with no change of what they
send or accept, but the one-item bound's number, which the list's count makes smaller.

Acceptance:

- `schema/secrets-v1.schema` reads `versioned record Pasted { items: list 3 PastedItem }`; the
  generated `src/generated/v1.rs` and the goldens under `golden/` are regenerated, and the drift
  test passes.
- `encode_pasted(&[(u16, &[u8])])` and `decode_pasted(&[u8]) -> Result<Box<[(u16, Secret)]>,
  PastedError>` as C1 says; `PASTED_PAYLOAD` is `PASTED_BYTES` less a one-item bundle's framing,
  computed from the codec's form (version, the list's `u32` count, the variant's tag, the
  secret's length), not a bare number.
- The codec suite: a one-item bundle of `PASTED_PAYLOAD` bytes is exactly 4096 bytes and decodes
  to its item, for codes 1 and 11; one byte more is refused; a bundle of codes 2, 3 and 5
  round-trips in its order; an empty list, a code twice, an empty value, four items, and an
  unknown code are each refused by `encode_pasted`, and the bytes of each such list (built with
  the generated types) by `decode_pasted` as `Unreadable`; a bundle of another version is
  `Version`.
- Every reader below compiles and its tests pass, sending and accepting what it did: a pasted App
  key is a bundle of one item 11, an admin's token one item 1; the service's `pasted` takes a
  bundle of exactly the item its caller names, else the caller's words.
- `temper/docs/design/protocol/credentials.status.md` loses the sentence that begins `2.2: a
  bundle of several items:` through `(\`temper-service/src/setup/public.rs\`, \`pasted\`).`.

## Inputs

- `temper/docs/plan/forgejo-connect/notes/common.md`, and `notes/contracts.md`, C1.
- `temper/docs/design/protocol/credentials.md`, 2.2 (*The value*, *The requests that carry one*).
- skein's `docs/design/codec.md`, its type table (a `list N T` is a `u32` count, then its items;
  an enumeration a byte tag, then its record).

## Owns and may change

- `temper/crates/temper-protocol-records/**`: the schema line above, the regenerated code and
  goldens (`SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p temper-codecs-world --test suite -E
  'test(drift::)'`, which also rewrites the other families' files: keep only this family's), and
  `src/codec.rs`: `encode_pasted` and `decode_pasted` as C1, `PASTED_PAYLOAD` computed. The
  per-item conversion (`generated`, `raw`, `pasted_item`) stays as it is; a code is checked by
  `parse` as today, 11 being the PEM.
- `temper/tests/codecs/tests/suite/records.rs`: the two `pasted_payload_*` tests move to the list
  API, and the acceptance's cases join them.
- Readers, each edited only at its call or its literal:
  - `temper-service/src/setup/public.rs`, `pasted(bytes, item, invalid)`: decode the list, and
    answer `Ok` only for exactly one item whose code is `item` and whose value is nonempty; else
    `Err(invalid)`; `Version` keeps its message. Its callers do not change.
  - `temper-service/src/tests/setup_admin.rs`, `bundle` (`:57`), and
    `temper-service/src/tests/setup_lifecycle.rs` at `:700`, `:856`, `:986` and `:1584`: the list
    call. `foreign_and_legacy_pastes_name_the_version_and_repaste` (`:1970`) keeps its first
    bytes (another version); its second, the former one-item layout at version 1, now reads as
    unreadable, so assert what `paste_ended` says for it (the caller's words, `the pasted value
    is not a key`) and rename the test for what it checks
    (`foreign_versions_and_former_layouts_are_refused`).
  - `temper-client-domain/src/forge/forges.rs`: `pasted_app_key` and `pasted_admin_token` call
    the list form with their one item; `PASTED_PAYLOAD`'s re-export stays.
  - `temper-client-domain/src/tests/admin.rs:100`, `the_bundle_bound_includes_its_seven_byte_header`:
    renamed `the_bundle_bound_includes_its_framing`, its two lengths `PASTED_PAYLOAD` and
    `PASTED_PAYLOAD + 1`; `tests/github_app.rs` (`:53`, `:162`, `:164`) likewise from the
    constant.
  - `temper-client-shell-cli/src/arguments.rs`, `key_bytes`: `forge connect`'s bound is
    `PASTED_PAYLOAD` (as `u32`, through `temper_client_domain::forge::forges::PASTED_PAYLOAD`), its
    comment saying the framing, not "seven bytes"; `host/input.rs:110-111` compares with that
    bound and its words name the new number; `tests/keys.rs` (`:116-120`, `:166`) from the
    constant, the test renamed for the framing, not "seven bytes".
  - `temper-client-tools/src/forges/connection.rs:278-280` (the failure's words) and
    `src/forges.rs:163` (`CONNECT`'s notes, "1 to 4089 bytes"): the new number;
    `tests/forge_connection.rs:14-15` from the constant.
  - The help: `temper-client-tools/src/tests/golden/help_forge_connect.txt` and
    `temper/docs/help/reference/ctl-forge.md` are regenerated, never hand-written, by
    `SKEIN_CODEGEN_WRITE=1` on `tests::help::the_help_prints_as_its_golden_files` and
    `tests::help::the_forge_reference_is_what_the_table_renders` (`heavy cargo nextest run -p
    temper-client-tools --lib -E 'test(=...)'`, the exact names from `--list`); their diff is the
    number alone. This departs from "Not yours: `docs/help/`" because the page is the table's
    rendering, which the test checks.

Pre-approved: if `rg -n '4089|4090|seven.byte|encode_pasted|decode_pasted' temper/crates temper/tests`
finds another reader, change it the same way (its call or literal only) and list it in the
report. A new reader's number in words names `PASTED_PAYLOAD`'s value; where code can use the
constant, it does. Nothing else of the client, the service or ctl changes: the bundle of several
items is used by later tasks.

## Steps

1. The schema, its regeneration and `codec.rs`; then the codec suite: `heavy cargo nextest run -p
   temper-codecs-world --test suite -E 'test(drift::) | test(records::)'`.
2. The service: `heavy cargo nextest run -p temper-service --lib -E 'test(setup_admin) |
   test(setup_lifecycle)'`.
3. The client domain, ctl and the CLI shell, with the help regenerated: `heavy cargo nextest run -p
   temper-client-domain --lib -E 'test(admin) | test(github_app)'`, then `-p temper-client-tools
   --lib`, then `-p temper-client-shell-cli --lib`, then `-p temper-docs`.
4. The status edit; the citations check (`notes/common.md`).
5. `cargo fmt`; old forms for temper; `heavy cargo clippy -p temper-protocol-records -p
   temper-service -p temper-client-domain -p temper-client-tools -p temper-client-shell-cli -p
   temper-codecs-world --all-targets -- -D warnings`; the focused tests of those packages, and
   `heavy cargo check -p temper-client-view --tests` and `-p temper-client-world --tests` (readers
   of the client domain). Commit the code and tests, then the status text; push.

## Report

As the implementer's text says, with `PASTED_PAYLOAD`'s new value, the regenerated files' `git
diff --stat`, and any reader found beyond the list.
