# client-api: code organization

The crates, modules and names this plan settles, the owner of each
shared interface, the checks it adds, the fixtures, the "Don't" list and
the size limits. Where the design names a crate or a module, the design
wins; names here are the plan's where the design leaves them to it.

## 1. Crates

### jig: renamed (jig 01), directories following the packages

| Old package and directory | New package | New directory |
|---|---|---|
| `jig-protocol-client-wire`, `crates/jig-protocol-client-wire` | `jig-client-protocol-wire` | `crates/jig-client-protocol-wire` |
| `jig-protocol-client-link`, `crates/jig-protocol-client-link` | `jig-client-protocol-link` | `crates/jig-client-protocol-link` |
| `jig-shell-native`, `crates/jig-shell-native` | `jig-client-shell-native` | `crates/jig-client-shell-native` |
| `jig-view`, `crates/jig-view` | `jig-client-view` | `crates/jig-client-view` |
| `jig-scripted-engine`, `testing/jig-scripted-engine` | `jig-client-scripted-engine` | `testing/jig-client-scripted-engine` |
| `jig-test-domain-client`, `testing/jig-test-domain-client` | `jig-client-test-domain` | `testing/jig-client-test-domain` |
| `jig-test-view`, `testing/jig-test-view` | `jig-client-test-view` | `testing/jig-client-test-view` |
| `jig-test-client-kit`, `testing/jig-test-client-kit` | `jig-client-test-kit` | `testing/jig-client-test-kit` |
| `jig-test-client-records`, `testing/jig-test-client-records` | `jig-client-test-records` | `testing/jig-client-test-records` |
| `jig-test-protocol-client-wire`, `testing/jig-test-protocol-client-wire` | `jig-client-test-protocol-wire` | `testing/jig-client-test-protocol-wire` |
| `jig-ops-domain-client`, `examples/ops/domain-client` | `jig-ops-client-domain` | `examples/ops/client-domain` |
| `jig-ops-view`, `examples/ops/view` | `jig-ops-client-view` | `examples/ops/client-view` |

`jig-client-world`, `jig-client-protocol-world`, `jig-client-wire-world`
and the engine's crates keep their names (jig's `README.md`, 15).

### jig: added, changed, removed

| Crate | Path | Kind | Holds | Made by |
|---|---|---|---|---|
| `jig-client-protocol-http` | `crates/jig-client-protocol-http` | step code under the subset, `no_std` with `alloc`; host and wasm32 | the default HTTP mapping's client end: an exchange as a method, a path under the prefix, headers and a body; a response as a reply or lost; the stream's events cut into messages (`protocol/client.md`, 17.4) | jig 10 |
| `jig-client-domain` | `crates/jig-client-domain` | step code under the subset | the client's domain: the link's state, watches by handle and their latest documents, the request table's use, kept work, families as bytes; the contract's types moved in from `jig-domain-client` (`client/domain/client.md`, 3) | jig 11 |
| `jig-client` | `crates/jig-client` | step code under the subset | `jig_client::Client`, composing `jig-client-domain` and `jig-client-protocol-link`; then the helpers as modules `shapes`, `facts`, `thread`, `words` (client.md, 3 and 19) | jig 12; helpers jig 15 |
| `jig-client-view` | `crates/jig-client-view` | step code under the subset | the reference UI: `jig-view`'s tree users and pages, and `jig-domain-client`'s page half (pages, places, navigation, the confirmation, drafts, notices, intents, cards) on `jig-client` and `skein-ui` alone; its state, `jig_client_view::Ui` (view.md, 18) | jig 01 renames; jig 13 grows it |
| `jig-domain-client` | `crates/jig-domain-client` | removed | its client half into `jig-client-domain`, its page half into `jig-client-view` | jig 11 to 13 |
| `jig-protocol-engine-clients` | unchanged | step code | owns no listener; takes `Mint`; its messages only, no HTTP (`protocol/client.md`, 4.2) | jig 17 |
| `jig-protocol-engine-clients-http` | `crates/jig-protocol-engine-clients-http` | step code | the default mapping's engine end and its example guards (`protocol/client.md`, 17.4; `protocol/browser.md`, 12.2) | jig 17 |
| `jig-client-shell-web` | `crates/jig-client-shell-web` | ordinary Rust, its platform half `#[cfg(target_arch = "wasm32")]` | the reference UI's browser parts over `skein-ui-web`: the mapping's client end over fetch, the printed link's sign-in, jig's look (`protocol/browser.md`, 5.1) | jig 19 |
| `jig-client-test-shell-web` | `testing/jig-client-test-shell-web` | ordinary Rust, wasm32 only (`cdylib`) | jig's test application's browser shell, written to be copied (`protocol/browser.md`, 3.1) | jig 19 |
| `jig-client-fake-person-browser` | `testing/jig-client-fake-person-browser` | ordinary Rust | jig's stories for a browser shell and its attack stories, on skein's face and harness (`protocol/browser.md`, 15) | jig 20 |
| `jig-client-browser-world` | `tests/client-browser` | a world, ordinary Rust | jig's browser tier: `browser_*` test binaries run by the `browser` profile | jig 19 |
| `jig-ops-client-protocol-wire` | `examples/ops/client-protocol-wire` | a codec crate generated from a schema | `ops`'s families' records (`examples.md`, 8; `protocol/client.md`, 11) | jig 21 |

