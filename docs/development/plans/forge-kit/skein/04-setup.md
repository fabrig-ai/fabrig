# Session 04: `skein-forge-setup`

Move reconcile onto given values and items, keeping configuration and files
with its owner; then build the design's differences and missing step tests.
Needs at start: **temper 01.2**, setup using `skein-forge`'s dialect on
temper's `origin/main`; **04.1 also needs skein 01.3**'s shared word check.

## Read first

- skein `docs/design/forge-setup.md`, 2–4 and 6–8; temper
  `docs/design/protocol/setup.md`, 3, 5, 8, 11 and 13: what the owner keeps.
- This plan's README (sessions, Rules, Done when), `code-organization.md`
  1–3 and 5–8, `decisions.md` 2, 3, 14, 19, 21 and Design questions 23–29;
  the designer's `forge-kit-answers-1.md` and `forge-kit-answers-2.md`.
- `findings/survey-04-setup.md`, corrected by `findings/trace-setup.md`
  1–4; `findings/base-runs.md`; `findings/base-skein-04.md`.
- skein `docs/foundation/programming-model.md`, 3, 5.2, 9 and 10;
  `docs/foundation/testing-strategy.md`, 2.1, 8 and 10 (all named rules).
- Move precedent: jig `docs/development/plans/skein-ui/skein/01-move.md`;
  temper `docs/development/plans/forge-setup/temper/09-reconcile-machine.md`.

## What the design asks

- 2–3: the identity, credentials, reach and optional hooks; the host keeps them.
- 4.1–4.2: owner-given values, five items, one outstanding need; `MAX_OUT = 1`.
- 4.3–4.4: the six steps, bounds, named failures and converging repairs.
- 6: pure step code and checks; files, waiting, locks and HTTP stay the host's.
- 7: step tests here; the world moves in skein 07.3 after temper 04.1 connects it.
- 8: use the existing dialect and recorded facts; conformance belongs to skein 06.

## Choices this plan makes

`given::Given`, `given::{Place, Refused}`, `item::Item` are this session's
names. `org_owner: Option<Box<[u8]>>` alone decides whether to make an org.
Answers 23–29 are settled: 23 gives Places, optional owner and pair order;
24–28 give 04.2's changes; 29 puts temper's mapping and check in step code.
Section 4.1: the owner supplies the nonempty admin and token names, without
word/login grammar; only the bot marker `<word>:bot` is derived. Scopes stay fixed.

## Rules that bind it

- **No rewrite of a moved part.** Copy temper's `origin/main` at 04.1's
  start, name that commit in the landing, and make only its Shape's forced
  changes. Counts and source paths below are **at the base**, not the copy.
- No system name in skein's code, comments or tests; no moved-type alias or
  re-export. Tests use `kit`; the owner later supplies its own word.
- Keep each moved test's name, budget and time. Test builds stay unoptimized;
  units <20 ms. A slow test is understood, never hidden (testing-strategy 8).

## Increments

### 04.1 Reconcile's machine moved
Scope: forge-setup 4 and 6, the new home and its boundary only.
Needs: temper 01.2's setup on the kit's dialect; skein 01.3's
  `skein_forge::marker::valid_word`, available before this move.
