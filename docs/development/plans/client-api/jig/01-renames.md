# Session 01: the client crates renamed, and the naming rule's check

jig's twelve client crates move under the `<project>-client` prefix
(jig's `README.md`, 15; `STATUS.md`, "jig's client API", step 1): five
crates of the wire, the link and the native shell with their test wire
and records, then the reference UI and the client's test kits, then
`ops`'s two client crates, each package renamed and its directory moved
with it, with no change of behaviour. Then `jig-boundary` gains the
naming rule as code, `jig_boundary::client_names::breaches`, public so
that an application's workspace can call it too, and a test that
fails the gate on a client crate of jig's outside the prefix, but
`jig-domain-client`, which the client's restructuring removes later in
this plan. The repository downstream of jig follows these renames in a
session of its own; nothing here edits it.

## Read first

- jig's `docs/design/README.md`: 15 (two pages).
- jig's `docs/design/STATUS.md`: section 3, "jig's client API", its
  step 1.
- `findings/renames.md` (105 lines).
- `code-organization.md`: 1 (the renames table), 2, 4 (the check), 6, 7.
- `decisions.md`: 8.
- jig's `tests/boundary/src/tests.rs` and `tests/boundary/src/tests/graph.rs`
  (the code this session changes).
- skein's `programming-model.md`: 4 and 10; `testing-strategy.md`: 8; by
  number only.

## What the design asks

- README 15: "**Client crates live under one prefix.** Every client
  crate, wherever it lives, is named
  `<project>-client-{domain,protocol,<kind>}-<...>`, so that everything
  client-related lives under `<project>-client`." The project is jig, or
  an application under its own name: `jig-ops` for `ops`.
- STATUS, step 1: "*the crates renamed* by the client naming rule
  (`README.md`, section 15), a rename and a crate move with no change of
  behaviour", the twelve of code-organization.md, 1. `jig-client-world`,
  `jig-client-protocol-world` and `jig-client-wire-world` keep their
  names; the engine's crates (`jig-protocol-engine-clients`,
  `jig-engine-clients-world`, `jig-fake-parties`) and
  `jig-protocol-transcript` "are not client crates and keep theirs".
- README 15, "Not client crates": the clients' component and its world,
  the default mapping's engine end, `jig-fake-parties`, and
  `jig-protocol-transcript`.

## Choices this plan makes

- **Directories follow the packages:** `crates/jig-client-*`,
  `testing/jig-client-*`, `examples/ops/client-domain`,
  `examples/ops/client-view` (`examples.md`, 2 names the last two); the
  worlds keep `tests/client*`.
- **Three rename increments in series,** cut as `findings/renames.md`, 3
  cuts them: they share the workspace `Cargo.toml`, `Cargo.lock`,
  `tests/boundary/src/tests/graph.rs`, `tests/client` and `tests/ops`.
  The check lands last, when only `jig-domain-client` breaches the rule.
- **The check is library code of `jig-boundary`,** ordinary Rust under
  `tests/clippy.toml`: `pub mod client_names` and `pub mod manifest`,
  not behind `#[cfg(test)]`, so that the application's workspace takes
  `jig-boundary` as a dev-dependency (decisions.md, 8). The manifest
  parser that `src/tests.rs` holds today (`CrateManifest`,
  `parse_manifest`, `workspace_manifest`, `quoted_value`,
  `dependency_name`, `walk_directory`, `source_files`, lines 7-104) moves
  into `manifest.rs`, and the graph tests read manifests through it:
  one parser.
- **What code-organization.md, 4 leaves open, settled** (each measured,
  `logs/w1/client-names-by-hand.txt`):
  - *Under the prefix* means a name `<x>-client…` or `<x>-<a>-client…`,
    where `<a>`, an application, is none of README 15's layer and kind
    words: `domain`, `protocol`, `engine`, `test`, `fake`, `scripted`,
    `conformance`, `shell`, `view`. With any word as `<a>`,
    `jig-domain-client` would pass as an application named `domain`, the
    check would flag nothing after the renames, and at the base it would
    pass `jig-test-client-kit` and `jig-test-client-records`.
  - The same shape names a seed of rule (a): a dependency `<x>-client…`
    or `<x>-<a>-client…` of any project, but the shared wires and
    records, or one of skein's five UI crates.
  - Rule (b)'s dependents are the workspace's packages: the workspace's
    own manifest, which has no package, is not one. Counted as one, it
    hides `jig-scripted-engine` at the base.
  - The result is crate names, sorted.

