# Session 01: the setup's documents, users and tokens

Reconcile's first four steps (setup.md 5) speak Forgejo's user and admin-token routes, all under the admin's token: the bot is
made with a random password nobody keeps, carries its marker `temper:bot` in its full name from the request that makes it, and
has its tokens listed, deleted and made on the admin's routes; nothing signs in as the bot (designer's answers 14, 15). This
session puts those routes in `temper-forge-forgejo` as documents, both sides as its others, and records on the throwaway Forgejo
the facts they rest on (setup.md 13.3, "The admin routes"). 01.1 holds the codec gaps that temper 02 shares and that are **owned
here**: `Method::Put`, the boolean in a request body (`BodyDecoder::scalar`), and the `User` members reconcile reads. 01.2 puts
the users in, 01.3 the tokens. Nothing calls the documents until temper 09 and 10: the fake forge takes only the arms that keep it
compiling (it serves the routes in temper 09.2). The conformance tool grows the group's recorder that temper 02 and 04 reuse.
Three increments, in order; each lands alone.

## Read first

- **Design:** setup.md 4 (the scopes), 5 (steps 1 to 4), 6.3 (the admin token's scopes), 9, 11, 13.3 (the admin routes);
  forge.md 3.2 (the routes the bot's tokens must reach); credentials.md 2 (the tokens' rows).
- **Plan:** decisions.md 5 (the profile's limits), 11 (`setup-` facts), 16 (no live service); `findings/forge-and-tiers.md` 1
  (adding documents), 3 (a new group); `findings/measurements.md` 4 (a plain recording rewrites 34 transcripts and drops the 40
  `component-` lines); `findings/forgejo-tokens.md` (the probe of the routes and scopes, rows 1-10).
- **skein:** `programming-model.md` 6.2 (validate before allocating), 6.3 (the worst case), 10.4 (errors are values);
  `testing-strategy.md` 2.1 (step tests), 8 (the two suites). temper `docs/development/workflow.md` 2.
- **Code:** `client/actions.rs:11-65,67-180` (a family of kinds with its decoder: the model for `client/setup.rs`);
  `server/writes.rs:21-460` (write bodies); `tests/codecs/tests/forgejo_writes.rs:3-12` and `forgejo_start.rs:74-111` (the replay
  tests to copy); `tools/forgejo-conformance/src/cases.rs:10-49`, `fixture.rs:84` (`Fixture::api`: a call by the admin's `all` token).

## What the design asks

- **setup.md 5, steps 1-4:** `GET /user` under the admin's token answers a site admin; the bot's user read by login and made by
  `POST /admin/users`, never a site admin, its full name the marker; each of its two tokens found among `GET
  /admin/users/{login}/tokens`, deleted by `DELETE …/tokens/{token}`, made again by `POST`; the bot's id from its user.
  This session: the documents and the facts for each.
- **setup.md 9:** the bot's password is random and kept nowhere: the codec holds it as `skein_lib::Secret`.
- **setup.md 11:** the documents are `temper-forge-forgejo`'s, both sides, served by the fake (temper 09.2).
- **setup.md 13.3:** record making a user with its full name; listing, making and deleting its tokens by the admin's token; that
  a second token of one name is refused; the least scopes the admin's token needs (6.3 names `write:admin`, `write:organization`,
  `write:user`) and the scopes each route of forge.md 3.2 needs. setup.md 4 names `read:user`, `write:issue`, `write:repository`
  for the API token and `write:repository` with `read:user` for the git token (answer 34): 01.3 checks each, and that
  `write:repository` alone is refused `GET /user`.
- **Probed already** (`findings/forgejo-tokens.md`, 17.0.0, by hand): rows 1-4 and 7 confirm the routes this session codes, the
  full name set at creation, the list without values, the delete by listed id or by name (204 both) and the admin's token making
  one for itself; rows 5 and 6, the git token's two scopes; row 10, a user made to change its password refused `GET /user` under
  basic auth. Probed by hand after the findings: the same user is refused under any token too (403 "You must change your
  password"), until the admin's `PATCH /api/v1/admin/users/{login}` clears the flag. The facts record them again, on the committed code.

## Choices this plan makes

- **The two user members are read only from a user document of its own** (`GET /user`, `GET /users/{login}`, the answers of the
  admin routes), by a new walk kind `Kind::Setup(SetupKind::Account)`. A user embedded in another document keeps `id` and `login`.
  Why: four decoders read a user's keys (`client/{start,items,reviews,branches}.rs`), each with a key table that ends in
  `unreachable!` (`items.rs:449-520`) or an `else` that would take `is_admin` for another field (`reviews.rs:286-291` sets
  `dismissed`, `branches.rs:222-225` `dismiss_stale_approvals`), and the engine's bound prices a user as `size_of::<User>()` and a
  few strings per slot (`engine-protocol-forge/src/limits.rs:325-335`), which more retained strings in every embedded user would
  outgrow. `Request::User` (`client.rs:121`) moves to the account kind. `email` and a profile description are read by nothing.
- **A flat `Request`, a file pair per family** (`client.rs:12-22`, `server.rs:12-18`), as temper 02: `client/setup.rs` and
  `server/setup.rs` hold users and tokens. Names, as temper 09 expects: `UserOf`, `CreateUser`, `Tokens`, `CreateToken`,
  `DeleteToken`. There is no `EditUser`: the marker is set at creation and no password is reset (code-organization.md still lists it).
- **No credential in a `Request`:** the host sets `Authorization: token`, the admin's, for all five. A password and a token's value
  are `Secret`s (Debug shows the length only). `DeleteToken` carries the listed row's id (Forgejo takes an id, else a name, both 204 by row 4:
  `forgejo-17-swagger.json`, `adminDeleteUserAccessToken`), so a name of digits is never read as an id.
- **The group runs under the admin token's scopes** (answer 18; setup.md 6.3): `setup::scoped` makes a token of `write:admin`,
  `write:organization`, `write:user` for the fixture's admin (a call by its `all` token, uncaptured), and every capture of the
  group is sent under it, but where a capture says otherwise. Sufficiency is then what every fact shows; 01.3 adds what is necessary.
- **The group is recorded last,** by `setup::record` in `tools/forgejo-conformance/src/setup.rs`, called after `cases::flags`
  (`lib.rs:26`) with a mask of its own (`Mask::new`, `mask.rs:11`): the transcripts already committed keep their numbers, and the
  masked ids the codec tests name as literals count from 1 in this group alone. Every fact reads captured cases (`facts.rs:44`), so
  the facts and the codec tests rest on the same bytes. temper 02 adds `orgs` and `hooks` to the same two functions.
- **Fresh secrets leave a transcript masked, and nothing random stays in one:** a password in a body and a token's value in an
  answer are registered in `Secrets` and the capture is scanned after masking (`mask.rs:63-71` turns a 40- or 64-digit run into a
  counted id of the same width); a token's last eight digits (`token_last_eight`) are handed to the mask as a literal, else two
  recordings differ. Never scanned raw, never kept.
- **Shipped limits:** the codec world's new tests take `forgejo::profile_limits()` (new in `tests/codecs/src/forgejo.rs`):
  `temper_engine_protocol_forge::limits::documents` over `temper_service::limits::profile(Profile::Small).forge_component`
  (`limits.rs:247`; `forge.rs:110-132`: `string_bytes` 512, `body_bytes` 16,384, `inner` 32, `target_bytes` 4,096). The crate's own
  unit tests use `fixtures::limits()` (`tests/fixtures.rs:12`: 1,024, 65,536, 64) and `fixtures::small()` (`:22`: 8 and 16), the
  crate being unable to name the profile; a cap test names its small value beside the profile's.
- **Size counts** `crates/*/src` outside `tests*`, plus `testing/` and `tools/` outside `tests*` (code-organization.md 6).

## Rules that bind it

- Every 2xx JSON capture lands with its server route (`json_cases`, `tests/codecs/src/forgejo.rs:361`, routes each, and `route`
  :95 panics on an unrouted one; a body that is not JSON, the git discovery, is skipped); facts are `setup-…`; no secret in a
  transcript, a fact line, a message or a Debug.
- Shared with temper 02 and 04 (not with client-api): `request.rs`, `document.rs`, `client.rs`, `server.rs`, `walk.rs`,
  `server/writes.rs`, the fake's lists, `tests/codecs`, the tool's `cases.rs`, `facts.rs`, `lib.rs`: rebase, never overwrite;
  after a rebase over one of them the group is recorded again.
- Files at most 800 lines: `server/writes.rs` is 460, this session adds under 100 and temper 02 more; past 700, move
  `BodyDecoder` (`:207-442`) to `server/body.rs` in a commit of its own first. The fake's `response.rs` is 741 and takes three arms
  per request (about 15 more here): past 780, split it first, the same way.

## Increments

### 01.1 The codec's gaps: Put, a boolean in a body, the user's two members
Scope: `Method::Put`; the dispatch that lets a write kind carry a boolean; `User` gains `is_admin` and `full_name`, read on a user
document of its own (Choices).
Needs: nothing beyond the session's start.
Files: `temper-forge-forgejo`: `request.rs:4`; `document.rs:35,178`; `decode/walk.rs:27,64,165` (`Kind::Setup`); `client.rs:121,139`
(`Request::User` decoded as `SetupKind::Account`); new `client/setup.rs`; `server/start.rs:62` (a writer for a whole user, used by
`Answer::User` only); `server/writes.rs:305-313` (`scalar`); a `Method::Put` arm in each `route`
(`server/{start,items,reviews,branches,actions,files}.rs`, `server/writes.rs:154`); a `Kind::Setup(_)` arm in each exhaustive `Kind`
list (`grep -rn 'Kind::Users' crates/temper-forge-forgejo/src` finds the 17 mentions, in 8 files). Elsewhere:
`engine-protocol-forge/src/connection.rs:203-208` (maps Put to skein-http's); the fake `server.rs:388-394` (maps Put: still 405, by
the codec); `tests/codecs/src/forgejo.rs:31-37` (`b"PUT"`); the 19 literals `User { id, login }` (the grep of the base command, `User \{ id[:,]` less `Heard::User` and `Unusable::User`, finds
20 lines at the base: 18 of them, the multi-line one at `tests/codecs/src/forgejo.rs:221` apart, and `document.rs:179` and
`engine-protocol-forge/src/start.rs:25`, another type) (`tests.rs:89,124,162,287`; `engine-protocol-forge/src/tests/reads.rs:170`, `tests/connection.rs:67`; fake `routes.rs:42,49`,
`answers.rs:154`, `tests/answers.rs:21,55,74,104,193`; `tests/codecs/src/forgejo.rs:221`, `tests/forgejo_start.rs:12,25,26,37`) take
`is_admin: false, full_name: Box::from([])` (two new fields named, no `..Default::default()`). No client-api hotspot.
Shape: `Method::Put`. `User` does not derive `Default` (step code disallows it); `document::user()` (`:178`) builds it. `SetupKind::Account` has `field(id)`,
`field(login)`, `optional(is_admin, Boolean)`, `optional(full_name, Text)` (`walk.rs:58-63`); `Setup` (the decoder, as `Actions`)
fills a `User` and ends in `Document::User`. `Start` keeps `Kind::User` for embedded users. `server/start.rs` writes all four
members for `Answer::User`; rows and embedded users stay `id`, `login`. `BodyDecoder::scalar`'s `Boolean` arm calls `flag(key,
value)` (`:313` today answers `Shape`): an `if let` chain per request with such a member, empty in this increment; 01.2 gives it its
first, temper 02.1 its others. `Number` stays refused: nothing in the setup sends one. **It is the brief's split:** nothing in 01.1
has a boolean member, so the arm has no test until 01.2. The memory bound is unchanged (Choices);
`the_worst_case_counts_answers_and_refuses_overflow` (`engine-protocol-forge/src/tests.rs:308`) guards it.
Test shape: `src/tests/accounts.rs` (new; `mod accounts;` in `tests.rs`), unit, each under 20 ms, with
`fixtures::limits()`/`small()`/`decode()` (`tests/fixtures.rs:12,22,55`) and `Request::User`:
`a_top_level_user_keeps_admin_flag_and_full_name`, `an_embedded_user_keeps_id_and_login_alone` (a comment's `user` carrying both),
`a_user_without_the_two_members_decodes_with_empty_defaults`, `a_full_name_past_string_bytes_is_too_long_naming_it` (`small()`: 8, the
profile's is 512), `an_admin_flag_that_is_not_a_boolean_is_malformed_naming_it`, `the_servers_user_answer_decodes_to_the_user_it_encodes`,
`a_put_is_refused_by_every_family_of_routes` (`/api/v1/user`, `/repos/a/b`, `/issues`, `/pulls/1/files`, `/actions/runs`, `/branches`
give `Unrouted::Method`). The existing `forgejo_start.rs:26` row now expects the real transcript's `is_admin` true.
Reference test:
```rust
// crates/temper-forge-forgejo/src/tests/accounts.rs
use super::fixtures;
use crate::{client, document::{Document, User}, request::Request};
use alloc::boxed::Box;

#[test]
fn a_top_level_user_keeps_admin_flag_and_full_name() {
    let limits = fixtures::limits();
    let body = br#"{"id":1,"login":"fixture","full_name":"temper:bot","email":"fixture@example.invalid","is_admin":true}"#;
    let decoder = client::decoder(&Request::User, &limits).unwrap();
    let user = User { id: 1, login: Box::from(*b"fixture"), is_admin: true, full_name: Box::from(*b"temper:bot") };
    assert_eq!(fixtures::decode(decoder, body, &limits).document, Ok(Document::User(user)));
}
```
Proof: unit, `cargo nextest run -p temper-forge-forgejo -p temper-codecs-world -p temper-fake-forge-protocol`: the site admin's
recorded `GET /user` reads `is_admin` true; the 7 tests of `tests::accounts` (`cargo nextest run -p temper-forge-forgejo -E
'test(/tests::accounts::/)' --no-tests=fail`); under 30 s once built; joint, `cargo nextest run
-p temper-engine-protocol-forge-world --test fake_protocol` (`tests/fake_protocol.rs:6`): the real component starts against the
fake's bytes, reading `GET /user` through the account kind from an answer of four members, and lists; under 30 s. No higher layer
yet: nothing sends a Put or reaches the boolean arm, and 01.2 connects them.
Don't: give `Kind::User` the new keys; add a route or a request; decode `full_name` of an embedded user.
Size: at most 100 lines of production code (codec).
Removes: nothing.

### 01.2 Admin users: create, read by login, and the group's recorder
Scope: two requests with their documents, both sides; the group's recorder in the conformance tool, and its basic-auth exchange;
the facts and transcripts (setup.md 5 steps 1, 2 and 4; 13.3 admin routes; findings row 10; a must-change user's token).
Needs: 01.1.
Files: `temper-forge-forgejo`: `request.rs` (after `DeleteBranch`, :219); `client.rs` (`encode` :34 and `decoder` :94 arms, `mod
setup`); `client/setup.rs` (encode, targets); new `server/setup.rs` (`route`); `server.rs:39` (`route` tries it after `read_route`'s
`NotFound`, so `/users/search` stays `UserSearch`); `server/writes.rs` (`WriteKind::NewUser`, its `fields` :37 and `flag` arm);
every exhaustive `Request` list (the 28 occurrences of `Request::DeleteBranch` at the base: `grep -rn 'Request::DeleteBranch'
--include='*.rs' crates testing | wc -l`), among them `engine-protocol-forge/src/connection.rs:726`, `reads.rs:706`, `writes.rs:122`.
The fake: arms in `answers.rs`, `response.rs`, `routes.rs`, `server.rs:474`, `tally.rs` (two `Route` variants; **`Map::with_capacity(40)`
at :106 and `expect("40 routes")` at :116 and `server.rs:432` hold exactly the 40 now: name `ROUTES = 64`**), refusing the new
requests (`Error::Refused`, `routes.rs:146`) until temper 09.2: transitional, ended there. Throwaway: `lib.rs` (`fresh_secret`,
over `instance.rs:229`'s `random`, made `pub(crate)`), `instance.rs` (`exchange_basic`, after `exchange` :196), `http.rs:46` (the
masked scheme follows the header's). Conformance: `lib.rs:22-27`, new `setup.rs` and `users.rs`, `fixture.rs:74` (`exchange_basic`
beside `exchange`), `cases.rs:10-33,43,295` (`Recorder`, `send`, `record`, `flags`: `pub(crate)` or `&mut Secrets`), `facts.rs:44`
(`case`, `Case` `pub(crate)`). `tests/codecs`: `src/forgejo.rs` (`profile_limits`, `setup_cases`), new
`tests/forgejo_setup.rs`, `Cargo.toml` (+`temper-service`, `temper-engine-protocol-forge`; regenerate `Cargo.lock`, a client-api
hotspot, with `cargo check -p temper-codecs-world --tests`; never merge it).
Shape: `Request::{UserOf, CreateUser}` (Sketches). Targets: `GET /api/v1/users/{login}`; `POST /api/v1/admin/users`
`{"username","email","password","full_name","must_change_password"}` in that order, all five required by the server side. Both
answer `Document::User` (the account kind). A password past `string_bytes` is `Malformed::TooLong` naming `password`.
The recorder: `Recorder.secrets` becomes `&mut Secrets` (`cases.rs:14`; `lib.rs:22,26` pass `&mut`); `send` is unchanged. New
`send_new(name, method, target, body, auth: Auth<'_>, fresh: &[(SecretKind, &str)])` beside it skips `send`'s raw scan
(`cases.rs:27`: a fresh secret is in the raw bytes by design), registers `fresh`, then masks and writes: `write::case` scans the
masked bytes with all the secrets. `Auth` is `Token(&Token)` or `Basic(login, secret)`, the second sent by `Fixture::exchange_basic`
over `Instance::exchange_basic` (`Authorization: Basic login:secret`; the recorded request says `Basic MASKED`), and the tool
registers the header, `fixture::base64("login:secret")` (`fixture.rs:136`), beside the secret.
`setup::record(f, out, secrets)` makes its own `Mask`, runs `users::record`, and is called after `cases::flags` (:26);
`setup::facts(f, out)` returns the group's facts, called after `facts::flags` (:27). `setup::scoped(f, secrets, login, name,
scopes)` makes a token for `login` on `POST /admin/users/{login}/tokens` by `Fixture::api` (uncaptured), registers it and answers
a `Token`; the group's first act is `scoped(.., "fixture", "setup-admin", ADMIN)` (Choices). The password of `bot` is
`temper_throwaway_forgejo::fresh_secret()`. Captures (10), in the order that fixes the masked ids, under the admin token but where
said: `setup-user-create` (user `bot`, `full_name` `temper:bot`, `must_change_password` false), `-create-again`,
`-create-by-writer` (403, the writer's token), `-of` (GET by login), `-missing` (404), `-admin` (`GET /user`), `-create-person`
(user `person`, `must_change_password` true, a fresh password), `-person-basic` (`GET /user` under `Basic person:<that
password>`: 403, findings row 10), `-person-token` (`GET /user` under a `read:user` token `scoped(.., "person", "setup-person", ..)`
made: 403 "You must change your password") and `-person-cleared` (the same call after `Fixture::api` sent, uncaptured, `PATCH
/api/v1/admin/users/person` `{"login_name":"person","source_id":0,"must_change_password":false}`: 200). The PATCH is a raw call, no
`Request`: temper 10.2's support sends the same one. `person` is the fact's own user, never temper's bot.
Facts (10, each `setup-`): `create-user-answers-201-with-login-email-and-full-name`,
`created-user-is-no-site-admin-and-keeps-its-full-name`, `create-user-again-is-refused-with-a-message`,
`user-of-a-login-reads-admin-flag-and-full-name`, `user-of-a-missing-login-is-404`, `a-non-admin-token-is-refused-the-admin-routes`,
`the-admin-token-reads-user-as-a-site-admin`, `every-recorded-user-carries-is-admin-and-full-name` (reads `user`, `user-fixture` and
the group's user transcripts: why the decoder may read both as optional),
`a-user-made-to-change-its-password-is-refused-the-api-under-basic-auth` (403, the message names the change; why the person of
temper 14 signs in on Forgejo's web first, setup.md 12),
`a-user-made-to-change-its-password-is-refused-under-its-token-until-the-admin-clears-the-flag` (`-person-token`'s 403 and its
message, then `-person-cleared`'s 200 as `person`: why temper 10.2's support clears the flag before the person acts).
Test shape: `tests/codecs/tests/forgejo_setup.rs`, world, each under 0.5 s, `forgejo::profile_limits()`, `forgejo::transcript` (`:25`),
`decode_answer` (`:73`), `decode_route` (`:102`), `shape` (`:190`), cases in `setup_cases()` modelled on `write_cases` (`:227`); the
password in the cases is the transcripts' masked id (64 digits, `…01` for `bot`'s, the next for `person`'s). Tests:
`setup_requests_encode_as_the_tool_sent_them`,
`setup_requests_route_on_the_servers_side_to_the_same_request`, `setup_transcripts_decode_to_their_documents`,
`setup_answers_the_server_encodes_decode_unchanged_with_the_transcripts_shape`, `a_refused_setup_write_reads_its_status_and_message`
(`-person-basic`'s and `-person-token`'s 403 among them), `setup_bodies_enforce_members_types_and_bounds` (`must_change_password` as text, a password past
`string_bytes`, a missing `full_name`). In `accounts.rs`: `a_requests_debug_shows_a_password_by_its_length_only`, `a_create_user_body_names_every_member_in_order`.
Throwaway `src/tests.rs`: `a_fresh_secret_is_64_hexadecimal_digits_and_never_repeats`,
`a_basic_request_records_its_scheme_and_holds_no_credential` (as `:24`). Conformance `src/tests.rs`: `a_fresh_password_in_a_body_leaves_its_transcript_masked` (as `:43`).
Reference test:
```rust
// tests/codecs/tests/forgejo_setup.rs
use temper_codecs_world::forgejo;
use temper_forge_forgejo::client;

#[test]
fn setup_requests_encode_as_the_tool_sent_them() {
    let limits = forgejo::profile_limits();
    for (case, request) in forgejo::setup_cases() {
        let transcript = forgejo::transcript(case);
        let encoded = client::encode(&request, &limits).expect("setup encoding");
        assert_eq!(encoded.method, transcript.method, "{case}");
        assert_eq!(&*encoded.target, transcript.target, "{case}");
        assert_eq!(encoded.body.as_deref().unwrap_or_default(), transcript.request_body, "{case}");
    }
}
```
Proof: process, the record command, after
`export TEMPER_FORGEJO=$HOME/src/rust/tmp/temper/forgejo/v17.0.0/forgejo-17.0.0-linux-amd64 TEMPER_FORGEJO_RUNNER=$HOME/src/rust/tmp/temper/forgejo/runner-v13.2.0/forgejo-runner-13.2.0-linux-amd64`
on its own line: `sh -c 'cargo run -p forgejo-conformance -- --scratch target/forgejo-conformance --out tools/forgejo-conformance &&
cargo run -p forgejo-conformance -- --component --scratch target/forgejo-conformance --out tools/forgejo-conformance'`
shows 10 `holds` lines of 01.2's facts (counted by the first words of their names, `grep -cE '^holds setup-(create-user|created-user|user-of|a-non-admin|the-admin-token|every-recorded|a-user-made)'`), the 40
`holds component-` unchanged, and 10 new `setup-user-*` directories; at most 120 s alone, on the committed code (the gate lane refuses a tree with changes); world, `cargo
nextest run -p temper-codecs-world` replays them through both sides; under 30 s built.
Don't: scan a fresh secret raw or register it after the capture; keep of a recording more than the new `setup-*`
directories and the `setup-` lines of `facts.txt` (the base's churn, measurements.md 4, is restored with `git checkout`);
serve the requests in the fake; log or `{:?}` an `Encoded` of `CreateUser` (its body holds the password); send basic auth as the
bot or for any user but `person`; add a `Request` for the PATCH.
Size: at most 160 lines of production code (codec).
Removes: nothing.

### 01.3 The bot's tokens on the admin's routes: list, make, delete; the scopes
Scope: three requests, `AccessToken`, both sides; the facts, including the admin token's least scopes (setup.md 13.3), the git
token's two scopes (answer 34) and the scopes of setup.md 4 against forge.md 3.2.
Needs: 01.2 (the recorder, `scoped`, `exchange_basic`, the request lists).
Files: as 01.2 for `request.rs`, `client.rs`, `client/setup.rs` (the token kinds), `server/setup.rs`, `server/writes.rs`
(`WriteKind::{NewToken, Scopes, Scope}`; `BodyDecoder::logins` :211 becomes `texts`, `walk.rs:290-300` names the key by kind),
`document.rs` (`AccessToken`, `Row::Token` :67, `Document::Token` :84), `server.rs:64` (`Answer::{Token, Tokens}`), `tally.rs`
(three `Route` variants), the exhaustive `Request`, `Document`, `Row` lists (`engine-protocol-forge/src/reads.rs:616-662`),
`server.rs:119` (`write`), the fake arms, the `expected` arms of `tests/codecs/tests/forgejo_{start,writes,items,reviews,files}.rs`.
Throwaway: none (`exchange_basic` is 01.2's). Conformance: new `tokens.rs`, `setup.rs`, `mask.rs` (`Mask::literal`), `cases.rs`
(`send_new` registers a 2xx answer's `sha1`), `component.rs:27-30` (below), `README.md:19`.
Shape: `Request::{Tokens, CreateToken, DeleteToken}`; `GET /api/v1/admin/users/{login}/tokens?page=&limit=` (rows `Row::Token`, then
`Document::Listed`); `POST` `{"name","scopes":[…]}` answering 201 `Document::Token` with `value` (`sha1`); `DELETE
/api/v1/admin/users/{login}/tokens/{id}`, no document. `scopes` is bounded by `limits.inner` (`Malformed::Bound { field: b"scopes" }`,
as `tests.rs:259`), its elements read as texts as `Contexts` are (`client/branches.rs:116-122`, `walk.rs:290-300`); `sha1` is optional
text, an empty or an absent one `value: None` (row 2: the list leaves it blank). The git discovery goes by 01.2's `Auth::Basic`
(`Basic bot:<token>`, as git sends it), the header registered beside the token. `Mask::literal(text)` rewrites a registered literal to
a counted id of its width, as the 40- and 64-digit runs are.
Actors and tokens: the group's `bot` (01.2) is made a collaborator of `fixture/specimen` by `Fixture::api` (`PUT …/collaborators/bot`,
write; uncaptured). `scoped` makes, uncaptured, under the `all` token, the probes `no-admin` (`write:organization`, `write:user`),
`no-user` (`write:admin`, `write:organization`), `read-user` (`write:admin`, `write:organization`, `read:user`), all for `fixture`, and
`user-only` (`read:user`), `git-alone` (`write:repository`) and `no-issue` (`read:user`, `write:repository`) for `bot`.
Captures (17; the mask continues from 01.2), under the admin token but where said: `setup-token-make` (`temper-api`: `read:user`,
`write:issue`, `write:repository`), `-make-git` (`temper-git`: `write:repository`, `read:user`), `-make-again` (the name taken), `-list`,
`-list-page-2` (`limit=1`), `-as-the-bot` and `-git-as-the-bot` (`GET /user` under each made token: 200, row 6 for the git one),
`-git-alone` (`GET /user` under `git-alone`: 403, row 5), `-delete` (by the listed id), `-deleted-use` (`GET /user` under it: 401),
`-delete-missing` (404), `-narrow-write` (`POST …/issues` under `no-issue`: refused), `-git-discovery` and `-git-discovery-refused`
(`GET /fixture/specimen.git/info/refs?service=git-receive-pack` under `Basic bot:<temper-git's token>` and `Basic bot:<user-only's>`); `setup-scope-no-admin` (`GET …/tokens` under `no-admin`), `-no-user` (`GET /user` under `no-user`),
`-read-user` (`GET /user` under `read-user`).
Facts (15, each `setup-`): `admin-route-makes-a-token-answering-201-with-name-scopes-and-value`, `token-of-an-existing-name-is-refused`,
`token-list-shows-names-and-scopes-without-values`, `token-list-pages-by-page-and-limit`, `a-made-token-answers-user-as-the-bot`,
`a-git-token-of-write-repository-alone-is-refused-user`, `a-git-token-of-write-repository-and-read-user-answers-user-as-the-bot`,
`token-delete-answers-204-and-ends-the-token`,
`token-delete-of-a-missing-id-is-404`, `admin-token-without-write-admin-is-refused-the-token-routes`,
`admin-token-without-a-user-scope-is-refused-user`, `admin-token-with-read-user-in-place-of-write-user-reads-user`,
`api-token-without-write-issue-is-refused-an-issue-write`, `git-token-with-write-repository-opens-the-push-discovery`,
`git-token-without-it-is-refused`. **The scopes of forge.md 3.2:** `component::run` (`component.rs:27-30`) gives the writer the token
`scoped(&f, &mut secrets, "writer", "temper-api", API)` makes (`API` the three scopes of setup.md 4) before `Hosted::new`; the 40
`component-` facts then hold under those scopes alone, every read and write of 3.2 included; necessity is sampled (an issue write, a
push), as the design claims a set, not a table. A fact that fails is a wrong premise of the design: raise it (Questions).
Test shape: `tests/forgejo_setup.rs` grows `setup_cases()` rows (the four replay tests cover them); new `src/tests/tokens.rs` (unit):
`a_made_token_holds_its_value_and_shows_only_its_length`, `a_listed_token_has_no_value`, `a_scope_list_past_inner_is_malformed_naming_scopes`,
`a_scope_list_encodes_in_the_order_given`, `a_delete_writes_the_id_as_one_segment`; in `forgejo_setup.rs`,
`setup_bodies_enforce_members_types_and_bounds` gains `scopes` as text and past `inner` (the profile's 32). Conformance `src/tests.rs`:
`a_made_tokens_value_and_tail_leave_its_transcript_masked`.
Reference test:
```rust
// crates/temper-forge-forgejo/src/tests/tokens.rs
use super::fixtures;
use crate::{client, document::Document, request::Request};
use alloc::boxed::Box;

#[test]
fn a_made_token_holds_its_value_and_shows_only_its_length() {
    let limits = fixtures::limits();
    let made = br#"{"id":3,"name":"temper-api","scopes":["read:user"],"sha1":"0123456789abcdef0123456789abcdef01234567","token_last_eight":"01234567"}"#;
    let request = Request::CreateToken { login: Box::from(*b"bot"), name: Box::from(*b"temper-api"), scopes: Box::from([]) };
    let decoded = fixtures::decode(client::decoder(&request, &limits).unwrap(), made, &limits);
    let Ok(Document::Token(token)) = decoded.document else { panic!("a token") };
    let Some(value) = token.value else { panic!("a value") };
    assert_eq!(value.len(), 40);
}
```
Proof: process, the record command of 01.2 (its `export` line first) shows 15 more `holds` lines, those of 01.3's facts (by the first
words of their names, as 01.2's are), the 40 `holds component-` (now under the narrow token), and 14 `setup-token-*` and 3 `setup-scope-*`
directories; at most 150 s alone; world, `cargo nextest run -p temper-codecs-world` replays the tokens'
transcripts through both sides; under 30 s built.
Don't: put a credential in a `Request`; put a token's value or its last eight in a fact line, a message or a kept transcript; use the
bot's password; use the bot's token on a token route.
Size: at most 190 lines of production code (codec).
Removes: nothing.

## Interface sketches

`Request` and `Method` are shared by the engine's component, the fake, reconcile's machine (temper 09) and its hosts (temper 10);
these are the new variants. `Secret` makes `{:?}` of a request safe; the sender forgets the password, no variant keeps a credential.
```rust
// crates/temper-forge-forgejo/src/request.rs
use alloc::boxed::Box;
use skein_lib::Secret;

/// An HTTP method of Forgejo's API; `Put` is new (01.1), first used by temper 02.1.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Method { Get, Post, Put, Patch, Delete }

/// One page of a listing, as at the base.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Page { pub page: Option<u32>, pub limit: Option<u32> }

/// The variants this session adds to `Request`; the 40 existing ones stay.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Request {
    /// `GET /api/v1/users/{login}` by a site admin: any user, whole. Answers `Document::User`. (01.2)
    UserOf { login: Box<[u8]> },
    /// `POST /api/v1/admin/users`; answers 201 `Document::User`. The password is the sender's to forget. (01.2)
    CreateUser { login: Box<[u8]>, email: Box<[u8]>, full_name: Box<[u8]>, password: Secret, must_change_password: bool },
    /// `GET /api/v1/admin/users/{login}/tokens`; rows `Row::Token`, then `Document::Listed`. (01.3)
    Tokens { login: Box<[u8]>, page: Page },
    /// `POST /api/v1/admin/users/{login}/tokens`; answers 201 `Document::Token`, the value once. (01.3)
    CreateToken { login: Box<[u8]>, name: Box<[u8]>, scopes: Box<[Box<[u8]>]> },
    /// `DELETE /api/v1/admin/users/{login}/tokens/{id}`, the id of a listed row; answers 204, no document. (01.3)
    DeleteToken { login: Box<[u8]>, id: u64 },
}
```
```rust
// crates/temper-forge-forgejo/src/document.rs
use alloc::boxed::Box;
use skein_lib::Secret;

/// A Forgejo user. `id` and `login` come from every user object; the last two only from a user document of its own, and
/// are false or empty in a user embedded in another document (01.1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct User { pub id: u64, pub login: Box<[u8]>, pub is_admin: bool, pub full_name: Box<[u8]> }

/// An access token as Forgejo lists (`value` none) or makes it (`value` the token, once). New in 01.3.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct AccessToken { pub id: u64, pub name: Box<[u8]>, pub scopes: Box<[Box<[u8]>]>, pub last_eight: Box<[u8]>, pub value: Option<Secret> }
```
```rust
// testing/temper-throwaway-forgejo/src/instance.rs
use crate::{Exchange, Failure, Outgoing};

/// The running instance, as at the base (fields elided).
pub struct Instance;

impl Instance {
    /// One exchange under `Authorization: Basic login:secret`, a token as git sends it or a person's password; the recorded request
    /// says `Basic MASKED`. (01.2)
    ///
    /// # Errors
    /// Returns a connection, framing or bounded-read failure.
    pub fn exchange_basic(&self, outgoing: &Outgoing<'_>, login: &str, secret: &str) -> Result<Exchange, Failure> { todo!() }
}
```
The conformance's `setup::record(f, out, secrets)` and `setup::facts(f, out)` are the group's two entries (01.2 makes them with
`users`, 01.3 adds `tokens`; temper 02 adds `orgs` and `hooks`), and `setup::scoped(f, secrets, login, name, scopes)` makes a token
for the groups' probes. `temper_throwaway_forgejo::fresh_secret()` answers 64 random hexadecimal digits (01.2). `Answer::User` writes
all four members, every other answer's users two; `Answer::Token(AccessToken)` and `Answer::Tokens(Box<[AccessToken]>)` are the
fake's (01.3).

## Questions to raise when reached

- **01.3: does `read:user` do in place of `write:user`?** setup.md 6.3 names `write:user`; `admin-token-with-read-user-in-place-of-
  write-user-reads-user` records it. If it holds, say so to the designer (the least is narrower); the plan builds the three until told.
- **A `setup-` or `component-` fact that fails** is a wrong premise of the design, never landed failing: a scope of setup.md 4 (a
  `component-` fact may need a scope the set lacks, e.g. `read:misc` for `GET /settings/api`). Raise it to the designer at the ready
  check, with the fact and the status seen. The routes themselves are probed (rows 1-4 and 7).
- **Byte-stability:** a recording at the base rewrote 34 transcripts. If two recordings of the group differ, raise it.

## Done when

- Counted by 01's own fact names (02's are `setup-org-`, `-team-`, `-hook-`; 04's `delivery-`): `test "$(grep -cE '^holds setup-(create-user|created-user|user-of|a-non-admin|the-admin-token|every-recorded|a-user-made|admin-route|token-|a-made-token|a-git-token|admin-token|api-token|git-token)' tools/forgejo-conformance/facts.txt)" = 25` and the same with `^fails` `= 0`.
- `test "$(grep -c '^holds component-' tools/forgejo-conformance/facts.txt)" = 40`.
- `test "$(ls -d tools/forgejo-conformance/transcripts/setup-user-* tools/forgejo-conformance/transcripts/setup-token-* tools/forgejo-conformance/transcripts/setup-scope-* | wc -l)" = 27`.
- `test "$(grep -c '^    Put,$' crates/temper-forge-forgejo/src/request.rs)" = 1` (`Method::Put`).
- `test -z "$(git ls-files 'crates/temper-forge-forgejo/*.rs' 'testing/temper-fake-forge-protocol/*.rs' 'testing/temper-throwaway-forgejo/*.rs' 'tools/forgejo-conformance/*.rs' 'tests/codecs/*.rs' | xargs wc -l | awk '$1 > 800 && $2 != "total"')"`.
- `test "$(git log origin/main --grep='^Plan: forge-setup$' -p --format= -- '*.rs' | grep -E '^\+' | grep -ciE 'actual|genuine|legacy|compat|workaround')" = 0`.
- `scripts/check.sh` passes.
