# Session 19: the browser parts, the test shell, the bundle and the browser tier

The client's crates build for the browser's target under the gate, the
reference UI gets its codec of places to paths, and
`jig-client-shell-web` gives an application jig's browser parts: the
client's adapter over skein's fetch, small store and transport, the
printed link's sign-in, and jig's look. jig's test application gets its
browser shell, `jig-client-test-shell-web`, written to be copied; the
gate builds its module, bundles it with skein's bundler, builds
`jig-test`, and runs the browser tier, a suite of its own, whose first
story is a person who opens the printed link and sees their inbox
(`protocol/browser.md`, 1 to 5, 9 to 11, 14, 15; `client/domain/testing.md`,
2 and 9).

## Read first

- jig's `docs/design/protocol/browser.md`: 1, 2, 3 (3.1 to 3.5), 4 (4.1
  to 4.3), 5 (5.1 to 5.3), 9, 10, 11, 13, 14, 15, 19.
- jig's `docs/design/protocol/client.md`: 12 (the bullet "The reference
  UI's browser parts"), 17.1 to 17.4.
- jig's `docs/design/client/domain/testing.md`: 2 (the browser tier's
  row), 9.
- skein's `docs/design/ui-web.md`: 3.1, 4.1 to 4.3, 5.1 to 5.3, 10, 14.2;
  skein's `examples/ui/shell-web/src/lib.rs` (the shell to copy, 275
  lines with `grow.rs`) and skein's `scripts/check.sh` (its wasm32 and
  browser lines).