The engine's step-2 changes stay in their crates: `jig-domain-core`,
`-tasks`, `-people`, `-authority`, `-views`, `-host`, `jig-protocol-store`,
`jig-protocol-config`, `jig-protocol-tools`, and their worlds.

### skein (skein 01)

`testing/skein-fake-person-browser`'s harness and face grow (the printed
line opened whole, extra origins, awaited trouble, a request with a given
`Host`); no crate is added. skein's code names no jig (skein's
`AGENTS.md`).

### temper

| Crate | Change | Session |
|---|---|---|
| `temper-protocol-client-wire` (`crates/temper-protocol-client-wire`) | renamed `temper-client-protocol-wire`, `crates/temper-client-protocol-wire` | temper 01 |
| `temper-shell-terminal` (`crates/temper-shell-terminal`) | renamed `temper-client-shell-terminal`, `crates/temper-client-shell-terminal`; its printer on `skein-ui-text`; its native client on `jig-client-shell-native`, the credential its host's | temper 01; temper 03 |
| `temper-client-domain` | the client root holding `jig-client`, two owners, routing by asker (temper's `client.md`, 3) | temper 03 |
| `temper-client-view` | on jig's reference UI as at the base (`jig-view`, which temper 02 follows to `jig-client-view`) until temper 07 (transitional, section 6), then temper's own pages on `skein-ui` and `jig-client` alone (client.md, 3.5 and 5) | temper 02, 03; temper 06, 07 |
| `temper-service` | the engine on step 2's operations and templates; the family on the collapsed wire; owns the clients' listener and mounts jig's component under `/jig` (client.md, 8) | temper 02, 04, 05 |
| `tests/client` (`temper-client-world`) | the naming check (temper 02.1); on `jig-client-world`'s referee and skein's fake person (temper 03); with jig's check, `jig_client`'s (jig 15.5), from temper 06 on (client.md, 10) | temper 02, 03, 06 |

## 2. Modules and names

- **The session files name modules** where the design leaves them open;
  a module the design names keeps the design's name.
- **One version constant per wire:** `jig_client_protocol_wire::VERSION`
  is the only definition of the client wire's version; the copy at
  `crates/jig-domain-client/src/contract/versions.rs:11` goes when the
  contract moves into `jig-client-domain` (jig 11).
- **No alias of a renamed crate or a moved type** (`pub use`, `type X =`,
  `extern crate … as`) kept for an old path, in jig or temper. One
  re-export is the design's: `jig-client`'s public API re-exports
  `jig-client-domain`'s types, the client's surface (client.md, 3), so
  that `jig-client-view` and an application depend on `jig-client`
  alone (view.md, 2).

## 3. Shared interfaces and their owners

| Interface | Owner | Users, by name |
|---|---|---|
| `jig_boundary::client_names::breaches` (section 4) | jig 01.4 | jig 13 (empties `PENDING`); temper 02.1 |
| a task's label, flags and last activity (`jig-domain-tasks`) | jig 02 | jig 03, 05, 08, 09, 14, 22 |
| `AsAllowed` and the escalation holder's check | jig 02 | jig 03, 04, 06, 14 |
| the parties' operations in `jig-domain-people` (`Create`, `Propose`, `Withdraw`) | jig 03 | jig 06, 08 (08.3), 14, 22 (22.4), 23 (23.1); the downstream repins |
| cancel with a grace, `Move`'s check, tracked and priority amended, the compound request | jig 04 | jig 06, 07, 09, 14 |
| `me`, the task line and task queries in the core | jig 05 | jig 08, 09, 22 |
| the agents' tools naming revisions | jig 06 | jig 14, 23 (23.2) |
| connectors' topics and procedures, the party's own scope and windows, an application's request by scope (07.6) | jig 07 | jig 14, 23 (23.2) |
| the ended indexes and read positions in the store | jig 08 | jig 08.5 (task queries' ended part, on jig 05's queries), jig 14, jig 22.3 |
| templates and policies' overrides in the configuration | jig 09 | jig 14 (a policy's edit), 22 (`me`, the policy document), 15 (`shapes`); temper 04 |
| `jig_client_protocol_http` | jig 10 | jig 12, 17, 19; temper 03 |
| `jig_client_domain`: the contract, handles and askers, the request table, kept work | jig 11 | jig 12, 13, 14, 22; temper 03.3 |
| `jig_client::Client`'s surface: the link's states, `me`, `SignIn { with }`, families as bytes | jig 12 | jig 13, 14, 15, 16, 19, 21; temper 03 |
| `jig-client-view`'s paths, with `jig-domain-client` moved in | jig 13.2 | jig 21; temper 03.4 |
| `jig_client_view::bridge` and the roots routing by asker; `jig_client_view::Ui` | jig 13.4 (`bridge`, ended by jig 21.3, 21.4); jig 21.3 (`Ui`) | jig 21; temper 03.5, 03.6 |
| the collapsed wire's records | jig 14 (requests), jig 22 (documents) | jig 15, 16, 17; temper 04 |
| `jig_client::{shapes, facts, thread, words}` | jig 15 | jig 16 (the reference UI's lists and acts); temper 06, 07 |
| jig's check for a client (`jig_client`'s, jig 15.5) | jig 15.5 | jig 16; temper 06.1, 06.3, 07.2 |
| `jig_protocol_engine_clients_http`'s mount and guards, `Mint` | jig 17 | jig 18; temper 05 |
| the clients' component without a listener, the test application's server owning it | jig 18.1 | jig 19; temper 05.2 |
| skein's harness: the printed URL, extra origins, awaited trouble | skein 01 | jig 19, 20 |

