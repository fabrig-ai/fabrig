+++
id = "link-readers-temper"
part = "link-values"
kind = "migration"
needs = []
files = ["temper/crates/temper-service/**", "temper/crates/temper-worker-domain/**", "temper/crates/temper-engine-domain/**", "temper/crates/temper-engine-smith/**", "temper/crates/temper/**", "temper/crates/temper-playground/**", "temper/tests/engine/domain/**", "temper/tests/client/**", "temper/tests/system/**", "temper/tests/conformance/**", "temper/tests/real-loop/**", "temper/tests/worker/domain/**", "Cargo.lock"]
effort = "medium"
minutes = 45
+++

# Temper names the link's values through `jig_domain_link`

## Goal

The first part of jig's D006. Temper reads the link's values through `jig_domain_host`, `jig_domain_fleet` and `jig_domain_accounts`: 202 uses in 54 files. They move to `jig_domain_link`, so jig's three `link-values-*` tasks can drop the re-exports. The re-exports stay until then.

Acceptance:

- This search finds nothing, where the search's names are the note's table (`notes/link-values.md`): `rg -n '\b(jig_domain_host|jig_domain_fleet|jig_domain_accounts|host|fleet|accounts)::(PROGRAM_ENDPOINTS|SourceName|valid_program_endpoints|Bounds|CountKind|Form|Grant|GrantTable|Hello|Hosting|JobBound|JobKind|Login|LoginMethod|LoginState|ModelBound|AgentOperation|WorkerProblem|Membership|Phase|Scheme|SignOutResult|SourceFailure)\b' temper -g '*.rs'`. A hit on another crate's alias of the same word, such as a local `host` or `accounts` module with its own `Form`, is exempted, not changed, and named in your report.
- Nor do `fleet::Worker`, `accounts::Worker` or `fleet::Operation` name a link value, and this finds nothing: `rg -nU 'jig_domain_(host|fleet|accounts)::\{[^}]*\b(PROGRAM_ENDPOINTS|SourceName|valid_program_endpoints|Bounds|CountKind|Form|Grant|GrantTable|Hello|Hosting|JobBound|JobKind|Login|LoginMethod|LoginState|ModelBound|AgentOperation|WorkerProblem|Membership|Worker|Operation|Phase|Scheme|SignOutResult|SourceFailure)\b' temper -g '*.rs'`. A name of the crate's own that happens to match, such as the hub's own `Operation`, is exempted.
- The net is at most +90 lines.

The searches are the check here: jig's own crates read the re-exports until the `link-values-*` tasks run, so temper can't be compiled without them yet. Those tasks compile temper's packages, and a reader you missed comes up there.

## Inputs

- `temper/docs/plan/debt-29/notes/link-values.md`, whole: the rule, the table, how a reader changes, what runs.
- `jig/docs/development/debt/D006-link-values-two-paths.md`.
- `jig/docs/design/domain/root.md`, 2.
- `temper/docs/plan/debt-29/notes/common.md`.

## Owns and may change

Owns, in the packages below, the `use` lines and the paths of these values, and each `Cargo.toml`'s dependency line. Nothing else of these files. `Cargo.lock` follows by cargo.

| Package | Files | Uses |
|---|---|---|
| `temper-service` | 20 | 84 |
| `temper-engine-domain-world` | 12 | 49 |
| `temper-worker-domain` | 6 | 22 |
| `temper-engine-domain` | 6 | 14 |
| `temper-engine-smith` | 2 | 12 |
| `temper-client-world` | 3 | 9 |
| `temper-system-world` | 2 | 6 |
| `temper-conformance-world` | 1 | 2 |
| `temper` | 1 | 2 |
| `temper-playground` | 1 | 2 |

A reader through `temper_worker_domain::host` (`temper/tests/real-loop/src/ending.rs`, `temper/tests/worker/domain/src/evidence.rs`) that names a link value takes the same edit, in those packages. If the temper boundary or dependency tests list a package's dependencies, the list gains `jig-domain-link`, and you say so.

Two of these files are other drafts' too: `temper-service/src/tests/first_model.rs` (`model-catalogue-1`) and `temper/tests/client/src/front.rs` (`chat-reading-c`). Your edits are `use` lines and paths alone, so fab's merge has little to meet. A use a merge brings takes the same edit.

Nothing in `temper-client-view`, `temper-client-domain` or `temper.css`: the cut found no reader there.

## Steps

Follow the note's **How a reader changes**, package by package, from the smallest. Then what the note's **What runs** lists, without `jig-boundary`, which reads jig's manifests alone, and without the temper check, which is for jig's tasks. Commit, push.

## Report

As your text says, with the searches' readings, the net lines, and each exemption.