## Rules that bind it

- **No alias of a renamed crate** (code-organization.md, 2 and 6): no
  `package = "<old name>"` key, no `pub use` of a renamed crate, no
  `extern crate … as`. Checked at each rename increment's end by
  `git grep -n -E '<its pattern>' -- . ':!docs'`, once, which prints
  nothing; each old name ends at a word boundary, so that `jig-view` does
  not match `jig-views-world` (at `a3b53546` the three patterns match
  263, 592 and 62 lines):
  - 01.1: `(jig[-_]protocol[-_]client[-_]wire\b|jig[-_]protocol[-_]client[-_]link\b|jig[-_]shell[-_]native\b|jig[-_]test[-_]protocol[-_]client[-_]wire\b|jig[-_]test[-_]client[-_]records\b)`;
  - 01.2: `(jig[-_]view\b|jig[-_]scripted[-_]engine\b|jig[-_]test[-_]domain[-_]client\b|jig[-_]test[-_]view\b|jig[-_]test[-_]client[-_]kit\b)`;
  - 01.3: `(jig[-_]ops[-_]domain[-_]client\b|jig[-_]ops[-_]view\b|ops/domain-client\b|ops/view\b)`.
- **A rename changes no behaviour:** no test's name and no expected
  value change but where it prints a name this session renames (README,
  Rules, "Edits outside an increment's Files"), and no golden file's
  content changes: `git diff -M --numstat origin/main...HEAD -- '*/golden/*' | awk '$1+$2>0' | wc -l`
  prints 0.
- **jig's code names no repository downstream of it:** the vocabulary
  tests of `jig-boundary` (`tests.rs:172-188`'s words) run in the gate,
  and code-organization.md, 6's `git grep` prints nothing at each
  increment's end. New doc comments say "an application".

## Increments

### 01.1 The wire, the link, the native shell, the test wire and the records renamed

Scope: five rows of code-organization.md, 1: `jig-protocol-client-wire`
to `jig-client-protocol-wire`, `jig-protocol-client-link` to
`jig-client-protocol-link`, `jig-shell-native` to
`jig-client-shell-native`, `jig-test-protocol-client-wire` to
`jig-client-test-protocol-wire`, `jig-test-client-records` to
`jig-client-test-records`, their directories moved with them (README 15;
STATUS step 1); the boundary's checks that select crates by prefix keep
covering them (`findings/renames.md`, 2).
Needs: nothing.
Files (102 files edited and 3,576 moved at the rehearsal,
`logs/w1/jig-01.1-numstat.txt`): the moved crates,
`crates/jig-client-protocol-wire` (its `Cargo.toml`; `golden/v7`, 3,032
files, moved), `crates/jig-client-protocol-link` (34 files),
`crates/jig-client-shell-native` (6, its `clippy.toml` moved),
`testing/jig-client-test-protocol-wire` (1; `golden/v1`, 446 files,
moved), `testing/jig-client-test-records` (2, its `clippy.toml` moved);
their users, `crates/jig-protocol-engine-clients` (22 files),
`crates/jig-protocol-config` (2), `testing/jig-test-service` (4),
`testing/jig-test` (2), `tests/client-protocol` (9), `tests/client-wire`
(4: the golden paths at `tests/drift.rs:7`, `fuzzy_client_wire.rs:721`,
`test_client_wire.rs:7`), `tests/engine-clients` (9), `tests/one-process`
(2), `tests/boundary/src/tests.rs` and `src/tests/graph.rs`. Shared: the
workspace `Cargo.toml` (members and `[workspace.dependencies]` paths),
`Cargo.lock` (regenerated by cargo, never edited by hand), `graph.rs`.
Found by `git grep` of each old name, kebab and snake, and the
rehearsal's `cargo check --workspace --all-targets` (0 errors,
`logs/w1/jig-renames-check.txt`). Nothing in `scripts/`,
`.config/nextest.toml`, `clippy.toml` or `tests/boundary/fixtures/`
names them.
Shape: each package's `name` and directory as above; the `use`
lines, paths and manifest keys follow (`jig_protocol_client_wire` to
`jig_client_protocol_wire`, and so on); in `tests/boundary`:
- `application_breaches` (`graph.rs:371-388`) selects
  `name.starts_with("jig-protocol-") || (name.starts_with("jig-client-protocol-") && !name.ends_with("-world"))`
  and flags a dependency starting `jig-ops-`, `jig-test-` or
  `jig-client-test-`. Without `!name.ends_with("-world")` the prefix
  selects `jig-client-protocol-world`, and
  `protocol_crates_link_no_application_crate_even_when_empty` fails on
  its six test-kit dependencies (measured at the rehearsal);
