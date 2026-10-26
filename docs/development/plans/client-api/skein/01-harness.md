# Session 01: the harness opens the printed URL, serves other origins, awaits trouble

skein's browser harness, in `skein-fake-person-browser`, learns what a
browser tier needs once an application prints a one-time URL and checks
its server against other sites: it opens the line a server prints, from
its stdout or its stderr, as a URL, and restarts the server with
arguments that name the address it had; it serves fixed pages at other
origins in its own loop and opens them by a step; and a step awaits
trouble with given words, which then does not fail the story, or sends a
request with a given `Host` and awaits its status. The harness stays
generic: it names no application, and skein's UI example's tier keeps
running as it does (skein's `ui-web.md`, 14.1 and 14.2).

## Read first

- skein's `AGENTS.md` (52 lines) and `docs/development/workflow.md`
  if present; skein's `docs/design/ui-web.md`: 4.4, 14, 14.1, 14.2;
  skein's `docs/design/browser.md`: 5 and 9 (the kit's pages and trouble).
- `findings/server-and-browser.md`: 3 (the last two bullets) and 4.
- skein's `testing-strategy.md`: 2.9 (the harness's one loop) and 8, by
  number only.

## What the design asks

skein's `ui-web.md`, 14.2, quoted, is what this session builds on:

- "**The harness,** in the same crate, ordinary Rust, runs the
  application's server binary and Chromium as children of one loop,
  through skein's io (testing-strategy.md, 2.9); opens the address the
  test gives, or one the binary prints, found by a prefix the test gives;
  runs each person's script through the face; fails a test on any trouble
  the kit reports that the story did not expect, a policy violation, an
  exception or a native dialog".
- "**stops the server binary and starts it again** at the address it
  had, which its arguments name, so that a page sees its server go and
  come back".
- 14.1: "The facts the face assumes, checked against Chromium on fixed
  pages that a small server on skein's HTTP serves in the test's loop".

What the harness cannot do at the base (`findings/server-and-browser.md`,
4): it reads the server's stderr only and takes the printed line as
`host:port` (`harness.rs:79-87`, `:558`); a restart replaces the first
argument with the address (`harness.rs:415`); it spawns only the server
and Chromium, with one accept (`harness.rs:138-139`, `:197-200`); trouble
is tolerated by `Setup.expected` for the run and cannot be awaited
(`harness.rs:336-345`); no step opens an absolute URL (`face.rs:265`).

## Choices this plan makes

- **The printed line's form is the test's:** `Names::Address` (the rest
  of the line after the prefix is `host:port`, opened at `/`: the UI
  example's) or `Names::Url` (the line, its prefix included, is an
  absolute URL, opened as printed). In the URL form, each person opens a
  line no earlier person opened, waiting for a newer one, since a printed
  URL may be one-time; the address the face's `go` uses is the URL's
  `host:port`.
- **A restart's arguments are the test's own,** `Setup.restart`, each
  `{address}` in them replaced by the address the server had; the UI
  example's tier passes `["{address}", "--bundle", <dir>]`, which keeps
  what it does today.
- **Other origins are fixed pages the test gives,** served by the
  harness on listeners of their own on `127.0.0.1`, each opened by a step
  at a host name the test gives (`127.0.0.1` for another port of the same
  host; `localhost` for another site). Each `{address}` in a page's body
  is replaced by the server's address when served, so a page can name
  the application's origin. The page server is the browser kit's own,
  `tests/browser/src/serve.rs`, moved into the harness with its pages as
  data; `tests/browser` passes its pages to it.
- **Awaited trouble is a step:** `Step::Trouble { words, within }`. Its
  words are expected on that person's pages from the script's start (so
  trouble that comes before the step is reached does not fail the run),
  and the step waits until trouble with those words has come.
- **A request with a given `Host`** is a step, `Step::Request`, sent by
  skein's HTTP client to the server's address, its status awaited. It is
  not in `ui-web.md`, 14.2's text; it is a design gap for skein's
  designer (the report names it), and the step is generic: it names no
  application.

## Rules that bind it

- **skein names no repository downstream of it** (the workspace's
  `AGENTS.md`: "Dependencies point upstream only"): at each increment's
  end, `git -C <worktree> grep -n -i -w jig -- crates testing tests
  examples` prints nothing (it prints nothing at the base), and `git -C
  <worktree> diff origin/main...HEAD | grep -i -w -E '^\+.*(jig|temper|smith)'`
  prints nothing (the base names temper and smith in 50 older lines,
  `logs/w8/skein-jig-names.txt`, which this session leaves alone).