## 4. Checks the plan adds

- **The client naming rule** (jig's `README.md`, 15; temper's
  `client.md`, 2): "Every client crate, wherever it lives, is named
  `<project>-client-{domain,protocol,<kind>}-<...>`."
  - **jig 01.4** adds `jig_boundary::client_names` (a module of
    `tests/boundary`'s library, ordinary Rust):
    `pub fn breaches(root: &Path, project: &str, allowed: &[&str]) -> Vec<String>`,
    which reads each package's `Cargo.toml` under `root`, as the
    boundary's manifest walk reads them today (the workspace's own
    manifest is no package and no dependent), and returns, sorted, each
    crate that is client-side and whose name does not start with
    `<project>-client` or `<project>-<application>-client`
    (`jig-ops-client`), where `<application>` is one word and never a
    layer or kind word (`domain`, `protocol`, `engine`, `test`, `fake`,
    `scripted`, `conformance`, `shell`, `view`), so that
    `jig-domain-client` is no application's client. jig 01.4's file gives
    the definition in full, and wins where this summary differs. Client-side,
    over the normal and target dependencies (not dev-dependencies):
    (a) it depends on a crate named `<p>-client` or `<p>-client-*`, other
    than a wire or record crate (`*-client-protocol-wire`,
    `*-client-test-protocol-wire`, `*-client-test-records`, which the
    engine reads too), or on `skein-ui`, `skein-ui-text`, `skein-ui-web`,
    `skein-fake-person` or `skein-fake-person-browser`; (b) every crate
    of the workspace that depends on it, the allowed ones aside, is
    client-side, and there is at least one; (c) its name, after
    `<project>-`, has a segment `client`, or starts `shell-`, or has a
    segment `view`, unless it starts `<project>-engine-`. `allowed` names
    the crates README 15 keeps outside the prefix.
  - The test `client_crates_carry_the_client_prefix` in
    `tests/boundary/src/tests/graph.rs` calls it for jig with
    `allowed = ["jig-protocol-engine-clients",
    "jig-protocol-engine-clients-http", "jig-engine-clients-world",
    "jig-fake-parties", "jig-protocol-transcript", "jig-test",
    "jig-test-service", "jig-one-process-world", "jig-ops-world"]` and
    asserts the result equals a `PENDING` list, `["jig-domain-client"]`
    at jig 01.4, emptied by jig 13, the session that removes that
    crate. Two unit tests on fake manifests show a breach reported, one
    by (a), one by (c).
  - **temper 02.1** adds the same check to temper: a test
    `client_crates_carry_the_client_prefix` in
    `tests/client/tests/names.rs` (`temper-client-world`, which takes
    `jig-boundary` as a dev-dependency) calling
    `jig_boundary::client_names::breaches(<temper's root>, "temper",
    &["temper", "temper-one-process-world"])` and asserting it is empty.
  - Both run in the default suite, so the gate refuses a client
    crate without the prefix from their first landing on.
