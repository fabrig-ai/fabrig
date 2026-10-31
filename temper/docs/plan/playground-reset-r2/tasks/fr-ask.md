+++
id = "fr-ask"
part = "forge-records"
kind = "interface"
needs = ["forge-families", "cl-wire"]
files = [
  "temper/crates/temper-engine-domain-forge/src/boundary.rs",
  "temper/crates/temper-engine-domain-forge/src/lib.rs",
  "temper/crates/temper-engine-domain-forge/src/records.rs",
  "temper/crates/temper-engine-domain-forge/src/domain.rs",
  "temper/crates/temper-engine-domain-forge/src/connect.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/**",
  "temper/crates/temper-engine-domain-forge/src/tests.rs",
  "temper/crates/temper-engine-domain/src/**",
  "temper/crates/temper-service/src/family.rs",
  "temper/crates/temper-service/src/route.rs",
  "temper/tests/engine/domain/**",
  "temper/tests/engine/forge/**",
]
effort = "high"
minutes = 60
+++

## Goal

Give the forge connector and the root the surface that makes forges
records (`temper/docs/design/draft/playground-reset.md, 3`), with
today's behavior kept: `SetUp` and `Connect` name their forge by a
`Target` (a kept forge or a new one), their answers name the forge's
number, a seed event and a configure call exist, and a forge made
emits `Made`. A `Target::New` and a seed are refused or ignored until
`fr-domain` builds them; every existing caller sends `Target::Kept(n)`
with the number it sends today. This task lands the names `fr-domain`,
`fr-registry` and D4's tasks build on, so it stays small.

Acceptance:

- Every package that depends on `temper-engine-domain-forge` or
  `temper-engine-domain` builds, and its focused tests pass unchanged in
  meaning (the edits to them are call sites).
- New unit tests in `temper-engine-domain-forge`
  (`src/tests/records.rs`):
  `a_set_up_naming_a_kept_forge_answers_its_number` (a SetUp with
  `Target::Kept(3)` on a configured owned forge 3 answers
  `SettingUp { forge: 3 }`), `a_new_forge_is_not_offered_yet`
  (`Target::New(..)` in SetUp and Connect is refused `NotOffered` /
  `NoSuchForge`), `configure_installs_a_recorded_forge_and_refuses_an_unrecorded_one`.
