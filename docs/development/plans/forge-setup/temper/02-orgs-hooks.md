# Session 02: the setup's documents, orgs, teams and org hooks

Reconcile (setup.md, 5, steps 5 and 6) makes each org's team for the bot, puts the bot in it, and makes, patches and
prunes the org's webhook. This session gives `temper-forge-forgejo` the documents those steps speak, both sides as
its others, and records on the throwaway Forgejo the facts they rest on: orgs, teams and members (02.1), then org
hooks (02.2). It extends temper 01's family (`Kind::Setup`, the recorder, the replay tests). Nothing calls the
documents until temper 09.3 (the machine, the fake's routes) and 10.1 (the host): the fake takes only the arms
that keep it compiling.

## Read first

- **Design:** setup.md 5 (steps 5, 6), 6.5, 11 (the crates), 13.3 (bullets "Orgs and teams", "Org hooks"; the
  host-list half is temper 04.1's); forge.md 5 (the events table; the listener's path) and 10 (no secret in a
  transcript). Forgejo's API: `forgejo-17-swagger.json` (`CreateOrgOption`, `CreateTeamOption`, `CreateHookOption`,
  `EditHookOption`, and the `orgDeleteHook` route).
- **Plan:** decisions.md 5, 11, 13; `findings/forge-and-tiers.md` 1 ("Adding the setup's documents") and 3 ("A new
  group"); `findings/forgejo-tokens.md` rows 8 and 9 (the org routes' scopes); `temper/01-users-tokens.md` (its Choices:
  `Kind::Setup`, the recorder, `setup::scoped` and the group's admin token, `profile_limits`, `setup_cases`).
- **skein:** programming-model.md 6.2 (validate before allocating), 10.4 (errors are values); testing-strategy.md
  2.1 (step tests), 5 (a fake follows the recorded facts).
- **Code:** `crates/temper-forge-forgejo/src/`: `request.rs:4,37`; `document.rs:35,67,84`; `client.rs:34,84,94,155`;
  `server.rs:39,64,119`; `server/writes.rs:21-90,208-313`; `decode/walk.rs:27-51,165,290-300`; a listing of users
  as rows: `client/reviews.rs:99`, `client.rs:119`. The tool: `tools/forgejo-conformance/src/{cases.rs:10-49,
  facts.rs:35-94,fixture.rs:19-92}`.

## What the design asks

- **setup.md 5, step 5; 6.5 (02.1):** each org read, and made for the person, private, when owned, by `POST
  /api/v1/admin/users/{person}/orgs`; in it a team named the bot's login on every repository
  (`includes_all_repositories`), its `permission` `write`, sent explicitly, and `write` on code, issues, pull requests
  and actions (`units_map`), the bot its member; a team a person edited is made again.
- **setup.md 5, step 6; forge.md 5 (02.2):** the org's hook, found by its URL, made when absent, patched at every
  start with the secret, the 22 events, JSON and active; of several with that URL the lowest id is patched and the
  others deleted; a hook of another URL is left alone; Forgejo never shows a secret.
- **setup.md 11, 13.3:** both sides of the documents, each fact checked on v17.0 before a machine relies on it.

## Choices this plan makes

- **Temper 01's shape, extended.** A flat `Request`; a file pair per family (`orgs`, `hooks`); answers' tables are
  new `SetupKind` variants (`Kind::Setup`, 01.1), request bodies' new `WriteKind` variants (`server/writes.rs:21`),
  so no new `Kind` variant is made. The group's recorder (`setup::record`, `setup::facts`, 01.2) calls an `orgs` and
  a `hooks` part after 01's, on its own `Mask`; the replay tests are 01.2's, over rows added to `setup_cases()`.
- **Names** are temper 09's working names: `Org`, `CreateOrg`, `Teams`, `CreateTeam`, `EditTeam`, `TeamMembers`,
  `AddMember`; `OrgHooks`, `CreateOrgHook`, `EditOrgHook`, `DeleteOrgHook`.
- **An org is private** (designer's answer 19): `CreateOrg` sends `{"username":name,"visibility":"private"}`, the
  visibility fixed by the encoder. Reconcile makes orgs on an owned forge alone; a managed forge's are left as they are.
  The route is the admin's, which takes `write:admin` alone, while `POST /orgs` needs `write:organization` too (findings
  rows 9 and 8): two facts say so, from three probe captures. No `Request` makes `POST /orgs`, and `org-made` already
  covers `CreateOrg` on both sides, so the probes are no `setup_cases()` rows (01.3's `-git-discovery` is the precedent).
- **A team is one value:** `TeamSpec` is what a make and an edit send and what `Team.spec` holds, so reconcile
  compares by `==`. Units go in `units_map` (answer 16; `units` is deprecated), each of the four at its access; an
  edit always carries `name` (v17 keeps a team's name when an edit omits it, a fact; ruling 119). `permission` `write`
  is sent beside `units_map` (setup.md 5, step 5;
  designer's answer 35), and a fact reads it back.
- **A hook is `type` `forgejo`, `content_type` `json`,** both fixed by the encoder. The read hook has no secret; its
  target is `config.url` (the answer also has a top-level `url`, not read; a fact records the target). **Twins**
  (answer 17): reconcile deletes every hook of the listener's URL but the lowest id, so the documents have
  `DeleteOrgHook` (204, no document), and a fact that Forgejo accepts a twin.
- **The secret is `skein_lib::Secret`** (Debug shows its length). The recorded hook secrets are two fixed
  64-hexadecimal-digit literals (reconcile's width), unregistered with the leak scan since they protect nothing; the
  mask turns them into counted ids in the transcripts, as it does a password, and the cases name those ids.
- **The facts run under the admin's scopes** (answer 18): the captured calls send 01.2's group token `setup-admin`
  (`setup::scoped(.., "fixture", "setup-admin", ADMIN)`: the fixture admin's `write:admin`, `write:organization` and
  `write:user`, setup.md 6.3), which `setup::record` hands to `orgs::record` and `hooks::record`, so a route that needs
  another scope fails its fact. This session makes no admin token of its own but 02.1's probe, `org-admin-only`
  (`write:admin` alone), made by `setup::scoped` under the `all` token: uncaptured, `scoped` registers its value.
  Provisioning (`Fixture::api`, `fixture.rs:84`) and the final live reads (`Fixture::get`, :92) stay on `f.owner.token`.
- **The fixture:** org `ensemble`, made for `fixture` (the site admin, owned mode's person, setup.md 6.5); team
  `writer`, member `writer` (the bot); `stranger` the control; `ensemble/stage`, private, made after the team.
- **Names of captures and facts:** `setup-org-`, `setup-team-`, `setup-hook-` (decisions.md 11), so 02 counts its own.

## Rules that bind it

- **Transcripts and `facts.txt` are written by the tool only.** A recording rewrites 34 other transcripts (01.2's
  Don't): `git checkout -- tools/forgejo-conformance/transcripts` first, then add the new `setup-` directories and
  the `setup-` lines of `facts.txt`. Masked ids count within the group: a rebase over 01.3's tokens renumbers
  these captures, so record again.
- **No client-api hotspot is touched; `Cargo.lock` does not change** (no dependency; 01.2 adds the codecs'). Shared
  with temper 01 and 04.2: `request.rs`, `document.rs`, `client.rs`, `server.rs`, `server/writes.rs`, `walk.rs`,
  the fake's lists, `tests/codecs/src/forgejo.rs`, the tool's `lib.rs`, `cases.rs`, `facts.rs`, `setup.rs`.

## Increments

### 02.1 Orgs, teams and members
Scope: setup.md 5 step 5, 13.3 "Orgs and teams". The org read and made private, the teams listed, made and edited,
the members listed and put; both sides; the facts, with a repository made later writable by a member and the two org
routes' scopes. The bullet's git-token half (`GET /user` under `write:repository` alone: 403; with `read:user`: 200) is
01.3's (findings rows 5 and 6).
Needs: temper 01.2 (the recorder `setup::record`, `setup::scoped` and the `setup-admin` token it makes, `profile_limits`,
the replay tests, `ROUTES`, and `cases.rs`'s `Recorder` and `send` `pub(crate)`, 01-users-tokens.md:162; make `w`
(`cases.rs:33`) and the fields `orgs.rs` reads `pub(crate)` too where 01.2 left them private; its Needs bring
01.1's `Method::Put` and the body decoder's numbers and booleans, refused today at `server/writes.rs:313`, and the
codec world's `b"PUT"` arm).
Files: `temper-forge-forgejo`: `request.rs` (after `DeleteBranch`, :219); `document.rs` (types below; `Row` :67,
`Document` :84); `client/setup.rs` (`SetupKind` gains `Org`, `Team`, `Teams`, `Units`; their `fields`/`element`
arms call `orgs::`); new `client/orgs.rs` (encode; the `Orgs` decoder) and `server/orgs.rs` (its `route` joins the
chain of `server.rs:39` after 01's); `client.rs` (mods :12-22, `encode` :34, `DecoderState` :84, `decoder` :94,
`step` :155); `server.rs` (mods :12-18, `Answer` :64, `write` :119); `server/writes.rs` (`WriteKind::{NewOrg,
NewTeam}`, `fields` :37, `element` :72, the `member` arm for `includes_all_repositories`); arms in
`client/{start,writes,files,reviews,items}.rs`, `server/{start,items,reviews,branches,files,actions}.rs` and
`server/writes.rs:245-282`, `:316-394` (`scalar`), `:398-439` (`set_id`); `client/writes.rs:38-124` (`target`), `:171`
(`write`); `client/start.rs:135`; `server/items.rs:252`. Elsewhere: `temper-engine-protocol-forge` `connection.rs:686`, `reads.rs:665`, and the
`Document`/`Row` lists (`reads.rs:391,634,662,724,746`, `reads/done.rs:93`, `contents.rs:219,287`, `actions.rs:232-242,329`,
`writes.rs:207,335`); the fake (`testing/temper-fake-forge-protocol/src`): `routes.rs:68,146,235` (`write`'s `Err(Refused)` list, :321), `answers.rs:229,292,456`,
`response.rs:118,172,176-216`, `server.rs:520`, `tally.rs` (`Route` :5, `route` :48; 01.2's `ROUTES` is 64: 40 + 6
+ the 7 here + 4 of 02.2 fit), refusing with `Error::Refused` until temper 09.3 (transitional, ended there);
`tests/codecs/src/forgejo.rs` (rows of `setup_cases()`) and the exhaustive `Answer` matches of `tests/codecs/tests/`
`forgejo_files.rs:153`, `forgejo_items.rs:146`, `forgejo_reviews.rs:112`, `forgejo_start.rs:70`, `forgejo_writes.rs:169`; the tool: new `src/orgs.rs`, called from `setup.rs`.
Counts: `grep -rn 'Request::DeleteBranch' --include='*.rs' crates testing` prints 28 lines at the base (17 files), and
`grep -rn 'Answer::CombinedStatus' --include='*.rs' crates testing tests` 18 (14 files): the arms in `crates/` are
about 140 of the Size.
Shape: the variants and types in the sketches. Targets (`/api/v1/` +): `orgs/{org}`; `admin/users/{person}/orgs`
(POST, `{"username":name,"visibility":"private"}`); `orgs/{org}/teams` (GET paged; POST); `teams/{id}` (PATCH);
`teams/{id}/members` (GET paged); `teams/{id}/members/{login}` (PUT, no body). A make and an edit send
`{"name","permission","includes_all_repositories","units_map":{"repo.code","repo.issues","repo.pulls",
"repo.actions"}}` in that order. Answers: `Document::{Org, Team}`, `Row::Team`; the members are `Row::User` rows
under `Kind::Users`, as collaborators are; `Answer::{Org, Teams, Team, Members}` (`Members(Box<[User]>)`).
Tables: `Org` (`id`, `name`; `visibility` is not read: a fact checks it in the bytes); `Team` (`id`, `name`,
`permission`, `includes_all_repositories` required, `units_map` an object `Units` of four optional texts, an absent
unit reading `Access::None`, the other keys of Forgejo's answer skipped); bodies `NewOrg`, `NewTeam` (whose
`units_map` is the same `Units` table). A word outside Forgejo's five accesses is `Malformed::Value`. `AddMember`
has no decoder (204: `client::decoder` returns `None`).
Test shape: crate tests in new `src/tests/orgs.rs` (`mod orgs;` at `src/tests.rs:1`), with `fixtures::limits()`
(`src/tests/fixtures.rs:12`) and `fixtures::decode` (:55), each under 20 ms:
`a_team_request_writes_its_booleans_and_four_units_as_literals`; `adding_a_team_member_is_a_put_with_no_body`;
`an_org_is_made_with_a_fixed_private_visibility`; `an_org_name_is_one_percent_encoded_path_segment`;
`a_team_answer_without_units_map_reads_every_unit_as_none`; `a_team_access_outside_forgejos_words_is_malformed`;
`a_team_body_with_a_text_where_a_boolean_belongs_is_refused` (on 01.1's boolean arm). The codec side adds rows to
`setup_cases()` and their expected answers; 01.2's four replay tests run them with `forgejo::profile_limits()`, the
shipped builder (`temper-engine-protocol-forge/src/limits.rs:281` over `temper-service/src/forge.rs:110`: `inner` 32,
`string_bytes` 512, `target_bytes` 4096; no name or list here passes them), each under 0.5 s. The tool's literals
are the encoder's output byte for byte, in its key order. The reference tests follow the sketches.
Captures, in this order (21): `setup-org-made`, `-org-made-again`, `-org-read`, `-org-missing`, then the probes:
`-org-route-orgs-refused` (`POST /orgs`, org `probe-a`, under `org-admin-only`: 403), `-org-route-orgs-made` (the same,
org `probe-b`, under `setup-admin`: 201) and `-org-made-by-write-admin-alone` (`CreateOrg`'s route, org `probe-c`, under
`org-admin-only`: 201); `-team-list-new-org`, `-team-made`, `-team-made-again`, `-team-list`, `-team-list-page-1`,
`-team-list-page-3`, `-team-edit`, `-team-edit-nameless`, `-team-members-none`, `-team-member-added`, `-team-members`,
`-team-member-missing-user`, then, after `ensemble/stage` is made and the team restored by provisioning calls,
`-org-issue-by-member` and `-org-repo-for-a-stranger`. Facts (18, all `holds`; a capture in parentheses when its name
differs): `setup-org-made-for-a-user-by-an-admin-is-201-and-private` (org-made: the answer's `visibility` is `private`);
`setup-org-made-again-is-422`; `setup-org-read-by-name-names-its-id`; `setup-org-missing-is-404`;
`setup-org-route-orgs-needs-write-organization` (route-orgs-refused: 403 whose message names `write:organization`;
route-orgs-made: 201, `private`); `setup-org-made-for-a-user-takes-write-admin-alone` (made-by-write-admin-alone: 201,
`private`); `setup-org-starts-with-only-the-owners-team` (team-list-new-org);
`setup-team-made-with-write-all-repositories-and-four-units` (team-made: the answer's `permission` `write`, as sent, and
the four units `write`); `setup-team-listing-reads-back-what-was-made` (team-list: both again);
`setup-team-made-again-is-422`; `setup-team-edit-without-a-name-keeps-the-name` (team-edit-nameless: 200, the
answer's `name` the team's before the edit);
`setup-team-edit-replaces-units-and-flags` (team-edit: `includes_all_repositories` false, and `units_map` the four units
as the encoder sends them, pulls `read` and actions `none`; ruling 127); `setup-team-member-put-is-204` (member-added); `setup-team-members-list-the-added-user` (members);
`setup-team-member-put-of-a-missing-user-is-404`; `setup-team-member-writes-in-a-repository-made-later`
(org-issue-by-member: 201); `setup-team-non-member-cannot-see-the-private-repository` (org-repo-for-a-stranger: 404);
`setup-team-listing-pages-by-limit` (list-page-1: one team; list-page-3: none).
Proof: process: the record command (below) on the pinned Forgejo and runner shows 18 new `holds`, no `fails`, and 21
new directories, in at most 150 s (01.3's 150 s, with this session's exchanges; the base takes 44 s plus 24 s for
`--component`); world: `cargo nextest run -p temper-codecs-world --test forgejo_setup` replays the rows through
both sides, under 30 s built; unit: `cargo nextest run -p temper-forge-forgejo -E 'test(/tests::orgs::/)'` runs the 7 tests of
`src/tests/orgs.rs`, under 30 s built. The proof stops at the recording and its replay because nothing in
production calls the codec: temper 09.3 connects it to reconcile's machine and the fake, temper 10.1 to a Forgejo.
Record command:
```sh
export TEMPER_FORGEJO=$HOME/src/rust/tmp/temper/forgejo/v17.0.0/forgejo-17.0.0-linux-amd64 TEMPER_FORGEJO_RUNNER=$HOME/src/rust/tmp/temper/forgejo/runner-v13.2.0/forgejo-runner-13.2.0-linux-amd64
sh -c 'cargo run -p forgejo-conformance -- --scratch target/forgejo-conformance --out tools/forgejo-conformance && cargo run -p forgejo-conformance -- --component --scratch target/forgejo-conformance --out tools/forgejo-conformance'
```
Don't: no remove or delete request (reconcile only makes again); no request for `POST /orgs` (its two captures are
probes); no `description`, `can_create_org_repo` or deprecated `units` sent; no credential in a `Request`; no route
served by the fake; no hand-edited transcript or fact; no admin call captured under the `all` token.
Size: 250 (codec); it lands at up to 1100 by the ruling of 2026-10-16T23:47:27Z's block.
Removes: nothing.

### 02.2 Org hooks
Scope: setup.md 5 step 6, 13.3 "Org hooks" (the delivery half is temper 04.1's). The org's hooks listed, made,
patched and deleted; both sides; the facts.
Needs: nothing beyond 02.1 (its org `ensemble`, its lists, and the hooks part in the recorder, which takes the group's
`setup-admin` token as 02.1's orgs part does).
Files: as 02.1 for the new variants (same lists, same counts), with new `client/hooks.rs`, `server/hooks.rs`,
`src/tests/hooks.rs` (`mod hooks;`), `tools/forgejo-conformance/src/hooks.rs`; `request.rs` (`HookSpec`);
`document.rs` (`Hook`, `Document::Hook`, `Row::Hook`); `SetupKind::{Hook, Hooks, Config, Events, Event}`;
`WriteKind::NewHook`; `walk.rs:290-300` (the text-array arm names `Event` and the key `events`, as 01.3 does for
scopes); `member` arm for `active`; the decoder's text list (`logins`, `texts` after 01.3).
Shape: the variants and types in the sketches. Targets: `orgs/{org}/hooks` (GET paged; POST), `orgs/{org}/hooks/{id}`
(PATCH; DELETE, answering 204 and no document, so `client::decoder` returns `None` as for `AddMember`). Bodies:
`{"type":"forgejo","config":{"url","content_type":"json","secret"},"events":[…],"active"}`; the edit's is the same
without `type`. Tables: `Hook` (`id`, `config` an object `Config`, `events` an array of texts, `active`), `Config`
(`url` required, `content_type` optional; the body's adds `secret`); body `NewHook` (`type` optional, `config`
required, `events` and `active` optional). Events past `limits.inner` are `Malformed::Bound { field: b"events" }` (the
22 of forge.md 5 fit the profile's 32); a `content_type` other than `json` is `Malformed::Value`. `Answer::{Hooks,
Hook}`.
Test shape: crate tests in new `src/tests/hooks.rs`, each under 20 ms: `a_hook_request_writes_type_config_events_and_active`;
`a_hook_patch_carries_the_config_and_the_secret_again`; `a_hook_requests_secret_shows_only_its_length`;
`a_hook_answer_reads_its_target_from_config_url`; `hook_events_past_the_inner_limit_are_refused` (the server's body
decoder, `inner: 2` beside the profile's 32, as `tests.rs:580`); `a_hook_body_with_a_content_type_other_than_json_is_refused`;
`a_hook_delete_is_a_delete_with_no_body_and_no_answer`. Codec rows as in 02.1. The tool writes the 22 event names of
forge.md 5's table once (`hooks::EVENTS`); the cases name the secrets' masked ids.
Captures (12): `setup-hook-list-none` (a new org lists `[]`), `-hook-made` (URL `http://127.0.0.1:3000/hooks/1`, sent
with the instance's port, which the mask turns into 3000; the throwaway's default host list), `-hook-made-twin` (the
same body), `-hook-made-other` (`http://127.0.0.1:3000/other`, `push` alone), `-hook-list` (three), `-hook-list-page-1`
(`limit=1`: one), `-hook-list-page-4` (none), `-hook-patched` (the first, the full list, the second secret),
`-hook-patched-narrow` (`push` alone, inactive), `-hook-patch-missing`, `-hook-deleted` (the twin, 204),
`-hook-delete-missing`. Facts (12, all `holds`): `setup-hook-for-a-loopback-address-is-made` (201);
`setup-hook-lists-the-events-it-was-made-with` (the 22 names, as a set: Forgejo orders them);
`setup-hook-answers-its-target-in-config-url`; `setup-hook-is-json-and-active`; `setup-hook-never-shows-its-secret`
(no answer has a `secret` key at any depth, and a final live read through `Fixture::get` holds neither literal);
`setup-hook-patch-narrows-events-but-keeps-the-workflow-job-ones` (-hook-patched-narrow: 200, `active` false,
`events` `push` and the four `workflow_job_*` of the full patch; ruling 131); `setup-hook-patch-leaves-the-orgs-other-hook` (a final live read);
`setup-hook-patch-of-a-missing-id-is-404`; `setup-hook-listing-pages-by-limit`;
`setup-hook-of-one-url-can-be-made-twice` (twin: 201, a new id, both listed);
`setup-hook-delete-is-204-and-leaves-the-orgs-other-hooks` (deleted; a final live read);
`setup-hook-delete-of-a-missing-id-is-404`.
Proof: process: the record command of 02.1 shows 12 new `holds`, no `fails`, 33 directories in all, in at most 150 s;
world: the replay of 02.1 over the hook rows, under 30 s built; unit: `cargo nextest run -p temper-forge-forgejo -E 'test(/tests::hooks::/)'`
runs the 7 tests of `src/tests/hooks.rs`, under 30 s built. As 02.1: temper 09.3 and 10.1 connect it.
Don't: no repository hook (the person's); no `workflow_run_*` or `action_run_*` event (the caller's list, forge.md 5);
no delete but a hook's; the secret never in a `Debug`, a message or a test name; log or `{:?}` an `Encoded` of a hook
request (its body holds the secret).
Size: 150 (codec), landing at up to 600 by the same ruling; the arms in `crates/` are about 90 of it (the delete adds about 20).
Removes: nothing.

## Interface sketches

`Request`, `Document`, `Row` and `Answer` are shared with the engine's component, the fake, reconcile's machine (temper
09) and its hosts (temper 10), which use these names. 02.1 creates what is marked (02.1), 02.2 the rest; the admin's
token is the host's header, never a member of a request; `AddMember` and `DeleteOrgHook` end in 204 and no document.

```rust
// crates/temper-forge-forgejo/src/request.rs
use crate::document::TeamSpec;
use alloc::boxed::Box;
use skein_lib::Secret;

/// One page of a listing, as at the base.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Page { pub page: Option<u32>, pub limit: Option<u32> }

/// What reconcile asks of an org's hook: a request's body. Type `forgejo` and JSON are fixed. (02.2)
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct HookSpec { pub url: Box<[u8]>, pub secret: Secret, pub events: Box<[Box<[u8]>]>, pub active: bool }
/// The variants this session adds to `Request`; the others stay. Admin calls use the admin's token, set by the host.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Request {
    /// `GET orgs/{name}`; answers `Document::Org`, or 404. (02.1)
    Org { name: Box<[u8]> },
    /// `POST admin/users/{person}/orgs`, `{"username": name, "visibility": "private"}`; answers 201 `Document::Org`. (02.1)
    CreateOrg { person: Box<[u8]>, name: Box<[u8]> },
    /// `GET orgs/{org}/teams`; rows `Row::Team`, then `Document::Listed`. (02.1)
    Teams { org: Box<[u8]>, page: Page },
    /// `POST orgs/{org}/teams`; answers 201 `Document::Team`. (02.1)
    CreateTeam { org: Box<[u8]>, spec: TeamSpec },
    /// `PATCH teams/{team}`, the whole spec; answers 200 `Document::Team`. (02.1)
    EditTeam { team: u64, spec: TeamSpec },
    /// `GET teams/{team}/members`; rows `Row::User`, then `Document::Listed`. (02.1)
    TeamMembers { team: u64, page: Page },
    /// `PUT teams/{team}/members/{login}`; answers 204, no document. (02.1)
    AddMember { team: u64, login: Box<[u8]> },
    /// `GET orgs/{org}/hooks`; rows `Row::Hook`, then `Document::Listed`. (02.2)
    OrgHooks { org: Box<[u8]>, page: Page },
    /// `POST orgs/{org}/hooks`; answers 201 `Document::Hook`. (02.2)
    CreateOrgHook { org: Box<[u8]>, hook: HookSpec },
    /// `PATCH orgs/{org}/hooks/{id}`, config, events and active; answers 200 `Document::Hook`. (02.2)
    EditOrgHook { org: Box<[u8]>, id: u64, hook: HookSpec },
    /// `DELETE orgs/{org}/hooks/{id}`; answers 204, no document. (02.2)
    DeleteOrgHook { org: Box<[u8]>, id: u64 },
}
```

```rust
// crates/temper-forge-forgejo/src/document.rs
use alloc::boxed::Box;

/// A team's permission, or a unit's access within it, in Forgejo's words. (02.1)
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Access {
    /// `none`.
    None,
    /// `read`.
    Read,
    /// `write`.
    Write,
    /// `admin`.
    Admin,
    /// `owner`.
    Owner,
}
/// The four units temper uses: `repo.code`, `repo.issues`, `repo.pulls`, `repo.actions`. (02.1)
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Units { pub code: Access, pub issues: Access, pub pulls: Access, pub actions: Access }
/// What a team is asked to be: a request's body, and what a read team is compared with. (02.1)
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct TeamSpec { pub name: Box<[u8]>, pub permission: Access, pub includes_all_repositories: bool, pub units: Units }
/// A team as Forgejo answers it: its id, and what it holds. (02.1)
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Team { pub id: u64, pub spec: TeamSpec }
/// An organization: its id and its login (`name`; the deprecated `username` is not read). (02.1)
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Org { pub id: u64, pub name: Box<[u8]> }
/// An org's hook as listed; Forgejo never shows its secret. `url` is `config.url`. (02.2)
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Hook { pub id: u64, pub url: Box<[u8]>, pub events: Box<[Box<[u8]>]>, pub active: bool }
/// The access a word names, or none for another word. (02.1)
#[must_use]
pub fn access(text: &[u8]) -> Option<Access> {
    todo!()
}
/// The word of an access. (02.1)
#[must_use]
pub fn access_text(access: Access) -> &'static [u8] {
    todo!()
}
```

```rust
// tools/forgejo-conformance/src/orgs.rs
use crate::{Result, cases::Recorder, facts::Fact, fixture::Fixture};
use std::path::Path;
use temper_throwaway_forgejo::Token;

/// Records the org, team and member captures under `admin`, the group's `setup-admin` token (01.2's `setup::scoped`),
/// and the three probes, one of them under `org-admin-only` that `setup::scoped` makes; after 01's. `hooks.rs` has the
/// same two functions and takes the same token (02.2).
pub(crate) fn record(r: &mut Recorder<'_>, admin: &Token) -> Result<()> {
    todo!()
}
/// The `setup-org-` and `setup-team-` facts: from the captures, and from the final state through `Fixture::get`.
pub(crate) fn facts(f: &Fixture, out: &Path) -> Result<Vec<Fact>> {
    todo!()
}
```

The reference tests, the first of each new test file (the others follow their form):

```rust
// crates/temper-forge-forgejo/src/tests/orgs.rs
use super::fixtures;
use crate::{
    client,
    document::{Access, TeamSpec, Units},
    request::{Method, Request},
};
use alloc::boxed::Box;

#[test]
fn a_team_request_writes_its_booleans_and_four_units_as_literals() {
    let spec = TeamSpec {
        name: Box::from(*b"writer"),
        permission: Access::Write,
        includes_all_repositories: true,
        units: Units { code: Access::Write, issues: Access::Write, pulls: Access::Write, actions: Access::Read },
    };
    let request = Request::CreateTeam { org: Box::from(*b"ensemble"), spec };
    let encoded = client::encode(&request, &fixtures::limits()).unwrap();
    assert_eq!(encoded.method, Method::Post);
    assert_eq!(&*encoded.target, b"/api/v1/orgs/ensemble/teams");
    assert_eq!(
        encoded.body.as_deref(),
        Some(
            br#"{"name":"writer","permission":"write","includes_all_repositories":true,"units_map":{"repo.code":"write","repo.issues":"write","repo.pulls":"write","repo.actions":"read"}}"#
                .as_slice()
        )
    );
}
```

```rust
// crates/temper-forge-forgejo/src/tests/hooks.rs
use crate::request::{HookSpec, Request};
use alloc::boxed::Box;
use skein_lib::Secret;

#[test]
fn a_hook_requests_secret_shows_only_its_length() {
    let hook = HookSpec {
        url: Box::from(*b"http://127.0.0.1:3000/hooks/1"),
        secret: Secret::new(Box::from(*b"0123456789abcdef")),
        events: Box::from([Box::<[u8]>::from(*b"push")]),
        active: true,
    };
    let request = Request::CreateOrgHook { org: Box::from(*b"ensemble"), hook };
    let Request::CreateOrgHook { hook, .. } = request else { panic!("a hook") };
    assert_eq!(hook.secret.len(), 16);
}
```

## Questions to raise when reached

- **A fact that fails on v17.0** (a team spelling, `config.url`, an event name Forgejo drops, the `private`
  visibility, a route that needs a scope beyond the admin's three; findings rows 8 and 9 answer the org routes' scopes,
  an org made for the admin itself included): the session stops and reports the fact, its capture and the options; it
  does not bend the document.
- **Byte-stability:** two recordings of the group that differ in a `setup-org-`, `-team-` or `-hook-` directory.

## Done when

From the repository root, after 02.2 has landed; `TEMPER_FORGEJO` and `TEMPER_FORGEJO_RUNNER` are the README's.

1. The facts hold: `grep -c -e '^holds setup-org-' -e '^holds setup-team-' -e '^holds setup-hook-' tools/forgejo-conformance/facts.txt` prints 30, and the same with `^fails` prints 0.
2. The captures are there: `ls -d tools/forgejo-conformance/transcripts/setup-org-* tools/forgejo-conformance/transcripts/setup-team-* tools/forgejo-conformance/transcripts/setup-hook-*` lists 33 directories.
3. Both sides agree and compile: `cargo nextest run -p temper-forge-forgejo -p temper-codecs-world -p temper-fake-forge-protocol -p temper-engine-protocol-forge` passes, `cargo nextest run -p temper-forge-forgejo -E 'test(/tests::orgs::/) | test(/tests::hooks::/)'` runs 14 tests, and `cargo nextest run -p temper-codecs-world --profile fuzzy --test fuzzy_forgejo` passes (2 tests).
4. Clippy is clean where the arms are: `cargo clippy -p temper-forge-forgejo -p temper-engine-protocol-forge -p temper-fake-forge-protocol -p temper-codecs-world -p forgejo-conformance --all-targets -- -D warnings`.
5. Each landing's report gives its hand-written production lines by code-organization.md's command, against Size 250 and 150 (codec).
