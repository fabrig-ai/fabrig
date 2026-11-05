+++
id = "adopt-plan"
part = "adopt-plan"
kind = "interface"
needs = []
files = [
  "skein/crates/skein-forge-setup/src/host.rs",
  "skein/crates/skein-forge-setup/src/host/boundary.rs",
  "skein/crates/skein-forge-setup/src/host/run.rs",
  "skein/crates/skein-forge-setup/src/forgejo/adopt.rs",
  "skein/crates/skein-forge-setup/src/forgejo/access.rs",
  "skein/crates/skein-forge-setup/src/tests/**",
  "skein/tests/forge/setup/**",
  "skein/docs/design/forge-setup.status.md",
]
effort = "high"
minutes = 45
+++

# Adopt given a repository's path

## Goal

`Plan::Adopt` names its repository by id or by path, so an owner who holds only the path a person gave can give the bot access to a repository the bot can't see yet. Reconcile's walk and the id form behave as they do now.

Acceptance:

- `host::Plan::Adopt { forge: u16, repository: host::Repository }`, with `host::Repository::{Id(NonZeroU64), Path { owner: Box<[u8]>, name: Box<[u8]> }}`.
- Given a path, the plan's step 2 is `GET /api/v1/repos/{owner}/{repo}` on the admin's token, each encoded as one path segment. Steps 3 and 4 use the path the answer spells, and a repository whose answer spells another case or name is the same repository.
- Given a path, a `404` stops `Repository`, `Missing`, a `403` stops `Repository`, `Refused(403)`, and every failure's subject is the path.
- The id form's requests, its check of the answer's id and its subject (the id in decimal before step 2 has read a path) are unchanged.
- The tests below pass and bite. `skein/docs/design/forge-setup.status.md` loses its `4.5` entry.

## Inputs

- `skein/docs/design/forge-setup.md`, 4.5, and 4.4 for a failure's subject.
- `temper/docs/plan/adopt-access/notes/common.md`.

## Owns and may change

Owns the `files` above.

- **Departure from your text:** you remove the `4.5` entry from `skein/docs/design/forge-setup.status.md`. That is the only `docs/design/` edit here.
- **Reader lines in the other tree, the value alone** (`repository: host::Repository::Id(repository)`): `temper/crates/temper-service/src/setup/lifecycle.rs`, the `host::Plan::Adopt` in `advance`; `temper/crates/temper-service/src/tests/setup_scopes.rs`, its `host::Plan::Adopt`. The `host::Plan::Adopt { .. }` patterns in `temper-service/src/setup.rs` and `lifecycle.rs` need nothing.
- **Readers in this tree, the value alone:** `skein/tests/forge/setup/src/plans.rs` (`adopt`), `skein/tests/forge/setup/tests/suite/hooks.rs`, `skein/tests/forge/setup/tests/suite/host.rs`, `skein-forge-setup/src/tests/host.rs`.
- **Options:** if `Plan` or a test needs `Repository` to derive more than `Plan` does, derive it. If an empty owner or name would need a new `Place`, don't add one: `Adopt::at` takes the bytes as they come, and the request's own encoding bounds answer them as for any request; say so in the report.

## Steps

`host/boundary.rs`:

```rust
/// The repository a plan gives the bot access to (forge-setup.md, 4.5).
pub enum Repository {
    Id(NonZeroU64),
    Path { owner: Box<[u8]>, name: Box<[u8]> },
}
```

Export it beside `Plan` in `host.rs`. `host/run.rs` builds `Adopt::new(given, id)` from `Id` and `Adopt::at(given, owner, name)` from `Path`.

`forgejo/access.rs`: `Access::named(at: Repo, bot, url)`, whose `begin` is `Request::Repository { at }` with `self.path` set from the start, so `subject` is the path throughout. `Phase::Repository`'s answer checks the id only when it was given one, then takes the answer's owner and name as now. `Request::Repository` answers the same `Document::Repository` as `RepositoryById` (`skein-forge/src/forgejo/client.rs`). Reconcile's walk and `create.rs` keep `Access::new` and `known`.

Tests in the world `skein/tests/forge/setup/tests/suite/host.rs`, beside `the_host_adopts_a_repository_and_hands_up_nothing`:

- `the_host_adopts_a_repository_by_its_path_and_hands_up_nothing`: `fixtures::repository_config(&[b"alice/specimen"])`, `Repository::Path`; the bot's `Grant::Write`, `done(1)`.
- `an_adopt_by_a_path_the_forge_lacks_stops_missing_naming_the_path`: `Failure { step: Stage::Repository, subject: b"alice/absent", why: Why::Missing, .. }`.

And in `skein/tests/forge/setup/tests/suite/hooks.rs`, beside its `Plan::Adopt`: by a path, with the listener, one hook at the listener's URL, and a second run leaves one.

In `skein-forge-setup/src/tests/` (the adopt runs in `fixtures.rs`, `AdoptRun`): by a path, the first request after the admin's is `GET /api/v1/repos/...` with a segment needing encoding encoded; a `403` there stops `Refused(403)` naming the path; the answer's own spelling is what the permission request uses.

Narrow tests, `cargo fmt`, `python3 -I -B skein/scripts/old_forms.py`, then:

```
heavy cargo clippy -p skein-forge-setup -p skein-forge-setup-world --all-targets -- -D warnings
heavy cargo nextest run -p skein-forge-setup -p skein-forge-setup-world
heavy cargo check -p temper-service --tests
```

Take the world's package name from `skein/tests/forge/setup/Cargo.toml`. Commit and push.

## Report

As your text says. Name each rule you broke by hand and the failure seen.
