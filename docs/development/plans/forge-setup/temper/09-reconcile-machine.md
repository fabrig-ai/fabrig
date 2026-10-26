# Session 09: reconcile's machine and its world

Reconcile (`temper forge reconcile`, setup.md 5) is a machine in `crates/temper-forge-setup`: from what the forge says it asks
for the one request, file or draw it needs next, and ends converged or naming its step; every request goes under the site
admin's token (designer's answer 14). Four increments build it against the fake forge's protocol layer, which learns to serve
the setup's routes: 09.1 the frame, the admin and the bot (unit); 09.2 the bot's tokens, and the world
`tests/forge-setup/reconcile` with its first story; 09.3 orgs, teams, members and hooks; 09.4 the other stories of setup.md
13.2 and the fuzzy sweep. The host (files, HTTP, the wait, the deadline) is temper 10.1's: nothing here names one.

## Read first

- setup.md: 3, 5, 6.5, 8, 9, 11, 13.1, 13.2. credentials.md: 2, 5. forge.md: 5, 9. domain/testing.md: 2.2, 4.2, 6, 6.1, 7.
- This plan: `decisions.md` 5, 13 and 19; `findings/forge-and-tiers.md` 1 and 3; `findings/configuration-and-binary.md` 2 and 6.
- temper 01 (`UserOf`, `CreateUser`, `Tokens`, `CreateToken`, `DeleteToken`; `User`, `AccessToken`), 02 (`Org`, `CreateOrg`,
  `Teams`, `CreateTeam`, `EditTeam`, `TeamMembers`, `AddMember`, `OrgHooks`, `CreateOrgHook`, `EditOrgHook`, `DeleteOrgHook`;
  `TeamSpec`, `HookSpec`) and 04.2 (`SUBSCRIBED`, `hook_target`): used by their names, both sides. temper 07.1: `setup::Entry`,
  `Setup` and `src/tests/fixtures.rs` of `temper-forge-setup`; 07.2: `place::File`.
- skein: `programming-model.md` 3, 9, 10; `testing-strategy.md` 2.1, 2.5, 6; `development.md` 5.3. temper: `workflow.md` 2.

## What the design asks

- **Setup.md 5:** six steps in order (admin; bot, made with the marker in its full name; its two tokens, by the admin's
  routes; its id; each org with its team and member; each org's hook), each read first and written only where it differs;
  listings paged with the forge's page cap; a failure names its step. Takes: the machine.
- **Setup.md 3, 8, 9; credentials.md 2, 5:** the files, `user` in decimal; the bot never a site admin, its password drawn,
  used once, never kept; no secret in an argument, a URL or a log. Takes: `Failure`, and 07.2's `place::File`.
- **Setup.md 11, 13.1, 13.2; domain/testing.md 2.2, 4.2; forge.md 5:** step tests from answers, a protocol world through bytes with
  the fake serving the setup's routes, the hook's URL. Takes: the tests, the world, `hook_url`.

## Choices this plan makes

- **One need at a time** (`MAX_OUT` 1): send, read, write, draw, then `Done` or `Stop`. The host maps `File` to a path and
  `Send`'s `token` to `Authorization: token ...`: no `Credential`, no basic auth (answer 14). `File` is 07.2's `place::File` (the
  host asks `Place::path`), of which reconcile uses `AdminToken`, `ApiToken`, `GitToken`, `User` and `WebhookSecret`: it has none
  of its own. Secrets are `skein_lib::Secret`; `Failure` holds names only.
- **The wait and the deadline are the host's** (10.1's `pause`): it resends the first request until the forge answers. The
  machine holds no timer and no limit; a later `Down`, or a status it has no rule for, stops the run at its step.
- **Randomness is the host's** (`Need::Draw(32)`): the bot's password and the hooks' secret, each 64 lowercase hexadecimal
  digits (decisions.md 5; precedent: `brief.rs:79` of temper-engine-domain-forge); a step test gives literal bytes.
