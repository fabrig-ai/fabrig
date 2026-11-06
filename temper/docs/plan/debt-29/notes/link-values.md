# debt-29: the link's values, one path each

For `link-readers-temper`, `link-values-accounts`, `link-values-fleet` and `link-values-host`, which together pay jig's D006. Read `jig/docs/development/debt/D006-link-values-two-paths.md` whole, and `jig/docs/design/domain/root.md`, 2.

## The rule

The link's values are named through `jig_domain_link`, and `jig-domain-fleet`, `jig-domain-host` and `jig-domain-accounts` don't re-export them. These tasks change paths and manifests, and nothing else: no type, value or behaviour.

| Re-exporter | Its `pub use jig_domain_link` names |
|---|---|
| `jig-domain-accounts` | `SourceName`, `Worker`, `Form`, `Grant`, `GrantTable`, `Scheme` |
| `jig-domain-fleet` | `Membership`, `PROGRAM_ENDPOINTS`, `Problem as WorkerProblem`, `SourceName`, `Worker`, `valid_program_endpoints`, `Bounds`, `CountKind`, `Form`, `Grant`, `GrantTable`, `Hello`, `Hosting`, `Login`, `LoginMethod`, `LoginState`, `ModelBound`, `Operation`, `Phase`, `Scheme`, `SignOutResult`, `SourceFailure` |
| `jig-domain-host` | `PROGRAM_ENDPOINTS`, `SourceName`, `valid_program_endpoints`, `Bounds`, `CountKind`, `Form`, `Grant`, `GrantTable`, `Hello`, `Hosting`, `JobBound`, `JobKind`, `Login`, `LoginMethod`, `LoginState`, `ModelBound`, `Operation as AgentOperation`, `Phase`, `Scheme`, `SignOutResult`, `SourceFailure` |

Most readers reach them through an alias, `use jig_domain_host as host;`, and `host::SourceName`. A file's alias stays for what the crate itself owns, `host::LinkDown` among it.

## How a reader changes

- **Import by name** where the file has nothing else of that name: `use jig_domain_link::{SourceName, Worker};`, and `SourceName` at each use.
- **Otherwise the full path:** `jig_domain_link::Form`. That's the case when the file already has an `Operation`, a `Form`, a `Problem` or a `Worker` of its own, as `temper-service` has for `Worker`.
- **Add no `link` alias.** `use jig_protocol_worker_link as link;` holds that name across `temper-service`, so one word would mean two crates.
- **The renames go, where the file can bear it:** `host::AgentOperation` becomes `jig_domain_link::Operation`, and `fleet::WorkerProblem` becomes `jig_domain_link::Problem`. A file with its own `Operation` or `Problem` writes the full path, or `use jig_domain_link::Operation as AgentOperation;` if that's what the file reads best with.
- **An alias left with no use goes,** and a dependency left with no use goes from the manifest. `cargo` won't tell you; check each manifest you touched by `rg` for the crate's name in its package.
- **A manifest gains `jig-domain-link.workspace = true`,** in `[dependencies]` or `[dev-dependencies]` as its reader needs it, in the section's order. `Cargo.lock` follows by cargo, never by hand.

## Inside the re-exporter

A private `use jig_domain_link::{..};` at the crate's root keeps `crate::SourceName` working for the crate's own modules (about 75 such uses across the three), so they don't change. Keep only the names the crate still reads there: rustc's `unused_imports` tells you.

## What's unchanged

- `jig-client-view` and `temper-client-view` read none of these values: `rg` says so at the cut. If fab's merge brings one, it's a question.
- `temper-worker-domain/src/lib.rs`'s `pub use jig_domain_host as host;` re-exports the hub's own, and stays. Its readers of a link value, if `link-readers-temper` finds any, name `jig_domain_link`.
- `pub use boundary::{.., Scheme, Worker}` in `jig-protocol-engine-workers` and the `Worker` types of `jig-protocol-config`, `jig-test-service` and `temper-service` are their own types, not the link's, unless the compiler shows otherwise.
- Generated code isn't touched: a `generated/` directory is its tool's.

## What runs

A path holds no behaviour, so this departs from the implementer's text: the focused tests of the packages you changed are the trunk's end check's, unless a compile repair changed more than a path, in which case run that package's tests.

- `heavy cargo clippy -p PKG [-p PKG ...] --all-targets -- -D warnings`, naming every package you changed. That compiles their tests.
- `heavy cargo nextest run -p jig-boundary`, since it reads every jig manifest.
- **The temper check, for a `link-values-*` task:** `heavy cargo check --tests -p temper-service -p temper-engine-domain-world -p temper-worker-domain -p temper-engine-domain -p temper-engine-smith -p temper-client-world -p temper-system-world -p temper-conformance-world -p temper -p temper-playground`, since the re-exports you removed were theirs too. A reader `link-readers-temper` missed takes the same edit, in the path alone, and your report names it: that's the one temper line such a branch may carry.
- `cargo fmt`; `(cd TREE && python3 -I -B scripts/old_forms.py)` for your tree; `python3 -I -B tools/citations/citations.py --all`; `python3 -I -B tools/debt/debt.py`.

## Lines

These tasks can't end with fewer lines: the item says each file gains about a line. Your spec's cap is on the net, and your report gives it. A branch stays in its tree: `link-readers-temper` edits temper alone, and each `link-values-*` edits jig alone, but for the temper check's path lines above.

## Who's next

Each `link-values-*` branch lowers the item's **Measure** by the two lines it removes, so an item left open by a run cut short tells the truth. `link-values-host` deletes it.