Files: from temper `crates/temper-forge-setup/src/` at the base to skein
  `crates/skein-forge-setup/src/`: `reconcile.rs` -> `reconcile.rs`,
  `reconcile/{hooks,orgs,tokens}.rs` -> the same paths; `limits.rs`'s
  `Limits` and `check` -> `limits.rs`; `setup.rs`'s `login` and `email`
  validators -> `given.rs`; `tests/reconcile.rs` -> `tests/reconcile.rs`;
  only `Run`, `ADMIN`, `secret`, `admin`, `person`, `team_spec`, `hook_row`
  and `subscribed` from `tests/fixtures.rs` -> `tests/fixtures.rs`.
  New `Cargo.toml`, `src/{lib,given,item,tests}.rs`, `src/tests/given.rs`;
  shared hotspots skein `Cargo.toml` (dependency entry) and `Cargo.lock`.
  `crates/*` already admits the member. Dependencies: skein-lib, skein-json,
  skein-forge only. Root: `#![cfg_attr(not(test), no_std)]`, forbid unsafe.
  Dialect imports name `skein_forge::forgejo`; source setup citations become
  `forge-setup.md, 4.3`. Read-only consumers found at the base: shell
  `src/{reconcile,exchange}.rs` and tests, `tests/forge-setup/reconcile`,
  `crates/temper/src/{forge,setup,box_command,lib}.rs` and tests, temper-box's
  `src/{host,machine,tests/fixtures}.rs`, and setup's `src/{check,setup}.rs`.
  Their edits/deletions belong to temper 04.1, not this copy.
