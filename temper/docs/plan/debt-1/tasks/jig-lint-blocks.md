+++
id = "jig-lint-blocks"
part = "jig-lints"
kind = "migration"
needs = []
files = [
  "jig/tests/*/Cargo.toml",
  "jig/testing/*/Cargo.toml",
  "jig/crates/jig-client-shell-native/Cargo.toml",
  "jig/crates/jig-client-shell-web/Cargo.toml",
  "jig/**/clippy.toml",
  "jig/docs/development/debt/D019-lint-config-copied.md",
  "jig/docs/development/debt/README.md",
]
effort = "high"
minutes = 150
+++

# jig D019, clauses 2 and 3: no world allows the replay bans, one lint block, every extra `clippy.toml` says why

## Goal

jig's 40 test and shell manifests that write their own lint block (the item's Measure) write
one block, the same in each; none allows a `disallowed_*` lint; and every `clippy.toml` beyond
`jig/clippy.toml` and `jig/tests/clippy.toml` states why it differs, or goes. Only manifests,
`clippy.toml` files and scoped `#[expect(..., reason = "...")]`s change; no code changes and
behaviour cannot.

**What stays open, and why** (the cut's survey; the plan's question `d019-shared-table`):
D019's first clause, a shared table, cannot hold within that limit. Cargo shares one table
among members, the root's `[workspace.lints]`, all or nothing (cargo 1.95 refuses a member's
overrides beside `workspace = true`, and has no named tables), and it is the step code's:
under it these crates would trip thousands of sites. So this task does not pay D019: it pays
clauses 2 and 3, writes the one form a later shared table or check would hold, and leaves the
item open, edited to say what is left. The answer to the question changes nothing here.

Acceptance:

- `rg -l 'disallowed_[a-z]+ = "allow"' -g Cargo.toml jig` finds nothing (8 at base).
- The 40 manifests' lint blocks are byte-identical, from `[lints.rust]` to the block's last
  line (below); a manifest kept apart (see Pre-approved) is named in the report with why.
- Every `clippy.toml` under `jig/` but `jig/clippy.toml` and `jig/tests/clippy.toml` opens
  with a comment that says, truly for its crate, why it differs from the file it would
  otherwise inherit; the files that would differ in nothing are gone.
- No world, kit or shell uses wall clock or threads under a lint switched off (none does at
  base: the one wall-clock use, `jig/tests/engine-clients/tests/suite/provider.rs`, is already
  a scoped expect, and stays).
- clippy is clean for every package below, natively and, for the two web shells, for
  wasm32; `tools/citations/citations.py --all` reads 0.

## Inputs

- `temper/docs/plan/debt-1/notes/common.md`.
- `jig/AGENTS.md` (every crate is step code but its test kits and the client's shells,
  which are ordinary Rust).
