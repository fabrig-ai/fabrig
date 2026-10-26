# Session 02: jig's renames followed, and the naming check

temper repins onto jig 01.4's landing, the newest jig step its Needs
name, and follows jig's twelve client crates to their new names (jig's
`README.md`, 15; code-organization.md, 1): its manifests and its `use`
paths name `jig-client-view`, `jig-client-shell-native`,
`jig-client-protocol-wire`, `jig-client-protocol-link` and
`jig-client-test-domain` in place of `jig-view`, `jig-shell-native`,
`jig-protocol-client-wire`, `jig-protocol-client-link` and
`jig-test-domain-client`. Then temper's joint world takes jig's naming
check, `jig_boundary::client_names::breaches`, as a dev-dependency, and
a test fails the gate on a client crate of temper's workspace outside
`temper-client`, with nothing pending, since temper 01 renamed its two.
The session repairs what jig 01 breaks, and adds the check; nothing
else.

## Read first

- jig's `docs/design/README.md`: 15; temper's `STATUS.md`: section 2,
  the `client.md` row ("the crates `temper-protocol-client-wire` renamed
  `temper-client-protocol-wire` and `temper-shell-terminal`
  `temper-client-shell-terminal`, by the client naming rule"); temper's
  `client.md`: 2.
- `code-organization.md`: 1 (jig's renames table), 3 (the check's
  owner), 4 (the check); `findings/renames.md`: 1 (temper's column) and
  5; `decisions.md`: 8; the README's Rules, "Pins" and "What a temper
  repin repairs".
- `jig/01-renames.md`, 01.4 and its interface sketch (the function this
  session calls).
- skein's `testing-strategy.md`: 8, by number only.

## What the design asks

- jig's README 15: "Every client crate, wherever it lives, is named
  `<project>-client-{domain,protocol,<kind>}-<...>`, so that everything
  client-related lives under `<project>-client`."
- temper's `STATUS.md`, section 2, the `client.md` row: "the crates
  `temper-protocol-client-wire` renamed `temper-client-protocol-wire` and
  `temper-shell-terminal` `temper-client-shell-terminal`, by the client
  naming rule" (temper 01 did both).
- code-organization.md, 4: "**temper 02.1** adds the same check to
  temper: a test `client_crates_carry_the_client_prefix` in
  `tests/client/tests/names.rs` (`temper-client-world`, which takes
  `jig-boundary` as a dev-dependency) calling
  `jig_boundary::client_names::breaches(<temper's root>, "temper",
  &["temper", "temper-one-process-world"])` and asserting it is empty."

## Choices this plan makes

- **One increment:** the renames followed and the check land together;
  the check reads the manifests the renames edit, and both are small.
- **The pin is jig 01.4's landing,** `<landing>` below: the commit
  `git -C ~/src/rust/jig fetch origin && git -C ~/src/rust/jig log origin/main --all-match --grep='^Plan: client-api$' --grep='^Step: jig-01.4$' --format=%H -1`
  prints, run once at the increment's start (older plans reuse step
  names, hence the plan's trailer too). Never jig's `origin/main`: what
  jig lands after 01.4 is for the temper session whose Needs name it.
- **The repin commit carries the manifests' renames.** `cargo update`
  refuses a manifest that names a package jig no longer has (temper
  01's probe met it with `jig-fake-person`, `findings/temper.md`, 1), so
  the dependency keys of the eight manifests change in the repin commit,
  with `Cargo.lock`; the `.rs` files follow in the increment's commit.
- **The root the test reads** is its package directory's `../..`,
  canonicalized, as jig's `kit_root` reads its own
  (`tests/boundary/src/tests.rs:8-12`): a binary from the build cache
  may have been built in a worktree since removed, so
  `env!("CARGO_MANIFEST_DIR")` is not used.

## Rules that bind it

- **Pins** (README, Rules), onto `<landing>`:
  - when `git -C ~/src/rust/jig merge-base --is-ancestor <landing>
    <temper's present jig pin>` exits 0 (the pin read with `grep -o
    'jig.git#[0-9a-f]*' Cargo.lock | sort -u`), no repin commit is made;
  - otherwise, after the repin, `grep -o
    'git+https://github.com/forgetic/[a-z]*.git#[0-9a-f]*' Cargo.lock |
    sort -u` prints three lines: `jig.git#<landing>`, and skein's and
    smith's lines equal to those `git -C ~/src/rust/jig show
    <landing>:Cargo.lock | grep -o
    'git+https://github.com/forgetic/[a-z]*.git#[0-9a-f]*' | sort -u`
    prints. A source with a second revision runs its `cargo update`
    once more (temper 01.1 measured the skein one needing it); a second
    revision still there after that is a question.
- **What a repin repairs** (README, Rules): jig 01's renames, by this
  increment's Shape; what else jig landed between temper 01's pin
  (`a3b53546`) and `<landing>`, adapted the way jig adapted `jig-client-test-domain`,
  `jig-test-service` and `ops` in the commit of that change, in at most
  200 lines besides `Cargo.lock`. More is a question, never a reason to
  pin an older jig.
- **No alias of a renamed crate:** no `package = "jig-view"` key or the
  like; after the increment `git grep -n -E
  '(jig[-_]protocol[-_]client[-_]wire\b|jig[-_]protocol[-_]client[-_]link\b|jig[-_]shell[-_]native\b|jig[-_]test[-_]protocol[-_]client[-_]wire\b|jig[-_]test[-_]client[-_]records\b|jig[-_]view\b|jig[-_]scripted[-_]engine\b|jig[-_]test[-_]domain[-_]client\b|jig[-_]test[-_]view\b|jig[-_]test[-_]client[-_]kit\b|jig[-_]ops[-_]domain[-_]client\b|jig[-_]ops[-_]view\b)'
  -- . ':!docs'`, once, prints nothing (each old name ends at a word
  boundary, so that `jig-view` does not match `jig-views-world`).

- **temper's `STORE_VERSION`** (README, Rules, "Format versions"): if
  `<landing>` has jig 02.3's landing as an ancestor (`git -C ~/src/rust/jig
  fetch origin && git -C ~/src/rust/jig merge-base --is-ancestor $(git -C
  ~/src/rust/jig log origin/main --all-match --grep='^Plan: client-api$'
  --grep='^Step: jig-02.3$' --format=%H -1) <landing>` exits 0), 02.1
  moves it once, with its goldens, and its report says so; otherwise a
  later temper increment does.

## Increments

### 02.1 jig's renames followed, and the naming check

Scope: temper's manifests and code on jig's renamed client crates
(code-organization.md, 1); the naming check of code-organization.md, 4
in temper (decisions.md, 8; jig's README 15).
Needs: jig 01.4's `jig_boundary::client_names::breaches`, with jig's
renamed crates before it, landed (`<landing>`, the Choices' command);
temper 01 landed (its two crates renamed, its pins on jig's base).
Files (measured at the plan's base on temper 01.1's rehearsal, then
named by temper 01.2's paths: 8 manifests and 34 source files, 113 lines
added of which 54 counted, `logs/w1/temper-02-files.txt` and
`logs/w1/temper-02.1-size.txt`):
- manifests: the workspace `Cargo.toml` (five `[workspace.dependencies]`
  keys, and `jig-boundary = { git = "https://github.com/forgetic/jig.git" }`
  added), `crates/temper-client-view/Cargo.toml`,
  `crates/temper-service/Cargo.toml`,
  `crates/temper-client-shell-terminal/Cargo.toml`,
  `crates/temper/Cargo.toml`, `tests/client/Cargo.toml` (and its new
  `[dev-dependencies]` `jig-boundary.workspace = true`),
  `tests/codecs/Cargo.toml`, `tests/one-process/Cargo.toml`; `Cargo.lock`;
- `crates/temper-client-view/src/{forge.rs, forge/bindings.rs,
  forge/page.rs, forge/words.rs, limits.rs, root.rs, words.rs,
  tests/fixtures.rs, tests/forge_page.rs}`;
- `crates/temper-service/src/{clients.rs, family.rs, family/refusals.rs,
  tests/clients.rs, tests/clients/peer.rs, tests/family.rs}`;
- `crates/temper-client-shell-terminal/src/{client.rs, limits.rs,
  route.rs, route/family.rs, terminal.rs, terminal/native.rs, tests.rs,
  tests/family.rs, tests/fixtures.rs}`,
  `crates/temper-client-shell-terminal/tests/support/mod.rs`;
- `crates/temper/tests/support/person.rs`;
- `crates/temper-client-protocol-wire/schema/client-wire-v1.schema` (a
  comment naming jig's wire);
- `tests/client/src/{app.rs, fixtures.rs, translate.rs}`,
  `tests/client/tests/names.rs` (new);
- `tests/codecs/tests/client_wire.rs`;
- `tests/one-process/src/{referee/client.rs, referee/client/cache.rs,
  world.rs}`.
Found by `git grep` of the twelve old names, kebab and snake, outside
`docs/`; `temper-client-domain` names none of them (it uses
`jig_domain_client`, which jig 01 keeps). Shared hotspot: `Cargo.lock`.
Shape:
1. The repin commit, unless the Pins rule's ancestor check exits 0: the
   eight manifests' dependency keys renamed, then,
   `cargo update -p jig-domain-core --precise <landing>`, `cargo update
   -p skein-lib --precise <skein's revision>`, `cargo update -p
   smith-host-protocol --precise <smith's revision>` (each revision from
   `git -C ~/src/rust/jig show <landing>:Cargo.lock | grep -o
   'git+https://github.com/forgetic/[a-z]*.git#[0-9a-f]*' | sort -u`),
   and the Pins rule's three lines.
2. Each `use`, path and type of the five renamed crates temper names
   follows: `jig_view` to `jig_client_view`, `jig_shell_native` to
   `jig_client_shell_native`, `jig_protocol_client_wire` to
   `jig_client_protocol_wire`, `jig_protocol_client_link` to
   `jig_client_protocol_link`, `jig_test_domain_client` to
   `jig_client_test_domain`; nothing else in those lines changes, then
   `cargo fmt`, which reorders the `use` lines the renames move (4
   files, 17 lines at temper `f6f01026`).
3. The repairs of what else jig landed between `a3b53546` and
   `<landing>`, each following
   jig's own commit for it, within the README's 200 lines.
4. The check, in `tests/client/tests/names.rs`, below.
Test shape: one new test file, `tests/client/tests/names.rs`, a test
binary of `temper-client-world` (`names`, in the focused suite: the
default profile excludes only `fuzzy_*`, `process` and `live`). The
reference test, the file's only one:
```rust
//! temper's client crates live under `temper-client` (jig's README.md, 15).

#[test]
fn client_crates_carry_the_client_prefix() {
    // Tests run from their package's directory, `tests/client`.
    let root = std::env::current_dir().expect("package directory").join("../..");
    let root = root.canonicalize().expect("temper's root");
    let breaches = jig_boundary::client_names::breaches(&root, "temper", &["temper", "temper-one-process-world"]);
    assert!(breaches.is_empty(), "client crates outside temper-client: {}", breaches.join(", "));
}
```
Its ceiling: under 20 ms (it reads 41 manifests). Its expected result,
measured by hand once on temper's manifests with jig's and temper's new
names: nothing, with `temper`, `temper-client-domain`,
`temper-client-protocol-wire`, `temper-client-shell-terminal`,
`temper-client-view`, `temper-client-world` and
`temper-one-process-world` client-side; with temper's old names it
lists `temper-protocol-client-wire` and `temper-shell-terminal`
(`logs/w1/client-names-by-hand.txt`, and jig 01.4's sketch run on both
trees, `logs/w1/temper-names-probe.txt`). No other test changes but by
the renamed paths.
Proof: the process tier, the real binary on jig 01's crates, once
through the build lane: `sh -c 'cargo build -p temper --bin
temper && NEXTEST_DOUBLE_SPAWN=0 cargo nextest run -p temper --profile
process -E "test(temper_open_signs_in_prints_the_inbox_and_exits_on_q)"'`:
`temper open` signs in by the printed link, prints `== Inbox == Live`
and exits 0 on `q`. Bound: 2 minutes (1.2 s for the test at the base).
`client_crates_carry_the_client_prefix` runs in the gate: from this
landing on, the gate refuses a client crate of temper's outside the
prefix.
Don't: copy jig's check into temper (decisions.md, "For the user", B);
give `breaches` an allowed name beyond `temper` and
`temper-one-process-world`; make any repair beyond the README's 200
lines without asking; change a test's expected value but by a renamed
path.
Size: at most 80 lines for jig 01's renames followed (54 at the
rehearsal, by code-organization.md, 7's temper command), and the
README's 200 for what else jig landed; the test file and manifests
under `tests/` are not counted.
Removes: temper's last names of jig's old client crates.

## Interface sketches

None of temper's own. The function called, jig 01.4's
(`jig/01-renames.md`, its sketch; checked at the base in jig's scratch
worktree with clippy `-D warnings`, `logs/w1/jig-boundary-clippy-2.txt`):

```rust
// jig_boundary::client_names (jig, tests/boundary/src/client_names.rs)
/// Each client-side crate under `root`, `allowed` aside, not named
/// `<project>-client…` or `<project>-<application>-client…`; sorted.
#[must_use]
pub fn breaches(root: &Path, project: &str, allowed: &[&str]) -> Vec<String>;
```

## Questions to raise when reached

- A crate the check flags in temper (none at the plan's base with the
  new names): ask; never add it to `allowed`.

## Done when

- 02.1 has landed with its gate.
- In a worktree of temper at `origin/main`, `cargo nextest run -p
  temper-client-world -E 'test(client_crates_carry_the_client_prefix)'`,
  once, passes one test.
- The Rules' `git grep` of jig's old names prints nothing, and `git -C
  <worktree> show origin/main:Cargo.lock | grep -o
  'git+https://github.com/forgetic/[a-z]*.git#[0-9a-f]*' | sort -u`
  prints `jig.git#<landing>` (or a later jig, pinned by a later temper
  session) and skein's and smith's lines of that jig's `Cargo.lock`.