- `smith_breaches` (`graph.rs:390-415`) selects `jig-client-shell-` in
  place of `jig-shell-`, and `jig-client-protocol-` names not ending
  `-world`;
- the sorted list of
  `engine_clients_link_only_their_boundary_wire_and_foundation`
  (`tests.rs:280-295`) puts `"jig-client-protocol-wire"` first.
Test shape: no new test. The renamed names in `tests.rs` and `graph.rs`
follow; no test function's name changes.
Proof: the gate, whose suites link the renamed crates: a rename
has no behaviour to show above the build and those suites. In it,
`engine_clients_link_only_their_boundary_wire_and_foundation`,
`protocol_crates_link_no_application_crate_even_when_empty` and
`only_the_charter_and_the_transcript_link_smith_even_when_empty`
(`jig-boundary`), and `client_wire_schema_matches_generated_code_and_goldens`
(`jig-client-wire-world`, reading the moved `golden/v7`) pass; each
matched one test at the base (`logs/w1/jig-base-filters.txt`). Bound:
the gate's.
Don't: let a drift test regenerate goldens (they write a missing golden
directory: move each with `git mv`, then `git status --short -- '*/golden/*'`
prints only renames); edit `docs/`; rename a world.
Size: at most 100 lines (69 at the rehearsal, by code-organization.md,
7's command, `logs/w1/jig-01.1-size.txt`).
Removes: the five old names.

### 01.2 The reference UI and the client's test kits renamed

Scope: `jig-view` to `jig-client-view`, `jig-scripted-engine` to
`jig-client-scripted-engine`, `jig-test-domain-client` to
`jig-client-test-domain`, `jig-test-view` to `jig-client-test-view`,
`jig-test-client-kit` to `jig-client-test-kit`, directories following
(code-organization.md, 1).
Needs: nothing beyond this session's 01.1, landed first (they share
the workspace `Cargo.toml`, `Cargo.lock` and `graph.rs`).
Files (110 files edited and 283 moved at the rehearsal,
`logs/w1/jig-01.2-numstat.txt`): `crates/jig-client-view` (its
`Cargo.toml`), `testing/jig-client-test-view` (18 files),
`testing/jig-client-test-kit` (4), `testing/jig-client-test-domain` (1),
`testing/jig-client-scripted-engine` (1); users: `examples/ops/view` (9),
`tests/client` (51), `tests/client-protocol` (8), `tests/ops` (7),
`tests/views` (7), `tests/boundary/src/tests/graph.rs`; shared: the
workspace `Cargo.toml`, `Cargo.lock`.
Shape: the names as above; in `graph.rs`, `client_step`,
`client_roles_cover_step_crates_and_exclude_the_world`,
`smith_breaches`' `name == "jig-client-view"` and the selections by name
follow, and three sorted lists are written sorted:
- `test_connector_vocabulary_and_client_value_kit_keep_their_exact_roles`
  (`graph.rs:253`): `["jig-client-test-domain", "jig-test-connector", "skein-lib"]`;