- jig 17's and jig 18's session files (the mount's routes and cookie;
  the server's bundle and page); skein 01's (the harness's `Printed`,
  `Setup.restart`).
- `findings/server-and-browser.md`: 3, 5, 6.
- skein's `programming-model.md`: 2.1, 10 (by number only); skein's
  `testing-strategy.md`: 2.9, 7, 8 (by number only).

## What the design asks

- **The wasm32 builds** (`browser.md`, 15): "`jig-client-shell-web`,
  `jig-client-test-shell-web` and every client crate, built for
  `wasm32-unknown-unknown` with clippy's lints as errors". **The `web`
  profile** (4.1): "it inherits `release`, with `opt-level = "s"`, `lto =
  "fat"`, `codegen-units = 1`, `panic = "abort"`, `overflow-checks =
  true` and `strip = true`". **One lock** (4.1): "jig's workspace builds
  skein's bundler beside its test application's module, so the glue's
  generator is locked with the macros' version in jig's `Cargo.lock`".
- **The reference UI's codec** (9): `jig-client-view`'s `address` module,
  the table of places and paths, "numbers are decimal, with no leading
  zero, within their type; one trailing slash and a query ... are
  ignored; any other path is a place with no page, shown as missing";
  under a prefix of the application's (`/work/inbox`), with its own
  segment names.
- **jig's two parts** (5.1): the client's adapter ("each exchange for
  fetch, same-origin, its answer read up to the client's largest record;
  the stream's request, each piece cut to the client's stream piece; the
  reports as the host's events"), and sign-in ("the printed link's
  secret, read from the address, the address rewritten first; a
  credential shown once in a dialog of its own, never kept"); **jig's
  look** (3.3): "the prefix `jig` and the names of jig's style tokens and
  region names ..., from `jig-client-view`"; jig's pictures; `jig.css`
  (13).
- **The test shell** (3.1): "written after skein's reference shell ...,
  and adds `jig-client` through the client's adapter, the printed link's
  sign-in, and the reference UI with jig's look. ... about 200 lines
  beside its client host (a guide)."
- **The bundle** (4.2): "under its prefix, `/bundle/`, ...: `index.html`,
  and under `/bundle/<digest>/` `boot.js`, `<name>.js`, `<name>_bg.wasm`,
  `snippets/`, `skein-ui.css`, `jig.css`, `<name>.css`, `pictures/jig/`,
  a `pictures/<prefix>/` for its test family's owner, and `files/`"; its
  size a guide, "at most 4 MiB before compression, measured when first
  built and kept with jig's budgets".
- **The browser tier** (15): "a suite of its own, `--profile browser`,
  run by jig's gate after it builds jig's test application's bundle with
  skein's bundler, within a minute on the slowest machine (a rule: its
  profile's global timeout ...). ... A missing Chromium or bundle fails
  the suite, naming the command that provides it; it is never skipped.
  The bundle's directory reaches the tests in `JIG_TEST_BUNDLE`." Its
  story: "a person opens the printed link and sees their inbox, under
  the reference policy, with no trouble". "jig's test application's
  shell takes jig's steps of 3.5 once each: its own page or skein's
  bundler's (the plan's choice); a root file; a prefix on the reference
  codec in one story."

## Choices this plan makes

- **The bundler's page,** not one of jig's own (`browser.md`, 15: "the
  plan's choice"): no `--page`.
- **The bundler is built, then run:** `cargo run -p skein-ui-bundle`
  fails in jig's workspace ("package(s) `skein-ui-bundle` not found in
  workspace", run at the base, `logs/w8/bundler-run.txt`), and `cargo
  build -p skein-ui-bundle` works once a member depends on it normally
  (`logs/w8/bundler-build.txt`). The browser world, `jig-client-browser-world`,
  depends on `skein-ui-bundle` (its fixture bundles use the library), and
  the gate runs `cargo build -p skein-ui-bundle` then
  `target/debug/skein-ui-bundle`. jig's `[profile.dev.package.skein-ui-bundle]`
  sets `opt-level = 3`, as skein's does.
- **Adding a crate of skein's re-resolves skein's git source** to its
  newest commit (seen at the base: the lock moved from `f12f29d4` to
  another commit when `skein-ui-bundle` was added). An increment that
  adds one (19.4's `skein-fake-person-browser`, `skein-ui-bundle` and
  `skein-scratch` among them) holds the pin in the same commit: `cargo update -p skein-lib --precise <jig's skein pin at the session's
  start>`, after which `grep -o 'skein.git#[0-9a-f]*' Cargo.lock | sort
  -u` prints the one commit it printed before; a failure in a smith
  crate is a question.
- **No repin:** jig 12.2's first commit moved jig's skein to skein-01.s3's
  landing, which holds skein-01.s2's, skein-01.s1's, skein 01.3's and
  forgejo-17's skein step's, with the one repair, `lines: 0, line_bytes:
  0` in jig's `skein_ui_text::Settings` literal (README, "Pins"; the
  design answer to jig 12.2's question of 2026-10-16T19:46:42Z). jig 19 moves skein
  in no commit. At its start, with `<skein-01.s2>` the commit `git -C
  ~/src/rust/skein fetch origin && git -C ~/src/rust/skein log
  origin/main --all-match --grep='^Plan: client-api$' --grep='^Step:
  skein-01.s2$' --format=%H -1` prints, `git -C ~/src/rust/skein
  merge-base --is-ancestor <skein-01.s2> <jig's pin of skein>`
  succeeds; if it fails, that is a question.
- **The test shell's stylesheet** is `testing/jig-client-test-shell-web/jig_client_test_shell_web.css`,
  so that the bundle holds `<name>.css` as `browser.md`, 4.2 lists it;
  its root file is `files/robots.txt` (jig 18's root file).
- **A prefix on the reference codec in one story:** the test shell's dev
  build reads a page opened under `/test/prefixed` as the codec
  `Codec { prefix: b"/test/prefixed", ..REFERENCE }`, as skein's example's
  dev build reads `/test/small-store`; a release (`web`) build has no
  such path.
- **`JIG_TEST_BUNDLE`** is read as an absolute path, or as a path under
  the workspace's root when relative, so that the README's command and
  the gate's `"$(pwd)/target/jig-test-bundle"` both reach it.

## Rules that bind it

- **The browser suite is a suite of its own** (README, "Rules"): `--profile
  browser`, `not binary(/^browser_/)` in the default and fuzzy profiles,
  its `global-timeout` 60 s; nothing raises another profile's budget,
  which README's "Done when", 12 checks.
- **A browser story's Proof is the gate:** the author runs no
  gate of its own for it; a failing story is run alone as the README's
  "One test of the browser suite" says, under that section's rule.
- **jig never names the repository downstream of it:** the check of
  code-organization.md, 6, quoted: "`git -C
  <the session's jig worktree> grep -n -i -w temper -- crates testing tests
  examples scripts` prints nothing at each jig increment's end".

## Corrections after the ready check (ruling 329)

The ready check (skein 0be1255e7, jig 7d3becac7, temper 9ea74ad40)
found these, against the increments below; they replace what differs.
- **19.1, a lint unfulfilled on wasm32:**
  `crates/jig-client-domain/src/contract/documents.rs:786-789`'s
  `#[expect(clippy::large_enum_variant, ..)]` on `HistoryChange` fails
  the new wasm32 clippy ("this lint expectation is unfulfilled"). It
  becomes `#[cfg_attr(target_pointer_width = "64",
  expect(clippy::large_enum_variant, reason = ".."))]`, its reason
  kept, and the file joins 19.1's Files. With it, the wasm32 clippy and
  jig's whole gate passed. Any further expectation unfulfilled on
  wasm32 takes the same form; the commit body lists it.
- **19.1 and 19.4, the gate's documents:** `docs/development/workflow.md`
  section 1 (`:12-17`, the gate's commands) takes 19.1's wasm32 clippy
  line, and joins 19.1's Files; 19.4 renames section 2 ("The two test
  suites", `:30`) for three, and the header comments of
  `scripts/check.sh` and `.config/nextest.toml`, which name two suites,
  name three.
- **19.2:** `crates/jig-client-view/src/tests.rs` registers the test
  modules (`mod ..;`), and joins the Files for `mod address;`. The
  crate is `no_std`: the sketch's `Box<[u8]>` needs `use
  alloc::boxed::Box;`, and
  `random_bytes_parse_to_a_place_without_a_panic` draws each path into
  a `[u8; 64]` and parses a slice of it, with no `Vec` and no
  `map`/`collect`, which the crate's `clippy.toml` disallows.
- **19.3 and 19.4, the boundary test:** `tests/boundary/src/tests/graph.rs`
  is in their Files, not only "if a role list names client crates":
  `client_engine_breaches` (`:459`) names `jig-client-shell-native`
  (`:469`), and names the new shell crates beside it;
  `smith_breaches` (`:412`) holds them already by its prefix
  (`:419`, `jig-client-shell-`); `application_breaches` (`:390`) is
  checked for them.
- **19.3, the browser header is the adapter's:** the mapping's client
  end, `jig-client-protocol-http`, sets no `Jig-Client` header; the
  mount reads it (`jig-protocol-engine-clients-http/src/lib.rs:59`,
  `jig-client: browser`). The adapter's pure half,
  `adapter::requests`, adds it to each exchange and stream, as 19.3's
  test asserts, and 19.4's test shell takes it from the adapter.
- **19.4:** the harness's `Setup` also has `server`, `chromium`,
  `expected` and `origins`, as the `Tier::setup` sketch has them;
  `restart` is `args` again, `["serve", <config>]`, with no `{address}`
  (ruling 338): 19.4's story stops no server, `jig-test serve` takes no
  listener argument, and the stories that stop it get a stable port
  from jig 20's tier ("The restarted stories' address").
- **Held:** `cargo update -p skein-lib --precise 0f093bb7..` leaves one
  skein in `Cargo.lock`, as "Choices" says; temper's `address.rs` wraps
  jig's `Address`, so 19.2 only adds to it, and temper uses no `web`
  profile, wasm32 or `browser_`.

## Increments

### 19.1 The client crates on the browser's target, and the `web` profile

Scope: `[profile.web]` in jig's workspace (`browser.md`, 4.1); the gate's
`cargo clippy --target wasm32-unknown-unknown` of the client crates with
lints as errors (`browser.md`, 15), and what that build needs of them.
Needs: nothing beyond jig 18 on main.
Files: `Cargo.toml` (workspace: `[profile.web]`), `scripts/check.sh`;
whatever the client crates need to build for wasm32 (none uses std time,
fs or env at the base, `findings/server-and-browser.md`, 5; a smith
crate that does not build is a question). Shared: workspace
`Cargo.toml`.
Shape: in `scripts/check.sh`, after the host clippy:
`cargo clippy --target wasm32-unknown-unknown -p jig-client-protocol-wire
-p jig-client-protocol-link -p jig-client-protocol-http -p
jig-client-domain -p jig-client -p jig-client-view -p
jig-client-test-protocol-wire -p jig-client-test-domain -p
jig-client-test-view -p jig-ops-client-protocol-wire -p
jig-ops-client-domain -p jig-ops-client-view -- -D warnings`, the client
crates of README 15's rule that are step code (19.3 adds
`jig-client-shell-web` to this line, 19.4 `jig-client-test-shell-web`).
Left out, each because it does not run in a browser: the native shell,
`jig-client-shell-native` (Linux io through skein's kernel ring,
`client.md`, 12); the test harnesses `jig-client-scripted-engine`,
`jig-client-test-kit` and `jig-client-test-records`, which the worlds run
on the host; and the worlds `jig-client-world`,
`jig-client-protocol-world`, `jig-client-wire-world` and
`jig-client-browser-world`. A crate of the line that does not build for
wasm32 is a question, never dropped from the line. `[profile.web]` with
`inherits = "release"`, `opt-level = "s"`, `lto = "fat"`,
`codegen-units = 1`, `panic = "abort"`, `overflow-checks = true`,
`strip = true`.
Test shape: no test; the gate's wasm32 clippy is the check.
Proof: the gate runs the wasm32 clippy. The run's machine has the
`wasm32-unknown-unknown` target (README, "Prerequisites"); the commands
the planner could not run at the base are listed in the README for the
supervisor's ready check. Nothing above it exists until 19.4 builds the
module.
Don't: give a client crate `std`; build the native shell for wasm32;
change a lint.
Size: at most 40 lines.
Removes: nothing.

### 19.2 The reference UI's codec: places to paths and back

Scope: `jig_client_view::address`'s codec (`browser.md`, 9): each of
jig's places printed as its path and parsed back, at the root or under a
prefix, with the reference's segment names or the application's; a
query and one trailing slash ignored; numbers decimal, no leading zero,
within their type; any other path under the prefix missing, a path
outside it the application's.
Needs: nothing beyond the session's start (jig 13's places, `Address`
and `Place`, are in `jig-client-view` by then).
Files: `crates/jig-client-view/src/address.rs` (beside the places jig 13
moved there), `crates/jig-client-view/src/tests/address.rs` (new).
Shape (sketch below): `Codec<'a> { prefix, names }`, `Names<'a>`,
`REFERENCE`, `print(codec, address) -> Box<[u8]>`, `parse(codec, path)
-> Place`. `/` and the prefix alone parse to the home, the inbox
(`client/domain/pages.md`, 2: "the inbox by default").
Test shape: unit tests in `crates/jig-client-view/src/tests/address.rs`,
each under 20 ms, its addresses in a `const` table of `(Address, &[u8])`
rows, one per row of `browser.md`, 9's table and each section. The
reference test:
```rust
const PRINTED: &[(Address, &[u8])] = &[
    (Address::Inbox { project: None }, b"/inbox"),
    (Address::Inbox { project: Some(3) }, b"/inbox/3"),
    (Address::Conversations { project: None }, b"/conversations"),
    (Address::Task { task: 18_446_744_073_709_551_615, section: TaskSection::Overview }, b"/tasks/18446744073709551615"),
    (Address::Task { task: 7, section: TaskSection::Budget }, b"/tasks/7/budget"),
    (Address::Project { project: 4, section: ProjectSection::Recurring }, b"/projects/4/recurring"),
    (Address::Deployment, b"/deployment"),
];
#[test]
fn each_place_prints_its_path_and_parses_back_at_the_root() {
    for (address, path) in PRINTED {
        assert_eq!(&*address::print(&address::REFERENCE, *address), *path, "{address:?}");
        assert_eq!(address::parse(&address::REFERENCE, path), Place::Jig(*address), "{path:?}");
    }
}
```
Tests: `each_place_prints_its_path_and_parses_back_at_the_root`,
`each_place_under_a_prefix_prints_and_parses_back`,
`other_segment_names_print_and_parse_their_own_paths`,
`a_query_and_one_trailing_slash_are_ignored`,
`a_leading_zero_or_a_number_past_its_type_is_missing`,
`random_bytes_parse_to_a_place_without_a_panic` (2,000 paths of up to 64
bytes drawn by `skein_lib::Rng` from seed 7, under 20 ms).
Proof: unit layer; nothing calls the codec until the test shell's
history and links in 19.4, which connects it.
Don't: print a place that `parse` reads otherwise; read the fragment;
change a path of `browser.md`, 9's table.
Size: at most 150 lines.
Removes: nothing.

### 19.3 `jig-client-shell-web`: the client's adapter, sign-in and jig's look

Scope: the crate of jig's browser parts (`browser.md`, 5.1, 3.3, 10, 11,
13): the client's adapter, its pure half (a step's transport requests
as the mapping's client end gives them into skein's transport's plain
data, writes first; fetch's and the transport's reports into the host's
events) and its platform half (`#[cfg(target_arch = "wasm32")]`, the
browser header and same-origin credentials on each exchange); sign-in,
its pure half (the secret read from the page's address at the link's
route, the address rewritten to the home first) and its platform half
(`history.replaceState` before the host starts; a service's credential
shown once in a dialog of its own, as selected text, closed by a
button, never kept); jig's look (jig's names from `jig-client-view`,
jig's pictures, the parts' phrases) and `jig.css`.
Needs: nothing beyond the session's start (jig 10's
`jig-client-protocol-http`, the mapping's client end, and jig 13's
`jig-client-view` names are on main by then).
Files: new `crates/jig-client-shell-web/{Cargo.toml, clippy.toml,
jig.css, pictures/}`, `src/{lib.rs, adapter.rs, sign_in.rs, look.rs,
web.rs}`, `src/tests.rs`, `src/tests/fixtures.rs`; workspace `Cargo.toml`
(the member; `skein-ui-web`, `wasm-bindgen = "=0.2.126"`, `js-sys =
"=0.3.103"`, `web-sys = "=0.3.103"`, the pins skein's workspace gives,
`Cargo.toml:66-68` of skein) and `Cargo.lock` (the skein pin held, as
"Choices" says); `scripts/check.sh` (the crate joins 19.1's wasm32
clippy); `tests/boundary/src/tests/graph.rs` (its roles, if a role list
names client crates). Shared hotspots: workspace `Cargo.toml`,
`Cargo.lock`, `tests/boundary/src/tests/graph.rs`.
Shape (sketch below): `adapter::{Asked, Heard, Header, requests,
fetched, carried}` over jig 10's types for the mapping's requests (the
sketch's `Asked` stands in for them); `sign_in::{Route, Taken, take}`
and, on wasm32, `sign_in::web::{take_from_page(route) -> Option<Box<[u8]>>,
show_once(words, credential)}`; `look::{NAMES, PICTURES, phrases}`, jig's
part of skein's `Look`. `jig.css` places jig's regions by `data-region`
and styles jig's tokens with jig's palette (`browser.md`, 13), at most
120 lines. The crate is ordinary Rust, forbids `unsafe`, and carries its
own `clippy.toml` after skein's `examples/ui/shell-web/clippy.toml`
(jig's root `clippy.toml` is step code's, which an ordinary crate does
not take).
Test shape: unit tests in `crates/jig-client-shell-web/src/tests.rs` on
`src/tests/fixtures.rs` (`exchange(id, body)`, `save(key, value)`,
`route()` with path `/jig/v1/sign-in/link` and home `/`), each under
20 ms. The reference test:
```rust
#[test]
fn a_step_s_writes_leave_before_its_exchanges() {
    let requests = adapter::requests(vec![fixtures::exchange(1, b"\x01"), fixtures::save(b"k", b"v")]);
    assert_eq!(requests.writes, [store::Record { key: Box::from(b"k".as_slice()), value: Box::from(b"v".as_slice()) }]);
    assert_eq!(requests.exchanges.len(), 1);
    assert_eq!(requests.exchanges[0].headers, [("Jig-Client".to_owned(), "browser".to_owned()), ("Content-Type".to_owned(), "application/octet-stream".to_owned())]);
}
```
Tests: `a_step_s_writes_leave_before_its_exchanges`,
`an_answer_past_the_bound_is_an_exchange_failed`,
`a_refused_write_is_heard_as_not_stored`,
`the_link_route_s_secret_is_taken_and_the_address_is_the_home`,
`another_path_takes_no_secret`, `the_credential_dialog_s_words_are_jig_s`,
`the_stream_s_pieces_are_cut_to_the_client_s_piece` (`browser.md`, 5.1:
"each piece cut to the client's stream piece": `requests` of an
`OpenStream { piece: 16 }` gives `StreamDuty::Open { bound: 16 }`, and
skein's `fetch::Fetch` opened with that bound and given a 40-byte read
reports pieces of 16, 16 and 8 bytes).
Proof: unit layer here, as `browser.md`, 15 places the parts' pure
halves; the platform halves are compiled only for wasm32, by the gate's
wasm32 clippy, which this increment's crate joins with `-D warnings`.
The run's machine has the `wasm32-unknown-unknown` target (README,
"Prerequisites"); the commands the planner could not run at the base are
listed in the README for the supervisor's ready check. 19.4's
first story connects both halves in Chromium against `jig-test`.
Don't: keep a credential the dialog shows; give the parts a loop; name
`jig-client-test-domain` or the test view; inline a script or a style.
Size: at most 300 lines, `jig.css` among them.
Removes: nothing.

