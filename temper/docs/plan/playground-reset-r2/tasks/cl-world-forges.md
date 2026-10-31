+++
id = "cl-world-forges"
part = "client"
kind = "feature"
needs = ["rm-domain", "cl-connect-view", "cl-welcome", "cl-tools"]
files = [
  "temper/tests/client/**",
]
effort = "high"
minutes = 85
+++

## Goal

The joined client world (`temper-client-world`: temper's domain and view
against the real engine domain) on forge records and their removal: its
own translation makes forges by `SetUp`/`Connect` with no number
(`Target::New`), sends `RemoveForge`, and answers the removal's kept acts
as the box and the service would (draft 3, 4, 6.4); its stories show
the client's Forges, Connect a forge, Remove confirmed and Start over
against the engine, end to end.

Acceptance (`temper-client-world`, `tests/suite/`):

- `forges.rs`: `connect_a_forge_makes_a_record_numbered_from_one`
  (the welcome's at-once set up: the engine's record 1, the row listed,
  `finished` once its first repository is adopted);
  `a_removed_forge_leaves_forges_and_its_work_ends` (Remove confirmed on
  the row: the row goes, a change on its repository is cancelled, the
  forge's record is erased once its kept acts are answered);
  `removal_names_what_stays_for_each_kind` (owned, GitHub: the words
  drawn, against the engine's rows).
- `connect_forge.rs` / `welcome.rs`:
  `a_set_up_cut_short_is_found_unfinished_and_started_over` (a failed
  set up: the welcome opens the Forge step on "not finished"; Start over
  confirmed; the choices again; a new set up makes record 2 and reuses
  the `Default` project); `github_goes_on_with_its_kept_forge` (an App
  connected and not installed: Go on installs on the same number).
- `fuzzy/client.rs`'s `random_people_and_faults_make_each_request_once`
  passes with removals among the people's acts.
- Every other story passes, rewritten where it named a configured slot
  (`forge setup 3`, `Choice::Forge(3)`, …; each listed in the report).

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md` and
  `temper/docs/plan/playground-reset-r2/notes/scope.md`.
- `temper/docs/plan/playground-reset-r2/notes/contracts.md`, 2 and 5.
- `temper/docs/plan/playground-reset-r2/tasks/fr-ask.md` and
  `fr-domain.md` (`Target`, `NewForge`, `Event::Seed`,
  `configure_forge`, `Request::Made`/`ForgeMade`), `rm-domain.md`
  (`Ask::RemoveForge`, `RemoveAnswer`, `Event::{KeptActAnswered,
  SendKeptActs, GroupRemoved, SecretsErased}`, `Request::SendKeptAct`,
  the root's `RemoveGroup`/`EraseSecrets`), `fr-registry.md` (what a
  definition holds: the world gives `{}` where a typed world gives none),
  and the client tasks `cl-connect.md`, `cl-remove.md`, `cl-tools.md`.
- `temper/docs/design/draft/playground-reset.md`, 3, 4, 6.3, 6.4.
- `temper/docs/design/domain/testing.md`, the joined client world's
  section (`rg -n 'client world|joined' temper/docs/design/domain/testing.md`).

## Owns and may change

Owns `temper/tests/client/**` (its `src/` and `tests/`). Calls the
engine domain's public items as the `fr-`/`rm-` tasks built them; a
missing public item there is the engine's: if a story needs one, say so
in the report and keep the story to what exists (never edit the engine
crates here).

## Steps

1. **Making forges (src/translate/forge.rs, src/front.rs).** The wire's
   `SetUp` → `forge::Ask::SetUp { target: Target::New(NewForge {
   configured, definition }), .. }` with `configured` the world's owned
   Forgejo projection (as its fixtures build a configured forge today)
   and `definition` `{}`; `Connect { forge: None, how }` likewise for
   GitHub; `Some(n)` → `Target::Kept(n)`. The answers' numbers reach the
   wire (`SettingUp(n)`, `Connecting(n)`, `Link { forge: n, .. }`).
   `Request::Made`/`ForgeMade`: the world installs the forge in its fake
   forge and setup stand-ins as its fixtures install a configured one.
   The world's start configures no forge; a fixture that needs one
   kept from the start seeds it (`Event::Seed` or a restored record, as
   `fr-registry.md` says engine worlds do).
2. **Removing (src/translate/forge.rs, src/front.rs).** The wire's
   `RemoveForge` → `forge::Ask::RemoveForge`, its answer to
   `Removing`/`RemoveRefused`; the root's `RemoveGroup { forge }` answered
   by `forge::Event::GroupRemoved { forge }` and `EraseSecrets { forge }`
   by `forge::Event::SecretsErased { forge }`, each after a fault-chosen
   delay (the world's randomness), as the box and the service would;
   `SendKeptActs` at the world's engine start, as the service sends it.
3. **Stories:** the tests above, written as the suite's other stories
   (the fake person, its pages, the world's checks).
4. **Checks:** `cargo fmt`; `heavy cargo clippy -p temper-client-world
   --all-targets -- -D warnings`; `heavy cargo nextest run -p
   temper-client-world`, then its fuzzy profile once (`heavy cargo
   nextest run -p temper-client-world --profile fuzzy`).
5. Old forms from `temper/`; commit and push.

## Report

- Each story rewritten or deleted, and why.
- Any engine item a story lacked.