- `test_client_view_links_only_client_children_and_foundation`
  (`graph.rs:270`): `["jig-client-test-domain", "jig-client-view", "jig-domain-client", "skein-lib", "skein-ui"]`;
- `ops_client_view_is_step_code_with_only_client_and_connector_vocabulary`
  (`graph.rs:357-364`): `"jig-client-view"` first.
Test shape: no new test; no test function's name changes.
Proof: the gate, as 01.1's, with
`test_client_view_links_only_client_children_and_foundation` and
`client_roles_cover_step_crates_and_exclude_the_world` passing in it.
Bound: the gate's.
Don't: re-export `jig_client_view` items under the old path; touch the
reference UI's code beyond its paths.
Size: at most 200 lines (148 at the rehearsal: rustfmt rewraps the lines
that `jig_client_view::` lengthens, `logs/w1/jig-01.2-size.txt`).
Removes: the five old names.

### 01.3 ops's client crates renamed

Scope: `jig-ops-domain-client` to `jig-ops-client-domain` in
`examples/ops/client-domain`, `jig-ops-view` to `jig-ops-client-view` in
`examples/ops/client-view` (code-organization.md, 1; `examples.md`, 2).
Needs: nothing beyond this session's 01.2, landed first (shared files).
Files (25 files edited and 29 moved at the rehearsal,
`logs/w1/jig-01.3-numstat.txt`): `examples/ops/client-domain` (its
`Cargo.toml`), `examples/ops/client-view` (9 files), `tests/ops` (12),
`tests/boundary/src/tests/graph.rs` (the names, and the paths at
`graph.rs:344` and `:366`); the workspace `Cargo.toml` (members at lines
42-43, paths at 167-168), `Cargo.lock`.
Shape: the names and directories as above.
Test shape: no new test.
Proof: the gate, with
`ops_client_domain_is_step_code_with_only_client_and_connector_vocabulary`
and `ops_client_view_is_step_code_with_only_client_and_connector_vocabulary`
passing in it, each reading its moved `src/lib.rs`. Bound: the gate's.
Don't: rename `jig-ops-domain-connectors` (README 15 keeps it), or any
`jig-ops-domain-*`, `jig-ops-world`, `jig-ops-fake-production`.
Size: at most 40 lines (15 at the rehearsal, `logs/w1/jig-01.3-size.txt`).
Removes: the two old names.

### 01.4 The naming rule's check

