+++
id = "setup-intent"
part = "setup-intent"
kind = "interface"
needs = []
files = [
  "temper/crates/temper-client-protocol-wire/**",
  "temper/crates/temper-protocol-store/**",
  "temper/crates/temper-engine-domain-forge/**",
  "temper/crates/temper-engine-domain/src/boundary.rs",
  "temper/crates/temper-engine-domain/src/route/forge_route.rs",
  "temper/crates/temper-service/src/family.rs",
  "temper/crates/temper-service/src/route.rs",
  "temper/crates/temper-service/src/setup/**",
  "temper/crates/temper-service/src/tests/**",
  "temper/crates/temper-client-domain/src/forge/**",
  "temper/crates/temper-client-domain/src/tests/**",
  "temper/crates/temper-client-tools/src/forges.rs",
  "temper/crates/temper-client-tools/src/tests/**",
  "temper/tests/codecs/**",
  "temper/tests/engine/**",
]
effort = "high"
minutes = 170
+++

# The engine's half: `SetUp`'s first repository, three ways

## Goal

`SetUp`'s first repository becomes `Empty`, `Copies` or `Nothing`, kept in the forge's record and told on its
row, changed in place. `Copies` and `Nothing` make no repository and connect at once. The client still sends
`Empty`: the sheet is `sheet-choices`. Request, section 7, *The engine*.

Acceptance:

- A `SetUp` with `Nothing` or `Copies` reaches connected with no `Create`, its row's `first` none and its
  `starts_with` saying so, and it holds across a restart of the engine. `Empty` behaves exactly as today, the
  playground's among it. `temper ctl forge setup` sends `Empty`, its flags and output unchanged but for the new
  fields of `forge list --json`.
- The set up's addresses are checked at admission as `MakeRepository`'s are (invalid), at most `FIRST_COPIES`.
- "Set up again" on a failed set up asks with the same three-way value.
- The step list has four steps for `Copies` and `Nothing`, the fourth done once connected, not a fifth under way
  for ever.
- `MakeRepository`'s address is bound by 512 bytes at admission, not `name_bytes`: an address of 100 bytes is
  admitted. This is what makes a copy of a long name reach the forge.

## Inputs

- `temper/docs/plan/ui-welcome/notes/common.md`, `notes/contracts.md`, *The set up's intent*.
- `temper/docs/plan/ui-welcome/inputs/request.md`, section 7.
- `temper/docs/design/protocol/setup.md`, 6.7. `temper/docs/design/client/README.md`, 4.6 (`SetUp`, the row).
  `temper/docs/design/client/forge.md`, 5.4. `temper/docs/design/protocol/forge.md`, 2.1.
  `temper/docs/design/protocol/store.md`, section 2.
- `temper/docs/design/draft/repository-copies.md`, 5.4 and 7.

## Owns and may change

Owns the files in `files`.

May also change: `temper/crates/temper-client-view/src/forge/{connect,forges}.rs` and
`temper/crates/temper-client-view/src/words.rs` for the step list alone (below);
`temper/tools/gallery/src/{worlds,samples}.rs`, `temper/tools/gallery/src/worlds/github.rs` and every other
`ForgeRow` or `SetUp` literal, a field's edit each; `temper/crates/temper-client-shell-cli/src/tests/fixtures/setup.rs`;
`temper/docs/design/protocol/setup.status.md`, `temper/docs/design/client/README.status.md`: delete what you
close.

Options, pre-approved:

- **A bound that fails** at 16 addresses of 512 bytes, 16 forges (`w::Forges::decoded_bound`, the clients'
  `document_bytes`, `v11::Row::encoded_bound`, jig's `FamilyRequest`'s 1 MiB): lower `FIRST_COPIES` in both
  schemas and `setup.md`, say by how much and why, rather than raise a budget.
