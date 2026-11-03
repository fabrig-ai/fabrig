+++
id = "forge-connect"
part = "engine"
kind = "feature"
needs = ["forge-surface"]
files = [
  "temper/crates/temper-engine-domain-forge/src/connect.rs",
  "temper/crates/temper-engine-domain-forge/src/domain.rs",
  "temper/crates/temper-engine-domain-forge/src/forges.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/connect.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/make.rs",
  "temper/crates/temper-engine-domain/src/tests/setup.rs",
  "temper/crates/temper-service/src/route.rs",
]
effort = "high"
minutes = 150
+++

# The forge domain admits a tokens connect and a Replace, and keeps the checked login

## Goal

The forge connector decides a connected Forgejo's `Connect` with its tokens (a new forge's record,
or a kept one connected again) and a `Replace` of a GitHub App's key or a connected Forgejo's
tokens, and asks the routing to run them; it keeps the login a `Check` found as the record's
bot. Until the routing stages them (`check-connect`, `replace-run`), the routing answers both
`not taken`, so nothing panics on the trunk meanwhile.

Acceptance:

- **A tokens connect of a new forge** (`Making::New(ForgeAsk::Connected { place, git_program })`,
  `How::ForgejoTokens`), decided in `connect::make` beside the managed one: the owner's role,
  then `NoKey` without a carried value, `Busy` when the tables are full, then the record made by
  `forges::make` as `(Kind::Forgejo, Mode::Connected, place, git_program)`, `read_only` false, no
  first repository, the bot `make::bot()`; then `Connecting` and `Request::Connect { forge,
  request, how: How::ForgejoTokens, carried }`. `ForgeAsk::Connected` with any other `How`, and
  `ForgeAsk::Managed` or `GitHub` with `ForgejoTokens`, are `NotTaken` with no record made.
- **A tokens connect of a kept forge** (`Making::Kept`), modelled on `admin` (`:335-418`): role;
  `NoSuchForge`; `NotTaken` unless a Forgejo of the connected mode, not ended; `Connected` while
  connected, connecting or setting up; `Busy`; `NoKey`; then the previous connection kept in
  `previous`, `Connecting`, `Request::Connect`.
- **A Replace** (`Ask::Replace { forge }`): role; `NoSuchForge` (none kept, or ended); `NotTaken`
  unless a GitHub forge (any mode but fake) or a Forgejo of the connected mode; `Busy` while a run
  holds the forge (connecting among them) or the tables are full; `NotConnected` unless connected
  or stopped; `NoKey`; then `previous` kept, `Connecting`, and `Request::Replace { forge,
  request, carried }`. Its answer is `Answered::Connect`, given by the routing's
  `ConnectAnswered` as a connect's is. Every refusal names the carried value unused, as today.
- **A run's end** (`connect::ended`): a tokens connect or a Replace that stops restores the
  connection kept in `previous` (connected, stopped or not connected) and shows its `why` on the
  row, with no typed `AdminStop`; done, it clears `previous`. `ended`'s `admin` flag, read today
  from `previous`, is read from the run's own kind, since these runs keep `previous` too; a
  GitHub Replace done marks the App's way `Pasted`, as a pasted App's connect does.
- **`Event::BotLogin { forge, login }`** (`notes/contracts.md`, C4, step 4): on a kept, not ended
  Forgejo of the connected mode whose record's `bot.login` differs, and a login of 1 to 255
  bytes, the record's `bot.login` set, `forges::save`d, and `Request::ForgeKept` pushed, as
  `send`'s `read_only` rewrite does (`connect.rs:266-273`); otherwise nothing.