Scope: code-organization.md, 4: `jig_boundary::client_names::breaches`
and the test that keeps jig's client crates under the prefix (README
15), with `PENDING = ["jig-domain-client"]`.
Needs: nothing beyond this session's 01.1 to 01.3, landed first. At the
base the rule flags thirteen crates, the twelve this session renames and
`jig-domain-client`; after 01.1 to 01.3, `jig-domain-client` alone (measured with the sketch below:
`logs/w1/jig-boundary-tests-base-2.txt` and
`logs/w1/jig-boundary-tests-renamed-2.txt`).
Files: `tests/boundary/src/lib.rs`, `tests/boundary/src/manifest.rs`
(new), `tests/boundary/src/client_names.rs` (new),
`tests/boundary/src/tests.rs` (the parser moved out; `manifests()`
calls `crate::manifest::manifests(&kit_root())`),
`tests/boundary/src/tests/graph.rs` (the three tests). Shared: `graph.rs`.
Shape (sketch below): `pub mod manifest` with `pub struct CrateManifest
{ pub path: PathBuf, pub name: Option<String>, pub dependencies:
Vec<String> }`, `pub fn parse_manifest(path: PathBuf, source: &str) ->
CrateManifest`, `pub fn manifests(root: &Path) -> Vec<CrateManifest>`
(the walk `tests.rs:160-170` does today, from `root`); `pub mod
client_names` with `pub fn breaches(root: &Path, project: &str,
allowed: &[&str]) -> Vec<String>` and `pub(crate) fn of(crates:
&[CrateManifest], project: &str, allowed: &[&str]) -> Vec<String>`. The
fields are `pub` because `path` is read by the graph tests only: a
`pub(crate)` field is dead code in the library build (clippy fails it,
measured). In non-test code, clippy's pedantic `assigning_clones` asks
`clone_into` at `parse_manifest`'s section line (measured).
Test shape: in `tests/boundary/src/tests/graph.rs`, beside the fixture
tests `a_protocol_crate_naming_an_ops_crate_is_reported`
(`graph.rs:481-488`), whose `parse_manifest(PathBuf::from("fixture/…"),
"<manifest>")` the unit tests reuse; the workspace test reads
`kit_root()` (`tests.rs:14-18`). The reference test:
```rust
/// Client crates outside the prefix (README.md, 15) until the increment
/// that removes them: `jig-domain-client`, which the client's
/// restructuring removes.
const PENDING: [&str; 1] = ["jig-domain-client"];

#[test]
fn client_crates_carry_the_client_prefix() {
    let allowed = [
        "jig-protocol-engine-clients",
        "jig-protocol-engine-clients-http",
        "jig-engine-clients-world",
        "jig-fake-parties",
        "jig-protocol-transcript",
        "jig-test",
        "jig-test-service",
        "jig-one-process-world",
        "jig-ops-world",
    ];
    assert_eq!(crate::client_names::breaches(&kit_root(), "jig", &allowed), PENDING);
}
```
and the unit test of rule (a), the shape of the other:
```rust
#[test]
fn a_crate_on_the_client_without_the_prefix_is_reported() {
    let page = parse_manifest(
        PathBuf::from("fixture/page/Cargo.toml"),
        "[package]\nname = \"jig-page\"\n[dependencies]\njig-client.workspace = true\n",
    );
    let client = parse_manifest(PathBuf::from("fixture/client/Cargo.toml"), "[package]\nname = \"jig-client\"\n");
    assert_eq!(crate::client_names::of(&[page, client], "jig", &[]), ["jig-page"]);
}
```
Tests, each under 20 ms (the workspace test took 10 ms at the
rehearsal): `client_crates_carry_the_client_prefix`,
`a_crate_on_the_client_without_the_prefix_is_reported` (rule (a)),
`a_crate_named_as_a_view_without_the_prefix_is_reported` (rule (c): a
manifest named `jig-pages-view` depending on `skein-lib` alone gives
`["jig-pages-view"]`).
Proof: the gate, in which `client_crates_carry_the_client_prefix`
passes with `PENDING = ["jig-domain-client"]`: from this landing on, the
gate refuses a client crate of jig's outside the prefix. The layer is
the check's own: it reads manifests, and nothing runs above it; its
call from the application's workspace is that repository's session.
At the rehearsal, on the renamed tree with the sketch, `jig-boundary`'s
69 tests passed (`logs/w1/jig-boundary-tests-renamed-2.txt`). Bound: the
gate's.
Don't: put `jig-domain-client` in `allowed`, or any crate the README 15
list does not keep outside the prefix; keep a second copy of the
manifest parser; add a lint exception; write the forbidden words of
`tests.rs:172-188` in a doc comment (the vocabulary tests read this
crate's source).
Size: at most 200 lines, counted by `git diff -M --numstat
origin/main...HEAD -- tests/boundary/src ':!*/tests.rs' ':!*/tests/*' |
awk '{a+=$1} END {print a}'` (code-organization.md, 7's command counts no
path under `tests/`): 194 at the sketch, of which 112 are `manifest.rs`,
moved from `tests.rs` with only its visibility and one `clone_into`
changed (`logs/w1/jig-01-4-size.txt`).
Removes: the parser's definitions in `src/tests.rs` (moved).

## Interface sketches

Checked at the base with clippy `-D warnings` (`logs/w1/jig-boundary-clippy-2.txt`),
with the bodies written in full in the scratch worktree; the bodies
below are elided but the classification, which is easy to get wrong.

```rust
// tests/boundary/src/lib.rs
pub mod client_names;
pub mod manifest;
```

```rust
// tests/boundary/src/manifest.rs — moved from src/tests.rs, made public.
/// One workspace manifest: its package's name, absent for the workspace's
/// own, and its normal and target dependencies (never dev or build ones).
#[derive(Debug)]
pub struct CrateManifest {
    pub path: PathBuf,
    pub name: Option<String>,
    pub dependencies: Vec<String>,
}
#[must_use]
pub fn parse_manifest(path: PathBuf, source: &str) -> CrateManifest;
/// The `Cargo.toml` files under `root`, but hidden, `golden` and root `target*`
/// directories (the walk the graph tests do today).
#[must_use]
pub fn manifests(root: &Path) -> Vec<CrateManifest>;
pub(crate) fn source_files(dir: &Path, at_root: bool, suffixes: &[&str], out: &mut Vec<PathBuf>);
pub(crate) fn walk_directory(name: &str, at_root: bool) -> bool;
pub(crate) fn workspace_manifest(source: &str) -> bool;
pub(crate) fn quoted_value(value: &str) -> Option<&str>;
```

```rust
// tests/boundary/src/client_names.rs
/// The layers and kinds of README.md, 15, which never name an application.
const KINDS: [&str; 9] = ["domain", "protocol", "engine", "test", "fake", "scripted", "conformance", "shell", "view"];
/// Records the engine reads too: depending on one makes no crate client-side.
const SHARED: [&str; 3] = ["-client-protocol-wire", "-client-test-protocol-wire", "-client-test-records"];
/// skein's UI kit and its person: only client crates depend on them.
const UI_KIT: [&str; 5] = ["skein-ui", "skein-ui-text", "skein-ui-web", "skein-fake-person", "skein-fake-person-browser"];

/// Each client-side crate under `root`, `allowed` aside, not named
/// `<project>-client…` or `<project>-<application>-client…`; sorted.
#[must_use]
pub fn breaches(root: &Path, project: &str, allowed: &[&str]) -> Vec<String>;
pub(crate) fn of(crates: &[CrateManifest], project: &str, allowed: &[&str]) -> Vec<String> {
    // (a) a dependency is a `seed`, or (c) the name is `shaped`.
    let mut side: BTreeSet<&str> = names(crates).filter(|n| deps(n).any(seed) || shaped(n, project)).collect();
    // (b), to a fixed point: the packages that depend on `n` (the workspace's
    // own manifest, which has no package, and `allowed` aside) are at least
    // one, and all in `side`.
    loop {
        let more: Vec<&str> = names(crates).filter(|n| !side.contains(n) && {
            let users = users(crates, n, allowed);
            !users.is_empty() && users.iter().all(|u| side.contains(u))
        }).collect();
        if more.is_empty() { break; }
        side.extend(more);
    }
    side.into_iter().filter(|n| !allowed.contains(n) && !(first(n) == project && under_client(n))).map(str::to_owned).collect()
}
/// `<x>-client…`, or `<x>-<a>-client…` with `<a>` not in KINDS.
fn under_client(name: &str) -> bool;
/// Rule (a): in UI_KIT, or `under_client` and ending in none of SHARED.
fn seed(dependency: &str) -> bool;
/// Rule (c): after `<project>-`, a `client` or `view` segment, or `shell-`
/// first; never a name starting `<project>-engine-`.
fn shaped(name: &str, project: &str) -> bool;
```

## Questions to raise when reached

- A crate landed by another session before 01.4 that the check flags
  (none is planned to: jig 10's and 11's crates are named under the
  prefix): ask; never add it to `PENDING` or to `allowed`.

## Done when

- 01.1 to 01.4 have landed, each with its gate.
- The three patterns of the Rules, each run once as `git -C <worktree>
  grep -n -E '<pattern>' origin/main -- . ':!docs'`, print nothing.
- `cargo nextest run -p jig-boundary -E 'test(client_crates_carry_the_client_prefix)'`,
  once, passes one test, and `git grep -n 'PENDING' origin/main --
  tests/boundary/src` prints a line containing `["jig-domain-client"]`.