- **The UI example's tier keeps passing as it is:** its browser tests'
  names and expected values are unchanged; only their `Setup` literals
  gain the new fields (the users of a type the Shape changes, README).
- **Step code is untouched:** the harness is ordinary Rust in
  `testing/`; nothing under `crates/` changes.

## Increments

### 01.1 The printed line, from stdout or stderr, as an address or a URL; restart arguments

Scope: `Setup.printed` (the prefix, the output it comes on, what it
names) in place of `Setup.listening`; the server's stdout read when the
test names it; the first page opened at the printed URL in the URL form,
and a person after the first at a line no earlier person opened;
`Setup.restart` with `{address}` (`ui-web.md`, 14.2, quoted above).
Needs: nothing.
Files: `testing/skein-fake-person-browser/src/{harness.rs, stories.rs,
tests.rs}`; the `Setup` literals of `tests/ui-web/src/lib.rs:63`,
`tests/ui-web/tests/browser_example.rs:37` and `:109`,
`tests/ui-web/tests/browser_stories.rs:9` (five construction sites with
`stories.rs:9`, found by `git grep -n -E 'Setup \{|listening:|listening_address'`,
`logs/w8/skein-setup-users.txt`); no user in jig, temper or smith
(`git grep skein_fake_person_browser` prints nothing in each).
Shape (sketch below): `pub struct Printed<'a> { pub prefix: &'a [u8],
pub on: Output, pub names: Names }`, `pub enum Output { Stdout, Stderr
}`, `pub enum Names { Address, Url }`; `Step::Reopen`, a go to the URL
this person's page first opened (a printed one-time URL opened again,
which an application's story of a spent link needs); `Setup.listening` replaced by
`Setup.printed: Printed<'a>`; `Setup.restart: Vec<OsString>`;
`listening_address(stderr, prefix)` replaced by `pub fn printed(output:
&[u8], printed: &Printed<'_>) -> Vec<Opened>` with `pub struct Opened {
pub url: String, pub address: String }`, one per printed line in order;
the server's spawn pipes fd 1 as well when `on` is `Stdout`.
Test shape: unit tests in `testing/skein-fake-person-browser/src/tests.rs`,
beside `the_listening_line_gives_the_address` (`tests.rs:164`), which
becomes a row of the new table test. The reference test:
```rust
const PRINTED: &[(&[u8], Names, &[(&str, &str)])] = &[
    (b"starting\nserver at 127.0.0.1:44327\n", Names::Address, &[("http://127.0.0.1:44327/", "127.0.0.1:44327")]),
    (b"server at 127.0.0.1:44327", Names::Address, &[]),
    (b"http://127.0.0.1:7100/a/link?secret=x\n", Names::Url, &[("http://127.0.0.1:7100/a/link?secret=x", "127.0.0.1:7100")]),
    (b"http://127.0.0.1:7100/a?s=1\nhttp://127.0.0.1:7100/a?s=2\n", Names::Url,
        &[("http://127.0.0.1:7100/a?s=1", "127.0.0.1:7100"), ("http://127.0.0.1:7100/a?s=2", "127.0.0.1:7100")]),
];
#[test]
fn a_printed_line_names_the_url_to_open_and_its_address() {
    for (output, names, expected) in PRINTED {
        let prefix: &[u8] = if *names == Names::Url { b"http://" } else { b"server at " };
        let found = crate::harness::printed(output, &Printed { prefix, on: Output::Stderr, names: *names });
        let found: Vec<_> = found.iter().map(|o| (o.url.as_str(), o.address.as_str())).collect();
        assert_eq!(found, *expected, "{}", String::from_utf8_lossy(output));
    }
}
```
Tests, each under 20 ms: `a_printed_line_names_the_url_to_open_and_its_address`,
`a_restart_replaces_each_address_placeholder`,
`reopen_goes_to_the_url_the_person_first_opened`,
`a_second_person_opens_a_line_the_first_did_not`,
`stdout_is_piped_only_when_the_line_comes_on_it`.
Proof: the browser suite, which is the gate here (skein's gate runs
`--profile browser`): the UI example's tier in the address form,
unchanged, `the_harness_reads_the_listening_line`
(`tests/ui-web/tests/browser_harness.rs`) and
`a_reload_keeps_a_pending_add_made_once` (`browser_example.rs:21`, which
stops the server and starts it again) passing with `Setup.restart`. The URL form
and the stdout pipe meet a real server first in jig 19.4's first browser
story (jig's tier), which connects them; nothing in skein prints a URL.
A failing browser test is run alone as the README's "One test of the
browser suite" says, under that section's rule.
Don't: change the UI example's server or its output; keep
`listening_address` beside `printed`; read the URL's path or query.
Size: at most 100 lines.
Removes: `Setup.listening`, `listening_address`, the restart's
first-argument rule.

### 01.2 Other origins: fixed pages served by the harness, and a step opening them

Scope: `Setup.origins`, fixed pages the harness serves on listeners of
its own in its loop, `{address}` in a page replaced by the server's
address; `Step::Visit { origin, path }`, a navigation to the absolute
URL `http://<the origin's host name>:<its port><path>` (`ui-web.md`,
14.1: "fixed pages that a small server on skein's HTTP serves in the
test's loop").
Needs: 01.1.
Files: `tests/browser/src/serve.rs` moved to
`testing/skein-fake-person-browser/src/origins.rs`, its pages made data
(`Page { path, kind, body, headers }`);
`tests/browser/tests/support/real.rs` (`#[path = "../../src/serve.rs"]
mod serve` at `:8`, `serve::Pages` at `:176`, `:221`, `:348`) and
`tests/browser/tests/page_server.rs` (`#[path]` at `:3`, `serve::` at
`:13` to `:57`), the two users (`logs/w8/skein-pages-users.txt`), each
then passing its pages to the kit's server, the constants `BUTTON` to
`PRESS` moved into a new `tests/browser/src/pages.rs`;
`tests/browser/Cargo.toml` (takes `skein-fake-person-browser`); `testing/skein-fake-person-browser/src/{harness.rs,
harness/script.rs, face.rs (a go to an absolute URL), lib.rs}`; the
harness's io limits (`harness.rs:138-139`: `accepts` and `backlog` from
1 to 4 each, for the origins' connections).
Shape: `pub struct Origin { pub host: Box<[u8]>, pub pages: Vec<Page> }`;
`pub struct Page { pub path: Box<[u8]>, pub kind: Box<[u8]>, pub body:
Box<[u8]>, pub headers: Vec<Header> }`; `Setup.origins:
Vec<Origin>`; `Step::Visit { origin: u32, path: Box<[u8]> }`;
`origins::Pages::new(pages: Vec<Page>, now: Time, wall: Wall)` (the moved
server), its io owners from `ORIGINS_FIRST`, a constant the harness's
other tokens never reach. An unknown path answers `404`.
Test shape: browser tests in `tests/ui-web/tests/browser_harness.rs` on
`fixture_bundle` and a `fixture_harness` that takes origins
(`tests/ui-web/src/lib.rs:46`, `:62`); the kit's own page tests in
`tests/browser/tests/page_server.rs`, their names and expected values
kept. The reference test:
```rust
#[test]
fn a_page_of_another_origin_is_opened_by_its_step() {
    let scratch = Scratch::new("harness-origin");
    let bundle = fixture_bundle(&scratch, READY);
    let origin = Origin { host: Box::from(b"localhost".as_slice()), pages: vec![page(b"/", b"<!doctype html><h1>Elsewhere</h1>")] };
    let mut person = Script::new(Box::new([
        harness::Step::Visit { origin: 0, path: Box::from(b"/".as_slice()) },
        harness::Step::Person(Step::See { find: Find::exactly_named(Role::Heading { level: 1 }, b"Elsewhere"), within: SECOND }),
    ]));
    assert_eq!(fixture_harness_with(&bundle, vec![origin]).run(&mut [&mut person]), Outcome::Done);
}
```
Tests: `a_page_of_another_origin_is_opened_by_its_step` (browser, about
1 s), `an_origin_page_names_the_server_s_address`,
`an_unknown_origin_path_is_404` (unit, under 20 ms, in
`testing/skein-fake-person-browser/src/tests.rs` on `origins::Pages`
driven by hand, after `tests/browser/tests/page_server.rs`).
Proof: the browser suite (the gate): `a_page_of_another_origin_is_opened_by_its_step`
shows a person's page at an origin the harness serves; the kit's
`browser_facts` and `browser_real` tests pass on the moved page server.
Don't: serve the application's pages; give an origin a policy of its
own unless its `Page.headers` say so; keep a second page server in
`tests/browser`.
Size: at most 300 lines, of which about 230 are `serve.rs` moved from
`tests/` (outside the counted paths, so the size command counts them as
added).
Removes: `tests/browser/src/serve.rs`.

### 01.3 Awaited trouble, and a request with a given `Host`

Scope: `Step::Trouble { words, within }`, its words expected on its
person's pages from the script's start and awaited by the step;
`Step::Request { method, path, host, headers, body, status, within }`,
sent with skein's HTTP client over the harness's io to the server's
address, the step met when the answer's status equals `status`
(`ui-web.md`, 14.2: "fails a test on any trouble the kit reports that the
story did not expect").
Needs: 01.2.
Files: `testing/skein-fake-person-browser/src/{harness.rs,
harness/script.rs, lib.rs}`, new `testing/skein-fake-person-browser/src/harness/request.rs`;
`testing/skein-fake-person-browser/Cargo.toml` (takes `skein-http`);
`tests/ui-web/tests/browser_harness.rs`.
Shape: `Step::Trouble { words: Box<[u8]>, within: Duration }`;
`Step::Request { method: skein_http::Method, path: Box<[u8]>, host:
Box<[u8]>, headers: Vec<Header>, body: Box<[u8]>,
status: u16, within: Duration }`; `Play::expected(&self) -> Vec<Box<[u8]>>`
(the script's `Trouble` words, default none), which the harness reads
for each person at the start; `Play::request(&mut self) -> Option<Asked>`
and `Play::answered(&mut self, status: u16)`, after `server_action`; a
trouble report reaches the person's script through
`BrowserFace::trouble(&self) -> &[Box<[u8]>]` (the troubles seen on its
page, in order). A step not met within its `within` fails the story,
naming the words or the status seen.
Test shape: unit tests in `testing/skein-fake-person-browser/src/harness/script.rs`'s
`mod tests` on `crate::tests::face()`; browser tests in
`tests/ui-web/tests/browser_harness.rs`. The reference browser test:
```rust
#[test]
fn a_framed_page_s_refusal_is_awaited_as_trouble() {
    let scratch = Scratch::new("harness-framed");
    let bundle = fixture_bundle(&scratch, READY);
    let framing = page(b"/", b"<!doctype html><iframe src=\"http://{address}/\"></iframe>");
    let origin = Origin { host: Box::from(b"localhost".as_slice()), pages: vec![framing] };
    let mut person = Script::new(Box::new([
        harness::Step::Visit { origin: 0, path: Box::from(b"/".as_slice()) },
        harness::Step::Trouble { words: Box::from(b"frame-ancestors".as_slice()), within: SECOND },
    ]));
    assert_eq!(fixture_harness_with(&bundle, vec![origin]).run(&mut [&mut person]), Outcome::Done);
}
```
(the UI example's server sends `frame-ancestors 'none'` on each answer,
`examples/ui/server/src/routes.rs:10`).
Tests: `a_framed_page_s_refusal_is_awaited_as_trouble`,
`a_request_with_another_host_has_its_status_awaited` (`GET /`, `Host:
other.test`, status `200`, which the UI example's server answers to any
`Host`), both browser tests of about 1 s;
`trouble_named_by_a_later_step_does_not_fail_the_run`,
`a_trouble_step_not_met_within_its_bound_fails_naming_its_words`,
`a_request_step_with_another_status_fails_naming_it` (unit, under 20 ms).
Proof: the browser suite (the gate): `a_framed_page_s_refusal_is_awaited_as_trouble`
shows a policy violation of the server's page, caused by a page of
another origin, awaited and not failing the story; and
`the_harness_fails_a_page_whose_module_throws` still fails a story on
trouble it did not name.
Don't: drop `Setup.expected` (the UI example's tier uses it); let a
`Trouble` step's words cover another person's pages; follow a redirect
or read the request's answer body beyond its status.
Size: at most 150 lines.
Removes: nothing.

The last increment's step is `skein-01.3`: the plan's README's lookup
of a step's landing (Rules, Pins) finds this session's last landing.

## One test of skein's browser suite

The README's "One test of the browser suite" holds skein's commands and
the rule for running them; this file keeps no copy. The run's machine
has the `wasm32-unknown-unknown` target (README, "Prerequisites"); the
commands the planner could not run at the base are listed in the README
for the supervisor's ready check. At the base, `cargo nextest list
-p skein-ui-web-tests -p skein-browser-world --profile browser` listed
the browser tests this file names (`the_harness_reads_the_listening_line`,
`a_reload_keeps_a_pending_add_made_once`,
`the_harness_fails_a_page_whose_module_throws`, and the 16 of
`browser_facts` and `browser_real`), and
`the_listening_line_gives_the_address` ran and passed.

## Interface sketches

Compiled at the base in a scratch module of `skein-fake-person-browser`
(the crate taking `skein-http`, as 01.3 does) with `cargo clippy -p
skein-fake-person-browser --all-targets -- -D warnings`, which passed
(`logs/w8/sketch-skein-01.txt`); bodies return a trivial value. `Header`
is `skein_ui_web::transport::Header`, `(Box<[u8]>, Box<[u8]>)`.

```rust
// testing/skein-fake-person-browser/src/harness.rs
/// The line the server prints that says where its pages are (ui-web.md, 14.2).
pub struct Printed<'a> {
    /// The line's first bytes, by which the harness finds it.
    pub prefix: &'a [u8],
    /// The output the server prints it on.
    pub on: Output,
    /// What the line names.
    pub names: Names,
}
/// One of the server's outputs.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Output {
    Stdout,
    Stderr,
}
/// What a printed line names.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Names {
    /// The rest of the line is `host:port`, opened at `/`.
    Address,
    /// The line, its prefix included, is an absolute URL, opened as printed; one person each.
    Url,
}
/// A line found: the URL a person opens, and the `host:port` the face's `go` and a restart use.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Opened {
    pub url: String,
    pub address: String,
}
/// A fixed page the harness serves at another origin (ui-web.md, 14.1).
pub struct Page {
    pub path: Box<[u8]>,
    /// Its `Content-Type`.
    pub kind: Box<[u8]>,
    /// Its bytes, each `{address}` replaced by the server's address when served.
    pub body: Box<[u8]>,
    /// More fields its answer carries.
    pub headers: Vec<Header>,
}
/// Another origin: a listener of the harness's own on 127.0.0.1, opened at `host`.
pub struct Origin {
    /// The host name a `Visit` opens it at: `127.0.0.1` for another port of the same host, `localhost` for another site.
    pub host: Box<[u8]>,
    pub pages: Vec<Page>,
}
pub struct Setup<'a> {
    pub server: PathBuf,
    /// The first start's arguments.
    pub args: Vec<OsString>,
    /// A restart's arguments, each `{address}` replaced by the address the server had.
    pub restart: Vec<OsString>,
    pub printed: Printed<'a>,
    pub chromium: PathBuf,
    /// Trouble tolerated on any page for the run.
    pub expected: Vec<Box<[u8]>>,
    pub origins: Vec<Origin>,
}
/// Each line of `output` that starts with the prefix, in order, as what it names.
#[must_use]
pub fn printed(output: &[u8], printed: &Printed<'_>) -> Vec<Opened>;
/// A request the story's step asks the harness to send.
pub struct Asked {
    pub method: skein_http::Method,
    pub path: Box<[u8]>,
    pub host: Box<[u8]>,
    pub headers: Vec<Header>,
    pub body: Box<[u8]>,
}
pub trait Play {
    fn next(&mut self, face: &mut BrowserFace<'_>, now: Time) -> Next<Box<[u8]>>;
    fn at(&self) -> u32 { 0 }
    fn check_deadline(&mut self, _face: &mut BrowserFace<'_>, _now: Time) {}
    fn server_action(&mut self) -> Option<ServerAction> { None }
    /// The words of trouble this person's pages may report: its `Trouble` steps'.
    fn expected(&self) -> Vec<Box<[u8]>> { Vec::new() }
    /// A request to send now, answered by `answered`.
    fn request(&mut self) -> Option<Asked> { None }
    /// The status of the request `request` gave, or 0 when it failed.
    fn answered(&mut self, _status: u16) {}
}
// testing/skein-fake-person-browser/src/harness/script.rs, beside the base's steps
pub enum Step {
    // ...the base's steps
    /// Opens again the URL this person's page first opened (01.1).
    Reopen,
    /// Opens `path` at the origin `Setup.origins[origin]`.
    Visit { origin: u32, path: Box<[u8]> },
    /// Waits until this person's page reported trouble holding `words`.
    Trouble { words: Box<[u8]>, within: Duration },
    /// Sends a request with its own `Host` to the server; met when the answer's status is `status`.
    Request { method: skein_http::Method, path: Box<[u8]>, host: Box<[u8]>, headers: Vec<Header>, body: Box<[u8]>, status: u16, within: Duration },
}
```

## Questions to raise when reached

- None the design leaves open for these increments. The two additions
  `ui-web.md`, 14.2 does not name (other origins opened by a step, and a
  request with a given `Host`) are reported as a design gap, with the
  proposed line, and built as generic harness steps meanwhile.

## Done when

Each command runs once, after the session's last increment lands.

- `git -C ~/src/rust/skein log origin/main --grep='^Plan: client-api$'
  --format='%(trailers:key=Step,valueonly,separator=)' | grep -c
  '^skein-01\.[0-9]'` prints 3 (the supplement step skein-01.s1, the
  terminal's room, is README's "Done when", 10).
- `git -C ~/src/rust/skein grep -n -E 'listening_address|listening:'
  origin/main -- testing tests` prints nothing.
- `git -C ~/src/rust/skein grep -n -i -w jig origin/main -- crates
  testing tests examples` prints nothing.