- The service's wire answers carry the domain's numbers: cl-wire's
  forge-0 bridges in `temper-service/src/family.rs` are gone (its test
  `temper-service` `src/tests/family.rs` checks a SetUp's answer names
  the owned forge's number; add `a_set_up_answer_names_its_forge` there
  if no existing test shows it).

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md` and
  `temper/docs/plan/playground-reset-r2/notes/scope.md`.
- `temper/docs/plan/playground-reset-r2/notes/contracts.md`, sections 2
  (the record, numbers, definition) and 5 (`SetUp`, `Connect`, answers
  naming the number).
- `temper/docs/design/draft/playground-reset.md`, section 3.
- `temper/docs/design/protocol/setup.md`, 6.7 (the first repository's
  intent) for what `First` means; `temper/docs/design/domain/forge.md`,
  4.1 (the rows and the offer).
- `tasks/forge-families.md` for the held rows and accessors it added
  (`forge_record`, `forge_records`, `forge_next`, `kept::save_forge`,
  `kept::give_number`).

## Owns and may change

Owns (after `forge-families`; `fr-domain` and `fr-registry` follow):

- `temper-engine-domain-forge`: `src/boundary.rs` (the new types and
  variants below), `src/lib.rs` (re-exports), a new `src/records.rs`
  (stub bodies, below), `src/domain.rs` and `src/connect.rs` (the
  decisions' matches on the new shapes only), `src/tests/records.rs`
  and its `mod` line in `src/tests.rs`.
- `temper-engine-domain`: `src/boundary.rs` (`Request::ForgeMade`),
  `src/domain.rs` (the pass-throughs), `src/route/forge_route.rs` (the
  `Made` arm), `src/route/applications.rs`, `src/decision.rs` and any
  other exhaustive match the new variants force.

May change (call sites, each keeping today's behavior):

- `temper-service/src/family.rs`: the `forge::Ask::SetUp` and
  `forge::Ask::Connect` literals take `target: forge::Target::Kept(n)`
  where cl-wire's bridges computed `n` today (its first owned forge; the
  `Connect`'s `Some(n)`, or the first forge of the kind `how` gives);
  the answers' bridges (`ForgeNumber { forge: 0 }`, `Link { forge: 0,
  .. }`) take the number the domain's answer carries; Install's
  `Connecting`/`Link` take the request's own `forge`. Remove the
  comments that said 0 is "not numbered yet".
- `temper-service/src/route.rs`: an arm for `root::Request::ForgeMade
  { .. }` that does nothing, commented "the registry builds the made
  forge (fr-registry)". `fr-registry` replaces it.
- `temper-service/src/setup/public.rs`, `src/route/box_link.rs` and
  `src/tests/*.rs`: the renamed answer variants (`SettingUp { forge }`,
  `Connecting { forge }`, `Link { forge, link }`), filled with the
  forge they serve.
- `temper/tests/client` (D5's crate): only its literals of
  `forge::Ask::SetUp`/`Connect` and matches on the renamed answers, as
  in `family.rs`. One line each; say each in the report.
- `temper/tests/engine/domain/**`, `temper/tests/engine/forge/**`:
  call sites of the new shapes (`qa_first_repository.rs` and the
  others the compiler names).
- Anything `notes/scope.md` lists as forced by these items, in any
  crate.

## Steps

1. In `temper-engine-domain-forge/src/boundary.rs` (cite
   `temper/docs/design/draft/playground-reset.md, 3` at type level):

   ```rust
   /// A forge to make, composed by the service in the request's
   /// translation from the request and the machine's document; the
   /// decision numbers it (playground-reset.md, 3).
   #[derive(Clone, PartialEq, Eq, Debug)]
   pub struct NewForge {
       /// The record's projection; its `number` is set by the decision.
       pub configured: Configured,
       /// The record's definition (contracts 2): JSON, at most `definition_bytes`.
       pub definition: Box<[u8]>,
   }
   /// The forge a SetUp or Connect works on.
   #[derive(Clone, PartialEq, Eq, Debug)]
   pub enum Target {
       Kept(u16),
       New(NewForge),
   }
   // Ask, in place:
   SetUp { target: Target, org: Box<[u8]>, repository: Box<[u8]>, project: SetUpProject, playground: bool },
   Connect { target: Target, how: How },
   // Event, new:
   /// A forge the service seeds at its start under a given number: kept
   /// unless that number has a record or was given before.
   Seed { number: u16, new: NewForge },
   // Answers, in place:
   SetUpAnswer::SettingUp { forge: u16 },
   ConnectAnswer::Connecting { forge: u16 },
   ConnectAnswer::Link { forge: u16, link: Box<[u8]> },
   SetUpRefusal::Full,   // no room for another forge (forge.forges) or no number left
   ForgeRefusal::Full,   // the same, for Connect
   // Request, new, before any request naming the forge:
   /// A forge was made in this decision; its record is readable.
   Made { forge: u16 },
   ```

   `InstallAnswer` stays as it is. The wire has no `full`: the
   service translates `SetUpRefusal::Full` as it translates `Busy` and
   `ForgeRefusal::Full` as `Busy`; say so in the report.

2. In `src/domain.rs` and `src/connect.rs`: a `Target::Kept(n)` goes
   on exactly as `forge: n` does today; a `Target::New(_)` is refused
   (`SetUpRefusal::NotOffered`, `ForgeRefusal::NoSuchForge`).
   `Event::Seed` is ignored (no output). Answers carry the forge's
   number.

3. A new `src/records.rs` (`pub(crate) mod records;`, cited at module
   level as `temper/docs/design/draft/playground-reset.md, 3`):

   ```rust
   /// Make a forge's record from `new`: its number given, its record and
   /// the numbers saved, its projection installed, `Request::Made` pushed.
   pub(crate) fn make_forge(
       d: &mut Domain,
       new: NewForge,
       first: Option<FirstIntent>,
       out: &mut Queue<Request>,
   ) -> Result<u16, Full>; // stub: Err(Full), with #[expect(dead_code, reason = "fr-domain makes forges")]
   ```

   Use the queue and full types `kept.rs` uses. And on `Domain`, real
   now:

   ```rust
   /// Install a recorded forge's projection (the service's, at its start).
   /// False when no record of `configured.number` is held.
   pub fn configure(&mut self, configured: Configured) -> bool;
   ```

   It inserts into `configured` (the map today's constructor fills) and
   the forge's identity `(n, u32::from(n))` into `identities` when none
   is there. The constructor's `forges` and `identities` arguments stay
   as they are.

4. In `temper-engine-domain`: `Request::ForgeMade { forge: u16 }` in
   `src/boundary.rs`, routed from `forge::Request::Made` in
   `src/route/forge_route.rs` as its neighbours are (`super::now`);
   `src/decision.rs` and `src/domain.rs` match it with the neighbours
   that need no answer (`StopGroup`). In `src/domain.rs`:

   ```rust
   pub fn configure_forge(&mut self, configured: forge::Configured) -> bool;
   pub fn forge_record(&self, forge: u16) -> Option<&forge::ForgeRecord>;
   pub fn forge_records(&self) -> Box<[&forge::ForgeRecord]>; // number order
   ```

   each passing to the forge child as `forge_connection` does.

5. Call sites (Owns and may change), then the tests in Goal.
6. `cargo fmt`; `heavy cargo clippy -p temper-engine-domain-forge -p
   temper-engine-domain -p temper-service --all-targets -- -D warnings`
   (and any other package edited); focused tests, one command at a time:
   `heavy cargo nextest run -p temper-engine-domain-forge -p
   temper-engine-domain`, then `-p temper-service`, then `-p
   temper-engine-domain-world -p temper-engine-forge-world`, then `-p
   temper-client-world` if you edited it; `python3 -I -B
   scripts/old_forms.py` from `temper/`.
7. Commit (types, then call sites, then tests) and push.

## Report

The standard report, plus: each call site edited outside the owned
files; the queue and full types `make_forge` takes; how the service
maps `Full` onto the wire.