- **The page cap** is read once, with `Request::ApiSettings`, at the first listing (the bot's tokens, else the teams); a page
  asks `limit` = the cap and a page of fewer rows ends the listing. **Strict order:** every org's step 5 before any org's hook.
- **The fake keeps the setup's state in its protocol layer** (designer's answer 26), a `Store` answered as `table` is
  (server.rs:474 of temper-fake-forge-protocol), not in its domain, whose call is per repository (`boundary.rs`). It keeps no
  password: the bot's is read and dropped.
- **The world speaks HTTP with skein-http's client over `Pipe`** and reads bodies with `decode_answer`
  (tests/codecs/src/forgejo.rs:73); 10.1's host does the same over skein-io: two small drivers, not one.
- **Deliveries to the org hook** reconcile made are proved at the process layer by temper 11.1 (decisions.md 19): no story here.

## Rules that bind it

- **No secret in a failure, a `Debug` or a trace:** the world scans the `Debug` of every `Need`, `Event` and failure, and the
  trace, for every secret the fake issued (`secrets_absent`: the reference story asserts it, 09.4's
  `no_secret_is_in_any_failure_debug_or_trace` does for the failures). The author keeps a secret out of an argument and a
  URL: a token is named only in `Need::Send`'s `token`, the password only in `CreateUser`.
- **The ceilings are aims** (development.md 5.3: a new world costs at most about 1 s focused and 5 s fuzzy; this one aims at 1 s
  and 2 s in all, decisions.md 13; unit tests 20 ms, focused world tests 0.5 s each): a miss is noted in the landing report, read
  from nextest's `PASS [ t ]` lines in the test log (the whole run's, from its `Summary [ t ]` line). The shipped profile's builder
  is `temper_service::limits::profile` (limits.rs:247); the machine takes no limit, the world's are the fake's own.

## Increments

