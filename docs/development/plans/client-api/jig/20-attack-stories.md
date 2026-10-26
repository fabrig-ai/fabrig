# Session 20: jig's stories for a browser shell, and its attack stories

jig's test kit gains `jig-client-fake-person-browser`: functions on
skein's face and harness that an application's browser tier calls with
its binary, its bundle and what its pages show, to check its wiring of
`jig-client`, the client's adapter and the printed link (a reload keeps
a pending request; the engine restarted shows offline, then live; a
spent link signs the person out), and jig's attack stories, which an
application may run against its own server (a cross-site request from
another port and from another host name, a rebound `Host`, a framed
page). jig's browser tier calls each of them against `jig-test`, and
skein's stories for a browser shell that one person plays
(`protocol/browser.md`, 15 and 19; `client/domain/testing.md`, 2).

## Read first

- jig's `docs/design/protocol/browser.md`: 11, 12, 12.1, 12.2, 15, 19.
- jig's `docs/design/client/domain/testing.md`: 2, 9.
- skein's `docs/design/ui-web.md`: 14.2 (the stories for a browser shell
  and what an application names for each).
- skein 01's session file (the harness's `Origin`, `Page`, `Step::Visit`,
  `Step::Trouble`, `Step::Request`, `Step::Reopen`, `Printed`) and jig
  19's (the tier's `Tier`, its `Setup`, the test shell's dev paths).
- jig 17's session file (the guards' statuses: `421`, `403`, `415`, no
  `OPTIONS`).
- skein's `testing-strategy.md`: 2.9, 7, 8 (by number only).

## What the design asks

- **jig's stories for a browser shell** (`browser.md`, 15): "functions an
  application's tier calls with its binary, bundle and codec, to check
  its wiring of `jig-client`, the client's adapter and the printed link,
  since its loop is its own. Each signs its person in by the printed
  link: a reload keeps a pending request, which the engine makes once;
  the engine killed and restarted shows offline, then live, where its
  presentation shows the link's state; a spent link is refused, and the
  person is signed out."
- **jig's attack stories** (15): "functions an application's tier may
  call with its binary, its page's address and its mapping's prefix,
  against its own server ... Another origin's page is served by jig's
  kit, in the story's own loop ...; skein's harness opens it as an
  address the test gives, and the story names the policy violation it
  causes as trouble it expects":
  - "*a cross-site request:* such a page, on another port of the same
    host and on another host name, makes the signed-in person's browser
    send a request under the prefix; it is refused, and nothing is done,
    which the person's page then shows;
  - *a rebound Host:* a request naming a host off the list, as a rebound
    name would, sent by skein's HTTP client; answered `421`, with nothing
    done;
  - *a framed page:* such a page frames the application's page; the
    browser shows nothing of it, under the page's policy."
- **jig's tier** (15): "calls skein's stories, jig's and jig's attack
  stories". "Checked, never mandated" (15).

## Choices this plan makes