- **One wire version** (section 2): from jig 11's last landing on,
  `git -C ~/src/rust/jig grep -n -E '(CLIENT_VERSION|VERSION): u32 = [0-9]'
  origin/main -- crates` prints one line, in
  `crates/jig-client-protocol-wire/src/lib.rs` (at the base it prints two:
  that crate's under its old name, and `jig-domain-client`'s
  `contract/versions.rs:11`). README's Done when, 14, runs it.

## 5. Fixtures

Each crate's fixtures live in one module, made by the first session
that tests the crate, extended by the others, never redefined:
`src/tests/fixtures.rs` in a crate, `src/fixtures.rs` or the world's own
module in a world. New crates: `jig-client-protocol-http` (jig 10),
`jig-client-domain` (jig 11), `jig-client` (jig 12),
`jig-protocol-engine-clients-http` (jig 17), `jig-client-shell-web`
(jig 19). A moved module takes its tests and fixtures with it.

## 6. The plan's "Don't" list

The first six lines are guides that review reads (the author's own
reading before landing, the supervisor's hourly sample of a diff); a
miss found is fixed by the session's next increment. The lines after
them are rules, each with its check.

- No type beside an existing one for the same thing: change the type in
  place, and its users in the same increment.
- No field or predicate that its counterpart across a boundary lacks.
- No test that checks several behaviours under one name. A table test
  whose rows are cases of one behaviour is one behaviour.
- No lint exception in tests.
- No expected value computed by the code under test.
- No alias of a renamed crate or a moved type (section 2), and no
  re-export of one crate's items from another to keep old paths.
- No name of the repository downstream of jig in jig's files: `git -C
  <the session's jig worktree> grep -n -i -w temper -- crates testing tests
  examples scripts`
  prints nothing at each jig increment's end.
- No shape of work in the engine or the wire: no operation, document,
  subject, record or index named for a chat, a goal or a list (jig's
  `client/domain/contract.md`, 1). The shape-specific code that exists
  at the base goes in jig 16; until then it is edited only where an
  increment's Shape names the edit.
- No HTTP in `jig-client`, `jig-client-domain`,
  `jig-client-protocol-link` or `jig-protocol-engine-clients` once jig 10
  and jig 17 have taken it out.
- **Transitional code and its end** (development.md, 5.3), each ended
  by the increment named, which removes it:
  - `PENDING` in the naming check: jig 01.4 to jig 13;
  - temper's view on jig's reference UI (`jig-view`, then
    `jig-client-view` from temper 02): to temper 07;
  - the configuration's template-1 check, `templates::chat`, and
    `Settings.chat` and `close_grace` with their readers: jig 09.1 to
    jig 16;
  - the clients' component's answers for refusals and outcomes the wire
    cannot carry yet, lossy or an error, each named in its increment:
    jig 02.3 to jig 14;
  - the clients' component turning a v7 `Ask::Amend` refused as
    proposable into the party's `Propose` of it, answered v7's
    `AmendProposed`: jig 03.5 to jig 14.5;
  - the documents' fields jig 02 to 09 add (a task's label, flags and
    last activity, `waits_on`, `creating`, templates, `Read::Tasks`),
    left out by the clients' component's translation: to jig 22;
  - `jig-domain-client`'s client half beside `jig-client-domain`: jig
    11.2 to 13.2; then held in `jig-client-view` (`watches.rs`,
    `reads.rs`, `link.rs`, `versions.rs`, the watch spine's functions,
    the frame's link judgment): jig 13.2 to 21.3;
  - `jig_client_view::bridge`: jig 13.4 to 21.3; in it, the client's
    `NotKept`, `Limit`, `Unconfirmed` and `Unknown` shown as
    `Progress::Unknown`, its `Dropped` shown by nothing, and no
    `KeptGone` for kept work answered: jig 13.4 to 21.4;
  - `Conversation::stream` beside 11.5's run text: jig 11.5 to 21.3;
  - the link outside the roots, the old roots' `RequestTable` and their
    `NotKept` mapped to `Fact::InvalidAnswer`: jig 12.2 to 13.4 (the
    test application's root) and 21.2 (`ops`);
  - `me` opened at `Start`: jig 11.2 to 12.1;
  - `me` as `Subject::Frame`: to jig 22.1, which gives `me` its own
    shape; the window and cursor inside the subject and the read: to jig
    14.1;
  - the by-value hand-offs (`Frame`, `FrameIs`, `Ask { ByValue }`) and
    `Kept { scope, requests }` with `KeptGone`: to jig 21.4.

  README's Done when, 15, checks that the client's pieces are gone.
  Nothing else transitional is allowed without a ruling.
- The words the plan forbids in code: the pulse's `words` (README).

## 7. Size limits and the count

- **Aims:** a file of at most 800 lines and a test function of at most
  40; a miss is noted in the landing's report, and the landing stands. A
  table test keeps its rows in a `const` beside the function. A moved
  file keeps its size.
- **Rule: a file over 800 lines at the base grows by at most 40 lines**
  in one increment: for each such file the increment changes, `git diff
  --numstat origin/main...HEAD -- <file>` prints added and removed lines,
  and added minus removed is at most 40; the landing's report gives the
  two numbers. Code past that goes in a new module beside the file;
  more is a question before landing. The splits of those files are
  `later.md`'s.
- **An increment's size** is its hand-written production lines, counted
  once before it lands, in its worktree:
  - jig: `git diff -M --numstat origin/main...HEAD -- crates examples
    testing ':!*/tests.rs' ':!*/tests_*.rs' ':!*/tests/*' ':!*/generated/*'
    ':!*/golden/*' ':!*/schema/*' | awk '{a+=$1} END {print a}'` (lines
    added; a pure move counts 0, `-M` follows it; `tests_*.rs` are
    `#[cfg(test)]` modules, as `jig-domain-authority/src/lib.rs:43-50`
    declares them, ruling 140);
  - temper: the same with `-- crates testing tools`;
  - skein: the same with `-- crates testing`.
  `tests/` (worlds), generated code, goldens and schemas are not
  counted. Each landing's report gives the number.
