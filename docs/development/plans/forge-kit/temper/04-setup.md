# Session 04: temper on `skein-forge-setup`

Put temper on skein's reconcile kit: supply temper's values to it, check them
in step code before every command, `temper serve` included, and delete the
copied machine. Three increments: the repin and the kit's limits, validators
and refusal words (04.1); the owner's values and the check for every command
(04.2); the shell, the world and the deletions (04.3).
Needs at start: **skein 04.1** on skein's `origin/main` (Given, Place, Refused,
Item, Limits, `Reconcile::new(&Given)`, `given::{check,login,email}`). Skein
04.2 may or may not have landed: every table row below is tested on a
constructed `Refused`, so none depends on it.

## Read first

- skein `docs/design/forge-setup.md` 4.1 and 6; temper
  `docs/design/protocol/setup.md` 5, 8, 11 and 13.1.
- This plan's README (sessions, Rules, Done when), `code-organization.md`
  1–3 and 5–8, `decisions.md` 2, 3, 13, 14, 18 and answers 23–29.
- `skein/04-setup.md` (04.1's Given/Place/Item shapes, 04.2's refusals, the
  “temper's side” lines); findings `base-temper-04.md`, `base-runs.md`,
  `trace-setup.md`, `survey-04-setup.md`.
- skein `docs/foundation/programming-model.md` 3 and 10, `testing-strategy.md`
  2.1, 2.5, 8 and 10; temper `docs/development/workflow.md` 2.

## What the design asks

- forge-setup 4.1: the owner supplies every value; the kit derives only the
  marker `temper:bot` from the word `temper`.
- forge-setup 6, setup 5, 11, 13.1 and answer 29: step code maps the setup to
  the kit's `Given` and runs the kit's check for every command; the host
  supplies items, HTTP, files, waiting and locking. A refused Place is named
  by setup.md 5's path; no value or secret is printed.

## Choices this plan makes

- The owner module is `temper_forge_setup::given`: `refusal` (04.1) and
  `values` (04.2). `check::checked` stays where it is and runs the kit's check.
- The kit's `Refused` carries a Place and no words. The words are temper's,
  each the reader's own at the same path (table, 04.1), so a refusal reads
  alike whichever check finds it.
- Paths and counts are **at the base** (temper dd12b8e, skein 064c55f). A Size
  or count is "estimated" unless `findings/base-temper-04.md` records it (the
  931 machine lines, the 13 moved tests, 236 -> 230 tests, the times).

## Rules that bind it

- The repin is its own first commit: `cargo update -p skein-lib`, then
  `cargo check --workspace --all-targets`; it repairs all intervening
  skein breakage (at the base the check fails on `Op::Lock` in
  temper-worker-protocol-git, findings). On a Cargo.lock conflict take
  `origin/main`'s and update again; never merge it by hand.
- No re-export or alias of a moved type: every user names `skein_forge_setup`.
  Moved tests keep names, budgets and times; no moved part is rewritten.
- A Size counts added lines with tests: `git diff --numstat origin/main
  -- . ':!Cargo.lock'`, first column summed. At the base it prints nothing.

## Increments

### 04.1 Repin, take the kit's limits and validators, name its refusals
Scope: setup 5 and 11; forge-setup 6. The copied machine stays until 04.3.
Needs: skein 04.1 on skein's `origin/main` (README table; it needed temper 01.2).
Files: root `Cargo.toml` (one `skein-forge-setup` git entry beside
  `skein-lib`), `Cargo.lock` (regenerated); `crates/temper-forge-setup/Cargo.toml`;
  in `crates/temper-forge-setup/src/`: new `given.rs`, `tests/given.rs`;
  changed `{lib,tests,limits,setup,check,problem}.rs`, `tests/{limits,read}.rs`.
Shape: commit 1 is the repin alone. Then `limits.rs` deletes `Limits`, `Unfit`
  and `check`; `small` and `apply` name `skein_forge_setup::limits::Limits`
  and `Refused` stays. `setup.rs` deletes its `login` and `email`; `account`
  and `strings` call `skein_forge_setup::given::{login, email}`, and
  `Setup::read` takes the kit's Limits. `check::checked` runs the kit's
  `limits::check`, even for an empty setup, and adds `given::refusal(0, why)`
  through a new `Problems::extend`; a failed pair is one line, in the kit's
  order. `problem.rs` adds `What::Own(&'static [u8])`, rendered as its label,
  `: invalid form` and a newline, with no path (pushed under `Path::root()`).
  `given::refusal` (sketch A) is an exhaustive `match` on Place, no wildcard
  arm (clippy denies it), reading `index` only for the entry Places. `Setup::
  entry` is by forge number, so `refusal` takes the entry's position.
  Forced edits in retained code: `tests/limits.rs` calls the deleted
  `limits::check`: it calls `skein_forge_setup::limits::check`, same
  assertion; `tests/read.rs`, `an_entry_past_the_orgs_limit_is_refused`,
  builds the deleted `limits::Limits { orgs: 1, ..limits::small() }`: it names
  `skein_forge_setup::limits::Limits`.
  Every Place the kit's check returns (13), as temper names it: 15 rows, the
  two pair refusals (`other` set) being two more. `L` is
  `application.limits.overrides.`, `i` the entry's position, `j` the org's:

  | Place (other) | Path temper names | Words |
  |---|---|---|
  | BotLogin | `application.setup[i].bot.login` | invalid form |
  | BotEmail | `application.setup[i].bot.email` | invalid form |
  | Orgs | `application.setup[i].orgs` | past its bound |
  | Org(j) | `application.setup[i].orgs[j]` | invalid form |
  | OrgOwner | `application.setup[i].mode.owned.person.login` | invalid form |
  | OrgsLimit | `L`+`setup.orgs` | outside its type |
  | Deadline | `L`+`setup.deadline_ms` | outside its type |
  | Backoff | `L`+`setup.backoff_ms` | outside its type |
  | BackoffMax | `L`+`setup.backoff_max_ms` | outside its type |
  | Backoff (BackoffMax) | `L`+`setup.backoff_ms` | must agree with `L`+`setup.backoff_max_ms` |
  | BackoffMax (Deadline) | `L`+`setup.backoff_max_ms` | must agree with `L`+`setup.deadline_ms` |
  | Word | none: `temper's word` | invalid form |
  | AdminName | none: `temper's admin name` | invalid form |
  | ApiName | none: `temper's API token name` | invalid form |
  | GitName | none: `temper's git token name` | invalid form |

  The last four are temper's own constants, not the setup's: no setup field
  is invented and no supplied content is printed. The words are the reader's
  at the same path (it refuses a bad login, email, org name or count first,
  and `apply` a zero limit), so through `checked` only the two pairs reach
  the kit's check; the rest are tested on constructed values.
Test shape: one new unit, `each_kit_place_names_its_temper_path`, in the new
  `tests/given.rs`: 15 rows, one per table row, entry Places at `i` = 1 and
  `Org(2)`, asserting the full line; <20 ms. Reference, three rows written
  out, the other twelve the table's with the same shape:

  ```rust
  #[test]
  fn each_kit_place_names_its_temper_path() {
      let l = "application.limits.overrides.";
      for (place, other, line) in [
          (Place::Org(2), None, "application.setup[1].orgs[2]: invalid form\n".to_owned()),
          (Place::Backoff, Some(Place::BackoffMax),
           format!("{l}setup.backoff_ms: must agree with {l}setup.backoff_max_ms\n")),
          (Place::Word, None, "temper's word: invalid form\n".to_owned()),
      ] {
          let problems = crate::given::refusal(1, Refused { place, other });
          assert_eq!(&*problem::lines(&problems), line.as_bytes(), "{place:?}");
      }
  }
  ```

  The three limits tests keep their names and bytes (`a_backoff_past_its_maximum_names_both_limits`
  still prints both paths once); `tests/steps.rs` and `tests/members.rs` are untouched here.
Proof: unit; `cargo nextest run -p temper-forge-setup`: 99 passed (98
  at the base, 0.043 s per skein/04-setup.md), the new test <20 ms, suite <1 s.
Proof: joint; `cargo nextest run -p temper-forge-setup
  -p temper-forge-setup-shell -p temper-box -p temper
  -p temper-forge-setup-reconcile-world`: origin/main's count at the step's
  start plus 1 (237 from the base's 236, 0.989 s), about 1 s.
Proof: joint; `scripts/check.sh`: origin/main's focused count at
  the step's start plus 1, and its fuzzy count, passed (1880 / 50 from the
  base's 1879 in 6.770 s and 50 in 26.715 s), clean clippy,
  focused <15 s, fuzzy <60 s.
Don't: map an Entry to Given (04.2); change the shell, the world or the
  binary; delete the copied machine or its tests; add a setup field for the
  word or the three names; derive any value.
Size: about 0 lines moved, 125 new (estimated: `given.rs` 45, `tests/given.rs`
  45, `problem.rs` 15, `check.rs` 8, other edits 12; Cargo.lock aside).
Removes: `Limits`, `Unfit`, `check` (limits.rs) and `login`, `email`
  (setup.rs): about 50 lines (estimated).

### 04.2 The owner's values, and the check for every command
Scope: forge-setup 4.1; setup 8 and 13.1; answer 29.
Needs: 04.1 of this session.
Files: `crates/temper-forge-setup/src/{given,check}.rs`, `src/tests/given.rs`,
  `src/tests/{fixtures,limits,members}.rs`; binary `crates/temper/src/setup.rs`
  and `crates/temper/tests/startup.rs`.
Shape: `given::values(entry, listener, limits) -> Given` (sketch A) copies the
  forge, bot login and email, orgs and limits; the owner is `Some(person
  login)` for an owned entry and `None` for a managed one; the listener is as
  given. Word `temper`, admin `temper-admin`, API `temper-api` and git
  `temper-git` are four literals of temper's own, with the setup 5 table; no
  bot login is derived. `check::checked` takes `listener: Option<SocketAddr>`
  in place of `webhooks: bool` and passes `listener.is_some()` to
  `check::members`, which keeps its bool. After a clean limits check and
  parse it maps each entry, calls the kit's `given::check` and adds
  `given::refusal(position, why)`; if the limits check failed it skips the
  per-entry check, so a pair is not emitted again per entry. Binary
  `setup::check` passes `named.webhooks`' socket through `super::address`
  (named has already required a loopback address), for every command.
  Forced edits in retained code: `tests/limits.rs` (two `checked` calls) and
  `tests/members.rs` (`refused` and `refused_webhooks`) pass `checked` the bool
  `false`/`true`: they pass `None` and `Some(fixtures::listener())`; `fixtures.rs`
  adds `listener()`, 127.0.0.1:9898, for these and for `tests/given.rs`.
Test shape: three units in `tests/given.rs`, reusing `fixtures::{TWO, setup,
  document, members}` and `limits::small`, each <20 ms, no world:
  `a_managed_setup_maps_every_given_value`: forge 1, bot `temper`/
  `temper@example.invalid`, org `acme`, owner None, listener None, the four
  literals, limits 8 orgs/120 s/100 ms/5 s (the shipped profile), and the
  kit's `given::check(&given)` is `Ok(())`;
  `an_owned_setup_maps_every_given_value`: forge 2, org `work`, owner
  `Some(ada)`, listener 127.0.0.1:9898, same literals, check `Ok(())` (these
  two prove the kit accepts the four literal names);
  `a_refused_limit_pair_keeps_its_order_and_is_printed_once`: `checked` with
  TWO (two entries) and with `[]` (none), overrides `setup.backoff_ms`
  121000 and `setup.backoff_max_ms` 120000, which break both relations: the
  one line `application.limits.overrides.setup.backoff_ms: must agree with
  application.limits.overrides.setup.backoff_max_ms\n`, so a per-entry repeat
  would show as three lines.
  In the binary's `tests/startup.rs`, `every_command_checks_setup_before_reading_files`,
  on `owned_example` (its files are absent, so a read would fail first) with
  `value["application"]["limits"]["overrides"]["setup.backoff_ms"]` = 6000
  written into its config.json, as `a_box_maximum_backoff_below_its_base_is_refused`
  sets the box's: `temper::prepare_in` returns the pair line prefixed
  `temper serve: `, `temper::sign_in::account(bytes, 1)`'s `Refused::line()`
  `temper sign-in: `, and `temper::run(Command::Box { .. })` `temper box: `.
  The four forge rows spawn the binary, as `support::owned` and `tests/forge.rs`
  do: `support::owned::refused(dir, verb, expected)` for `prepare`,
  `reconcile` and `run` (its else arm is `run forgejo`), and
  `support::owned::command(dir, "runner").output()` for `run runner` (`refused`
  has no runner arm); each exits 1 with stderr `temper forge <verb>: forge 1: `
  and the pair line (verb `run` for both run rows). Seven rows, no Forgejo
  started: <0.5 s.
Proof: unit; `cargo nextest run -p temper-forge-setup`: 102 passed (99
  after 04.1), each new test <20 ms, suite <1 s.
Proof: joint; `cargo nextest run -p temper-forge-setup
  -p temper-forge-setup-shell -p temper-box -p temper
  -p temper-forge-setup-reconcile-world`: 04.1's count plus 4 (241 from
  the base's 236 and 04.1's 237; the binary 59, from 58), about 1.5 s.
Proof: joint; `scripts/check.sh`: 04.1's focused count plus 4
  (1884 from the base's 1879), and its fuzzy count (50), passed, clean clippy, focused <15 s, fuzzy <60 s.
Don't: touch the shell, the world or `place`; remove the copied machine; add a
  value the kit derives; check in the shell what step code checks.
Size: about 0 lines moved, 170 new (estimated: `values` 25, `checked` 15,
  `tests/given.rs` 65, `startup.rs` 45, binary 8, other edits 12).
Removes: nothing.

### 04.3 Connect the shell and the world, delete the copy
Scope: setup 11; forge-setup 6. The Item-to-file mapping stays temper's.
Needs: 04.2 of this session.
Files: `crates/temper-forge-setup/src/{place,lib,tests}.rs`,
  `src/tests/{given,fixtures}.rs`; deleted `src/reconcile.rs`,
  `src/reconcile/{hooks,orgs,tokens}.rs`, `src/tests/reconcile.rs`; shell
  `Cargo.toml`, `src/{reconcile,exchange}.rs`, `tests/{reconcile,exchange}.rs`,
  `tests/support/mod.rs`; world `tests/forge-setup/reconcile/Cargo.toml`,
  `src/{world,exchange,fixtures}.rs`, `tests/reconcile.rs`; binary
  `crates/temper/src/forge.rs`. Read-only users: binary `src/{lib,sign_in,box_command}.rs`,
  `tests/{forge,process}.rs`, `tests/support/owned.rs`; temper-box (Step only).
Shape: `place::item(Item) -> File` is five arms, exhaustive. Shell `Settings`
  (sketch A) drops `forge`, `deadline`, `backoff`, `backoff_max`, `entry` and
  `listener` for `setup_index` and `given`; `reconcile` first builds
  `Reconcile::new(&settings.given)`, before its io checks and the kernel, and
  a refusal is `Refused::Setup`, whose `line` is `temper forge reconcile:
  forge N: ` and the problem lines. Read and Write map `Item` through
  `place::item`, then `bound` and `file_path`; `file_path` returns
  `settings.admin_token` for AdminToken always (the binary chose it:
  managed -> the setup's, owned -> `directory/admin_token`), so the shell no
  longer reads `Mode`. Imports in shell `reconcile.rs`, `exchange.rs`,
  `tests/exchange.rs` and in the world name `skein_forge_setup::reconcile`.
  Binary `reconcile_settings` keeps its signature: `Setup::entry` returns no
  index, so `setup_index` is `checked.setup.entries.iter().position(|e| e.forge
  == number)`, then `given::values(entry, webhook_listener(named)?, &checked.limits)`;
  it maps and checks nothing itself. World: `fixtures::entry(owned)` becomes
  `fixtures::given(owned)` over `given::values`; `World` keeps `given`
  (`missing_org` sets its orgs, `listening` its listener), builds
  `Reconcile::new(&self.given).expect("fixture admitted")`, maps Items to its
  File store; `tests/reconcile.rs` re-points its three
  `use ...::reconcile::{Failure, Stage, Why}`.
  Deleted from `tests/fixtures.rs`: `Run` (its three `impl Run` blocks),
  `ADMIN`, `admin`, `person`, `team_spec`, `hook_row`, `subscribed`. Retained:
  `secret` (`tests/upgrade.rs` and `failed()` call it), `run_command`,
  `runner_added`, `failed`; `pub mod reconcile;` and `mod reconcile;` leave
  `lib.rs` and `tests.rs`.
Test shape: the 13 moved names leave (now skein's), each 0.003–0.007 s at the
  base, listed in `base-temper-04.md`; every other name stays. New:
  `each_kit_item_names_its_temper_file` in `tests/given.rs`, five Item rows
  to the five File variants of the same name, <20 ms; shell
  `invalid_given_values_are_refused_before_io` in `tests/reconcile.rs`:
  `support::settings(..)` with `given.bot_login` empty and `io.output` 0
  returns `Refused::Setup`, line `temper forge reconcile: forge 1:
  application.setup[0].bot.login: invalid form\n`, with no peer served and no
  kernel opened, <20 ms. `support::settings` builds its Entry, with one org
  `acme` for the empty list (skein 04.2's admission), through `given::values`
  and keeps its 2 s, 50 ms and 100 ms waits; the two deadline cases set
  `settings.given.limits.deadline` to 300 ms and 200 ms.
Proof: unit; `cargo nextest run -p temper-forge-setup
  -p temper-forge-setup-shell`: 115 passed (90 and 25; 98 and 24 at the
  base), each new test <20 ms.
Proof: joint; `cargo nextest run -p temper-forge-setup
  -p temper-forge-setup-shell -p temper-box -p temper
  -p temper-forge-setup-reconcile-world`: 04.2's count less 11 (230 from
  04.2's 241; 236 at the base, 0.989 s), about 1 s; focused worlds <0.5 s each.
Proof: world; `cargo nextest run -p temper-forge-setup-reconcile-world
  --profile fuzzy`: 1 passed (0.493 s at the base), under 5 s.
Proof: joint; `scripts/check.sh`: 04.2's focused count less 11
  (1873 from the base's 1879), and 50 fuzzy, passed, clean clippy, focused <15 s, fuzzy <60 s.
Proof: process; `TEMPER_FORGEJO=$WS/tmp/temper/forgejo/v17.0.0/forgejo-17.0.0-linux-amd64
  TEMPER_FORGEJO_RUNNER=$WS/tmp/temper/forgejo/runner-v13.2.0/forgejo-runner-13.2.0-linux-amd64
  scripts/proof-one-process.sh` (README Rules), once here, its
  production setup path having changed: its count at the step's start passed (19 at
  the base), <5 minutes; 145.694 s at
  the base, alone (`findings/base-runs.md`).
Don't: delete the crate, its member or dependency (preparation, setup and
  Step stay); move the world; change the box's Step; map or check in the
  shell; recreate skein 04.2's behaviour.
Size: about 0 lines moved, 150 new (estimated: shell 35, shell tests 30,
  world 30, `place` and its test 25, binary 12, imports 18).
Removes: the machine, 931 lines (findings); `tests/reconcile.rs` (the 13
  tests, 210 lines); the fixtures above (126 lines counted at the base,
  findings; they leave no import unused).

## Interface sketches

A. Owner mappings and the shell's boundary. Given, Refused, Item are the
kit's; `values` and `refusal` stay in step code; Settings and Refused stay in
the shell. `refusal`'s index is the entry's position.

```rust
// temper_forge_setup::given
pub fn values(entry: &Entry, listener: Option<SocketAddr>, limits: &Limits) -> Given;
pub fn refusal(index: u32, why: Refused) -> Problems;
// temper_forge_setup::check; Checked keeps setup: Setup, limits: Limits.
pub fn checked(document: &Document, at: u32, members_value: &Members,
    listener: Option<SocketAddr>, overrides: &[(Box<[u8]>, u64)]) -> Result<Checked, Problems>;
// temper_forge_setup::place
pub fn item(item: skein_forge_setup::item::Item) -> File;
// temper_forge_setup_shell::reconcile
/// Owner's checked values and host resources; consumed by one reconcile run.
pub struct Settings {
    pub setup_index: u32, pub given: skein_forge_setup::given::Given,
    pub address: SocketAddr, pub host: Box<[u8]>, pub user_agent: &'static str,
    pub directory: PathBuf, pub admin_token: PathBuf,
    pub token_bytes: u32, pub hook_secret_bytes: u32,
    pub http: skein_http::client::Limits,
    pub documents: skein_forge::forgejo::limits::Limits, pub io: skein_io::Limits,
}
/// Terminal host refusals, sent to the command's diagnostic renderer.
pub enum Refused {
    Unanswered { forge: u16, deadline: skein_lib::Duration }, // No answer by the deadline.
    Limits { forge: u16, name: &'static str },                // HTTP cannot fit the io bounds.
    File { forge: u16, file: File, why: String },             // An item could not be read or written.
    Stopped(skein_forge_setup::reconcile::Failure),           // The kit ended at a step.
    Setup { forge: u16, problems: Problems },                 // New: the constructor's refusal.
}
```

## Done when

Commands run from temper's root; the Proofs above run once and do not repeat.

- `cargo check --workspace --all-targets` prints Finished and exits 0.
- `rg -n 'pub mod reconcile|pub struct (Limits|Unfit)|^pub fn check|^fn (login|email)' crates/temper-forge-setup/src/{lib,limits,setup}.rs`
  prints nothing (exit 1): no local machine, limits check or validators (6
  lines at the base: lib.rs:11, limits.rs:5, 19 and 57, setup.rs:99 and 110).
- `rg -l --sort path 'skein_forge_setup::' crates/temper-forge-setup/src crates/temper-forge-setup-shell/src`
  prints exactly ten paths: `crates/temper-forge-setup-shell/src/{exchange,reconcile}.rs`
  and `crates/temper-forge-setup/src/{check,given,limits,place,setup}.rs` and
  `crates/temper-forge-setup/src/tests/{given,limits,read}.rs` (nothing at the
  base, exit 1).
- `rg -n 'webhooks: bool|listener: Option<SocketAddr>' crates/temper-forge-setup/src/check.rs`
  prints two lines: `members`' `webhooks: bool` (line 60 at the base) and
  `checked`'s `listener: Option<SocketAddr>` (the base's second is `webhooks:
  bool`, line 137).
- `rg -n 'Reconcile::new' crates tests -g '*.rs'` prints two lines, in
  `crates/temper-forge-setup-shell/src/reconcile.rs` and
  `tests/forge-setup/reconcile/src/world.rs`: the hosts build the kit's machine
  (four lines at the base; the two in `tests/fixtures.rs` go with `Run`).
- `python3 -c 'from pathlib import Path; root=Path("crates/temper-forge-setup/src"); paths=[root/"reconcile.rs",*[root/"reconcile"/(name+".rs") for name in ("hooks","orgs","tokens")]]; remaining=[str(p) for p in paths if p.exists()]; print("reconcile copies:",len(remaining)); print("\n".join(remaining)); raise SystemExit(bool(remaining))'`
  prints `reconcile copies: 0` and exits 0 (4 at the base).
- `rg -n -U 'pub\s+use\b[^;]*(Given|Reconcile|Need|Event|Reply|Failure|Stage|Why|Item|Place|Limits|Unfit|Refused)\b|(?m:^\s*(pub(\([^)]*\))?\s+)?type\s+\w+\s*=\s*([A-Za-z0-9_]+::)*(Given|Reconcile|Need|Event|Reply|Failure|Stage|Why|Item|Place|Limits|Unfit|Refused)\b)|skein_forge_setup[^;]*\bas\s+\w+' crates/temper-forge-setup/src crates/temper-forge-setup-shell/src crates/temper/src`
  prints nothing (exit 1): no moved-type re-export, alias or renamed import.