### 09.1 The machine: its frame, the admin, the bot, the bot's id
Scope: `reconcile` in `temper-forge-setup`: the vocabulary (sketch A) and setup.md 5 steps 1, 2 and 4.
Needs: temper 07.1 (the crate, `Entry`, `Setup`, the fixtures module), temper 07.2 (`place::File`, which `Need` and `Event` name) and
temper 01.1 (`User`'s `is_admin` and `full_name`, 01-users-tokens.md:93) and 01.2 (`UserOf`, `CreateUser` with its `full_name`). 07.1 names the crate's edge to
`temper-forge-forgejo` (setup.md 11): 09.1 adds none.
Files: `crates/temper-forge-setup/`: `src/reconcile.rs` (new), `src/lib.rs` (`pub mod reconcile`, shared with 07.x),
`src/tests/reconcile.rs` (new), `src/tests/fixtures.rs` (07.1's, extended).
Shape:
- `Reconcile::new(entry, listener)` takes one 07.1 `Entry` (the host finds it with `Setup::entry`) and keeps what it needs of
  it. `Begin` gives `Read(AdminToken)`; an absent or empty file stops `Admin` with `Missing`.
- Admin: `Send(Request::User)` under the admin's token; a `Document::User` with `is_admin` goes on, else `NotAdmin`. Every
  step: a `Status` it has no rule for (401 and 403 on the admin check among them) is `Refused(status)`, `Down` is `Down`, a
  reply of another kind than it reads is `Unexpected`; each stops at its step with the subject's name.
- Bot: `UserOf` as admin. `is_admin` is `BotIsAdmin`; a full name without `temper:bot` is `Unmarked` (subject: the login);
  else keep the id. 404: `Draw(32)` (another length is `Unexpected`), then one `CreateUser` (login, email, the password,
  `full_name` `temper:bot`, `must_change_password` false, answer 15), whose user's id is kept; the password is dropped.
- Id: `Read(User)`; bytes not equal to the id's decimal digits (or none) give `Write(User, digits)`; then `Done`. 09.2 inserts
  step 3 before the id, 09.3 steps 5 and 6 after it.
Test shape: `src/tests/reconcile.rs`. 09.1 adds to the fixtures: `Run::{managed, owned}() -> Run`, which take forge 1 and forge 2
of 07.1's `TWO` (`fixtures::document`, `Setup::read`, `Setup::entry`); `Run::step(&mut self, Event) -> Need`, which feeds one
`Event` into a `Queue::with_capacity(MAX_OUT)` and returns the one `Need`; `Run::forge(&self) -> u16` and `Run::bot(&self) ->
Box<[u8]>`, the entry's number and bot login; `ADMIN: &[u8]`, the admin's token; `secret(&[u8]) -> Secret`; and, of
`temper_forge_forgejo::document::User`, `admin() -> User` (the site admin `temper-admin`) and `person(login: &[u8], full_name:
&[u8]) -> User` (not a site admin). Tests:
`an_admin_token_of_a_plain_user_stops_at_the_admin`,
`an_absent_bot_is_made_with_a_drawn_password_and_the_marker` (draw `[0x0a; 32]`, password `"0a"` x 32, one request),
`a_bot_with_the_marker_is_kept_and_nothing_is_written`, the reference below,
`a_bot_that_is_a_site_admin_is_refused`, `a_refused_request_stops_naming_its_step_and_subject` (403 to the bot's creation),
`a_forge_that_stops_answering_stops_the_run_at_its_step`, `the_bots_id_is_written_in_decimal_only_when_the_file_differs`.
```rust
// crates/temper-forge-setup/src/tests/reconcile.rs
use super::fixtures;
use crate::{place::File, reconcile::{Event, Failure, Need, Reply, Stage, Why}};
use temper_forge_forgejo::{document::Document, request::Request};

#[test]
fn a_user_of_the_bots_login_without_the_marker_is_refused_naming_the_login() {
    let mut run = fixtures::Run::managed();
    assert_eq!(run.step(Event::Begin), Need::Read(File::AdminToken));
    let check = run.step(Event::Read(Some(fixtures::secret(fixtures::ADMIN))));
    assert_eq!(check, Need::Send { request: Request::User, token: fixtures::secret(fixtures::ADMIN) });
    run.step(Event::Reply(Reply::Document(Document::User(fixtures::admin()))));
    let need = run.step(Event::Reply(Reply::Document(Document::User(fixtures::person(&run.bot(), b"a note")))));
    let why = Failure { forge: run.forge(), step: Stage::Bot, subject: run.bot(), why: Why::Unmarked };
    assert_eq!(need, Need::Stop(why));
}
```
Proof: unit; `cargo nextest run -p temper-forge-setup -E 'test(/tests::reconcile::/)'` passes its 8 tests, each under 20 ms
(Rules: a miss is noted), in under 1 s in all. No higher layer exists: nothing hosts the machine until 09.2's world connects it.
Don't: format text, name a host, a path or a clock; hold a timer or a limit; keep a record of its own; define a `File` beside
`place::File`; read the admin's token but through `Read(AdminToken)`; set `is_admin` on the bot; send the password but in
`CreateUser`.
Size: 250.
Removes: nothing.

### 09.2 The bot's tokens, the fake's users and tokens, and the world
Scope: setup.md 5 step 3 in the machine; the fake serves admin users, the admin's token routes and the tokens' sign-in by
token (domain/testing.md 4.2); the world `tests/forge-setup/reconcile` with its first story, an empty forge given its bot and tokens.
Needs: 09.1; temper 01.3 (the bot's tokens on the admin's routes: list, make with scopes, delete by the listed id; both sides). 01.2 and
01.3 leave arms in the fake's exhaustive matches (routes.rs `repository` :66, `op` :143, `write` :235; tally.rs `route` :48;
server.rs `table` :474) that answer `Error::Refused` "until temper 09.2": this increment replaces them with the real answers.
Files: `crates/temper-forge-setup/src/reconcile/tokens.rs` (new), `reconcile.rs`. In `testing/temper-fake-forge-protocol/src/`:
`setup.rs`, `setup/{accounts,tokens}.rs` (new), `lib.rs`, and the hotspots shared with 01.2, 01.3, 02.x and 05.1 (rebase, never
overwrite): `service.rs` (the `Store` field and `setup_mut`; `user` :104 resolves a `token` against `Config` then the store;
:80 and :116 count the store) and `server.rs` (`decoded` :428 asks the store before `table`). The route map's 40 is `ROUTES`,
64, after 01.2 (tally.rs:106); `tally.rs:122`'s `expect("users bounded by token table")` reworded once the store's tokens count. Tests: the fake's `src/tests.rs` (`mod setup;`), `src/tests/fixtures.rs` (`ask_as`), `src/tests/setup.rs` (new). The world: `tests/forge-setup/reconcile/` (new), the root `Cargo.toml` (member), `Cargo.lock`.
Shape:
- Machine, after the bot, for `ApiToken` then `GitToken`: `Read(file)`; some secret: `Send(User)` under it, kept when the
  user's id is the bot's. Another user, 401 or no file: made again. `Tokens` as admin, by page; the token of its name
  (`temper-api`, `temper-git`), if listed, `DeleteToken` by the listed row's `id` (findings/forgejo-tokens.md row 4; 204 or 404
  both go on); `CreateToken` with the scopes of setup.md 4 (the API token's `read:user`, `write:issue`, `write:repository`; the
  git token's `write:repository` and `read:user`, answer 34: `write:repository` alone is refused `GET /user`, rows 5 and 6);
  `Write(file, value)` of the `AccessToken` (made without a value: `Unexpected`). Other statuses and `Down` stop at `Token`,
  subject the token's name.
- Fake `Store`: accounts and issued tokens in `List`s of `documents.inner` rows; it serves every request of 01.2 and 01.3,
  `Tokens` and `DeleteToken` (by id, 404 for an id it has not) among them. `GET /user` under a store token answers its
  account's full user; the admin routes (users, their tokens) need a site admin's token, else 403, and none or an unknown one
  401; a second token of one name is refused; statuses and bodies as 01.2's and 01.3's transcripts record. `Service::setup_mut` is the scenario's way in (sketch B).
- World crate `temper-forge-setup-reconcile-world`: `World::{managed, owned}(seed)` reads its own entry through 07.1's
  `Setup::read` from a document of its own, not `TWO` (forge 1, bot `temper`, orgs `acme` and `work`; owned: person `ada`) and makes a forge with the site admin
  `temper-admin` (id 1) and its token (owned adds the site admin `ada`, id 2). `reconcile() -> Ended` (`Done` or the `Failure`)
  runs one machine through bytes (`exchange.rs`: a `Need::Send` as HTTP text through skein-http's client and `Pipe`; the fake's
  `Server` and `Service` driven as `Joint::tick` drives them, joint.rs:147 of the protocol-forge world; `decode_answer` makes a
  page's rows and document a `Reply`; a `Draw` is the world's seeded bytes; one connection and a fresh `Server` per `Send`,
  `Cut::Never`). Also `listening(addr)`, `file`, `put`, `store`, `store_mut`, `tally`, `trace`, `secrets_absent`.
Test shape: `tests/forge-setup/reconcile/{Cargo.toml (lints as protocol-forge's, :33-47), src/{lib,fixtures,world,exchange}.rs,
tests/reconcile.rs}`; reuse `Pipe` and `Cut` (pipe.rs:96, :6) and `fake_limits` (fixtures.rs:116) of `tests/engine/protocol-forge`
(`temper-engine-protocol-forge-world`) and `decode_answer` of `tests/codecs` (`temper-codecs-world`), as dependencies; the world
sets only the fake's `max_response_items`, 2 (09.3's three teams need a second page); `token_absent` (fixtures.rs:190) is the
model of `secrets_absent`. World tests: the reference below, `the_tokens_hold_the_scopes_of_setup_section_4`,
`a_token_file_that_answers_as_the_bot_is_kept_untouched`,
`a_token_file_holding_another_users_token_is_made_again` (the bot's token of that name is deleted first, by its listed id: one
`DELETE` of that id in the `tally`),
`a_token_file_holding_a_secret_the_forge_never_made_is_made_again`. Fake unit tests in `src/tests/setup.rs` (`Harness::ask`,
`src/tests/fixtures.rs:240`, gains `ask_as` taking a token): `a_plain_user_is_refused_the_admin_routes`,
`a_made_token_authenticates_as_its_user_and_a_deleted_one_does_not`, `a_second_token_of_one_name_is_refused`.
```rust
// tests/forge-setup/reconcile/tests/reconcile.rs
use temper_forge_setup::place::File;
use temper_forge_setup_reconcile_world::{Ended, World};

#[test]
fn an_empty_forge_is_given_its_bot_and_both_tokens() {
    let mut world = World::managed(7);
    assert_eq!(world.reconcile(), Ended::Done);
    let bot = world.store().find(b"temper").expect("the bot is made");
    assert_eq!((bot.admin, &*bot.full_name), (false, b"temper:bot".as_slice()));
    for file in [File::ApiToken, File::GitToken] {
        let token = world.file(file).expect("the token is written");
        assert_eq!(world.store().token_user(token.expose()), Some(bot.id), "{file:?} answers as the bot");
    }
    assert_eq!(world.file(File::User).expect("the id is written").expose(), b"2".as_slice(), "the id in decimal");
    assert!(world.secrets_absent(), "no secret in a failure, a debug or the trace");
}
```
Proof: world; `cargo nextest run -p temper-forge-setup-reconcile-world --test reconcile` runs the first story to `Done`: the
bot made, marked and not an admin, both tokens answering as it, the id file `2`; each test under 0.5 s (Rules). No higher layer
yet: the host that meets a real Forgejo is 10.1's.
Proof: unit; `cargo nextest run -p temper-fake-forge-protocol -E 'test(/tests::setup::/)'` passes 3 tests, each under 20 ms (Rules).
Don't: add a field to the fake's `Config` or `Limits` (every world builds them); keep a password in the store; let a token
route answer without an admin's token; wire the store into the fake domain's permissions; let a document's `Debug` show a
token (the scan fails then).
Size: 250.
Removes: nothing.

### 09.3 Orgs, teams, members and hooks
Scope: setup.md 5 steps 5 and 6 in the machine; the fake serves orgs, teams, members and org hooks; their stories.
Needs: 09.2; temper 02.2 (org hooks and the delete of one; 02.1's orgs, teams and members come before it in session 02);
temper 04.2 (`SUBSCRIBED` and `hook_target` in `temper-forge-forgejo`'s `delivery.rs`, which 04.2 owns).
Files: `crates/temper-forge-setup/src/reconcile/{orgs,hooks}.rs` (new), `reconcile.rs`; the fake's `setup/{orgs,hooks}.rs` (new),
`setup.rs`, the hotspots of 09.2 and `server.rs`'s `Put` arm (:393: the members' `PUT` is routed, once 01.1 has `Method::Put`);
the world's `tests/reconcile.rs` and `src/world.rs`; `crates/temper-forge-setup/src/tests/reconcile.rs` (09.1's, extended).
Shape:
- Machine, after the id: `ApiSettings` once, if no listing has read the cap. Per org in order: `Org`; 404 managed is
  `OrgMissing`, owned `CreateOrg` for the person (02.1's encoder makes it private, answer 19). Teams listed by page for the
  one named as the bot: absent, `CreateTeam` with the wanted `TeamSpec` (the bot's login, `Access::Write`,
  `includes_all_repositories`, `Units` all `Access::Write`, answer 16); present with another `spec`, `EditTeam`. Members
  listed by page: the bot absent, `AddMember`.
- Then, with a listener: `Read(WebhookSecret)`, none gives `Draw(32)` and `Write`; per org, hooks by page, those whose `url` is
  `hook_url(listener, forge)` being the listener's: none, `CreateOrgHook`; else the lowest id gets `EditOrgHook` and each other
  `DeleteOrgHook` (answer 17), the spec being that url, the secret, `SUBSCRIBED`, active. Hooks of other URLs are left alone.
- The walk keeps one id, the lowest match so far, and assumes no order: a match above it is deleted at once; one below it
  deletes the id kept and is kept; after any delete the same page is listed again, since the rows after it moved up. The
  listing's end gives the patch (or the make).
- `pub fn hook_url(SocketAddr, u16) -> Option<Box<[u8]>>` is `http://`, the listener (IPv4 loopback or `[::1]`, written with
  `Writer` and `Decimal`) and 04.2's `hook_target(forge)`; `None` for any other address.
- Fake: orgs (owner), teams (`Team`, members) and hooks (`Subscription`, sketch B) in `List`s of `documents.inner`
  rows; listings paged with `routes::page` (routes.rs:52) and `max_response_items`, in id order; no answer shows a hook's
  secret; non-admins get 403. It serves every request of 02.1 and 02.2, ending their `Refused` arms. `World::managed` makes the
  two orgs; `World::owned` leaves them for reconcile to make for `ada`.
Test shape: world tests: `an_owned_forge_makes_a_missing_org_for_its_person`,
`a_managed_forge_missing_an_org_is_refused_naming_it`,
`a_team_named_for_the_bot_is_made_with_write_on_every_repository` (the bot a member),
`a_listing_longer_than_a_page_is_read_to_its_end` (three teams, cap 2),
`an_org_hook_is_made_with_the_secret_the_events_json_and_active` (events: `SUBSCRIBED`),
`the_webhook_secret_is_drawn_when_missing_and_kept_when_present`,
`two_hooks_with_the_listeners_url_keep_the_lowest_patched_and_the_rest_deleted` (a third of another URL stays). Unit, in the
crate's `src/tests/reconcile.rs`: `hook_urls_are_written_for_loopback_v4_and_v6` (literals, and `None` for 192.0.2.1),
`a_lower_hook_met_later_deletes_the_higher_one_kept` (rows 7, 3: `DeleteOrgHook` 7, the page listed again, `EditOrgHook` 3).
Proof: world; `cargo nextest run -p temper-forge-setup-reconcile-world --test reconcile` shows an owned empty forge converged:
two orgs owned by `ada`, each with the bot's team holding the bot and one hook at `/hooks/1` carrying the secret of `webhook_secret`;
12 tests, each under 0.5 s (Rules).
Proof: unit; `cargo nextest run -p temper-forge-setup -E 'test(/hook_urls_are_written|a_lower_hook_met_later/)'` passes 2 tests, each
under 20 ms (Rules).
Don't: send a repository request; remove a hook or a member it did not make; make a hook when no listener is given.
Size: 250.
Removes: nothing.

### 09.4 The remaining stories and the fuzzy sweep
Scope: setup.md 13.2's stories not yet told, and a sweep over random initial forges.
Needs: 09.3.
Files: the fake's `setup.rs` (scenario setters `delete_account`, `revoke`, `remove_member`, `delete_hook`, `edit_team`, a few
lines each); the world's `tests/reconcile.rs`; `tests/fuzzy_reconcile.rs` (new).
Shape: the setters; and the sweep: for seeds 0..32, `World::owned(seed)` starts from a random subset of {bot, tokens, user
file, team, member, hook, a second hook of its URL, a hook of another URL, secret file} present, each possibly stale; one run
ends `Done`; the state meets the invariants of 09.3's proof, written as literals apart from the machine, the other URL's hook
untouched; a second run writes only hook patches (the route counts before and after).
Test shape: world tests: `a_second_run_writes_only_the_hooks_patches`, `a_deleted_bot_is_made_again` (a new id in `user`),
`a_revoked_token_is_made_again`, `a_removed_team_member_is_added_again`, `a_deleted_hook_is_made_again`,
`a_team_edited_by_a_person_is_repaired`, `a_person_under_the_bots_login_is_refused`,
`no_secret_is_in_any_failure_debug_or_trace` (the failing stories, scanned). Fuzzy, under 2 s:
`random_initial_forges_converge_and_a_second_run_writes_only_hook_patches`.
Proof: world; `cargo nextest run -p temper-forge-setup-reconcile-world` passes in under 1 s in all, and `... --profile fuzzy`
in under 2 s (Rules): the world's whole cost.
Don't: sleep or draw outside the seed; weaken an earlier story to fit.
Size: 40.
Removes: nothing.

## Interface sketches

A. Reconcile's vocabulary, final shape. 09.1 makes it; 09.2 and 09.3 add the uses of `Token`, `Org` and `Hook`. `Entry` is 07.1's,
`File` 07.2's.

```rust
// crates/temper-forge-setup/src/reconcile.rs
use alloc::boxed::Box;
use core::net::SocketAddr;
use skein_lib::{Queue, Secret};
use temper_forge_forgejo::{document::{Document, Row}, request::Request};
use crate::{place::File, setup::Entry};

pub const MAX_OUT: u32 = 1;
/// What reconcile asks of its host, one at a time; the host answers with one `Event`.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Need {
    /// Send `request` under `Authorization: token <token>`; answered by `Event::Reply`.
    Send { request: Request, token: Secret },
    /// Read `file` whole; answered by `Event::Read`.
    Read(File),
    /// Write `bytes` to `file`, whole and private; answered by `Event::Written`.
    Write { file: File, bytes: Secret },
    /// Draw this many random bytes; answered by `Event::Drawn`.
    Draw(u32),
    /// Terminal: converged.
    Done,
    /// Terminal: stopped; the process exits 1.
    Stop(Failure),
}
/// What the host hands back.
#[derive(Clone, PartialEq, Eq, Debug)]
#[expect(clippy::large_enum_variant, reason = "a reply's document moves whole, as the store's records do")]
pub enum Event {
    /// Starts the run.
    Begin,
    /// A file's bytes, or none when it is absent.
    Read(Option<Secret>),
    /// A request's outcome.
    Reply(Reply),
    /// The write is whole.
    Written,
    /// The drawn bytes.
    Drawn(Secret),
}
/// A request's outcome as the host decoded it, once the forge has answered at all.
#[derive(Clone, PartialEq, Eq, Debug)]
#[expect(clippy::large_enum_variant, reason = "a document moves whole, as the store's records do")]
pub enum Reply {
    /// 2xx, one document.
    Document(Document),
    /// 2xx, a listing's page of rows in order.
    Rows(Box<[Row]>),
    /// 2xx, no body.
    Empty,
    /// The API answered with this status, not 2xx.
    Status(u16),
    /// No answer: a failed connection, a timeout, or a body that does not decode.
    Down,
}
/// Why a run stopped: names only, never a secret. `subject` is the login, org, team or token name the step was about.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Failure { pub forge: u16, pub step: Stage, pub subject: Box<[u8]>, pub why: Why }
/// The step of setup.md 5 the run was at: `Admin` (1), `Bot` (2 and 4), `Token` (3, one of the two), `Org` (5: the org, its
/// team, the bot's membership), `Hook` (6: an org's hook and the secret's file).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Stage { Admin, Bot, Token, Org, Hook }
/// What went wrong at it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Why {
    /// A file the step needs is absent or empty.
    Missing,
    /// The API answered this status where the step has no rule for it.
    Refused(u16),
    /// The admin's token answers a user who is not a site admin.
    NotAdmin,
    /// The bot's login is a site admin.
    BotIsAdmin,
    /// The bot's login is a user without the marker.
    Unmarked,
    /// A managed forge lacks the org.
    OrgMissing,
    /// The forge stopped answering after its first answer.
    Down,
    /// A reply of another kind than the step reads, or a draw of another length.
    Unexpected,
}
/// One forge's reconcile. Its other fields are the step's own state.
#[derive(Debug)]
pub struct Reconcile { forge: u16 }
impl Reconcile {
    /// Reconcile of `entry`'s forge; `listener` is `application.webhooks`'.
    #[must_use] pub fn new(entry: &Entry, listener: Option<SocketAddr>) -> Reconcile { todo!() }
}
/// One entry point: an event in, at most `MAX_OUT` needs out.
pub fn step(machine: &mut Reconcile, event: Event, out: &mut Queue<Need>) {
    todo!()
}
```

B. The fake's setup store. 09.2 makes the accounts and tokens. 09.3 adds `org(name, owner)`, `Team { id, org, spec: TeamSpec,
members }` with `team(org, name)`, `Subscription { id, org, url, secret, events, active }` with `subscriptions(org)` and
`hook(org, url, secret)`; 09.4 the setters.

```rust
// testing/temper-fake-forge-protocol/src/setup.rs
use alloc::boxed::Box;
use skein_lib::{List, Secret};