- **Moves and renames:** an increment whose Shape is a move or a rename
  (jig 01.1 to 01.3, jig 11.1, jig 13.2, temper 01.2, temper 02.1's
  path rewrite, temper 03.3, temper 03.4, jig 15.4, jig 17.3) counts
  only what it adds
  beyond the move: its report gives the command's number and, beside
  it, the added lines that name a moved path or a new name, counted
  once with `git diff -M -U0 origin/main...HEAD -- <the same paths and
  exclusions> | grep -E '^\+[^+]' | grep -c -E '<the new names and
  paths, joined by |>'`; its Size is checked against the difference.
  Lines that rustfmt re-wraps because a path grew (a `use` list's
  continuation lines, a signature or a call re-wrapped) are rewrite
  lines too, though they name no new path (ruling 118). A file the Shape
  moves onto a path main already holds (jig 13.2's `domain.rs` onto the
  orphan `ui.rs`), which `-M` reads as an edit, counts as a move: its added
  lines are those of `git diff --numstat origin/main:<old path>
  HEAD:<new path>`, in place of `-M`'s for that file, and the old file's
  removal adds none (ruling 196). When the
  difference passes twice the Size, the report lists, by file, the
  difference's lines that are no such re-wrap; within twice the Size,
  the increment lands without asking. Nothing else is counted.
- An increment lands within twice its Size without asking (a codec or
  schema increment within 2.5 times); past that it asks, with the count
  by file.
