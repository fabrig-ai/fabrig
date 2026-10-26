# Session 02: the browser shell

temper has no browser shell at the base: its client runs in the terminal alone. `client/browser.md` 2.1 asks for one,
`temper-client-shell-web`, copied from jig's reference shell with temper's root, view and records in its place; 2.3 gives
each page a path, by a codec in `temper-client-domain`; 2.5 a stylesheet after skein's; 4.3 a gate that lints the
client's crates for wasm32 and builds the bundle. 02.1 writes the codec and lints the three client crates that exist for
wasm32. 02.2 makes the shell, which starts the root and the view on a page, signs in, and moves on the root's `Show`, with
no server yet. 02.3 gives it jig's mapping under `/jig`, the small store and the timer, and writes `scripts/bundle.sh`,
`temper.css` and `files/robots.txt`: the bundle that temper 04.1 needs and the gate builds. 02.4 adds the history, the
links' addresses and the title. Each increment needs the one before. 02.1 → 02.2 → 02.3 lands beside the plan's critical
path, before temper 04.1 needs it (README), and 02.4 later. The session waits on nothing of temper 01 or 03 (decision 3),
and plans against `main` after `tools/temper-repin` (landed, `8c02b3b9e`).

## Read first

- **temper:** `docs/design/client/browser.md` 2 (its list), 2.1, 2.3, 2.4 (the start's sign-in), 2.5, 4.3;
  `docs/design/client/README.md` 2 (the limits), 3.5 (the pages), 7 (the small store);
  `crates/temper-client-domain/src/address.rs` (`Address`, `Filter`, `Place`), `src/boundary.rs` (`Event`, `Request`,
  `Action`), `src/domain.rs` (`new`, `step`, `fire`, `take`, `next_deadline`), `src/route.rs` (a `Went` moves the root);
  `crates/temper-client-shell-terminal/src/limits.rs` (`native`, `view`, `worst_case`), `src/client.rs`
  (`NativeClient::new`'s `View::new`, `go`, `outputs`), `src/route.rs` (`down`); `scripts/check.sh`.
- **jig:** `docs/design/protocol/browser.md` 3.1 (its browser shell), 10 (transport and the small store), 11 (signing
  in); the shell to copy, `testing/jig-client-test-shell-web/` (`src/lib.rs`, `src/host.rs`, `src/records.rs`,
  `src/mapping.rs`, `Cargo.toml`, `clippy.toml`, `files/robots.txt`); `crates/jig-client-view/src/address.rs` (`print`,
  `parse`, `segment`, `decimal`, `project_decimal`); `crates/jig-client-shell-web/src/adapter.rs`, `src/sign_in.rs`;
  `scripts/check.sh` (its wasm32 clippy and its bundle).
- **skein:** `docs/design/ui-web.md` 3.2 (the look), 4.2 (the bundler), 5.2 (the turn), 12 (the stylesheet);
  `crates/skein-ui-bundle/src/main.rs` (`USAGE`), `src/lib.rs` (`bundle` replaces `out` whole), `src/page.rs`.
- **skein foundation:** `programming-model.md` 2.1 (the shell), 3 (step functions), 6.3 (the worst case), 10 (the Rust
  subset); `testing-strategy.md` 2.1 (step tests), 10 (a test's shape).
- **Plan:** the README's "Rules of this plan", "The critical path" and "One test of a suite that needs a built
  artifact"; `code-organization.md` 1 to 3, 5 and 6; decisions.md 2, 3, 6, 7, and Design question 4.

## What the design asks

- **browser.md 2.1:** a `cdylib` copied from `jig-client-test-shell-web`, with none of jig's look; the modules `lib`,
  `host`, `records` and `limits`; the test address `/test/panic`; the small store under `temper.`, holding
  `jig-client`'s records alone; no service credential shown, and no line for kept work dropped. 02.2 and 02.3 build it.
- **browser.md 2.3:** the five pages' paths; a path that is no page opens the inbox at `/`; no reserved path printed.
  02.1 writes the codec. Moving is skein's history, which 02.4 wires.
- **browser.md 2.4:** the link's secret is taken before anything is sent, and the client signs in with `Link` or, without
  one, `Transport`. 02.2.
- **browser.md 2.5:** `temper.css` after `skein-ui.css`, with the palette as variables under `temper-` and the system's
  monospaced font. 02.3 writes them. The tokens' rules, the navigation, the borders and the composers are temper 03's
  (decision 3).
- **browser.md 4.3:** the gate lints four crates for wasm32 (02.1, then 02.2 adds the shell) and builds the bundle once,
  with the dev profile (02.3). The tier itself is temper 04's.
- **decisions.md, Design question 4 and decision 7:** `skein-ui.css`, then `temper.css`; the name
  `temper_client_shell_web`, the prefix `/bundle/`, the noscript words and the title "temper". 02.3 and 02.4.

## Choices this plan makes

- **The codec has no `Codec` value,** no prefix and no names: temper's paths are fixed (2.3). `print` takes `&Address`,
  which is `Clone` and not `Copy`. As jig's, it ignores the query and one trailing `/`, takes numbers in canonical decimal
  only, and does no percent-decoding. The inbox prints `/` whatever its filter.
- **The shell's client limits are the terminal's `native().client`, copied,** as 2.1 says ("derived as the terminal's
  are"). `EXCHANGES` is 8 and `BODY_BYTES` 32 MiB, as the terminal's. The stream's piece is 4096, as jig's shell has it.
- **The restart's words are "The application failed.",** jig's (`jig-client-shell-web`'s `look::phrases`), written as a
  literal: 2.1 takes none of jig's look and names no words.
- **A `Show` steps `Went`,** since temper's root moves only on `Went`. When its address differs from the one shown, the
  shell steps `Went` with it, as the terminal's `outputs` and `go` do. 02.4 adds the history entry. A path that is no
  page, at start or on back and forward, is the inbox, with `/` in the address bar.
- **The transport's requests wait for 02.3.** 02.2 drops them, in one arm that 02.3 removes. The timer comes with them:
  the client's deadlines only matter once it talks.
- **`scripts/bundle.sh [--release] OUT`** reads `OUT` from the repository's root, as `check.sh` and the README's commands
  run it, and prints `OUT`'s absolute path alone on stdout. The bundler's own output goes to stderr. It refuses an
  existing `OUT` that holds no `index.html`, since `skein_ui_bundle::bundle` removes `out` whole.
- **`skein-ui-bundle` joins temper's graph as the shell's dev-dependency,** so that `cargo build -p skein-ui-bundle`
  resolves before temper 04.1's crate exists. It gets `opt-level = 3` in the dev profile, as skein's and jig's roots do.
- **`temper.css` takes the mockups' `:root`:** `color-scheme: light`, and each colour and the shadow as `--temper-<its
  mockup name>`. The body takes the canvas, the ink and a monospaced font, and controls inherit the font. temper 03.3
  adds the rules for its tokens after these (decision 3). It needs 02.3 for `temper.css`, and 02.2 for the shell's `Look`.

## Rules that bind it

- **The shell's host is empty:** every dependency is under `cfg(target_arch = "wasm32")`, with none on the host's
  `wasm-bindgen`, `web-sys` or `js-sys`. There is no dependency on jig's test shell or test service: their code is copied
  (code-organization.md 5).
- **One codec:** no filter in the address, and no path under `/jig`, `/bundle/` or `/test/`, nor `/robots.txt`, printed.
  No other crate parses or prints a path (code-organization.md 2 and 5).
- **Dependency lines copy the URL** temper's `Cargo.toml` already uses for that repository (README, Rules). `temper.css`
  has no web font, icon or external origin (code-organization.md 5).

## Increments

### 02.1 The address codec, and the client's crates linted for wasm32
Scope: browser.md 2.3: `parse` and `print` over the existing `Address` and `Place`. browser.md 4.3: the gate's wasm32
clippy over `temper-client-domain`, `temper-client-view` and `temper-client-protocol-wire`.
Needs: nothing.
Files: `crates/temper-client-domain/src/address.rs`; `src/tests.rs` (`mod address;`); `src/tests/address.rs` (new);
`scripts/check.sh` (a hotspot: the real-loop plan's process tier, README). `Address`, `Place` and `Filter` are unchanged,
so their users are too.
Shape: `pub fn parse(path: &[u8]) -> Place` and `pub fn print(address: &Address) -> Box<[u8]>`, from jig's
`crates/jig-client-view/src/address.rs` `parse` and `print`, with its private `segment`, `decimal` and `project_decimal`
copied. Unlike jig's, there is no codec, prefix, section or `Elsewhere`. `parse` cuts the query at
`skein_lib::bytes::find(path, b"?")` and strips one trailing `/`. An empty path is `Inbox { filter: Filter::ALL }`. A path
still ending in `/`, or not starting with one, is `Missing`. Then it matches `chats`; `chats/<task>`; `tasks/<task>`;
`projects/<project>/adopt`, with nothing after. `print` writes with `skein_lib::{Decimal, Writer}` and checked sizes, as
jig's does. `alloc::boxed::Box`: the crate is `no_std`. In `scripts/check.sh`, after the workspace clippy, with a comment
citing browser.md 4.3: `cargo clippy --target wasm32-unknown-unknown -p temper-client-domain -p temper-client-view -p
temper-client-protocol-wire -- -D warnings`, split over lines as jig's `scripts/check.sh` does.
Test shape: unit, in `crates/temper-client-domain/src/tests/address.rs`, with no fixture. The reference test:

```rust
use crate::address::{Address, Filter, Place, parse, print};
#[test]
fn each_page_round_trips_through_its_path() {
    let pages: [(Address, &[u8]); 5] = [
        (Address::Inbox { filter: Filter::ALL }, b"/"),
        (Address::Chats, b"/chats"),
        (Address::Chat { task: 7 }, b"/chats/7"),
        (Address::Task { task: 7 }, b"/tasks/7"),
        (Address::Adopt { project: 3 }, b"/projects/3/adopt"),
    ];
    for (address, path) in pages {
        assert_eq!(&*print(&address), path, "{address:?}");
        assert_eq!(parse(path), Place::Address(address), "{path:?}");
    }
}
```

Beside it:
- `the_inbox_prints_slash_whatever_its_filter`: a project, a kind and a goal.
- `a_query_is_ignored`: `/chats/7?x=1`, and `/?secret=s` is the inbox.
- `one_trailing_slash_is_ignored`: `/chats/7/`, `/chats/`; `/chats/7//` is `Missing`.
- `a_path_that_is_no_page_is_missing`: `/jig/v1/sign-in/link`, `/bundle/x`, `/test/panic`, `/robots.txt`, `chats`,
  `/chats/x`, `/tasks`, `/projects/1`, `/projects/1/board`, `/projects/3/adopt/x`, `//`, `/chats/%37`.
- `a_number_not_in_canonical_decimal_is_missing`: `07`, `+7`, `-7`, `7a`, 21 digits.
- `each_number_spans_its_types_range`: `u64::MAX` as a task and `u32::MAX` as a project round trip; one more is `Missing`.

Ceilings: each under 20 ms.
Proof: unit: these seven, focused, in the gate. Nothing above them is within reach: the codec's first user is
02.2's shell, which the host does not build. temper 04's `a_non_page_address_opens_the_inbox_at_slash` later reads a
path through it in a browser. The wasm32 clippy shows that the three crates build for the browser.
Don't: add a filter, a prefix or a percent-decoder; return `Option`; parse in the shell.
Size: at most 110 lines (`address.rs`, `check.sh`).
Removes: nothing.

### 02.2 The shell runs temper's root and view on the page
Scope: browser.md 2.1: `temper-client-shell-web`'s start and turn, with temper's root and view and no server yet;
browser.md 2.4: the link's secret is taken first; browser.md 4.3: the shell joins the wasm32 clippy.
Needs: 02.1's `parse`.
Files: `Cargo.toml` (the member `crates/temper-client-shell-web`; workspace dependencies `skein-ui-web` and
`jig-client-shell-web` with the URLs already used for skein and jig, and `wasm-bindgen = "=0.2.126"` and
`web-sys = "=0.3.103"`, as skein's and jig's roots pin them); `Cargo.lock`. Both are hotspots (README).
`crates/temper-client-shell-web/` (new): `Cargo.toml`, `clippy.toml`, `src/lib.rs`, `src/host.rs`, `src/limits.rs`.
`scripts/check.sh` (`-p temper-client-shell-web` in the wasm32 clippy).
The copies have users:
- `limits::client` and `limits::view` copy `crates/temper-client-shell-terminal/src/limits.rs`'s `native().client` and
  `view`. A later change to either changes both.
- The shell's `View::new` call is a user of temper 03's browser words. Whichever lands second updates it.
Shape: copied from jig's `testing/jig-client-test-shell-web`:
- **`Cargo.toml`:** the package fields `crates/temper-client-shell-terminal/Cargo.toml` inherits from the workspace
  (README, Rules, a new crate's license); a `cdylib` whose `[target.'cfg(target_arch = "wasm32")'.dependencies]` are
  `jig-client`, `jig-client-shell-web`, `skein-lib`, `skein-ui`, `skein-ui-web`, `temper-client-domain`, `temper-client-view`,
  `temper-client-protocol-wire`, `wasm-bindgen`, and `web-sys` with the features `Window`, `Document`, `Location` and
  `HtmlElement`. Its `[lints]` are `crates/temper-client-shell-terminal/Cargo.toml`'s.
- **`clippy.toml`:** jig's, as it is.
- **`src/lib.rs`, from jig's `src/lib.rs`:**
  - `#![cfg(target_arch = "wasm32")]`.
  - `Report::{Start, Act}`, and a `Shell` of `host`, `page` and `dom`.
  - `start()`: `Restart::new(3, Duration::from_secs(60), "temper-restarts", "The application failed.").install("app")`;
    `sign_in::web::take_from_page(&sign_in::Route { path: b"/jig/v1/sign-in/link", home: b"/" })`; the path, as the
    location's pathname and search; the debug `/test/panic` `assert_ne!`; `Host::new(facts.seed, facts.now, facts.wall)`
    and `host.start(path, secret)`.
  - `Page::new` with `Look { names: &[], pictures: &[], draw: |_, _, _, _, _| false, components: &[], elements: &[] }`,
    `&host.bounds.tree`, `b"temper"` and `facts.primary`; then `Dom::new`.
  - `turn`: `platform::clocks`; `Start` and `Act`; `host.requests()`; `host.build`, `page.apply`, `dom.apply`.
- **`src/host.rs`, from jig's `src/host.rs`:**
  - `Host { root, view, bounds, env, out, address }`.
  - `new`: `limits::client()` and `limits::view`, `temper_client_domain::new`, `View::new(&bounds, &client, words,
    temper_client_protocol_wire::FORGE)`, as the terminal's `NativeClient::new` makes them.
  - `start`: `parse`; `Missing` is `Inbox { filter: Filter::ALL }`. It steps `Event::Start { address, kept }` with an
    empty `List`, then `Event::Client` with `SignIn { with }`: `With::Link(Secret::new(bytes))` from the taken secret,
    else `With::Transport`.
  - `clocks`, `act` (`temper_client_view::act`) and `build`.
  - `requests()` drains `out`, then calls `temper_client_domain::take`:
    - `Show` steps `Went { place: Place::Address(address) }` and keeps the address, unless it is the one kept.
    - `Request::SignOut` steps `jig_client::Event::SignOut`, as the terminal's `Duty::SignOut`.
    - `Dropped` and `Fact` are ignored (2.1).
    - `Client { .. }` is dropped: the transitional arm, which 02.3 removes.
- **`src/limits.rs`:** `client()`, the terminal's `native().client` with its `#![deny(…)]` header, and
  `view(&client)`. Temper's `Look` names come with temper 03.3.
Test shape: none. The crate holds no host code: `#![cfg(target_arch = "wasm32")]`, as jig's. What the page shows is
the root's and the view's, step code tested in their crates (2.1, "It decides nothing").
Proof: no layer within reach: nothing serves the shell until 02.3's bundle and temper 01.2's routes. The gate's wasm32
clippy builds it. temper 04.1's `the_printed_link_signs_in` runs it first.
Don't: depend on jig's test shell or `jig-client-view`; use jig's `look` (`NAMES`, `PICTURES`, `phrases`) or show a
service credential; add a host dependency; send anything before the secret is taken.
Size: at most 290 lines (the crate's five files, `check.sh`; the root's `Cargo.toml` and `Cargo.lock` are not counted).
Removes: nothing.

### 02.3 The client talks to its server, and the bundle (the step temper 04.1 needs)
Scope: browser.md 2.1: jig's mapping under `/jig` and its stream, the small store under `temper.`, the root's timer, the
`records` and `limits` modules; browser.md 2.5: `temper.css`'s palette and font; browser.md 4.3 and Design question 4: the
bundle, built once by the gate.
Needs: 02.2's shell.
Files:
- `Cargo.toml`: workspace `skein-ui-bundle` (the skein URL), and `[profile.dev.package.skein-ui-bundle] opt-level = 3`.
  `Cargo.lock` too. Both are hotspots.
- The shell's `Cargo.toml`: wasm32 `jig-client-protocol-http`, and `[dev-dependencies] skein-ui-bundle`.
- The shell's `src/lib.rs`, `src/host.rs`, `src/limits.rs`, and `src/records.rs` (new).
- `crates/temper-client-shell-web/temper.css` and `files/robots.txt` (new); `scripts/bundle.sh` (new).
- `scripts/check.sh`, a hotspot.
Shape: copied from jig's `testing/jig-client-test-shell-web`:
- **`src/lib.rs`, from its `src/lib.rs`:**
  - `Report` gains `Due`, `Fetch` and `Carried`; the `Shell` gains `platform`, `store` and `timer`.
  - `platform::Browser::new(fetch::web::Fetch::new(limits::EXCHANGES + 1, …, Report::Fetch))`;
    `Store::new(b"temper.", None)`, whose `read_all` goes to `host.start`; `timer::web::Timer::new(…, || Report::Due)`.
  - The turn: `adapter::requests`, `transport::carry`, `Report::Carried`, `read_on`; `fired`, `due`; and, at its end,
    `timer.set(host.deadline(), now)`.
- **`src/host.rs`, from its `src/host.rs`:**
  - `mapping` (`Mapping::new(b"/jig")`, as its `src/mapping.rs`), `stream`, `tickets` and `next`.
  - `heard`, `carried`, `due` and `deadline`, as jig's.
  - `start` takes the store's records, `records::decode`d, into `kept`, as the terminal's `iterate` sizes it.
  - `requests() -> Vec<Asked>`:
    - `Save { records }` is an `Asked::Save` for each of `records.into_boxed()`, through `records::encode`.
    - `Erase { records }` is an `Asked::Erase` for each key.
    - `Exchange` has `bound: limits::BODY_BYTES`; `OpenStream` gets a new `Stream` and `piece: limits::PIECE`; then
      `CloseStream`.
    - `ServiceCredential`, `Told`, `Down` and `jig_client::Request::SignOut` are ignored, by name.
- **`src/records.rs`:** `encode` and `decode`, from its `src/records.rs`'s `Record::Client` arm and its 16-byte or
  `STREAM_KEY` test. The forge's pending adoption is not kept: 2.1's small store holds `jig-client`'s records alone.
- **`src/limits.rs`:** `EXCHANGES`, `BODY_BYTES` and `PIECE`; `stream()`; `worst_case(&client)`. The sum is the root's
  (`temper_client_domain::worst_case`), the view's, the stream's (`jig_client_protocol_http::worst_case`), and
  `EXCHANGES + 1` bodies, all checked. `Host::new` `expect`s it (programming-model.md 6.3).
- **`temper.css`:**
  - `:root` sets `color-scheme: light` and `--temper-canvas` … `--temper-red-pale` and `--temper-shadow`, the values of
    `docs/design/client/mockups/style.css`'s `:root`.
  - `body` takes `background: var(--temper-canvas)`, `color: var(--temper-ink)` and `font: 14px/1.5 ui-monospace,
    monospace`.
  - `button, input, textarea, select { font: inherit; }`.
  - It has no `@import`, `@font-face` or `url(`.
- **`files/robots.txt`:** jig's, `User-agent: *` and `Disallow: /`.
- **`scripts/bundle.sh`,** `set -eu`, from jig's `scripts/check.sh`'s bundle lines:
  - It takes `[--release] OUT`, and gives its usage on any other.
  - It changes to the root, and refuses an existing `OUT` without `index.html`.
  - `cargo build --target wasm32-unknown-unknown [--release] -p temper-client-shell-web`; `cargo build -p
    skein-ui-bundle`.
  - `target/debug/skein-ui-bundle` with the module from `debug` or `release`, `--name temper_client_shell_web`,
    `--prefix /bundle/`, `--stylesheet crates/temper-client-shell-web/temper.css`, `--files
    crates/temper-client-shell-web/files`, `--noscript 'temper needs WebAssembly and scripts.'` and `--out "$OUT"`,
    with its stdout to stderr.
  - It prints `OUT`'s absolute path.
- **`scripts/check.sh`:** after the fuzzy suite, `scripts/bundle.sh target/temper-bundle >/dev/null`, with a comment
  citing browser.md 4.3. temper 04.1 runs the tier on it.
Test shape: none on the host, as 02.2. The bundler's own refusals (a reserved stylesheet name, a missing file) are
skein's tests.
Proof: world: the browser suite, `--profile browser`, temper 04.1's `the_printed_link_signs_in`. It is the shell's first
run, and its meeting point with temper's server (decision 2). Until it lands, the gate builds the bundle on every
landing, which fails on a shell that does not build for wasm32 or a file the bundler refuses. `worst_case` is checked at
the page's start, in a browser only.
Don't: add `--pictures` or `--page`; ship `mockups/style.css` (Design question 4); keep the forge's records; set a token's
rule in `temper.css` (temper 03.3's).
Size: at most 280 lines (`host.rs`, `lib.rs`, `limits.rs`, `records.rs`, `temper.css`, `bundle.sh`, `robots.txt`,
`check.sh`, the shell's `Cargo.toml`).
Removes: 02.2's arm that drops the transport's requests.

### 02.4 History, links and the title
Scope: browser.md 2.3, "Moving": a link pressed is a new history entry, back and forward are moves, and a path that is
no page is replaced with `/`; decision 7: the title "temper".
Needs: 02.3's shell.
Files: `crates/temper-client-shell-web/src/lib.rs`, `src/host.rs`.
Shape: copied from jig's `testing/jig-client-test-shell-web`:
- **`src/lib.rs`, from its `src/lib.rs`:**
  - `Report::Place`, and `history::web::History::new(…, Report::Place)` and `title::web::Title::default()` in the
    `Shell`.
  - In the turn: `Place` to `host.went`; `host.show.take()` to `history.show(&path, replace)`, a new page unless
    replaced.
  - After `apply`, `Page::links()` with `host.href` and `set_href`.
  - `finish`, as jig's: `title.set(heading, b"temper", b"en", closing_loses)`; a new page scrolls to the top and
    focuses its heading or `main`; then `history.restore()` and `timer.set`.
- **`src/host.rs`:**
  - `pub show: Option<(Box<[u8]>, bool)>`.
  - A `Show` also prints its address into `show`, with its `replace`, unless its path is the one the address bar shows.
    A filter's `Show` of the inbox so adds no entry (2.3: filters are not in the address).
  - `start` shows the printed path, replaced, when it differs from the path it read. So a path that is no page becomes
    `/`, and `/chats/` becomes `/chats`.
  - `went(path)`: `parse`, then `Went`, with `Missing` as the inbox and `/` shown replaced.
  - `href(node)`: through `temper_client_view::act` with `Act::Press`.
    - `Action::Pages(pages::Act::Go { address })` prints `address`.
    - `Action::Forge(forge::Act::Go { address: forge::Address::Adopt { project } })` prints `Address::Adopt { project }`.
    - Any other action is `None`.
Test shape: none on the host, as 02.2. The codec's choices are 02.1's tests.
Proof: world: the browser suite, `--profile browser`, temper 04's `back_then_forward_returns_to_the_chat`,
`a_reload_on_a_chat_signs_in_by_its_cookie` and `a_non_page_address_opens_the_inbox_at_slash`, which need this step.
Until they land, the gate's wasm32 clippy and its bundle.
Don't: keep a record of the address (2.3: the address is the browser's); push an entry for a `replace`; print in
`temper-client-view`.
Size: at most 90 lines (`lib.rs`, `host.rs`).
Removes: nothing.

## Interface sketches

The codec is `temper-client-domain`'s: the shell is its one user (code-organization.md 2). The planner compiles it
against `temper-client-domain` on the host, with stub bodies.

```rust
// crates/temper-client-domain/src/address.rs

/// The page's path (client/browser.md, 2.3): `/`, `/chats`, `/chats/<task>`, `/tasks/<task>`,
/// `/projects/<project>/adopt`. The inbox prints `/` whatever its filter.
#[must_use]
pub fn print(address: &Address) -> Box<[u8]>;

/// The page a path names, ignoring its query and one trailing slash; any other path is `Place::Missing`.
#[must_use]
pub fn parse(path: &[u8]) -> Place;
```

The shell's items are private to a `cdylib` built for wasm32 alone: no type crosses a crate boundary, and the gate's
wasm32 clippy compiles them. They are sketched for their signatures.

```rust
// crates/temper-client-shell-web/src/limits.rs
pub const EXCHANGES: u32 = 8;                // 02.3
pub const BODY_BYTES: u32 = 33_554_432;      // 02.3
pub const PIECE: u32 = 4096;                 // 02.3
pub fn client() -> temper_client_domain::Limits;
pub fn view(client: &temper_client_domain::Limits) -> Option<(temper_client_view::Limits, temper_client_view::words::Words)>;
pub fn stream() -> Option<jig_client_protocol_http::Limits>;                // 02.3
pub fn worst_case(client: &temper_client_domain::Limits) -> Option<u64>;    // 02.3

// crates/temper-client-shell-web/src/records.rs (02.3)
pub fn encode(record: jig_client::Record) -> (Box<[u8]>, Box<[u8]>);
pub fn decode(record: skein_ui_web::store::Record) -> Option<jig_client::Record>;

// crates/temper-client-shell-web/src/host.rs
impl Host {
    pub fn new(seed: u64, now: Time, wall: Wall) -> Self;
    /// 02.2 without `records`; 02.3 adds them.
    pub fn start(&mut self, path: &[u8], records: Vec<skein_ui_web::store::Record>, secret: Option<Box<[u8]>>);
    pub fn clocks(&mut self, now: Time, wall: Wall);
    pub fn act(&mut self, node: NodeId, act: Act);
    /// 02.2 returns nothing; 02.3 the adapter's requests.
    pub fn requests(&mut self) -> Vec<Asked>;
    pub fn build(&mut self, patches: &mut Queue<Patch>);
    pub fn due(&mut self);                                              // 02.3
    pub fn heard(&mut self, heard: Heard);                              // 02.3
    pub fn carried(&mut self, reports: Vec<transport::Carried>, stored: bool); // 02.3
    pub fn deadline(&self) -> Option<Time>;                             // 02.3
    pub fn went(&mut self, path: &[u8]);                                // 02.4
    pub fn href(&mut self, node: NodeId) -> Option<Box<[u8]>>;          // 02.4
}
```

## Questions to raise when reached

- **02.1:** if the wasm32 clippy over the three crates fails at the base for a cause outside the codec (a dependency
  that does not build for `wasm32-unknown-unknown`), report the error and ask before repairing it.
- **02.3:** record the dev bundle's files and bytes (`find OUT -type f | wc -l`, `du -sb OUT`). If they pass decision 6's
  128 files or 16 MiB, ask: temper 01.1 owns those limits.

## Done when

Each in a worktree of temper at `origin/main`, after `git fetch`:
- `git show origin/main:crates/temper-client-domain/src/address.rs | grep -q -F 'pub fn parse(path: &[u8]) -> Place {'`
- `git show origin/main:crates/temper-client-domain/src/address.rs | grep -q -F 'pub fn print(address: &Address) -> Box<[u8]> {'`
- `sh -c 'for c in domain view protocol-wire shell-web; do git show origin/main:scripts/check.sh | sed -n
  "/--target wasm32-unknown-unknown/,/-D warnings/p" | grep -q -E -- "-p temper-client-$c( |\$)" || exit 1; done'`
- `cargo clippy --target wasm32-unknown-unknown -p temper-client-domain -p temper-client-view -p
  temper-client-protocol-wire -p temper-client-shell-web -- -D warnings` exits 0.
- `sh -c 'test "$(cargo tree -p temper-client-shell-web -e normal --depth 1 | wc -l)" -eq 1'`: the host has no
  dependency of the shell's.
- `scripts/bundle.sh target/done-02 && ls target/done-02/index.html target/done-02/*/skein-ui.css
  target/done-02/*/temper.css target/done-02/*/files/robots.txt`
- `tr -d '\n' < target/done-02/index.html | grep -q -E 'skein-ui\.css.*temper\.css'`
- `git show origin/main:scripts/check.sh | grep -q -E '^scripts/bundle\.sh '`
- `git show origin/main:crates/temper-client-shell-web/temper.css | awk '/--temper-canvas:/{c=1} /monospace/{m=1}
  /@import|@font-face|url\(/{bad=1} END{exit !(c && m && !bad)}'`
- `git show origin/main:crates/temper-client-shell-web/files/robots.txt | grep -q -x 'Disallow: /'`
- `git show origin/main:Cargo.toml | grep -q -E '^skein-ui-web = \{ git = "[^"]*/skein\.git"'`, and the same for
  `skein-ui-bundle`; `git show origin/main:Cargo.toml | grep -q -E '^jig-client-shell-web = \{ git = "[^"]*/jig\.git"'`
- `git show origin/main:crates/temper-client-shell-web/src/lib.rs | grep -q -F 'History::new('`
- `git show origin/main:crates/temper-client-shell-web/src/lib.rs | grep -q -F '.set_href('`