- **`keep_first` and `with_intent`** on a forge later given a managed connect: `Copies` and `Nothing` yield no
  `FirstAsk`; the managed connect's own `first` replaces them, as it does today.
- **`label` and `playground`** exist on `Empty` alone. `setup/making.rs` falls back as it does with none.
- A view's forge line with neither first repository nor target reads as a forge with none today does.

Not this task's: the sheet, `not_yet`, Ready, Home.

## Steps

1. **The schemas,** in place, by `notes/contracts.md`. Regenerate
   `temper-client-protocol-wire/src/generated/v1.rs` and goldens with `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest
   run -p temper-codecs-world --test suite -E 'test(drift::)'`; it deletes nothing, so remove a dropped type's
   `.bin` by hand. The store stays at 11 (`temper-protocol-store/src/tests/sizes.rs`), its payload sum counting
   the list.
2. **The engine,** `temper-engine-domain-forge`: `FirstIntent` as the contract says; `make.rs`'s `setup`
   admission and `restore_first` (state `Making` for `Empty` alone: a `Copies` row must not read underway);
   `connect.rs`'s `with_intent`, `keep_first`; `Row.starts_with`. `making.rs`: the address's bound.
3. **The service.** `setup/runtime.rs`: `intent()` returns `None` but for `Empty`, and `record.first.is_some()`
   comes apart into "has an intent" and "has a repository to make". `setup/lifecycle.rs`: `create_local` returns
   false for them, so `adopt_local` connects, as it does with no intent; read
   `temper-service/src/tests/setup_create.rs` first, for how a second ask refills `local.first`. `family.rs`
   decodes `SetUpFirst`, canonicalising addresses as it does `MakeRepository`'s, and encodes `starts_with`.
4. **The client domain.** `forge/family.rs`, `forge/forges.rs` (`Row.starts_with`; `SetUp`, `SetupValues`,
   `retry_fields` and `side.set_up` carry the three-way value; `send_setup` sends `Empty` for the sheet's empty
   choice, as today), `forge/connect.rs`'s `underway` and `failed` for a row with none. `forge/listed.rs`.
5. **The step list.** `temper-client-view/src/forge/connect.rs` `progress_in`: four steps when the row's
   `starts_with` is `Copies` or `Nothing`.
6. **Tools.** `forges.rs`: `Intent` sends `Empty`; `first_json` and `forge list --json` gain `starts_with`, and,
   for copies, `copies`; its text says no repository for them; `made:FORGE` stays false for them.
7. **Tests,** the intent through store and wire for each of the three, full and smallest (goldens); `temper-engine-domain-forge`'s
   `tests/make.rs`: admission of each, an invalid address, one past the bound, `Empty` with `playground`;
   `temper-service`'s `tests/setup_create.rs` and `setup_lifecycle.rs`: `Nothing` and `Copies` connect with no
   `Create` run, a restart keeps them, "Set up again" asks the same; `tests/engine/domain/tests/suite/first_repository.rs`:
   the row, and its restart; `temper-client-domain`'s decode; `temper-client-tools`' `tests/forges.rs`: the JSON.
   A view test of the four steps.
8. **Check,** `notes/common.md`: a change of meaning runs its readers: `temper-engine-domain-forge`,
   `temper-engine-domain`, `temper-protocol-store`, `temper-client-protocol-wire`, `temper-codecs-world`
   (`drift`, then fuzzy), `temper-service`, `temper-client-domain`, `temper-client-tools`, `temper-client-view`,
   `temper-client-shell-cli`, `temper-client-gallery`, `temper-engine-domain-world`, `temper-engine-forge-world`,
   `temper-client-world`, and `temper`'s `process` and `suite` for the here-key, startover and forge-land walks;
   `heavy cargo check --tests` for `temper-client-shell-web`, `temper-client-shell-terminal`,
   `temper-client-browser-world`, `temper-system-world`. Commit and push.

## Report

What each bound came to, the goldens moved, and what a row of each kind reads as.