/// A user of the setup store; its password is never kept.
#[derive(Debug)]
pub struct Account { pub id: u64, pub login: Box<[u8]>, pub admin: bool, pub full_name: Box<[u8]> }
/// A token the store issued: its id (what a listing shows and a delete takes), user, name, scopes and secret.
#[derive(Debug)]
pub struct Issued { pub id: u64, pub user: u64, pub name: Box<[u8]>, pub scopes: Box<[Box<[u8]>]>, pub secret: Secret }
/// The setup's state, behind `Service::setup_mut`.
#[derive(Debug)]
pub struct Store { accounts: List<Account>, tokens: List<Issued> }
impl Store {
    /// Scenario: an account (its id) and a token of `user` as `secret`; then the world's reads.
    pub fn account(&mut self, login: &[u8], admin: bool, full_name: &[u8]) -> u64 { todo!() }
    pub fn token(&mut self, user: u64, name: &[u8], scopes: &[&[u8]], secret: Secret) { todo!() }
    #[must_use]
    pub fn find(&self, login: &[u8]) -> Option<&Account> { todo!() }
    #[must_use]
    pub fn token_user(&self, secret: &[u8]) -> Option<u64> { todo!() }
    #[must_use]
    pub fn scopes(&self, user: u64, name: &[u8]) -> Option<&[Box<[u8]>]> { todo!() }
}
```

## Questions to raise when reached

- **09.1: does reconcile repair an existing bot's email?** Answered: no. setup.md 5, step 2: "A bot that passes is never
  edited: its email is the setup's `bot.email` when reconcile makes it, and stays as it is after." 09.1 landed so; nothing
  changes.

## Done when

Each command runs from the repository root.

- The machine, the fake and the world pass: `cargo nextest run -p temper-forge-setup -p temper-fake-forge-protocol -p
  temper-forge-setup-reconcile-world`. The world's `Summary [ t ]` line (`cargo nextest run -p
  temper-forge-setup-reconcile-world`) reads under 1.0 s, and under 2.0 s with `--profile fuzzy` (the aims of Rules: a miss was
  noted in 09.4's landing report and fails nothing here).
- The stories exist: `git grep -c '^#\[test\]' -- crates/temper-forge-setup/src/tests/reconcile.rs
  tests/forge-setup/reconcile/tests/reconcile.rs tests/forge-setup/reconcile/tests/fuzzy_reconcile.rs
  testing/temper-fake-forge-protocol/src/tests/setup.rs` prints the counts 10 (the crate), 3 (the fake), 1 (fuzzy) and 20 (the world), in that order.
- Lints: `cargo clippy -p temper-forge-setup -p temper-fake-forge-protocol -p temper-forge-setup-reconcile-world
  --all-targets -- -D warnings` is clean, and no file passes 800 lines: `git ls-files -z -- crates/temper-forge-setup
  testing/temper-fake-forge-protocol tests/forge-setup | xargs -0 wc -l | awk '$1 > 800 && $2 != "total"'` prints nothing.
- The step crate names no host: `! git grep -nE 'std::|skein_io|skein_http|format!' -- crates/temper-forge-setup/src/reconcile.rs
  crates/temper-forge-setup/src/reconcile`. No forbidden word in the session's new files: `! git grep -nEi
  'actual|genuine|legacy|compat|workaround' -- crates/temper-forge-setup/src/reconcile.rs crates/temper-forge-setup/src/reconcile
  crates/temper-forge-setup/src/tests/reconcile.rs testing/temper-fake-forge-protocol/src/setup.rs
  testing/temper-fake-forge-protocol/src/setup testing/temper-fake-forge-protocol/src/tests/setup.rs tests/forge-setup`.
