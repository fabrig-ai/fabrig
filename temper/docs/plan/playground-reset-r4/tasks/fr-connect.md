+++
id = "fr-connect"
part = "forge-records"
kind = "feature"
needs = ["fr-make-forge"]
files = [
  "temper/crates/temper-engine-domain-forge/src/connect.rs",
  "temper/crates/temper-engine-domain-forge/src/origin.rs",
  "temper/crates/temper-engine-domain-forge/src/lib.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/connect_new.rs",
  "temper/crates/temper-engine-domain-forge/src/tests.rs",
]
effort = "high"
minutes = 45
+++

## Goal

`Connect` on a forge to make (`ConnectForge::GitHub`,
`ConnectForge::Forgejo { address }`, from `forge-asks`) makes the
forge's record (`make::make`, `fr-make-forge`) and then connects it as
`Connect` connects a kept forge today (contracts, section 5: "`SetUp`
and `Connect` make the forge"). The person's input is the record's
source: GitHub's public addresses, or the origin they gave a Forgejo.

Acceptance:

- `GitHub` with `How::GitHubManifest` or `How::GitHubApp` makes a record
  as the document's GitHub forge reads (`startup/application.rs`'s
  `forge`, GitHub's null defaults): kind GitHub, api `Host {
  "api.github.com", 443 }`, `Tls { server_name: "api.github.com", trust:
  None }`, git `https://github.com`, web `https://github.com`,
  `read_only` false, identity with empty login and author (the bot's
  defaults), no files, no setup; then today's path on its number.
- `Forgejo { address }` with `How::ForgejoAdmin` makes a managed
  Forgejo record from the origin: `https://HOST[:PORT]` gives api `Host
  { HOST, PORT or 443 }` with `Tls { server_name: HOST }`;
  `http://IP:PORT` with a loopback IP gives api `Socket` and
  `Plaintext` (any other `http` origin is refused, as the document
  refuses plaintext off loopback); git the origin with a trailing `/`;
  web none; setup entry `Managed { admin_token: None }`, directory empty,
  bot `temper` / `temper@localhost`, no orgs; identity login `temper`,
  author `Temper` / `temper@localhost` (the template's forge 3's,
  `temper/crates/temper-init/template/temper.json`); then today's
  admin path on its number (its `first`, if any, as today).
- Refusals that do not depend on the forge (role, no key, busy
  capacity) are decided before the make, so a refused connect makes no
  forge. A malformed origin, a `How` that does not fit the kind
  (`ForgejoAdmin` on GitHub, a GitHub `How` on Forgejo) answer
  `ForgeRefusal::NotTaken` and make nothing. A make refused `Busy` or
  `Full` answers `ForgeRefusal::Busy`; `Invalid` answers `NotTaken`.
- A connect that is admitted and then fails leaves its forge kept,
  unfinished (draft 6.4: the welcome starts it over).
- `Existing(n)` is unchanged.

## Inputs

- `temper/docs/plan/playground-reset-r4/notes/common.md` and
  `temper/docs/plan/playground-reset-r4/notes/contracts.md` (section 5).
- The draft, sections 3 and 6.4.
- `temper/docs/design/protocol/setup.md`, sections 4, 5.5 and 17
  (connected, managed and GitHub's connects).
- `temper/crates/temper-service/src/startup/application.rs`, `forge` and
  `forge_endpoint` (the document's defaults and its plaintext rule).
- The merged `forge-asks` and `fr-make-forge` reports.

## Owns and may change

Owned: `connect.rs` (`start` and `admin`: the new-forge branch),
the new `origin.rs` (the origin's parse), `lib.rs` (its `mod` line), the
new `tests/connect_new.rs` and its `mod` line.

Neighbours: none.

Pre-approved options:

- If an origin with a path (`https://host/forgejo`) is wanted by a test
  or fixture, accept the path into `git` and refuse nothing; default:
  refuse a path, `NotTaken`.
- Ports: an explicit `:443` on `https` is kept as given; `http` needs a
  port.

## Steps

1. `origin.rs`: `pub(crate) fn forgejo(address: &[u8]) ->
   Option<(record::Address, record::Transport, Box<[u8]>)>` (api,
   transport, git), pure, `no_std`; module comment cites setup.md, 4.
2. `connect.rs`: at `start`, for `GitHub`/`Forgejo`, the
   forge-independent refusals, then the record, `make::make`, then the
   existing path with the number.
3. **Tests**, `tests/connect_new.rs`:
   `a_github_manifest_connect_makes_github_and_asks_its_manifest`,
   `a_github_app_connect_makes_github_and_connects_with_its_key`,
   `a_forgejo_admin_connect_makes_a_managed_forgejo_from_its_origin`,
   `a_loopback_http_origin_is_plaintext`,
   `an_http_origin_off_loopback_is_refused_and_makes_nothing`,
   `a_refused_role_makes_no_forge`,
   `a_failed_connect_leaves_its_forge_kept`; and `origin::forgejo`'s
   table test (`origins_parse_or_refuse`).
   Narrow: `-p temper-engine-domain-forge --lib -E 'test(connect_new::)
   | test(connect::)'`.
4. `cargo fmt`; `heavy cargo clippy -p temper-engine-domain-forge
   --all-targets -- -D warnings`; the crate's focused tests; `heavy
   cargo nextest run -p temper-engine-domain --lib -E 'test(setup::)'`.
   Commit and push.

## Report

fab's sections, and `## Interfaces`: `origin::forgejo`, the records each
`ConnectForge` makes, and the refusal mapping.