### 19.4 The test shell, its bundle in the gate, and the browser tier's first story

Scope: `jig-client-test-shell-web` (`browser.md`, 3.1: start, the turn of 5.2,
the client host's functions of 3.2, the sign-in part at start, history
through the codec); its bundle built in the gate; `[profile.browser]`
and the default and fuzzy profiles' filters; the browser tier,
`tests/client-browser` (`jig-client-browser-world`), and its first story;
the third suite in jig's `docs/development/workflow.md`, 2, with the
module's size (`browser.md`, 4.2).
Needs: skein 01's harness (`Printed` in the URL form on stdout,
`Setup.restart`; its last landing, `Step: skein-01.3`), and
skein-01.s2's landing, both held by jig's pin of skein since jig 12.2's
move ("No repin").
Files: `Cargo.lock`; new `testing/jig-client-test-shell-web/{Cargo.toml,
clippy.toml, jig_client_test_shell_web.css, files/robots.txt, pictures/}`,
`src/lib.rs`; new `tests/client-browser/{Cargo.toml, clippy.toml}`,
`src/lib.rs`, `tests/browser_tier.rs`; `.config/nextest.toml`;
`scripts/check.sh`; `docs/development/workflow.md` (section 2); workspace
`Cargo.toml` (the two members, `skein-fake-person-browser`,
`skein-ui-bundle`, `skein-scratch`, `[profile.dev.package.skein-ui-bundle]`);
`tests/boundary/src/tests/graph.rs` (roles, if a list names client
crates); `crates/jig-client/src/client.rs` and the exhaustive matches on
`jig_client::Request` (ruling 337); `testing/jig-test/src/bundle.rs` and
its tests (ruling 344: the module's file cap);
`testing/jig-test-service/src/{clients.rs, startup.rs, lib.rs}`, where the
clients child is built and clocked (ruling 348). Shared hotspots: workspace
`Cargo.toml`, `Cargo.lock`, `tests/boundary/src/tests/graph.rs`.
Shape:
- `jig-client-test-shell-web`: `crate-type = ["cdylib"]`, its
  dependencies under `[target.'cfg(target_arch = "wasm32")'.dependencies]`,
  `#![cfg(target_arch = "wasm32")]` at its root, after skein's
  `examples/ui/shell-web/src/lib.rs`: `#[wasm_bindgen(start)] fn
  start()`, `Report` (skein's parts' reports and the host's events),
  `Shell::turn` in 5.2's order; restart under the key `jig-restarts`
  outside the small store's prefix `jig.`; title `heading · jig test`.
  About 200 lines beside its host (a guide).
- `scripts/check.sh`: `jig-client-test-shell-web` joins the wasm32
  clippy line of 19.1 (`-D warnings`; the host clippy sees nothing of a
  crate whose root is `#![cfg(target_arch = "wasm32")]`); and after the
  fuzzy suite, the lines below, whose bundler arguments are the README's
  "One test of the browser suite" ones, `target/` the worktree's own:

  ```sh
  # The browser tier (protocol/browser.md, 15): the test application's module, bundled, served by jig-test.
  cargo build --target wasm32-unknown-unknown -p jig-client-test-shell-web
  cargo build -p skein-ui-bundle
  target/debug/skein-ui-bundle \
      --module target/wasm32-unknown-unknown/debug/jig_client_test_shell_web.wasm \
      --name jig_client_test_shell_web --prefix /bundle/ \
      --stylesheet crates/jig-client-shell-web/jig.css \
      --stylesheet testing/jig-client-test-shell-web/jig_client_test_shell_web.css \
      --pictures jig=crates/jig-client-shell-web/pictures \
      --pictures jig-test=testing/jig-client-test-shell-web/pictures \
      --files testing/jig-client-test-shell-web/files \
      --noscript 'jig needs WebAssembly and scripts.' \
      --out target/jig-test-bundle
  cargo build -p jig-test
  JIG_TEST_BUNDLE="$(pwd)/target/jig-test-bundle"
  export JIG_TEST_BUNDLE
  cargo nextest run --workspace --profile browser
  ```

  (`jig-test` is the test family's owner prefix: the test shell gives
  it in its look's `Names` for `jig-client-test-view`'s owner, and the
  bundle holds that owner's pictures under `pictures/jig-test/`; the
  directory holds at least one picture of the closed list's types, which
  the bundler requires of a directory it is given.)
- `.config/nextest.toml`: the default and fuzzy profiles' filters gain
  `& not binary(/^browser_/)`; a new profile:

  ```toml
  # The browser suite (docs/development/workflow.md, 2): Chromium against jig-test, within 1 minute.
  [profile.browser]
  default-filter = "package(jig-client-browser-world) & binary(/^browser_/)"
  global-timeout = "60s"
  slow-timeout = { period = "20s", terminate-after = 3 }
  test-threads = 1
  ```
- `tests/client-browser/src/lib.rs`: `jig_test()` (the binary beside the
  test's own directory, `target/debug/jig-test`, failing with the command
  that builds it), `chromium()` (after skein's `tests/ui-web/src/lib.rs:28-43`),
  `bundle()` (`JIG_TEST_BUNDLE`, failing with the gate's commands that
  make it), `Tier::new(name)` writing a configuration in a scratch
  directory (after `testing/jig-test/tests/support/mod.rs:43-63`, with
  `service.components` `["clients"]`, no workers, and `application.server
  { listener: {"socket": "127.0.0.1:0"}, bundle: <bundle()> }`), and
  `Tier::setup(&self) -> Setup` with `printed: Printed { prefix:
  b"http://127.0.0.1:", on: Output::Stdout, names: Names::Url }`, `args:
  ["serve", <config>]`, `restart` the same.
- `docs/development/workflow.md`, section 2: a third bullet, "The browser
  suite", its profile, its 60-second budget, its commands (the block
  above), and the module's size, the guide of `browser.md`, 4.2 ("jig's
  test application's module, built with `web`, at most 4 MiB before
  compression"): `sh -c 'cargo build --target wasm32-unknown-unknown
  --profile web -p jig-client-test-shell-web && stat -c %s
  target/wasm32-unknown-unknown/web/jig_client_test_shell_web.wasm'`,
  once; the number printed is written there and in the landing's report.
  Past 4 MiB is a guide missed: recorded in `decisions.md`'s rulings, the
  landing standing; nothing else is measured.
Test shape: `tests/client-browser/tests/browser_tier.rs` on
`tests/client-browser/src/lib.rs`'s `Tier`; limits are the harness's and
`jig-test`'s small profile (the configuration's `limits.profile`
`"small"`, `testing/jig-test/tests/support/mod.rs` at the base). The
reference test:
```rust
#[test]
fn a_person_opens_the_printed_link_and_sees_their_inbox() {
    let tier = Tier::new("printed-link");
    let mut person = Script::new(Box::new([harness::Step::Person(Step::See {
        find: Find::exactly_named(Role::Heading { level: 1 }, tier.words(b"inbox")),
        within: Duration::from_secs(5),
    })]));
    assert_eq!(Harness::new(tier.setup()).run(&mut [&mut person]), Outcome::Done);
}
```
(`tier.words(b"inbox")` is the inbox page's heading in jig's English
phrasebook, read from `jig-client-view`'s phrases, not from the page;
the harness fails the story on any trouble, a policy violation among
them, so `Done` shows "with no trouble").
Tests: `a_person_opens_the_printed_link_and_sees_their_inbox`, one
browser story (about 3 s); and a unit test of `Tier`'s configuration in
`tests/client-browser/src/lib.rs`'s own `#[cfg(test)]` module (its lib's
test binary, not a `browser_*` one, so the default suite runs it),
`the_tier_s_configuration_names_the_bundle_and_a_loopback_listener`,
under 20 ms.
Proof: the gate, which builds the bundle and runs the browser suite
with this story: the person's page opens the URL `jig-test` printed,
the sign-in part takes the secret and posts it with the browser header,
the engine end answers with the cookie, and the inbox shows. A failing
story is run alone as the README's "One test of the browser suite"
says.
Don't: write a page of jig's own; carry a JavaScript snippet (`browser.md`,
17: none by default); give the tier a world's seed or a fixed sleep;
raise a profile's budget; move skein in any commit.
Size: at most 250 lines (the shell's; `tests/` is not counted). Ruling
342: 581 with the host, its limits and records, the manifests and ruling
337's forwarding, which the Scope holds; stop at 650, ruling 344's
module cap among them.
Removes: nothing.

### 19.5 The test shell's steps of 3.5: a root file, and the codec under a prefix

Scope: the tier's checks of what jig's test application takes of
`browser.md`, 3.5 (`browser.md`, 15): its root file served, and a
prefix on the reference codec in one story.
Needs: skein 01's `Step::Request` (its last landing, as 19.4).
Files: `testing/jig-client-test-shell-web/src/lib.rs` (the dev build's
`/test/prefixed` codec), `tests/client-browser/tests/browser_tier.rs`;
`testing/jig-test-service/src/server.rs` (`Server::done`, ruling 355); under ruling 358, the one file of
`testing/jig-test-service/` or `tests/client-browser/` its focused log names;
`testing/jig-test-service/src/limits.rs` (`Profile::Small`'s server connections, 8,
ruling 362) and each value or test derived from that count.
Shape: in the shell, `#[cfg(debug_assertions)]`, a page opened at a path
under `/test/prefixed` takes `Codec { prefix: b"/test/prefixed",
..address::REFERENCE }` for the session.
Test shape: two stories in `tests/client-browser/tests/browser_tier.rs`
on `Tier`, each about 3 s:
`the_root_file_is_served_as_the_bundle_s_robots_txt` (a `Step::Request`
`GET /robots.txt`, status `200`) and
`under_a_prefix_the_inbox_s_links_carry_the_prefix` (the person goes to
`/test/prefixed/inbox`, sees the inbox's heading, and a link of a task
in it leads to `/test/prefixed/tasks/<n>`, which `Step::Person(Press)`
then `See` the task's page shows).
Proof: the gate runs both stories; one failing is run alone as the
README's "One test of the browser suite" says.
Don't: keep the prefixed path in a `web` build; print a place outside
the prefix.
Size: at most 40 lines.
Removes: nothing.

The designer's answer of 2026-10-18T14:27Z (jig `e444994d4`, ruling
366, point 3): "**A client is one run of `jig-client`,** ... Each client
holds a stream of its own, so two clients of one sign-in, two tabs or a
page and the page that follows it, hold two. The engine never refuses or
ends a stream for another stream of its sign-in" (`protocol/client.md`,
4); "**The predecessor's stream is sent gone.**" (4, 4.3, 10). This part
replaces ruling 366, point 4's "the refusal at `watches.rs:123-132`";
the rest of 19.5 holds, and its two stories come after this part.
Files, added:
- `crates/jig-protocol-engine-clients/src/watches.rs`: `stream_open`
  (`:123-132`) loses its `existing` loop; its capacity check, `Conflict`
  at `self.streams.capacity()`, stays.
