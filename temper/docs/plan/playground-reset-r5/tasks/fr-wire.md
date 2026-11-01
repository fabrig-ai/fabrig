+++
id = "fr-wire"
part = "forge-records"
kind = "interface"
needs = ["pg-wire"]
files = ["temper/crates/temper-client-protocol-wire/**", "temper/tests/codecs/tests/suite/client_wire.rs", "temper/tests/codecs/tests/suite/fuzzy/client_wire.rs"]
effort = "high"
minutes = 75
+++

## Goal

The client wire of parts `forge-records` and `remove-forge`, so the
client part builds on it at once: `SetUp` and `Connect` name a
`ForgeChoice` (a new forge's choice, or a kept forge's number), the
request `remove_forge`, and their outcomes. Every sender keeps today's
behaviour by sending `kept`; the service answers a `new` choice and
`remove_forge` with a refusal until `fr-svc-route` and `rf-service`
serve them.

Acceptance:

- `temper/crates/temper-client-protocol-wire/schema/client-wire-v1.schema`
  holds exactly the forms of `notes/forge-records.md`, section 6:
  `NewGitHub`, `NewForgejo`, `NewForge { owned, github, forgejo }`,
  `KeptForge`, `NewForgeChoice { forge: NewForge }`, `ForgeChoice { new,
  kept }`; `SetUp.forge` and
  `Connect.forge` typed `ForgeChoice`; `RemoveForge { forge: u16 }` and
  `Request::remove_forge` appended after pg-wire's requests;
  `MakeRefusal`, `MakeRefused`, `Removing`, `RemoveRefusal`,
  `RemoveRefused`, and `Outcome` gaining `make_refused`, `removing`,
  `remove_refused` after pg-wire's `reset`. `Install`, `Reachable` and
  `Accept` keep `forge: u16`. Generated `src/generated/v1.rs` and
  goldens regenerated in place. The family version and `CLIENT_VERSION`:
  pg-wire raised them in this run; leave them.
- `src/lib.rs`'s request validation (`decode_requesting`, about line
  256): a `forgejo` choice's address is nonempty, at most 2048 bytes,
  begins `http://` or `https://`, has a host, no credentials, query,
  fragment or trailing slash (the checks today's `git_base` makes in
  `temper-service/src/startup/application.rs`, written here on the bytes);
  a `kept` forge is nonzero; `remove_forge`'s forge nonzero.
- Tests in `src/tests.rs`: `set_up_and_connect_round_trip_each_choice`,
  `a_forgejo_choice_refuses_an_address_with_credentials_query_or_no_host`,
  `remove_forge_and_its_outcomes_round_trip`,
  `a_kept_choice_or_removal_of_forge_zero_is_refused`; the codecs world's
  `client_wire.rs` and `fuzzy/client_wire.rs` gain the new forms as
  pg-wire's are there.
- Senders and translators compile with today's meaning (forced edits,
  each a line or a few, named in the report):
  - `temper-service/src/family.rs` `to_root`: `kept { forge }` becomes
    today's `forge`; a `new` choice is answered `make_refused {
    not_offered }` without reaching the root, `remove_forge` is answered
    `remove_refused { no_such_forge }`, each as an invalid request is
    answered there today, with a comment naming `fr-svc-route` /
    `rf-service`.
  - `temper-client-domain` (`src/forge/family.rs` `set_up`,
    `src/forge/admin.rs`, `src/forge/github.rs`), `temper-client-tools`
    (`src/forges.rs`, `src/forges/connection.rs`), `temper/tests/client`
    (`src/translate/forge.rs`), and the test fixtures of
    `temper-client-shell-cli`, `-terminal`, `-web`, the view and the
    gallery: `ForgeChoice::Kept(KeptForge { forge })` where they wrote
    `forge`; the new outcomes treated as the nearest arm treats an
    unexpected outcome.

## Inputs

- `temper/docs/plan/playground-reset-r5/notes/common.md`,
  `temper/docs/plan/playground-reset-r5/notes/contracts.md` (section 3,
  the client acts), `temper/docs/plan/playground-reset-r5/notes/forge-records.md`
  (sections 3 and 6).
- `temper/docs/design/client/README.md`, 4.6 (the family's records) and 8.
- `temper/docs/plan/playground-reset-r5/tasks/pg-wire.md` (what the
  trunk's schema gained just before you).
- Survey `temper/docs/plan/playground-reset-r5/notes/surveys/s1b-connector-store-acts.md`,
  section 3 ("Wire definitions", "Act/wire test readers").

## Owns and may change

- Owns: `temper/crates/temper-client-protocol-wire/**`;
  `temper/tests/codecs/tests/suite/client_wire.rs` and
  `fuzzy/client_wire.rs`.
- May change, forced edits only, as listed in the acceptance:
  `rg 'v1::SetUp \{|v1::Connect \{|w::SetUp \{|w::Connect \{|Outcome::'`
  across `temper/` finds them. No sender gains a new behaviour (the
  client part's tasks and `fr-ctl` do that).
- Options: if a sender builds `SetUp`/`Connect` through a helper used by
  many tests, change the helper alone and say so.

## Steps

1. Schema forms after pg-wire's; regenerate; goldens.
2. Validation in `lib.rs`; library tests.
3. Forced edits across temper (`heavy cargo check -p PKG --tests` while
   errors last).
4. Codecs world cases. Narrow: `heavy cargo nextest run -p
   temper-client-protocol-wire --lib`; `heavy cargo nextest run -p
   temper-codecs-world --test suite -E 'test(/client_wire/)'`.
5. `cargo fmt`; `heavy cargo clippy -p temper-client-protocol-wire -p
   temper-codecs-world -p temper-service -p temper-client-domain -p
   temper-client-tools --all-targets -- -D warnings`; `heavy cargo check
   --tests` for `temper-client-view`, `temper-client-shell-cli`,
   `temper-client-shell-terminal`, `temper-client-shell-web`,
   `temper-client-world`, `temper-client-gallery`; focused tests of
   `temper-client-protocol-wire` and `temper-client-domain`. Old-forms
   from `temper/`. Commit and push.

## Report

As `notes/common.md` says; every file outside the owned list you edited.

## Answers

2026-11-01. Question: may `ForgeChoice::new` carry a wrapper record
holding `NewForge`, since skein's schema rejects an enum as a variant's
payload? Answer: yes. Add `record NewForgeChoice { forge: NewForge }`
and type `ForgeChoice { new: NewForgeChoice, kept: KeptForge }`;
`notes/forge-records.md`, section 6, now says so. No skein change: the
rule is skein's grammar, pinned by its tests. `NewGitHub`, `NewForgejo`
and `NewForge` stand as written, so `machine::fill(&Machine,
&v1::NewForge)` (`fr-machine`) is unchanged; validation reads the
`forgejo` address at `new.forge`. Go on from the pushed commit
`5343c927fb`.