- `AGENTS.md`, 2 (a lint exception is a scoped `#[expect]`, never `#[allow]`; test crates are
  ordinary Rust under their tree's `tests/clippy.toml`: one thread, one loop, a seed replays).
- `jig/docs/development/debt/D019-lint-config-copied.md`, whole.
- `skein/docs/foundation/programming-model.md`, 10.2 and 10.3 (what ordinary Rust and step
  code may use); `skein/docs/foundation/testing-strategy.md`, 1.

## Owns and may change

**The 40 manifests** (`rg -l '^\[lints\.clippy\]' -g Cargo.toml jig`: 26 under `jig/tests/`,
12 under `jig/testing/`, `jig/crates/jig-client-shell-native` and
`jig/crates/jig-client-shell-web`): each one's `[lints.rust]` and `[lints.clippy]` tables
become exactly this, where the block is today, every other table untouched:

```toml
[lints.rust]
unsafe_code = "forbid"
elided_lifetimes_in_paths = "deny"
unused_must_use = "deny"

[lints.clippy]
pedantic = { level = "warn", priority = -1 }
missing_errors_doc = "allow"
missing_panics_doc = "allow"
wildcard_enum_match_arm = "deny"
match_wildcard_for_single_variants = "deny"
unwrap_used = "deny"
allow_attributes = "deny"
allow_attributes_without_reason = "deny"
disallowed_types = "deny"
disallowed_methods = "deny"
disallowed_macros = "deny"
```

The survey's variants, for orientation: 27 are this block without `missing_errors_doc` and
`disallowed_macros`; 6 (`tests/core`, `tests/conformance`, `testing/jig-conformance`,
`testing/jig-fake-parties`, `testing/jig-fake-store`, `testing/jig-fake-workers`) and 2
(`testing/jig-client-scripted-engine`, `testing/jig-client-test-kit`) allow the three
`disallowed_*`; `testing/jig-test` adds `missing_errors_doc`; 4
(`tests/client-browser`, `testing/jig-client-test-shell-web`,
`testing/jig-client-fake-person-browser`, `tests/boundary`) keep pedantic and the doc allows
alone. Their per-line comments go: the block reads the same everywhere.

**Scoped expects** in the crates of those 40: where the form denies what a crate did not, each
site gets `#[expect(clippy::<lint>, reason = "...")]` (or `rust::` for a rustc lint) on the
narrowest item, its reason the site's own; an `#[allow]` the form now denies becomes such an
`#[expect]`. The survey expects about 7 (`wildcard_enum_match_arm` in `tests/client-browser`,
`testing/jig-client-fake-person-browser` and `tests/boundary`); an expect that the form's
`missing_errors_doc = "allow"` leaves unfulfilled goes.

**`clippy.toml` files** (23 at base; the bases are `jig/clippy.toml`, the step-code subset,
and `jig/tests/clippy.toml`, the replay bans):

- Delete those that say nothing their inherited file does not (the survey found
  `jig/tests/{channel,client-wire,engine-clients,one-process,store}/clippy.toml` identical to
  `tests/`'s, and `jig/tests/{ops,client-browser}/clippy.toml` the same entries reworded):
  delete each whose paths equal `tests/clippy.toml`'s, and head any that does not.
- Add one to each of the six `jig/testing/` kits that allowed the bans because they inherit
  the step-code subset (`jig-conformance`, `jig-client-scripted-engine`,
  `jig-client-test-kit`, `jig-fake-parties`, `jig-fake-store`, `jig-fake-workers`): the
  entries of `jig/tests/clippy.toml`, under a header such as "This kit is ordinary Rust, a
  world's (jig/AGENTS.md): it takes the replay bans of tests/clippy.toml, not the step-code
  subset of jig/clippy.toml that it would otherwise inherit."
- Head every other one truly for its crate: `jig/tests/client-protocol` (why it adds
  `core::cell::Cell`, from its code); `testing/jig-client-test-records` and
  `testing/jig-test-profile` (their header says "under tests/", which they are not);
  `testing/jig-test`, `jig-test-store-files`, `jig-test-system`, `jig-ops-fake-production`;
  `crates/jig-client-shell-native` (its header calls it a fake; it is the native client's
  shell, `src/lib.rs`); `crates/jig-client-shell-web` (true; keep);
  `testing/jig-client-test-shell-web`.
- `testing/jig-client-fake-person-browser/clippy.toml` drops `Instant` and `SystemTime`, which
  its code does not use: restore `tests/`'s full list, and head it.
- `testing/jig-test-patches{,-jig,-smith}/clippy.toml` empty the lists, so their crates may
  use `HashMap`, `Instant` and `thread::spawn`, under a header citing programming-model.md
  10.2, which does not say that. They are ordinary Rust: give them `tests/`'s entries and a
  true header.

**The item** (it stays open; keep its file name, which skein's D036 and temper's D011 cite):

- Title: "D019: jig's test crates and shells copy one lint block, which no shared table
  holds", in the file and in its row of `jig/docs/development/debt/README.md`.
- **What** becomes, in the item's words: the 40 manifests write one `[lints]` block, the same
  in each, by hand; Cargo shares among members only the root's `[workspace.lints]`, all or
  nothing, and that table is the step code's; nothing holds the 40 copies to one form.
  **Why it matters** keeps its first clause (a copied block drifts) and loses the second (no
  world allows the bans now). **What pays it back** adds that the way is the user's, asked as
  plan debt-1's question `d019-shared-table`. **Checked** is today at your branch's base.
  **Paid when**, **Measure** (still 40), **Value**, **Size** and **Status** (open) stay.

Pre-approved:

- **Wall clock or threads:** if a crate, once its lint is on, uses `Instant`, `SystemTime`,
  `thread::spawn` or another real-time or threading item outside an expect it already has,
  stop and ask: never switch the lint off for it.
- **`HashMap`/`HashSet`:** a use whose iteration order nothing observes may take a scoped
  expect saying so; one whose order reaches an output or a trace is a question.
- **A manifest kept apart:** if bringing one manifest to the form needs more than 10 new
  expects, give it the form's `disallowed_*` lines and keep the rest of its block as it is
  (no `disallowed_*` allowed), and report it with the count; the item's **What** says so.
- **A lint the form lacks:** if a manifest sets one its crate needs (a `check-cfg`, say), keep
  that one line after the form's and list it.

## Steps

1. Manifests; then the `clippy.toml` files; then the expects that clippy asks for.
2. The package names: `for m in $(rg -l '^\[lints\.clippy\]' -g Cargo.toml jig); do grep -m1
   '^name' "$m"; done`, plus `jig-test-patches`, `jig-test-patches-jig` and
   `jig-test-patches-smith`. One run over all of them:
   `heavy cargo clippy -p <each> ... --all-targets -- -D warnings`; then, as fab's
   clippy-wasm32 step does for the packages of `.config/gate/wasm32.txt`,
   `heavy cargo clippy --target wasm32-unknown-unknown -p jig-client-shell-web -p jig-client-test-shell-web -- -D warnings`.
3. This departs from the implementer's text: no test suite runs. The change is lint levels,
   clippy's lists and `#[expect]` attributes, which change no code; clippy over
   `--all-targets` compiles every target, and is the check.
4. `cargo fmt` (expects only); `python3 -I -B scripts/old_forms.py` from `jig/`; the
   acceptance's `rg`; `python3 -I -B tools/citations/citations.py --base origin/main` and
   `--all` from the repository root.
5. Commits: the manifests and their expects; the `clippy.toml` files; the item's text.
   Pushed.

## Report

As the implementer's text says, and also: the manifests by what moved in each (the
disallowed allows, the doc allows, the four brought up), every expect added with its lint
and reason, every `clippy.toml` deleted, added or headed, and any manifest kept apart.