- **The routing's interim:** `temper-service/src/route.rs`'s arms for `How::ForgejoTokens` and
  `root::Request::Replace`, `unreachable!` since `forge-surface`, push the forge domain
  `Event::ConnectAnswered { forge, request, answer: ConnectAnswer::Refused(ForgeRefusal::NotTaken)
  }` (through `engine.events`, as `setup::public`'s `event` does), each with a comment that
  `check-connect` and `replace-run` stage them.
- Tests below pass; `forge-surface`'s two stub tests are replaced by them.

## Inputs

- `temper/docs/plan/forgejo-connect/notes/common.md`; `notes/contracts.md`, C3 and C4.
- `temper/docs/design/protocol/forge.md`, 2.1 (the bot), 2.2 (making a forge; a Check's end; a
  forge's user stays; Replace) and 8.1.
- `temper/docs/design/protocol/setup.md`, section 4; `client/README.md`, 4.6 (`ForgeRefusal`).

## Owns and may change

- `temper-engine-domain-forge/src/connect.rs`: `make`, `start`, `admin` or a sibling for the
  tokens connect, a `replace` admission, `send`, `connected` (the routing's first answer, whose
  refusal already restores `previous`), `ended`, and the run's kind kept in `running` (an enum
  of your naming: admin, tokens, replace, app) so `ended` reads it; a `bot_login` for
  `Event::BotLogin`.
- `domain.rs`: the dispatch (`:1237-1254`) and `Event::BotLogin`'s arm call into `connect.rs`.
- `forges.rs` only if `save` or `make` needs a parameter for this; say so if it does.
- Tests, in `src/tests/connect.rs` (copy `admin_connect_*` at `:658-901` and
  `admitted_connect_marks_connecting_answers_then_erases_once_at_its_end` at `:125`):
  - a new connected Forgejo made, numbered next, its record's fields as above, `Connecting`,
    `Request::Connect` with `ForgejoTokens` and the carried value; `ForgeKept` pushed;
  - its refusals in order (role, not taken for each pairing above, busy, no key), each naming
    the carried value unused and making no record;
  - a kept connected Forgejo: admitted not connected and stopped; refused `Connected` while
    connected or connecting; a stop restoring `Stopped { .. }` with the run's `why` on the row;
  - a Replace admitted on GitHub connected and stopped, and on a connected Forgejo connected and
    stopped, each `Request::Replace`; refused `NotTaken` on a managed, owned or fake Forgejo,
    `NotConnected` on a forge not connected or not set up, `Busy` while connecting, `NoKey`
    without a value; a Replace that stops on a connected forge leaves it connected, with its
    `why`;
  - `BotLogin`: the login rewritten, saved and `ForgeKept` once; the same login, a managed or
    GitHub forge, an ended forge, an empty login and one of 256 bytes, each changing nothing.
- `src/tests/make.rs`: a test beside `connect_new_managed_makes_a_record_with_its_intent`
  (`:144`) for the connected record.
- `temper-engine-domain/src/tests/setup.rs`: beside
  `admin_connect_routes_the_whole_how_and_erases_at_both_ends` (`:650`), a tokens connect and a
  Replace routed to the root's `Request::Connect` and `Request::Replace` with their carried
  values, and erased at both ends.
- `temper-service/src/route.rs`: the interim arms above, nothing else.

Pre-approved: if an existing test asserts that a Forgejo of the connected mode is `NotTaken` for
a connect it now takes (`tests/connect.rs:114` asserts it for a GitHub App's, which stays
`NotTaken`), keep it for the ways it still refuses and say so. Rows (`Domain::forges`) do not
change: the client reads both offers from kind, mode and connection.

## Steps

1. The admissions and their tests: `heavy cargo nextest run -p temper-engine-domain-forge --lib
   -E 'test(connect) | test(make)'`.
2. `ended`, `BotLogin` and their tests, the same run.
3. The root's routing tests: `heavy cargo nextest run -p temper-engine-domain --lib -E
   'test(setup)'`.
4. The service's interim arms: `heavy cargo nextest run -p temper-service --lib -E 'test(route) |
   test(setup_github)'`.
5. `cargo fmt`; old forms for temper; `heavy cargo clippy -p temper-engine-domain-forge -p
   temper-engine-domain -p temper-service --all-targets -- -D warnings`; the focused tests of
   those packages, and of `temper-engine-domain-world` (the forge domain's world, whose stories
   make and remove forges). Commit; push.

## Report

As the implementer's text says, with the run's kind's name and where `ended` reads it.