Shape: retain `reconcile::{Event, Reply, Failure, Stage, Why, step, MAX_OUT}`
  and `hook_url(address: SocketAddr, forge: u16) -> Option<Box<[u8]>>`.
  Replace `Reconcile::new(&Entry, Option<SocketAddr>) -> Self` with
  `Reconcile::new(given: &Given) -> Result<Self, Refused>`, calling
  `given::check` before cloning values. `Orgs::new(given: &Given) -> Self`
  copies orgs and owner: Some(login) makes an absent org for that login;
  None stops with OrgMissing. No `Entry`, `Mode` or `Account` survives.
  Replace `Reconcile.bot: Account` with owned bot-login and bot-email bytes.
  `Tokens::new(id: u64, api_name: &[u8], git_name: &[u8]) -> Self` copies
  the given names; `Tokens::name(&self) -> &[u8]` borrows the current name.
  Reconcile retains the given admin/token names until their steps. `Given`
  owns its bytes; the machine owns its copies, as in the source.
  Expose the copied validators as `given::login(bytes: &[u8]) -> bool` and
  `given::email(bytes: &[u8]) -> bool`; temper 04.1 changes setup's `account`
  and `strings` callers to these paths (the person's email still needs them).
  Drop the copied limits' `Unfit`; `limits::check` returns `given::Refused`,
  replacing dotted paths with Places and preserving the two relation checks.
  **temper's side:** shell `src/reconcile.rs`'s `reconcile` and the world's
  `src/world.rs` constructor calls become `Reconcile::new(&given)` and handle
  refusal; their Read/Write arms map `Item` to files (`file` becomes `item`).
  In temper 04.1, temper-forge-setup's step code maps every setup entry to
  Given and runs `given::check` through `src/check.rs::checked`, for every
  command, `temper serve` too, with no shell. It maps `Refused.place/other`
  to protocol/setup.md 5's paths; `src/setup.rs::{account,strings}` use the kit
  validators. Binary `crates/temper/src/forge.rs::reconcile_settings` takes
  those mapped values for the host; it does not own their mapping/check.
  `src/setup.rs::check` supplies the listener to that step-code mapping;
  `src/lib.rs`'s serve path keeps calling it; `src/box_command.rs::run` calls
  `checked` through it and `steps`. temper-box's `host::Service::new`,
  `machine::Machine::new` and test fixtures still take temper's `Step`: no
  machine call or Step signature changes there. temper 04.1 repairs the move;
  a later interface change is repaired by whichever temper increment repins next.
```rust
// skein_forge_setup::given; Box from alloc, SocketAddr from core::net.
/// Owner's inputs; the checked machine copies these for one run.
pub struct Given {
    pub forge: u16,
    pub bot_login: Box<[u8]>,
    pub bot_email: Box<[u8]>,
    pub admin_name: Box<[u8]>,
    pub api_name: Box<[u8]>,
    pub git_name: Box<[u8]>,
    pub orgs: Box<[Box<[u8]>]>,
    pub org_owner: Option<Box<[u8]>>,
    pub listener: Option<SocketAddr>,
    pub word: Box<[u8]>,
    pub limits: Limits,
}
/// Refusal locations; the owner maps them to its configuration paths.
pub enum Place {
    Word,       // Owner's word.
    BotLogin,   // Bot's login.
    BotEmail,   // Bot's email.
    AdminName,  // Admin's failure name.
    ApiName,    // API token's name.
    GitName,    // Git token's name.
    Orgs,       // Given org count.
    Org(u32),   // Given org name, zero-based position.
    OrgOwner,   // Optional owner's login.
    OrgsLimit,  // Maximum org count.
    Deadline,   // Whole run's duration.
    Backoff,    // First retry duration.
    BackoffMax, // Largest retry duration.
}
/// A refused input; no value or secret crosses this diagnostic boundary.
pub struct Refused { pub place: Place, pub other: Option<Place> }
pub fn check(given: &Given) -> Result<(), Refused>;
// skein_forge_setup::limits; Duration is skein_lib::Duration.
/// Owner's four bounds, mirroring the source Limits field for field.
pub struct Limits {
    pub orgs: u32, pub deadline: Duration,
    pub backoff: Duration, pub backoff_max: Duration,
}
pub fn check(limits: &Limits) -> Result<(), Refused>;
// skein_forge_setup::item
/// Values kept by the host, never paths in the kit.
pub enum Item {
    AdminToken,    // Host answers Read; retained for this run.
    ApiToken,      // Host answers Read or Written.
    GitToken,      // Host answers Read or Written.
    User,          // Decimal bot id; host answers Read or Written.
    WebhookSecret, // Host answers Read or Written.
}
// skein_forge_setup::reconcile; Request is skein_forge::forgejo::request::Request.
/// One need, answered once by the host unless terminal.
pub enum Need {
    Send { request: Request, token: Secret }, // Answer: Reply.
    Read(Item),                              // Answer: Read.
    Write { item: Item, bytes: Secret },      // Answer: Written.
    Draw(u32),                               // Answer: Drawn.
    Done,                                    // Terminal: converged.
    Stop(Failure),                           // Terminal: stopped.
}
```
  Internal `given::marker(word: &[u8]) -> Box<[u8]>` makes only `<word>:bot`.
  Keep source Clone/PartialEq/Eq/Debug derives; Place/Item/Refused also Copy.
  Check the word with `skein_forge::marker::valid_word`; copied login/email
  shapes check bot/orgs/optional owner. Refuse each empty admin/API/git name
  at its Place. Every `u16` forge number and optional SocketAddr is admitted:
  neither has a refusal Place; non-loopback still means no hooks (4.3 step 6).
  Limits' relations stay in written order (zero checks belong to 04.2):
  `(Backoff, BackoffMax)` before `(BackoffMax, Deadline)`. Other refusals
  have `other: None`. Org count admission is 04.2. Keep substring matching,
  refusal precedence and empty-item/token-status behavior until 04.2.
  Keep `draw`'s `CreateUser { must_change_password: false, .. }`, asserted by
  the moved `an_absent_bot_is_made_with_a_drawn_password_and_the_marker` (4.3 step 2).
  Forced fixture changes: `Run` holds Given instead of parsing TWO.
  `managed()` uses forge 1/org `acme`/owner None; `owned()` uses forge 2/org
  `work`/owner Some(`ada`); `hook_listing()` starts from managed, sets listener
  127.0.0.1:9898 and rebuilds with `Reconcile::new(&given)`. Each expects Ok;
  `forge()` and `bot()` read Given's forge and bot_login. Keep the reply scripts.
  Forced renames: `temper` -> `kit` in moved tests, `admin()`'s login, `person`
  call arguments, `kept_tokens`/`hook_listing` bytes and `team_spec`'s name;
  `File` -> `Item` in `bot_lookup`/`kept_tokens`. The machine retains
  `given::marker(word)` for both `draw` and `id`.
Test shape: **13 moved tests**, all names in base-skein-04's nextest listing;
  at the revised base they take 0.002–0.004 s each (98-test package: 0.043 s).
  Shared `fixtures::limits()` copies shipped `limits::small`: 8 orgs, 120 s,
  100 ms, 5 s; no production default moves. `fixtures::given()` uses forge 1,
  bot `kit`, email `kit@example.invalid`, org `acme`, no owner/listener, word
  `kit`, with `kit-admin`, `kit-api`, `kit-git` supplied separately.
  Add 4 unit tests in `tests/given.rs`: `the_word_derives_only_the_bot_marker`
  checks `given::marker(b"kit") == b"kit:bot"`;
  `the_given_names_are_used_verbatim` changes only Given's word to `other`:
  answer-driven Run prefixes yield Admin/kit-admin/Refused(401); separate
  API/git runs answer with bot marker `other:bot`, and their CreateToken
  requests name `kit-api`/`kit-git` (literal bytes).
  `valid_given_values_are_accepted` checks profile and validator boundaries
  (word 1/16, login 1/40, email 3/254), expected `Ok(())`; include forge
  numbers 0/65535 and None/loopback/non-loopback listeners, all accepted.
  `invalid_given_values_name_their_place` checks word 0/17/uppercase,
  login 0/41/leading punctuation, email 2/255/control/two @, bad org and
  owner, plus an empty admin/API/git name separately, expected `Refused` with
  Word/BotLogin/BotEmail/Org(0)/OrgOwner/AdminName/ApiName/GitName respectively,
  and `other: None`; both check and constructor refuse before a need.
  Reference, using the shared builder; extend it with those boundary rows:
```rust
// src/tests/given.rs
#[test]
fn valid_given_values_are_accepted() {
    let given = super::fixtures::given();
    assert_eq!(crate::given::check(&given), Ok(()), "shipped profile admitted");
}
```
Proof: unit; `cargo nextest run -p skein-forge-setup` shows 17 passing
  tests, each <20 ms, suite <1 s. Base form: `cargo nextest run -p
  temper-forge-setup`, 98 passed in 0.043 s. No caller of the new crate yet;
  temper 04.1 connects the host/world, skein 07.3 moves the world here.
Don't: move `setup`, `check`, `problem`, `state`, `place`, `ini`, `runner`,
  `spec`, `prepare`, `run`, `upgrade`, `step`, the limits' `apply`/`small`,
  `setup.*` names or any setup-shell code; move the members/read/steps tests;
  add zero-limit refusal before 04.2 or duplicate the kit's word check.
Size: about 900 lines moved, 280 new (hand-written production; tests aside).
Removes: nothing in skein; temper 04.1 removes the source copies and maps Items.

### 04.2 The design's differences
Scope: forge-setup 4.1, 4.3–4.4 and 7, the settled answers 24–28.
Needs: 04.1's machine on Given and Item; no additional cross-session dependency.
Files: `crates/skein-forge-setup/src/{given,limits,reconcile}.rs`,
  `src/reconcile/{tokens,hooks}.rs`, `src/tests/{given,reconcile,fixtures}.rs`.
  Temper repin repair: `crates/temper-forge-setup-shell/tests/support/mod.rs`
  (`settings`), alongside the existing affected caller expectations.
Shape: no new public signature. Against the base, retained through 04.1:
  `Reconcile::id` uses `bytes::find` alone, accepting any substring; now the
  whole `<word>:bot` needs both edges: start or a preceding byte outside
  `[a-z0-9-]`, AND end or a following byte outside that set. Scan past rejected
  occurrences. `id` checks BotIsAdmin first at base; now Unmarked wins when
  both hold. Normalize `Read(Some(empty))` to `Read(None)` for every Item:
  base already refuses an empty AdminToken and rewrites an empty User, but
  sends empty API/git tokens to User and keeps an empty WebhookSecret.
  Base `Tokens::event` remakes only on 401; add 403. Every other status keeps
  stopping with Refused(status), and Down keeps stopping with Down.
  Base setup parsing checks the org count; the machine itself does not.
  Add that check to `given::check` and therefore `Reconcile::new`: zero or
  above `limits.orgs` -> `Refused { place: Orgs, other: None }` before any
  clone/need. Base `limits::check` checks relations alone; first refuse zero
  in each of its four fields at that limit's Place, `other: None`, then
  check relations in answer 23's order.
  **temper's side:** temper 04.1 builds the step-code mapping/check described
  in 04.1, for every command including serve, naming each refused Place by
  protocol/setup.md 5's path, with no shell. Shell `src/reconcile.rs::reconcile`
  and world `src/world.rs` retain checked construction; setup `src/check.rs::checked`
  runs the strengthened check. The next temper repin repairs affected
  refusal/marker/empty-item/token-status expectations as these tests do.
  Its shell-test `support::settings` must give `Entry::orgs` one valid org,
  `Box::new([b"acme".as_slice().into()])`, so shell stories reach HTTP after
  strengthened admission. Refresh further copies/derived expectations reached
  by that repair and list them in the repin commit body.
Test shape: **0 moved, 16 new**, 33 total; shared 04.1 fixtures, all <20 ms.
  In `tests/given.rs`: `no_org_is_refused_before_the_machine_starts` (0),
  `an_org_count_at_the_bound_is_accepted` (PROFILE_ORGS = 8),
  `an_org_count_past_the_bound_is_refused` (9): check and constructor;
  expected `Refused { place: Orgs, other: None }` or successful construction.
  `a_zero_limit_names_its_place`: zero each field separately with the other
  profile values kept; `limits::check`, `given::check` and the constructor
  refuse OrgsLimit, Deadline, Backoff or BackoffMax respectively,
  `other: None`, before any need.
  In `tests/reconcile.rs`, driven only by answers:
  - `a_marker_at_a_word_boundary_is_accepted`: `kit:bot`, `(kit:bot)`,
    `note kit:bot`, `xkit:bot kit:bot`, `kit:botx kit:bot` -> Read(ApiToken).
  - `a_marker_inside_another_word_is_refused`: word `b`, full name `ab:bot`,
    `0b:bot` or `-b:bot` -> Stop Bot/kit/Unmarked (bot login stays `kit`).
  - `a_marker_followed_by_word_bytes_is_refused`: word `b`, full name
    `b:botx`, `b:bot0` or `b:bot-` -> Stop Bot/kit/Unmarked.
  - `an_unmarked_admin_is_refused_as_unmarked`: `is_admin = true`, `a note` ->
    Stop Bot/kit/Unmarked, never BotIsAdmin.
  - `an_empty_api_token_is_absent`, `an_empty_git_token_is_absent`:
    empty read at each token's start -> Send ApiSettings under ADMIN;
    the API token was kept in the git case, so no cap has been read.
  - `an_empty_bot_id_is_absent`: empty read -> Write User, `b"42"`.
  - `an_empty_hook_secret_is_absent`: empty read -> Draw(32).
    The moved `an_empty_admin_token_stops_before_any_request` covers the
    fifth Item: empty -> Stop Admin/kit-admin/Missing, no request.
  - `a_revoked_token_is_deleted_made_and_written`: 401 checking either token
    (API kept in git case) -> ApiSettings under ADMIN; cap 2, id 7 -> DeleteToken(7),
    Empty/404 -> CreateToken, value `new-token` -> Write the corresponding Item.
  - `a_forbidden_kept_token_is_remade`: 403 while checking either token ->
    the same full remake script as 401; assert API scopes `read:user`,
    `write:issue`, `write:repository`, git scopes `write:repository`, `read:user`.
  - `a_non_auth_status_checking_a_token_stops`: 404/429/500/502/503 for either
    token -> Token's named Stop with Refused(the status), no remake.
  - `a_down_checking_a_token_stops`: Down for either token -> named Stop/Down.
  Failure expectations include forge 1 and literal subject `kit`, `kit-api`
  or `kit-git`. Helpers prepare the phase without assertions on the way.
Proof: unit; `cargo nextest run -p skein-forge-setup` shows 33 passing
  tests, each <20 ms, suite <1 s. 04.1's 98-test base form lacks these 16 tests.
  temper 04.1 connects the host; skein 07.3 moves the world. Neither blocks this step.
Don't: alter scopes, check scopes of a kept token that answers as the bot,
  retry a later Down, change an unrelated transition or add a machine timer.
Size: about 0 lines moved, 140 new.
Removes: substring-only acceptance, old precedence and empty-secret retention.

### 04.3 Section 7's step tests
Scope: prove the remaining transitions, page walks and limits of forge-setup 7.
Needs: 04.2's marker, items, token-status rules and org admission.
Files: `crates/skein-forge-setup/src/tests.rs`, `src/tests/fixtures.rs`;
  new `src/tests/{steps,limits}.rs`. No production changes.
Shape: extend the shared Run fixtures: explicit replies reach a phase;
  assert its next need. For an ended machine, call step with a MAX_OUT Queue
  directly: Run::step expects one need and cannot inspect no output.
Test shape: **0 moved, 21 new**, 54 total; each <20 ms; no new world or fuzzy
  sweep. All use 04.1's profile builder. PAGE_CAP = 2 (rather than Forgejo's
  default 50) makes a two-page walk with three rows; DRAW_BYTES = 32 from 4.3.
  SHORT_DRAW_BYTES = 31 is one byte below that requested draw.
  In `tests/steps.rs`, these 18 tests drive the machine with answers:
  - `an_owned_forge_makes_a_missing_org_for_its_person`: `Run::owned()`,
    kept tokens/id, ApiSettings cap PAGE_CAP, then Org(work) answered 404 ->
    Send `CreateOrg { person: b"ada", name: b"work" }` under ADMIN.
  - `a_managed_forge_missing_an_org_is_refused_naming_it`: `Run::managed()`,
    same prefix to Org(acme), answered 404 -> Stop with forge 1,
    Stage::Org, subject `b"acme"`, Why::OrgMissing, no CreateOrg.
  - `a_missing_team_is_made`: empty teams -> CreateTeam for acme, name kit,
    write permission, all repositories true, all four units Write.
  - `a_changed_team_is_repaired`: team id 7 with Read -> EditTeam(7),
    same literal wanted spec; use the shared fixture for that expectation.
  - `an_absent_bot_member_is_added`: empty members -> AddMember(7, kit).
  - `an_absent_hook_is_made`: no matching URL -> CreateOrgHook(acme),
    URL `http://127.0.0.1:9898/hooks/1`, kept secret, subscribed events, active.
  - `a_kept_hook_is_patched`: one id 3 at that URL -> EditOrgHook(3), same
    literal URL/secret/events/active; decode JSON is the dialect's test.
  - `an_out_of_order_event_stops_each_phase`: data rows of explicit event
    prefixes reach every outer State and Tokens/Orgs/Hooks Phase; a wrong
    event -> Stop/Unexpected at that phase's literal stage and subject.
  - `an_ended_machine_ignores_every_event`: Done and Stop, then every Event
    variant -> queue length 0, including a late reply and Begin.
  - `a_listener_off_loopback_skips_hooks`: None, 192.0.2.1:9898 and
    [2001:db8::1]:9898 -> Done after membership, no secret read/draw.
  - `tokens_are_paged_until_the_named_token`: two unrelated rows, then
    wanted id 7 -> page 2 limit 2 followed by DeleteToken(7).
  - `teams_are_paged_until_the_named_team`: two unrelated teams then id 7
    with the wanted spec -> page 2 limit 2 then TeamMembers(7).
  - `members_are_paged_until_the_bot`: two other users then kit -> page 2
    limit 2 then Done (no listener), with no AddMember.
  - `a_hook_on_page_two_restarts_that_page_after_delete`: page 1 has id 7
    and another URL, page 2 has id 3 -> DeleteOrgHook(7), Empty/404 ->
    re-list page 2 (not 1 or 3); final short page -> EditOrgHook(3).
  - `a_zero_page_cap_is_unexpected`: first listing via tokens or orgs,
    cap 0 -> Stop/Unexpected at Token or Org, with the literal subject.
  - `a_short_secret_draw_is_unexpected`: 31 bytes -> Hook/acme/Unexpected.
  - `an_admin_401_names_the_given_admin`: 401 -> Admin/kit-admin/Refused(401).
  - `a_token_create_conflict_stops_without_writing`: create status 422 ->
    Token/kit-api/Refused(422), no Write; no speculative retry of the race.
  In `tests/limits.rs`, 3 pure tests: `limits_at_both_relations_are_accepted`
  (backoff = backoff_max; separately max = deadline minus one nanosecond ->
  Ok); `a_limit_one_past_its_relation_names_the_pair` (backoff = max+1 ns;
  max = deadline -> respective literal pairs); `the_first_broken_relation_wins`
  (backoff 121 s, max 120 s, deadline 120 s -> Backoff/BackoffMax).
  Reference tests, with no assertions in phase setup:
```rust
// src/tests/steps.rs; hook_secret_read is a shared explicit prefix fixture.
#[test]
fn a_short_secret_draw_is_unexpected() {
    let mut run = super::fixtures::hook_secret_read();
    run.step(Event::Read(None));
    let short = [0x0a; super::fixtures::SHORT_DRAW_BYTES];
    let need = run.step(Event::Drawn(super::fixtures::secret(&short)));
    let expected = Failure {
        forge: 1, step: Stage::Hook, subject: b"acme".as_slice().into(),
        why: Why::Unexpected,
    };
    assert_eq!(need, Need::Stop(expected), "a draw has the requested length");
}
// src/tests/limits.rs
#[test]
fn limits_at_both_relations_are_accepted() {
    let mut limits = super::fixtures::limits();
    limits.backoff = limits.backoff_max;
    assert_eq!(crate::limits::check(&limits), Ok(()), "equality is admitted");
}
```
  Extend the reference with max = profile deadline minus 1 ns. steps.rs imports
  `crate::reconcile::{Event, Failure, Need, Stage, Why}`.
Proof: unit; `cargo nextest run -p skein-forge-setup` shows 54 passing
  tests, each <20 ms, suite <1 s. 04.1's 98-test base form lacks the new tests.
  No higher kit caller until skein 07.3; temper 04.1 connects the host.
  Keep tests/steps.rs under 800 lines; each test <=40.
Don't: change production to fit a test, duplicate Run or its builders, move
  the world, use a real clock/socket or add a fuzzy sweep to a unit crate.
Size: about 0 lines moved, 250 new, all tests (estimated: 21 tests, about 210, fixtures 40).
Removes: nothing.

## Done when

Commands from skein's root; base forms and results are in base-skein-04.

- `cargo nextest run -p skein-forge-setup` prints 54 passed, 0 skipped,
  suite <1 s, every unit <20 ms; the 13 moved names still appear unchanged.
- `cargo nextest list -p skein-forge-setup -E 'test(/^tests::reconcile::/)'`
  prints 25 tests: the 13 moved names plus 04.2's 12; no filter is used to run
  the suite. At the base its source form lists exactly the 13 moved names.
- `cargo clippy -p skein-forge-setup --all-targets -- -D warnings`
  exits 0, with no warnings.
- `rg -n -i 'temper' crates/skein-forge-setup` prints nothing (exit 1
  means no matches); the base source crate contains the names being removed.
