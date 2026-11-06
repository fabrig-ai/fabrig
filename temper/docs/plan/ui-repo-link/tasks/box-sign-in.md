+++
id = "box-sign-in"
part = "box"
kind = "feature"
needs = ["provider-public", "repo-link"]
files = [
  "temper/crates/temper-init/**",
  "temper/crates/temper/src/init.rs",
  "temper/crates/temper/src/forge.rs",
  "temper/crates/temper/src/forge_sign_in.rs",
  "temper/crates/temper/src/commands.rs",
  "temper/crates/temper/src/box_command.rs",
  "temper/crates/temper-forge-setup/src/machine.rs",
  "temper/crates/temper-forge-setup/src/sign_in.rs",
  "temper/crates/temper-forge-setup/src/ini.rs",
  "temper/crates/temper-forge-setup/src/spec.rs",
  "temper/crates/temper-forge-setup/src/step.rs",
  "temper/crates/temper-forge-setup/src/tests/**",
  "temper/crates/temper-box/src/**",
  "temper/crates/temper-service/src/clients.rs",
  "temper/crates/temper-service/src/clients/forge_sign_in.rs",
  "temper/crates/temper-service/src/route/box_link.rs",
  "temper/crates/temper-service/src/startup/application.rs",
  "temper/crates/temper-service/src/startup/application/sign_in.rs",
  "temper/crates/temper-service/src/tests/forge_sign_in.rs",
  "temper/crates/temper-client-view/src/forge_sign_in.rs",
  "temper/crates/temper-client-view/src/forge/forges.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-shell-web/src/lib.rs",
]
effort = "high"
minutes = 170
+++

# A box's owned Forgejo signs its person in through temper, by itself

## Goal

On a fresh `temper box`, with no command for the person: `init --box` makes the provider's key and writes the provider, the owner's address and the client; the owned Forgejo's `app.ini` has `ACCOUNT_LINKING = auto`; the last step of its set-up adds the `temper` source; then the page's facts offer sign-in, and "Open Forgejo ↗" and a repository's link go through temper. Any failure of this leaves the box running and the set-up succeeding, with plain links, one line in `box.log` and a line on the Forges row. Read `sign-in.md`, 9.9 whole first, for its reasons bind you.

`box-proof` runs it against a real Forgejo. Here the worlds are the steps' and the box's. One real-Forgejo check is yours too: acceptance 5.

Acceptance:

1. **The document.** `init --box` writes what 9.9 shows: `owners[0].subject` the person's `email`, `clients.provider` with `key` `provider.key`, the public client `forgejo`, the owned `listen`'s redirect and `addresses: [1]`, and `machine.owned.sign_in` `"forgejo"`. A bare `temper init` changes in no byte. Its documents still start: `temper-init`'s tests and `temper-service`'s start of a box's document.
2. **The key.** Made before the document, as `forge_sign_in::make_missing(path, true)` makes it: 0600, PKCS#8 PEM, 3072 bits; a key there is left as it is and `provider.key` is not among init's refusals. With no `openssl`, or if it fails: `clients.provider` and `sign_in` `null`, init succeeds, `init --box` says one line, and the box's `box.log` opens with `First run: no sign-in through temper: WHY` (`setup.md`, 7.2). A test runs init with a `PATH` that has no `openssl`.
3. **The claims.** With `application.sign_in` `null` and `machine.owned.sign_in` set, the printed link signs in `person.login`, named `Owner`; otherwise `owner` and `Owner` as today. A service-world story takes jig's provider all the way for a box's document: its `preferred_username` is the person's login, its `email` the person's address, `email_verified` true, `sub` `1:ADDRESS`.
4. **Refused members,** each a test naming its path: `machine.owned.person.login`, folded, `temper`, `temper-admin` or `work`, and its `email`, folded, the bot's or the set-up's admin's, when `sign_in` is set; `machine.owned.sign_in` naming no client of the provider; a `sign_in.members` login, folded, `temper`, whatever the forges. A test pins the refused bot's login and address to `temper-engine-domain-forge`'s `bot()`, so that they can't part.
5. **`ACCOUNT_LINKING = auto`** in `app.ini` for this arrangement alone (golden beside today's); a managed Forgejo's printed settings and `temper serve`'s owned `app.ini` keep `login` byte for byte. **`forgejo admin auth add-oauth`** with no `--secret` on the builder's real Forgejo (`skein_throwaway_forgejo`, as `temper/crates/temper/tests/support/forgejo_sign_in.rs` does): if it registers and the token exchange succeeds, the design stands. If not, the option below.
6. **The step.** `temper forge source DIRS N` lists the sources and adds `temper` unless it's listed, with `--work-path` and `--config`, and no secret anywhere; under a deadline; says `source added` or `source failed WHY`; exits 0. Running it twice leaves one source. It's the last of the owned group, only when `machine.owned.sign_in` is set.
7. **Never a welcome failure.** The box's machine: a step whose ending is ignored (`source`) advances its group on any exit, a non-zero one or a signal or "did not start", never `failed`, and is never restarted. A test in `temper-box/src/tests/machine.rs` for each ending, the group `started` and the box running. `source FORGE added|failed WHY` goes down the pipe before `started`, with the box's own line for a step that ended saying nothing (`setup.md`, 7.1); `box.log` has `<label>: no sign-in through temper: WHY` once. A `start` for a standing group says its last line again.
8. **Serving.** `clients/forge_sign_in.rs` holds the box's entry, for the kept forge of mode owned, whatever its number, under 9.7's check, and adds `Unserved::NoSource`: not served until `source FORGE added`; named once in the log as the other reasons are. With a source failed, or no key, the `temper-forge-sign-in-off` meta. The metas change as 9.7 says (`refresh`); `ETag` follows.
9. **The Forges row** says `forge.md`, 5.4's line while the off fact holds, for that row alone, in the browser; a test with `Forges` page facts in `tests/forges.rs`. `temper-client-shell-web` reads the meta beside the others.
10. Nothing about `temper ctl` or a coding run's credentials changes; say in the report what you checked for that.

## Inputs

- `temper/docs/design/protocol/sign-in.md`, 2.3, 9.3, 9.4, 9.7, 9.9; `setup.md`, 6.2, 6.8, 7.1, 7.2, 8.2 and section 8's `owned.*` table; `configuration.md`, 3 and 6.
- `temper/docs/design/client/forge.md`, 5.4, *A box's Forgejo that can't sign in through temper*.
- `temper/docs/plan/ui-repo-link/notes/contracts.md`, `notes/common.md`.

## Owns and may change

- Owns: this spec's `files`.
- Reader edits: `OwnedMachine` literals (`temper-service/src/system.rs`, `src/tests/setup.rs`, `src/tests/fixtures.rs`) gain `sign_in: None`. `Clients::new`'s callers and `Entries::new`'s, arguments alone. `temper/crates/temper-engine-domain-forge/src/make.rs`: `bot()`'s words may become a `pub const`, with no change of value. `temper-forge-setup`'s `FirstSignIn`: tests and golden readers (`temper/crates/temper/tests/suite/forge.rs`). Help: `temper/docs/help/box.md` gains a line on the link, within its bound (`notes/common.md`); `docs/help/reference/*` regenerated; `temper/crates/temper/src/tests.rs`' command table if it lists verbs. Test fixtures' documents that were built by `init --box` and assert `owner` or `owner-a` of a box: `temper/crates/temper/tests/**`, `temper/crates/temper-init/src/tests/**`; gallery and journeys where they print a box's login: regenerate.
- **A change of meaning runs its readers' tests:** the printed link's login on a fresh box changes. Run `temper-service`, `temper-box`, `temper-init`, `temper-forge-setup`, `temper-client-view`, and `temper`'s focused tests.
- Options, each pre-approved:
  - **If Forgejo refuses a source with no secret** (acceptance 5): pass the fixed `--secret public-client`, a named constant, in the argument, since it isn't a secret; the issuer already ignores it. Change 9.9's *The client is public* in two lines to say it isn't a secret and why that's safe. Say so in the report.
  - **If key-making pushes a tier over its budget** (a fresh box per test, about a second of CPU each: measure it): test support seeds a fixture key in the data directory, for init leaves one there. Only the fixture's published test key (`skein/docs/design/oauth-issuer.md`, 9). Report the measurements.
  - `fill` substitutes quoted `"$name"`s with JSON: use it for `$provider` and `$owned_sign_in` with raw JSON values. If that needs a new `Value`, add it.
  - Group stage of the `source` step: `Stage::Run`. If the client's row or a test reads `step N run` as "Forgejo is starting", a stage of its own. Default: `Run`.
  - `Fetched`'s path in `temper-box/src/host.rs` (`control.fetched`) is the model of the relay; a second slot beside it is fine.
  - `forgejo admin auth list`'s output is a table: match the second column exactly against `temper`, with a test of real output as the fixture. If the list can't be read, try `add-oauth` and report its refusal as the failure.

## Steps

Failing tests first, at the lowest tier: the document (`temper-init`), the refusals (`temper-forge-setup`, `temper-service`'s start), `app.ini` (`golden.rs`), the machine's ending, the serving rule. Sketches:

```rust
// temper-forge-setup/src/sign_in.rs
pub enum FirstSignIn { MakesAccount, LinksOnly, /** A box's owned Forgejo: makes or links by name and address, no password (sign-in.md, 9.9). */ LinksByName }
pub fn refused_person(login: &[u8], email: &[u8]) -> bool;

// temper-forge-setup/src/step.rs
impl Step { /** Its group goes on however it ends (setup.md, 7.2). */ pub fn ending_ignored(self) -> Self; }

// temper-forge-setup/src/spec.rs
pub fn source_list(owned: &Owned, place: &Place) -> Spec;
pub fn source_add(owned: &Owned, place: &Place, client: &[u8], discovery: &[u8]) -> Spec;

// temper-box/src/pipe.rs
pub fn source(forge: u16, outcome: &Source) -> Box<[u8]>;

// temper-service/src/route/box_link.rs
Out::Source { forge: u16, added: bool }   // WHY goes to the log, never the page

// temper-service/src/clients/forge_sign_in.rs
pub enum Unserved { NotKept, NotForgejo, Origin { origin: Box<[u8]> }, NoRoom, NoSource }
```

The discovery URL is built as `forge_sign_in.rs`'s `run` builds it. The prepare path, in `temper/crates/temper/src/forge.rs`'s `.with_sign_in(...)`: `Some(FirstSignIn::LinksByName)` when `machine.owned.sign_in` is set and the entry is the owned kind, otherwise as today.

Then the narrow tests, `cargo fmt`, old forms for temper, clippy naming each package you changed, `heavy cargo nextest run` of those and of their direct dependents, `python3 -I -B tools/citations/citations.py --base origin/main`. Drop the **Not yet** entries this closes from `sign-in.status.md`, `setup.status.md` and `forge.status.md`. Commit and push as you reach each acceptance item: this task is long.

## Report

As the implementer's text says, with: what `add-oauth` did with no secret; the key's cost and what you did about it; what login and name the printed link now gives a box; what you checked of `temper ctl` and a coding run's credentials.