- `crates/jig-client-protocol-link/src/{component.rs, store.rs, lib.rs}`:
  at each hello (`stream_event`, `component.rs:563`), a `Request::Save`
  of the stream's name under `STREAM_KEY`, `b"stream"`, its bytes a
  `w::Kept` of the 16-byte name at `TABLE_VERSION`; `start_store`
  (`store.rs:167`) takes that one record beside at most
  `requests.requests` of the table's, and keeps its name; the first
  `OpenStream` after `Start` waits behind that name's `StreamGone`
  exchange, built as `drop_stream` builds it (`component.rs:666-672`),
  until its reply or `Event::Lost`; `crates/jig-client/src/lib.rs`
  re-exports `STREAM_KEY`.
- The three routers that hand `jig-client` its records by `key.len() ==
  16` hand it `STREAM_KEY`'s as well:
  `testing/jig-client-test-shell-web/src/records.rs:30`,
  `tests/client-protocol/src/native.rs:248`,
  `tests/client-protocol/src/front/persistence.rs:67`; the kept lists
  filled from them hold one more:
  `testing/jig-client-test-shell-web/src/host.rs:53-54`,
  `testing/jig-client-test-domain/src/route.rs:26`.
Tests, added, aim for under 20 ms each:
`tests/engine-clients/tests/stream.rs`,
`two_streams_of_one_sign_in_each_open_with_their_own_names`; in
`crates/jig-client-protocol-link/src/tests/`,
`the_stream_s_name_is_saved_at_each_hello`,
`a_kept_stream_name_is_sent_gone_before_the_first_open` (no
`OpenStream` until the gone's reply, and one after `Lost`), and
`a_full_table_and_the_stream_s_name_start`.
Don't: change the closing warning's test,
`testing/jig-client-test-shell-web/src/lib.rs:143` (a 16-byte key is a
request; the stream's name warns of nothing); bound streams per sign-in;
rename `exchanges_per_client`; change the mapping's `streams` refusal
(`jig-protocol-engine-clients-http/src/lib.rs:69-90`) or a profile's
`streams`.
Unchanged: ruling 358, point 4; ruling 362; the stories' assertions;
every deadline and lifetime.
Size: about 150 lines more than 19.5's 40; past 250, report.
Removes: the refusal of a second stream for one sign-in.

The designer's answer of 2026-10-18T16:11Z (skein `f7a4e7022`): "**A
button, a tab or a menu item takes its words one way:** as its name,
holding no node, or, with no name, as the text nodes it holds. A browser
shows a name as the element's text (ui-web.md, section 6), so a text
node inside a named one shows its words twice" (`ui.md`, 3). jig's
controls take the first form; skein's click on the second waits
(`later.md`). This part comes before the two stories, and replaces
ruling 358, point 2's stop for the files it names; the rest of 19.5
holds.
Files, added, each site a named button, tab or menu item holding a node:
- `crates/jig-client-view/src/frame.rs`: `control` (`:150-170`) calls
  `b.text` only when `role == Role::Link` (a link's words are its text:
  ui-web.md, 6, names it by `aria-label`); `membership` (`:180-182`);
- `crates/jig-client-view/src/`: `confirmation.rs:92-95`,
  `pages/work.rs:30-32` and `:48-50`, `sections/project/owner.rs:615-618`,
  `:1109-1112` and `:1169-1173`, `turns/call.rs:42-44` and `:108-110`;
- `crates/jig-client-view/src/sections/project/notes.rs:90-95`: the
  button holds no node, and `owner_authority::pattern`'s nodes follow
  it, after its `close`;
- `examples/ops/client-view/src/infrastructure.rs:150-152`, `:185-187`;
  `examples/ops/client-view/src/root.rs:126-129`;
- `testing/jig-client-test-view/src/family.rs:289-291`,
  `proposal.rs:79-81`, `root.rs:244-247`;
- each count of nodes that a test asserts equal to a built tree and that
  counts a dropped node (`crates/jig-client-view/src/sizes.rs`,
  `sizes/`, a fragment's `*_nodes`): one fewer for each.
Shape: at each site, the `text` call (`b.text`, `fragment.text`,
`words.text`) after `name` goes; the name, states, styles and binding
stay.
Tests, added, aim for under 20 ms each: in
`crates/jig-client-view/src/tests/pages.rs`, `whole_page` asserts, for
each node of the tree it builds, that a button, a tab or a menu item
with a name has `size == 1`, so `each_of_jigs_pages_keeps_the_outline`
checks each page it builds; in `tests/frame_facts.rs`,
`a_named_control_holds_no_node` (`control` with `Role::Button`: size 1,
its name the label; with `Role::Link`: its text the label).
Don't: change skein, or jig's skein pin (README, "Pins"); drop a name;
give a link no text; change a test's words or the text shell's goldens
(the text shell prints a control's name when it holds no text:
skein-ui-text's `render.rs`, `content`).
Unchanged: the stories' assertions; a test finding a button by role and
name. A test finding one by `Find::text` finds it by role and name, with
the same words.
Size: about 60 lines more; past 120, report.
Removes: the text node of the name inside jig's buttons, tabs and menu
items.

## One test of the browser suite

The README's section of that name holds the commands, built on 19.4's
`scripts/check.sh` lines above, and the rule for running them; this file
keeps no copy.

## Interface sketches

Compiled at the base, the pure halves only (the platform halves need
wasm32, which the planner's machine lacked; the run's machine has the
target, README, "Prerequisites", and the commands the planner could not
run at the base are listed in the README for the supervisor's ready
check), in two scratch crates of jig's
workspace with stand-ins for jig 10's and jig 13's types: `cargo
clippy -p jig-sketch-w8 -p jig-sketch-w8-web --all-targets -- -D
warnings`, which passed (`logs/w8/sketch-19.txt`), with the skein pin
held as "Choices" says.

```rust
// crates/jig-client-view/src/address.rs (step code)
/// The segments the codec prints: each one segment of `a` to `z`, `0` to `9` and `-`.
pub struct Names<'a> {
    pub inbox: &'a [u8],
    pub conversations: &'a [u8],
    pub tasks: &'a [u8],
    pub projects: &'a [u8],
    pub deployment: &'a [u8],
    /// `activity`, `plan`, `history`, `budget`, `panels`, in `TaskSection`'s order after `Overview`.
    pub task_sections: [&'a [u8]; 5],
    /// `goals` to `recurring`, in `ProjectSection`'s order after `Overview`.
    pub project_sections: [&'a [u8]; 9],
}
/// The reference codec's settings (browser.md, 9).
pub struct Codec<'a> {
    /// Empty for the root, or one segment or more with a leading `/` and none trailing: `/work`.
    pub prefix: &'a [u8],
    pub names: Names<'a>,
}
/// The reference's: at the root, browser.md 9's table's segments.
pub const REFERENCE: Codec<'static>;
/// The path of an address under the codec's prefix.
pub fn print(codec: &Codec<'_>, address: Address) -> Box<[u8]>;
/// The place a path names: a query and one trailing slash ignored; `/` (or the prefix alone) the home, the inbox;
/// under the prefix but no place, `Place::Missing`; outside it, `Place::Elsewhere`.
pub fn parse(codec: &Codec<'_>, path: &[u8]) -> Place;
```

```rust
// crates/jig-client-shell-web/src/adapter.rs (ordinary Rust; the pure half)
/// A field's name and value, as the mapping gives them.
pub type Header = (Box<[u8]>, Box<[u8]>);
/// One of the client host's transport requests of a step, as the mapping's client end gives it
/// (jig 10's type in place of this stand-in).
pub enum Asked {
    /// An exchange: a POST of its record under the prefix, the browser's header among its headers.
    Exchange { id: u32, method: &'static str, path: Box<[u8]>, headers: Vec<Header>, body: Box<[u8]>, bound: u32 },
    /// The stream's GET, its body read a piece at a time.
    OpenStream { path: Box<[u8]>, headers: Vec<Header>, piece: u32 },
    /// The stream let go.
    CloseStream,
    /// A small store's record kept.
    Save { key: Box<[u8]>, value: Box<[u8]> },
    /// A small store's record erased.
    Erase { key: Box<[u8]> },
}
/// What the platform said, as one of the client host's events.
pub enum Heard {
    /// A response's status and body, to the mapping's client end.
    Answered { exchange: u32, status: u16, body: Box<[u8]> },
    /// The exchange failed, was abandoned, or its body passed its bound.
    Failed { exchange: u32 },
    /// The stream's response began.
    StreamOpened,
    /// A piece of the stream's body.
    Piece { bytes: Box<[u8]> },
    /// The stream's body ended or failed.
    StreamEnded,
    /// A record the small store kept.
    Stored { key: Box<[u8]> },
    /// A record the small store refused: its step's exchanges fail too.
    NotStored { key: Box<[u8]> },
}
/// A step's requests as skein's transport carries them: writes first, then exchanges, then the stream.
pub fn requests(asked: Vec<Asked>) -> skein_ui_web::transport::Requests;
/// fetch's report, as the host's event.
pub fn fetched(report: skein_ui_web::fetch::Report) -> Heard;
/// The transport's report, as the host's event.
pub fn carried(report: skein_ui_web::transport::Carried) -> Heard;

// crates/jig-client-shell-web/src/sign_in.rs (the pure half)
/// The link's route under the mapping's prefix, and the home the address goes to.
pub struct Route<'a> { pub path: &'a [u8], pub home: &'a [u8] }
/// The page's start at the link's route: the secret, and the address to show at once in its place.
pub struct Taken { pub secret: Box<[u8]>, pub address: Box<[u8]> }
/// The secret of `path_and_query` at the link's route; none at any other path or with no `secret`.
pub fn take(route: &Route<'_>, path_and_query: &[u8]) -> Option<Taken>;
```

## Questions to raise when reached

- None the design leaves open for these increments. The two-person
  stories of skein's that need a second printed link are jig 20's
  question.

## Done when

Each command runs once, after the session's last increment lands.

- `git -C ~/src/rust/jig log origin/main --grep='^Plan: client-api$'
  --format='%(trailers:key=Step,valueonly,separator=)' | grep
  '^jig-19\.' | sort -u | wc -l` prints 5.
- `git -C ~/src/rust/jig show origin/main:Cargo.lock | grep -o
  'skein.git#[0-9a-f]*' | sort -u` prints one line, the commit of jig
  17.3's move (`0f093bb74`, for `skein_lib::sha256`; ruling 302, as
  ruling 383 reads it), and with `<skein-01.s2>` the commit of "Choices"'
  command, `git -C ~/src/rust/skein merge-base --is-ancestor
  <skein-01.s2> <that commit>` succeeds.
- `git -C ~/src/rust/jig grep -n -e wasm32-unknown-unknown -e 'profile
  browser' origin/main -- scripts/check.sh` prints a line for each
  (README, "Done when", 7).
- `git -C ~/src/rust/jig ls-tree -d --name-only origin/main
  crates/jig-client-shell-web testing/jig-client-test-shell-web
  tests/client-browser | wc -l` prints 3.