- **The kit takes what the application names, as skein's stories do**
  (skein's `ui-web.md`, 14.2: "What the application names is what a story
  takes, and nothing else"): `States { live, offline, signed_out }`, finds
  of what its page shows; for the attack stories, `Target { prefix,
  page, request, done }`: its mapping's prefix, a path of its page, a
  keyed request's record its client would send, and what its page shows
  once that request is made.
- **The forging pages** are built by the kit: one `fetch` with the
  browser's header and the mapping's content type and `credentials:
  "include"` (a preflight the mapping never answers, which Chromium
  reports as a CORS refusal on the page's console), and one simple
  `fetch` with `mode: "no-cors"`, `Content-Type: text/plain` and the
  cookie (no preflight: it reaches the server, which refuses it, `403`
  from the same site, `415` from another), each naming the server's
  origin by `{address}`. The trouble the stories await is Chromium's
  report of the preflighted one, the words `blocked by CORS policy`; the framed
  story's is `frame-ancestors`.
- **"Nothing is done"** is shown on the person's page: after the forged
  requests, the person goes back to the application's page, sees `live`,
  and `done` is gone (`Step::Gone` within 1 s): a page that shows it holds
  its server's state, then the absence (skein's `ui-web.md`, 14.2: "A
  story looks for something absent only once its page shows that it
  holds its server's state").
- **"Which the engine makes once"** is the client's request table's and
  the engine's, which the protocol world counts
  (`a_reload_restores_a_pending_request_from_the_small_store_s_bytes`);
  the browser story shows the pending request kept across the reload and
  made, and names that test for the once.
- **The restarted stories' address:** the harness restarts `jig-test`
  with `Setup.restart`; `jig-test` names its listener in its
  configuration, not its arguments, so jig's tier writes the
  configuration of each story that stops the server (20.2's two) with a
  port it reserved (a `std::net::TcpListener` bound to `127.0.0.1:0`,
  read, and dropped before the server starts), and gives `restart` the
  same arguments. The other stories keep port `0`.
- **"Killed" is the harness's `StopServer`:** it sends the server the
  terminate signal (skein's harness, `harness.rs:401-409` at skein's
  base), on which `jig-test` drains and ends (`service.md`, 5); the page
  sees its server gone either way, which is what the story checks. A
  kill with no drain is the process tier's (`service.md`, 8), not this
  tier's.
- **skein's stories in jig's tier** are the two one person plays: back
  and forward, and a panic (the test shell's dev build panics at start
  at `/test/panic`, as skein's example's does). The two that need a
  second person signed in are `later.md`'s (Questions, below).

## Rules that bind it

- **A browser story's Proof is the gate:** the author runs no
  gate of its own for it, and runs a failing story alone as the README's
  "One test of the browser suite" says, under that section's rule.
- **The browser suite's budget, 60 s, is a rule** (`browser.md`, 15; the
  `browser` profile's `global-timeout`): a browser suite past 60 s fails
  the gate, so the increment that takes it past makes its stories
  cheaper before landing, never the timeout raised; 4 s per story is an
  aim nothing measures; what counts is the browser suite's `Summary`
  seconds in the landing's `Gate-Result`. After 20.5 the suite holds 13
  stories (jig 19's 3, this session's 10), about 45 s at 3.5 s each,
  near the rule: each landing's report gives that number.
- **jig never names the repository downstream of it:** the check of
  code-organization.md, 6, quoted: "`git -C
  <the session's jig worktree> grep -n -i -w temper -- crates testing tests
  examples scripts` prints nothing at each jig increment's end".

## Corrections after the ready check (ruling 332)

The ready check (skein 867095d14, jig 899bd8e68, temper 5cbbc1c0a) found
these; they replace what differs below. Code is named by path and
symbol.
- **20.1, "Killed":** the terminate signal of `Step::StopServer` is
  `ServerAction::Stop` in skein's
  `testing/skein-fake-person-browser/src/harness.rs`, the step in
  `harness/script.rs`; not the child-exited handler.
- **20.1, `signed_out`:** jig's sign-in page says "Sign in"
  (`JigPhrase::SignInPage` in `jig-client-view/src/phrases/english.rs`,
  shown by `pages/not_the_work.rs`), and no words to open the printed
  link. `States::signed_out` is a `Find` of jig's `SignInPage` words;
  its doc comment says so.
- **20.2, the expected errors:** skein's
  `tests/ui-web/tests/browser_example.rs` gives
  `net::ERR_CONNECTION_REFUSED` and
  `net::ERR_INCOMPLETE_CHUNKED_ENCODING` in its restart tests' own
  `expected` lists, not in the `printed`/`restart` part of its Setup.
- **20.3, `expected`:** skein's `stories.rs` replaces `expected` with a
  story's own list; it adds nothing to the application's. jig's stories
  build `expected` as the application's list plus `Failed to load
  resource`.
- **20.3, the forging page:** the second `fetch`, preflighted with
  `Jig-Client`, is refused, and Chromium reports "Uncaught (in
  promise)", which no `expected` holds. Both `fetch` calls end in
  `.catch(() => {})`. With it, the page passed in real Chromium, same
  site and other site; as written, it failed both.
- **20.5:** `stories::back_and_forward` takes two `Shown { address,
  sees }`, not paths: the inbox's (`/inbox`, its title) and a task's
  (`/tasks/<n>`, its title), the task made before the story starts.
  `stories::panic(setup, address, failed)` takes `/test/panic`'s address
  too.
- **Interface sketches:** each of the eight `pub fn` takes
  `#[must_use]`, or clippy's `must_use_candidate` refuses it.
- **Held:** every landed name the file gives; the guard order, header
  values, `/jig` prefix and `SameSite=Strict` cookie of
  `jig-protocol-engine-clients-http`; the 13 browser tests of "Done
  when"; jig names temper nowhere. The reload order of 20.2 passed in
  real Chromium. Not checked: the wire's, inbox rows' and printed
  link's formats, 20.5's "application failed" words, and 20.1's
  `Step::Reopen` against skein's `Step`; an author who finds one
  off names it in its report.

## Increments

### 20.1 The kit, and a spent link signs the person out

Scope: `testing/jig-client-fake-person-browser`, ordinary Rust on skein's
face and harness, with its first story: the person opens the printed
link and sees `live`, opens the same URL again (`Step::Reopen`), and is
signed out, which `signed_out` shows (`browser.md`, 15; `client.md`, 8);
jig's tier calls it.
Needs: nothing beyond the session's start (jig 19's tier,
`jig-client-browser-world` with `Tier`, and skein 01's harness steps,
on jig's pin since 19.4).
Files: new `testing/jig-client-fake-person-browser/{Cargo.toml,
clippy.toml}`, `src/{lib.rs, stories.rs}`; workspace `Cargo.toml` (the
member); `tests/client-browser/{Cargo.toml, src/lib.rs (Tier::states())}`,
new `tests/client-browser/tests/browser_jig_stories.rs`. Shared:
workspace `Cargo.toml`, `Cargo.lock`.
Shape (sketch below): `States { live: Find, offline: Find, signed_out:
Find }`; `stories::a_spent_link_signs_the_person_out(setup: &Setup<'_>,
states: &States) -> Outcome`. In jig's tier, `Tier::states()` gives the
reference UI's: `live` the banner's live state, `offline` its offline
state, `signed_out` the sign-in page's words to open the printed link,
each as jig's English phrasebook words them (read from `jig-client-view`'s
phrases).
Test shape: `tests/client-browser/tests/browser_jig_stories.rs`, on
`Tier`. The reference test:
```rust
#[test]
fn jig_s_spent_link_story_holds_for_jig_test() {
    let tier = Tier::new("spent-link");
    assert_eq!(stories::a_spent_link_signs_the_person_out(&tier.setup(), &tier.states()), Outcome::Done);
}
```
Tests: `jig_s_spent_link_story_holds_for_jig_test` (about 3 s); unit
tests of the kit's script, each under 20 ms, in
`testing/jig-client-fake-person-browser/src/tests.rs`:
`the_spent_link_story_reopens_after_seeing_live`.
Proof: the gate runs the story: the reference UI shows its sign-in
page after the spent secret is refused.
Don't: give the kit a type of `jig-test`'s or of the test view's (it is
an application's tier's kit); open a URL the story makes up.
Size: at most 120 lines.
Removes: nothing.

### 20.2 A reload keeps a pending request; a restart shows offline, then live

Scope: two of jig's stories for a browser shell (`browser.md`, 15), and
the closing warning (`browser.md`, 10: "Closing a tab with requests
pending asks the platform to warn (`beforeunload`) ... only while the
client holds requests not yet answered durable"): the server stopped,
the person makes a request (the application's script), sees it pending,
reloads and meets the closing warning, the server started, and the
request shows made; with nothing pending, a reload meets no warning; the
server killed shows `offline`, started again shows `live`. A reload
is a new client, whose page sends the old page's stream gone before it
opens its own (`protocol/client.md`, 4; 19.5's designer's answer); the
stories assert nothing of it, and its gone failing while the server is
stopped is the `net::ERR_CONNECTION_REFUSED` that `Setup.expected`
names below.
Needs: 20.1.
Files: `testing/jig-client-fake-person-browser/src/stories.rs`;
`tests/client-browser/src/lib.rs` (`Tier::reserved(name)`: a
configuration on a port it reserved, and `Tier::request_script()`, the
reference UI's acts that make a request, a task made from the inbox, in
jig's words); `tests/client-browser/tests/browser_jig_stories.rs`;
`testing/jig-client-test-domain/src/domain.rs` (jig-test's navigation
offers the Conversations page, ruling 386), a test in its
`src/tests/root/`, and each test or golden of jig-test's frame the
new link changes.
Shape: `stories::a_reload_keeps_a_pending_request(setup, states, request:
&mut dyn Play, pending: Find, made: Find) -> Outcome`;
`stories::a_reload_with_nothing_pending_meets_no_warning(setup, states,
request: &mut dyn Play, made: Find) -> Outcome`;
`stories::a_restart_shows_offline_then_live(setup, states) -> Outcome`;
`pending` is what the application's page shows of a request kept and
not yet answered (the reference UI's card of a request in flight, in
jig's words). Each is a `Script` of skein's steps, with no fixed wait:
for the first, `See(live)`, `StopServer`, the application's script,
`See(pending)`, `Person(Reload)`, `Warned(true)`, `StartServer`,
`See(made)` within 5 s; for the second, `See(live)`, the application's
script, `See(made)`, `Person(Reload)`, `Warned(false)` (the person acted
on the page first, as skein's `ui-web.md`, 14.2 asks of a story that
looks for no warning); for the third, `See(live)`, `StopServer`,
`See(offline)`, `StartServer`, `See(live)`. The first and third name in
`Setup.expected` the connection failures a
stopped server causes (`net::ERR_CONNECTION_REFUSED`,
`net::ERR_INCOMPLETE_CHUNKED_ENCODING`, as skein's example's restart
tests do, `tests/ui-web/tests/browser_example.rs:41-45` of skein).
Test shape: three stories in `browser_jig_stories.rs`, each about 4 s:
`jig_s_pending_request_story_holds_for_jig_test`,
`jig_s_no_warning_story_holds_for_jig_test`,
`jig_s_restart_story_holds_for_jig_test`.
Proof: the gate runs the three; the once of the made request is the
protocol world's `a_reload_restores_a_pending_request_from_the_small_store_s_bytes`,
which this does not run again.
Don't: wait a fixed time for the server (the steps await what the page
shows); add an argument to `jig-test`.
Size: at most 120 lines.
Removes: nothing.

### 20.3 A cross-site request, from another port and from another host name

Scope: two attack stories (`browser.md`, 15, 12.2): the signed-in person
opens a page of another origin, served by the harness (`Setup.origins`),
on `127.0.0.1` at another port (same site) or at `localhost` (another
site), whose script sends the forged requests of "Choices"; the story
awaits Chromium's report (`Step::Trouble`, `blocked by CORS policy`),
goes back to the application's page, sees `live`, and `done` is gone.
Needs: 20.1.
Files: `testing/jig-client-fake-person-browser/src/{attacks.rs (new),
pages.rs (new), lib.rs}`; `tests/client-browser/src/lib.rs`
(`Tier::target()`: prefix `/jig`, page `/`, the request a keyed request
that makes a task named `Forged` in jig's wire, and `done` the inbox's
row named `Forged`); new `tests/client-browser/tests/browser_attacks.rs`.
Shape: `Target<'a> { prefix: &'a [u8], page: &'a [u8], request: &'a [u8],
done: Find }`; `attacks::a_cross_site_request_from_another_port_does_nothing(setup,
states, target) -> Outcome`, `attacks::a_cross_site_request_from_another_host_does_nothing(setup,
states, target) -> Outcome`; `pages::forging(target) -> Page`, the page's
HTML with the request's bytes in base64 and `{address}` for the server.
Each story builds its harness from the application's `Setup` with
`Failed to load resource` added to `expected` (Chromium's console reports
of the refused requests' statuses, `403` or `415`, and of the blocked
one), as skein's stories add their own (`stories.rs:9-17` of skein).
Test shape: a unit test of the forging page in
`testing/jig-client-fake-person-browser/src/tests.rs`, under 20 ms,
`the_forging_page_posts_to_the_mapping_s_request_route`, its expected
page a literal; and in `browser_attacks.rs`, the reference story:
```rust
#[test]
fn a_page_on_another_port_cannot_make_a_request_as_the_person() {
    let tier = Tier::new("cross-port");
    let target = tier.target();
    assert_eq!(attacks::a_cross_site_request_from_another_port_does_nothing(&tier.setup(), &tier.states(), &target), Outcome::Done);
}
```
Tests: `a_page_on_another_port_cannot_make_a_request_as_the_person`,
`a_page_of_another_site_cannot_make_a_request_as_the_person`, each about
4 s; `the_forging_page_posts_to_the_mapping_s_request_route` (unit).
Proof: the gate runs both stories: from the same site and from
another, the person's browser sends the forged requests, Chromium reports
the refused one, and the person's inbox shows no task named `Forged`.
Don't: give the forging page the application's policy (it is another
site's page); send the forged request from the harness instead of the
browser; expect trouble on the application's own pages.
Size: at most 150 lines.
Removes: nothing.

### 20.4 A rebound `Host`, and a framed page

Scope: the last two attack stories (`browser.md`, 15, 12.1, 12.2): a
request naming a host off the list, sent by skein's HTTP client through
the harness (`Step::Request`), answered `421`, and the person's page then
shows nothing done; a page of another origin frames the application's
page, Chromium reports `frame-ancestors` (`Step::Trouble`), and the
framing page shows nothing of the application (`Step::Gone` of `live`).
Needs: 20.3.
Files: `testing/jig-client-fake-person-browser/src/{attacks.rs,
pages.rs}`; `tests/client-browser/tests/browser_attacks.rs`.
Shape: `attacks::a_rebound_host_is_answered_421_and_does_nothing(setup,
states, target) -> Outcome` (the request: `POST <prefix>/v1/request`,
`Host: rebound.test`, the mapping's content type and the browser
header, `target.request` as its body, status `421`);
`attacks::a_framed_page_shows_nothing(setup, states, target) -> Outcome`;
`pages::framing(target) -> Page` (`<iframe src="http://{address}<page>">`).
Test shape: two stories in `browser_attacks.rs`, each about 4 s:
`a_rebound_host_is_answered_421_and_makes_nothing`,
`a_framed_page_shows_nothing_of_the_application`; a unit test,
`the_framing_page_frames_the_application_s_page`, under 20 ms.
Proof: the gate runs both: the `421` is awaited and the inbox shows
no `Forged`; the framed page's refusal is reported and nothing of the
application shows in it.
Don't: send the rebound request from the browser (it cannot set `Host`);
await a fixed time.
Size: at most 100 lines.
Removes: nothing.

### 20.5 skein's stories for a browser shell, in jig's tier

Scope: jig's tier calls skein's stories that one person plays (skein's
`ui-web.md`, 14.2, "skein's stories for a browser shell"; `browser.md`,
15: "jig's tier calls them with jig's binary, bundle and codec"): back
and forward between the inbox and a task's page, and a panic at start,
which the test shell's dev build makes at `/test/panic`.
Needs: 20.1.
Files: `testing/jig-client-test-shell-web/src/lib.rs` (the dev build's
`/test/panic`, `#[cfg(debug_assertions)]`, as skein's example's);
new `tests/client-browser/tests/browser_skein_stories.rs`.
Shape: `skein_fake_person_browser::stories::back_and_forward` and
`::panic`, called as skein's `tests/ui-web/tests/browser_stories.rs`
calls them, with `Tier::setup()`, the codec's paths (`/inbox`, a task's
`/tasks/<n>`), and jig's words for "the application failed" (the restart
part's, `browser.md`, 5).
Test shape: two stories in `browser_skein_stories.rs`, each about 4 s:
`back_and_forward_move_between_the_inbox_and_a_task`,
`a_panic_at_start_reloads_then_says_jig_failed`.
Proof: the gate runs both.
Don't: keep `/test/panic` in a `web` build; call skein's two-person
stories (`later.md`).
Size: at most 30 lines. Ruling 393: the back-and-forward story waits
(`later.md`): skein's `back_and_forward` goes to its first address at
once, before jig's sign-in by the printed link is ready (401), and
jig's skein moves past its cap (ruling 296) only in temper's `tools`.
20.5 lands the panic story and `/test/panic` alone.
Removes: nothing.

## Interface sketches

Compiled at the base, `cargo clippy -p jig-sketch-w8 -p
jig-sketch-w8-web --all-targets -- -D warnings`, which passed
(`logs/w8/sketch-19-20.txt`; again after the review, `logs/w8/sketch-review.txt`), with `Origin` and `Page` stubbed after skein
01's sketch (they land in skein 01) and trivial bodies.

```rust
// testing/jig-client-fake-person-browser/src/lib.rs
/// What an application's tier names of its pages for jig's stories.
pub struct States {
    /// What its page shows once it holds the engine's state and its stream is live.
    pub live: Find,
    /// What it shows while the engine is out of reach.
    pub offline: Find,
    /// What it shows signed out: its sign-in page, saying to open the printed link.
    pub signed_out: Find,
}
/// What an attack story needs of an application: where its mapping is, and what a forged request would do.
pub struct Target<'a> {
    /// The prefix its server mounts jig's mapping at: `/jig` in jig's test application.
    pub prefix: &'a [u8],
    /// A path of its page, opened and framed: `/` in jig's.
    pub page: &'a [u8],
    /// A keyed request's record, as its client would send it under the prefix.
    pub request: &'a [u8],
    /// What its page shows once that request is made, which a refused one never shows.
    pub done: Find,
}
// testing/jig-client-fake-person-browser/src/stories.rs
/// A reload keeps a pending request, meeting the closing warning, and the engine makes it once (browser.md, 15, 10).
pub fn a_reload_keeps_a_pending_request(setup: &Setup<'_>, states: &States, request: &mut dyn Play, pending: Find, made: Find) -> Outcome;
/// With nothing pending, a reload of a page acted on meets no closing warning (browser.md, 10).
pub fn a_reload_with_nothing_pending_meets_no_warning(setup: &Setup<'_>, states: &States, request: &mut dyn Play, made: Find) -> Outcome;
/// The engine killed and started again shows offline, then live.
pub fn a_restart_shows_offline_then_live(setup: &Setup<'_>, states: &States) -> Outcome;
/// A spent link is refused, and the person is signed out.
pub fn a_spent_link_signs_the_person_out(setup: &Setup<'_>, states: &States) -> Outcome;
// testing/jig-client-fake-person-browser/src/attacks.rs
/// A page on another port of the same host makes the person's browser send `target.request`; refused, nothing done.
pub fn a_cross_site_request_from_another_port_does_nothing(setup: &Setup<'_>, states: &States, target: &Target<'_>) -> Outcome;
/// The same, from a page of another host name, another site.
pub fn a_cross_site_request_from_another_host_does_nothing(setup: &Setup<'_>, states: &States, target: &Target<'_>) -> Outcome;
/// A request naming a host off the list is answered `421`, and nothing is done.
pub fn a_rebound_host_is_answered_421_and_does_nothing(setup: &Setup<'_>, states: &States, target: &Target<'_>) -> Outcome;
/// A page of another origin frames the application's page; the browser shows nothing of it.
pub fn a_framed_page_shows_nothing(setup: &Setup<'_>, states: &States, target: &Target<'_>) -> Outcome;
// testing/jig-client-fake-person-browser/src/pages.rs
/// The other origin's page of a cross-site story: the two forged fetches of "Choices", `{address}` the server's.
pub fn forging(target: &Target<'_>) -> Page;
/// The other origin's page of the framed story.
pub fn framing(target: &Target<'_>) -> Page;
```

The forging page, whose order matters (the refused preflight is the
trouble awaited; the simple request is the one that reaches the guards):

```html
<!doctype html><title>Elsewhere</title><script>
const body = Uint8Array.from(atob("<the request, base64>"), c => c.charCodeAt(0));
fetch("http://{address}<prefix>/v1/request", {method: "POST", mode: "no-cors", credentials: "include", headers: {"Content-Type": "text/plain"}, body});
fetch("http://{address}<prefix>/v1/request", {method: "POST", credentials: "include", headers: {"Content-Type": "application/octet-stream", "Jig-Client": "browser"}, body});
</script>
```

## Questions to raise when reached

- **20.5: skein's two-person stories** (typing while a patch arrives; a
  write refused) are not called in this plan (decided by the planner,
  2026-10-15; `later.md`): a second person signs in only after the
  printed link was spent and refused, which draws a fresh one
  (`protocol/client.md`, 8), and skein's harness hands a person no link
  printed after its start. 20.5 calls the two one-person stories, and
  nothing is to raise.

## Done when

Each command runs once, after the session's last increment lands.

- `git -C ~/src/rust/jig log origin/main --grep='^Plan: client-api$'
  --format='%(trailers:key=Step,valueonly,separator=)' | grep
  '^jig-20\.' | sort -u | wc -l` prints 5.
- `git -C ~/src/rust/jig ls-tree -d --name-only origin/main
  testing/jig-client-fake-person-browser` prints one line.
- `git -C ~/src/rust/jig log origin/main --all-match
  --grep='^Plan: client-api$' --grep='^Step: jig-20.5$'
  --format='%(trailers:key=Gate-Result,valueonly)' -1` names the browser
  suite's 13 tests and its seconds, under 60.
