# Session 04: the browser tier

temper has no browser tier at the base: no crate drives Chromium, nextest has no `browser` profile, and from temper 02.3
the gate builds a bundle that nothing opens. `client/browser.md` 4 asks for one: Chromium against `temper serve`, ten
stories of what a browser alone shows (4.1), in a suite, crate and profile of its own, failing when Chromium or the
bundle is missing (4.3). 04.1 repins temper's skein and jig in a commit of its own, then copies jig's
`tests/client-browser` into `temper-client-browser-world`, gives nextest its `browser` profile and the gate its run,
and lands the first story. 04.2 to 04.5 add the other nine, in three binaries (decision 8). The session needs temper
01.2 and 02.3, and skein 01.4, so that the repin takes all four of skein 01's steps. The tier is a world, so its lines
count as tests (code-organization.md 6) and the sizes are the gate's lines and the repin's repairs.

## Read first

- **temper:** `docs/design/client/browser.md` 2.1, 2.3, 2.4, 2.5, 4 (what it runs against), 4.1, 4.3, 6;
  `docs/design/domain/testing.md` 2 (its opening: the browser tier) and 2.6 (the rebound Host stays there);
  `crates/temper/tests/support/configuration.rs` (`documents`), `crates/temper/tests/support/mod.rs` (`configuration`),
  `crates/temper/src/lib.rs` (`serve`, `prepare`: paths join the working directory);
  `crates/temper-client-view/src/frame.rs` (the status node), `src/pages/chats.rs`, `src/pages/chat.rs` (`main`),
  `src/words.rs`; `crates/temper-client-domain/src/pages/chats.rs` (`Chats::start`, `Chats::told`);
  `.config/nextest.toml`; `scripts/check.sh`; `tests/client/Cargo.toml`.
- **jig:** `docs/design/protocol/browser.md` 12.1, 12.2, 15; `tests/client-browser/src/lib.rs` (`jig_test`, `chromium`,
  `bundle`, `private`, `absolute_paths`, `Tier`), `src/forgery.rs`, `tests/browser_*.rs`;
  `testing/jig-client-fake-person-browser/src/lib.rs` (`States`, `Target`), `src/stories.rs`
  (`a_spent_link_signs_the_person_out`), `src/attacks.rs`, `src/pages.rs` (`forging`); `crates/jig-client/src/shapes.rs`
  (`chat`, `fill`); `crates/jig-domain-core/src/documents/small.rs` (`chip_from_row`); `.config/nextest.toml`
  (`[profile.browser]`); `scripts/check.sh` (its browser lines).
- **skein:** `docs/design/ui-web.md` 14.1, 14.2; `testing/skein-fake-person-browser/src/harness.rs` (`Setup`,
  `Printed`), `src/harness/script.rs` (`Step`), `src/stories.rs` (`panic`, `back_and_forward`, `Shown`);
  `testing/skein-browser/src/boundary.rs` (`Key`, `Request::Key`); `docs/foundation/development.md` 5.7.
- **skein foundation:** `testing-strategy.md` 8, 10; `programming-model.md` 10.2.
- **Plan:** the README's "Rules of this plan" (the repin, browser tests, dependency lines, hotspots) and "One test of a
  suite that needs a built artifact"; `code-organization.md` 1 to 6; decisions.md 2, 4, 5, 7, 8, 9, Design questions 1
  to 3, For the user 1; `findings/base-runs.md` 1b, 2, 9; `skein/01-prerequisites.md` 01.1 to 01.3;
  `temper/01-serving.md` 01.1 to 01.3; `temper/02-shell.md` 02.2 to 02.4; `temper/03-look.md` 03.1, 03.2.

## What the design asks

- **browser.md 4.3:** a suite of its own in `tests/client-browser`, run by the gate with the profile `browser`, one test
  at a time, on the bundle the gate builds once; at most 12 tests and about 60 s; Chromium from `SKEIN_TEST_CHROMIUM`; a
  missing Chromium or bundle fails the suite. 04.1.
- **browser.md 4, "What it runs against"; decisions.md, Design questions 1:** `temper serve` with an unreached forge
  and LLM endpoint on closed loopback ports, no peer; the peers join with milestone 2's first story that needs them.
  04.1.
- **browser.md 4.1, story 1:** the printed link shows the inbox, the address `/` once it shows (Design questions 2); a
  reload on a chat signs in by the cookie; a spent link shows 2.4's signed-out words. 04.1, 04.2.
- **browser.md 4.1, story 2:** skein's `back_and_forward` between the chats and a chat; a non-page address opens the
  inbox at `/`. 04.3.
- **browser.md 4.1, stories 3 and 4:** Enter starts a chat once, Shift+Enter a line break in it (decision 9, Design
  questions 3); skein's `panic` at `/test/panic`. 04.4.
- **browser.md 4.1, story 5; jig's protocol/browser.md 15:** jig's three attack stories that need a browser. 04.5.
- **decisions.md 4, For the user 1:** each story is there for what only a browser shows; each also fails on a policy
  violation, an uncaught exception or a native dialog, which skein's harness reads.

## Choices this plan makes

- **The repin** (04.1's first commit; README, Rules; development.md 5.7): `cargo update -p skein-lib -p
  jig-client` moves both git sources to their `origin/main`, then `cargo check --workspace --all-targets`. smith
  stays at `f47f4df9`, which jig's `origin/main` also pins. jig's crates then build against skein's `main`, ahead of
  jig's own skein pin; skein 01 only adds variants that jig matches with a fallback arm (`skein/01-prerequisites.md`).
- **The configuration is the process tier's, copied:** `documents` and `support::configuration`'s writes, into the
  tier's `src/configuration.rs` (it is a test module of package `temper`, not importable). Three changes: the LLM
  endpoint at `127.0.0.1:9`, in `agent.json`'s endpoint and `engine.llm.endpoints[0]`, beside the forge already there
  (`Forge::Unreached`), not `34443`, where another gate's fake LLM may listen on the shared machine;
  `application.clients.bundle` the bundle's canonical directory (temper 01.1's key); and every relative path made
  absolute under the scratch by a walk, as jig's `absolute_paths`, since `serve` joins paths to its working directory
  and skein's `Setup` names none (the child runs in the test's, `tests/client-browser`). The worker and its agent
  program stay, the agent being `temper()`: the configuration starts as the process tier's does.
- **Starting a chat with the LLM unreached (wave3's question): it records the person's words and shows the chat at its
  address; the turn fails after.** "Start" sends `jig_client::shapes::chat`'s `Ask::Create` (`Chats::start`). jig's
  core answers `OwnerOutcome::Created { first, .. }` when the tasks' child has made the task
  (`jig-domain-core/src/sibling_routes.rs`), apart from any run; `Chats::told` shows `Address::Chat { task: first }`,
  whose page shows "Chat N" and the task's title. The run then goes to the in-process worker, whose agent meets the
  closed port. The words are the task's specification; the page shows their first line, the chip's title
  (`chip_from_row`). No story reads a turn.
- **The tier's helpers, from jig's `lib.rs`:** `temper()` beside the test executable, as `jig_test()`; `chromium()` as
  jig's; `bundle()` from `TEMPER_TEST_BUNDLE`, as jig's `bundle()`. Each panics with how to build what is missing.
- **`temper_client_browser_world::Tier`, as jig's and code-organization.md 2 name it:** `Tier::new(name)`, `setup`,
  which borrows the tier's scratch, `states`, and from 04.5 `target`; the bundle's variable `TEMPER_TEST_BUNDLE`.
- **The states:** live and offline are `Find::exactly_named(Role::Status, …)` on `frame.rs`'s status node, since
  `Find::text(b"Live")` would also find "Live chats"; signed out is 03.1's words, `Find::text`.
- **A story's chat is started through the page** by `start_chat(words)`, and is `Chat 1` at `/chats/1`: jig numbers
  tasks from 1, and temper's start makes none (a question if not).
- **`back_then_forward_returns_to_the_chat` makes its chat in a first harness run,** then runs skein's
  `back_and_forward` against a second `temper serve` on the same scratch: the story's script starts at its first address
  and the chat must exist. The state outlives the restart, as the process tier's
  `a_persons_change_lands_on_the_forge_and_is_found_after_a_restart` shows, and `people.sign_ins` 2 admits the second
  sign-in.
- **The composer story types through Chromium:** the harness's `Step::Type`, `Step::KeyWithShift` and `Step::Key`, so
  that the shell's key listener and the text area's own line break run; the person's `Send` acts a Submit and would skip
  both. It reads the break through the title: a started chat's title is its words' first line, cut at the first `\n`
  (jig `crates/jig-domain-core/src/documents/small.rs` 49-50, `chip_from_row`), which the Chat page shows as a Paragraph
  named by it (`crates/temper-client-view/src/pages/chat.rs` 35); the first words are on no conversation line. So
  "Hello", Shift+Enter, "there", Enter gives a paragraph named exactly "Hello"; no break gives "Hellothere", and
  Shift+Enter submitting leaves the Chats page before "there" is typed, or starts a second chat.
- **The forgery is a chat.** jig's `forgery.rs` forges a person task for role 0, which jig's tier admits by adding a
  delegation kind; temper's engine document admits charter 1 alone (jig's `tests/channel/src/fixtures.rs`), so jig's
  request would be refused by the engine and the story's `Gone` would hold with no guard. 04.5 forges what "Start"
  sends: `fill`'s task for the chat template, by hand, the words "Forged"; the target page `/chats`.
- **Dependencies all in 04.1,** so that `Cargo.toml` and `Cargo.lock` change in one increment (README, hotspots).
- **The profiles:** the default gains `& not binary(/^browser_/)`; the fuzzy one's `binary(/^fuzzy_/)` already
  excludes the tier, and `measure` stays unfiltered. `[profile.browser]` takes jig's values (`global-timeout = "60s"`,
  `slow-timeout = { period = "20s", terminate-after = 3 }`, one thread): `findings/base-runs.md` ran no jig browser
  suite. Its skein suite (2: 22 tests in 13.6 s, each 0.39 to 0.95 s) puts ten stories well inside browser.md 4.3's
  60 s.

## Rules that bind it

- **Browser tests sparingly:** exactly the ten stories of decisions.md's "For the user"; the tier's other tests are
  host unit tests of its configuration (README, Rules; code-organization.md 5).
- **No peer, no jig test service or test shell, no `#[ignore]` or early return** when Chromium or the bundle is
  missing (code-organization.md 5; browser.md 4.3).
- **The repin is 04.1's own commit;** a repair past the increment's Size is a question to the supervisor (README).

## Increments

### 04.1 The repin, the tier and the printed link
Scope: README "The repin"; browser.md 4.3; 4.1 story 1, the printed link (Design questions 1 and 2).
Needs: temper 01.2, 02.3; skein 01.4.
Files:
- The repin's commit: `Cargo.lock`, and what `cargo check --workspace --all-targets` then names.
- `Cargo.toml`: `tests/client-browser` in `members`; `skein-fake-person-browser`, `skein-browser` and
  `jig-client-fake-person-browser`, each copying the URL its repository's lines use (`skein-scratch` is there at the
  base). `Cargo.lock`.
- `tests/client-browser/Cargo.toml`, `src/lib.rs`, `src/configuration.rs`, `tests/browser_sign_in.rs` (new).
- `.config/nextest.toml`, `scripts/check.sh`.
- Copied, unchanged: `crates/temper/tests/support/configuration.rs`, `support/mod.rs` (real-loop 10.1, 10.2 and 13.3
  change `support/`; README, hotspots).
Shape:
- **Manifest:** the package fields `tests/client/Cargo.toml` inherits; its `[lints]`; dependencies, workspace lines:
  `skein-fake-person-browser`, `skein-fake-person`, `skein-browser` (to name `skein_browser::boundary::Key`, which
  `skein-fake-person-browser` does not re-export), `skein-scratch`, `skein-lib`, `skein-ui`, `serde_json`,
  `skein-tls-world`, `jig-channel-world`, `jig-client`, `jig-client-protocol-wire`, `jig-client-protocol-link`,
  `jig-client-fake-person-browser`.
- **`src/configuration.rs`:** `documents(directory, program, bundle)` and `write(directory, program, bundle)`, copied
  with the three changes above; `absolute_paths` over the names it writes, `state`, `accounts/records`, `worker-root`;
  files 0o600, as jig's `private`. Its doc cites the two sources.
- **`src/lib.rs`:** `temper()`, `chromium()`, `bundle()`, `WITHIN` (5 s) and `ABSENT` (1 s), jig's waits; `Tier {
  scratch }`; `new` makes the scratch and writes; `setup`: `args` and `restart` `["serve", <scratch>/config.json]`,
  `Printed { prefix: b"http://127.0.0.1:", on: Output::Stdout, names: Names::Url }`, no expected line, no origin;
  `states`. The crate's doc cites browser.md 4.3 and jig's `tests/client-browser/src/lib.rs`.
- **`.config/nextest.toml`:** the default filter's `& not binary(/^browser_/)`; `[profile.browser]`, with the comment
  "Chromium against temper serve, one test at a time (client/browser.md, 4.3).", `default-filter =
  "package(temper-client-browser-world) & binary(/^browser_/)"` and jig's three values.
- **`scripts/check.sh`,** after 02.3's `scripts/bundle.sh target/temper-bundle >/dev/null`, kept: a comment citing
  browser.md 4.3; `cargo build -p temper`; `TEMPER_TEST_BUNDLE="$(pwd)/target/temper-bundle"`; `export
  TEMPER_TEST_BUNDLE`; `cargo nextest run --workspace --profile browser`, as jig's lines.
Test shape: world, `tests/browser_sign_in.rs`, its doc citing browser.md 4.1 story 1. The reference test:

```rust
#[test]
fn the_printed_link_signs_in() {
    let tier = Tier::new("printed-link");
    let inbox = Find::exactly_named(Role::Heading { level: 1 }, b"Inbox");
    let mut script = Script::new(Box::new([
        Step::Person(PersonStep::See { find: inbox, within: WITHIN }),
        Step::Person(PersonStep::See { find: tier.states().live, within: WITHIN }),
        Step::Address(Box::from(b"/".as_slice())),
    ]));
    assert_eq!(Harness::new(tier.setup()).run(&mut [&mut script]), Outcome::Done);
}
```

Beside it, unit, in `src/lib.rs`'s tests over `documents(Path::new("/private/test"), …, Path::new("/test/bundle"))`:
- `the_tier_names_the_bundle_for_its_server`: `application.clients.bundle` is `/test/bundle`.
- `the_tier_listens_on_loopback_at_a_dynamic_port`: `application.clients.listener.socket` is `127.0.0.1:0`.
- `the_tiers_forge_is_unreached`: the forge's `api.socket` is `127.0.0.1:9`.
- `the_tiers_llm_endpoint_is_unreached`: `engine.llm.endpoints[0].address.socket` and the agent's endpoint are
  `127.0.0.1:9`.
- `the_tiers_paths_are_absolute`: the forge's `api_token` and the worker's `root` are under `/private/test`.

Ceilings: each unit under 20 ms; the story about 1 s (`findings/base-runs.md` 1b).
Proof: world: the browser suite, `--profile browser`, `the_printed_link_signs_in` (README, "One test of a suite that
needs a built artifact"), above the units, then the gate. process: the README's process tier once, after the repin's
commit, which moves the engine `temper serve` runs.
Don't: skip on a missing Chromium or bundle; import from `crates/temper/tests/support`; depend on `jig-test`,
`jig-test-service` or `jig-client-test-shell-web`; start a peer; replace 02.3's bundle line; change the default or fuzzy
budgets (README, Done when 6); repin smith or hearth.
Size: at most 300 lines: about 6 of `check.sh`; the rest is the repin's repairs, if any.
Removes: nothing.

### 04.2 A reload by the cookie and a spent link
Scope: browser.md 4.1 story 1, a reload on a chat and a spent link.
Needs: temper 02.4 (the chat's address), 03.1 (the signed-out words).
Files: `tests/client-browser/src/lib.rs` (`start_chat`, `FIRST_CHAT`, `FIRST_CHAT_ADDRESS`), `tests/browser_sign_in.rs`.
Shape:
- `start_chat(words)`: See the live state, Press the link "Chats", See the text area "Start a chat", Type the words,
  Press the button "Start", See the heading 1 `FIRST_CHAT`, each find `exactly_named`.
- `a_spent_link_shows_the_signed_out_words` calls jig's `stories::a_spent_link_signs_the_person_out` with the tier's
  setup and states.
Test shape: world. The reference test:

```rust
#[test]
fn a_reload_on_a_chat_signs_in_by_its_cookie() {
    let tier = Tier::new("reload-on-a-chat");
    let see = |find| Step::Person(PersonStep::See { find, within: WITHIN });
    let mut steps = temper_client_browser_world::start_chat(b"Hello");
    steps.extend([
        Step::Person(PersonStep::Reload),
        see(Find::exactly_named(Role::Heading { level: 1 }, FIRST_CHAT)),
        see(tier.states().live),
        Step::Address(Box::from(FIRST_CHAT_ADDRESS)),
    ]);
    let mut script = Script::new(steps.into_boxed_slice());
    assert_eq!(Harness::new(tier.setup()).run(&mut [&mut script]), Outcome::Done);
}
```

Beside it: `a_spent_link_shows_the_signed_out_words`.
Ceilings: each about 1 s.
Proof: world: the browser suite, `--profile browser`, these two. The refusal is jig's mount's tests, the words 03.1's.
Don't: read the cookie or the small store; add a third story.
Size: 0 lines.
Removes: nothing.

### 04.3 Back, forward and a non-page address
Scope: browser.md 4.1 story 2.
Needs: temper 02.4.
Files: `tests/client-browser/tests/browser_pages.rs` (new).
Shape: `back_then_forward_returns_to_the_chat`: a first harness run of `start_chat(b"Hello")`, then skein's
`back_and_forward(&tier.setup(), chats, chat, loaded)` on the same tier: `Shown { address: b"/chats", sees: heading 1
"Chats" }`, `Shown { address: FIRST_CHAT_ADDRESS, sees: heading 1 FIRST_CHAT }`, `loaded` the live state (skein 01.2).
`a_non_page_address_opens_the_inbox_at_slash`: See live, Press "Chats", See its heading, Go `/nowhere`, See the
heading 1 "Inbox", `Step::Address` of `/`.
Test shape: world. The reference test:

```rust
#[test]
fn back_then_forward_returns_to_the_chat() {
    let tier = Tier::new("back-then-forward");
    let mut started = Script::new(temper_client_browser_world::start_chat(b"Hello").into_boxed_slice());
    assert_eq!(Harness::new(tier.setup()).run(&mut [&mut started]), Outcome::Done, "the chat to return to");
    let chats = Shown { address: b"/chats", sees: Find::exactly_named(Role::Heading { level: 1 }, b"Chats") };
    let chat = Shown { address: FIRST_CHAT_ADDRESS, sees: Find::exactly_named(Role::Heading { level: 1 }, FIRST_CHAT) };
    assert_eq!(stories::back_and_forward(&tier.setup(), chats, chat, tier.states().live), Outcome::Done);
}
```

Beside it: `a_non_page_address_opens_the_inbox_at_slash`.
Ceilings: about 2 s for the first (two serves), 1 s for the second.
Proof: world: the browser suite, `--profile browser`, these two. The codec's choice of the inbox is 02.1's units.
Don't: copy `back_and_forward`'s script; split the first into two tests.
Size: 0 lines.
Removes: nothing.

### 04.4 The composer and a panic
Scope: browser.md 4.1 stories 3 and 4; decision 9.
Needs: temper 03.2 (and skein 01.3, in 04.1's repin).
Files: `tests/client-browser/tests/browser_pages.rs`.
Shape: `a_panic_shows_the_failure_page`: skein's `stories::panic(&tier.setup(), b"/test/panic",
Find::text(b"The application failed."))` (decision 7; 02.2's restart part).
Test shape: world. The reference test:

```rust
#[test]
fn enter_starts_one_chat_keeping_a_shift_enter_break() {
    let tier = Tier::new("composer");
    let see = |find| Step::Person(PersonStep::See { find, within: WITHIN });
    let composer = || Find::exactly_named(Role::TextArea, b"Start a chat");
    let chats = || Step::Person(PersonStep::Press { find: Find::exactly_named(Role::Link, b"Chats") });
    let mut script = Script::new(Box::new([
        see(tier.states().live),
        chats(),
        see(composer()),
        Step::Type { find: composer(), words: Box::from(b"Hello".as_slice()) },
        Step::KeyWithShift(Key::Enter),
        Step::Type { find: composer(), words: Box::from(b"there".as_slice()) },
        Step::Key(Key::Enter),
        see(Find::exactly_named(Role::Heading { level: 1 }, FIRST_CHAT)),
        Step::Address(Box::from(FIRST_CHAT_ADDRESS)),
        see(Find::exactly_named(Role::Paragraph, b"Hello")),
        chats(),
        see(Find::exactly_named(Role::Link, FIRST_CHAT)),
        Step::Person(PersonStep::Gone { find: Find::exactly_named(Role::Link, b"Chat 2"), within: ABSENT }),
    ]));
    assert_eq!(Harness::new(tier.setup()).run(&mut [&mut script]), Outcome::Done);
}
```

Beside it: `a_panic_shows_the_failure_page`.
Ceilings: each about 1 s.
Proof: world: the browser suite, `--profile browser`, these two. The view's Submit and line break are 03.2's units and
joint story.
Don't: send with the person's `Send`; read a turn; add a story for the message composer.
Size: 0 lines.
Removes: nothing.

### 04.5 The attacks that need a browser
Scope: browser.md 4.1 story 5; jig's protocol/browser.md 12.1, 12.2, 15.
Needs: temper 01.3 (the policy).
Files: `tests/client-browser/src/forgery.rs` (new), `src/lib.rs` (`Tier`'s `forged`, `target`),
`tests/browser_attacks.rs` (new).
Shape:
- `forgery.rs`, copied from jig's `tests/client-browser/src/forgery.rs`, citing it, over `jig_client::contract as c`:
  the task's executor `c::OwnerExecutor::Agent { charter: 1 }`, contract `c::NewContract::Report { words: 32 }`, budget
  `c::NewBudget::AsAllowed { cap: None }`, flags `{ conversation: true, listed: false, retention: c::Retention::Kept }`,
  label 1, deadline `Named`, jig's zero authority, the words "Forged"; jig's key and `w::Keyed` wrapping, unchanged.
- `Tier::target`: `Target { prefix: b"/jig", page: b"/chats", request: &self.forged, done: Find::text(b"Forged") }`.
Test shape: world. The reference test:

```rust
#[test]
fn a_cross_site_request_from_another_port_does_nothing() {
    let tier = Tier::new("another-port");
    let outcome =
        attacks::a_cross_site_request_from_another_port_does_nothing(&tier.setup(), &tier.states(), &tier.target());
    assert_eq!(outcome, Outcome::Done);
}
```

Beside it: `a_cross_site_request_from_another_host_does_nothing`, `a_framed_page_shows_nothing`.
Ceilings: each about 1 s.
Proof: world: the browser suite, `--profile browser`, these three: Chromium sends what jig's guards refuse, and enforces
`frame-ancestors` from 01.3's policy. The refusals are jig's tests; the headers 01.3's units.
Don't: run jig's rebound Host story here (`domain/testing.md` 2.6); add a role to the engine's delegation kinds, as
jig's tier does.
Size: 0 lines.
Removes: nothing.

## Interface sketches

The planner compiles these against the tier's dependencies, with stub bodies, once skein 01.1 and 01.3 have landed.

```rust
// tests/client-browser/src/lib.rs (04.1; 04.2 adds start_chat and the chat's names; 04.5 forged and target)
pub const WITHIN: skein_lib::Duration = skein_lib::Duration::from_secs(5);
pub const ABSENT: skein_lib::Duration = skein_lib::Duration::from_secs(1);
pub const FIRST_CHAT: &[u8] = b"Chat 1";
pub const FIRST_CHAT_ADDRESS: &[u8] = b"/chats/1";
#[must_use]
pub fn temper() -> std::path::PathBuf;
#[must_use]
pub fn chromium() -> std::path::PathBuf;
#[must_use]
pub fn bundle() -> std::path::PathBuf;
#[must_use]
pub fn start_chat(words: &[u8]) -> Vec<skein_fake_person_browser::Step>;
pub struct Tier {
    scratch: skein_scratch::Scratch,
    forged: Box<[u8]>,
}
impl Tier {
    #[must_use]
    pub fn new(name: &str) -> Self;
    #[must_use]
    pub fn setup(&self) -> skein_fake_person_browser::Setup<'_>;
    #[must_use]
    pub fn states(&self) -> jig_client_fake_person_browser::States;
    #[must_use]
    pub fn target(&self) -> jig_client_fake_person_browser::Target<'_>;
}

// tests/client-browser/src/configuration.rs (04.1)
pub(crate) fn documents(directory: &Path, program: &Path, bundle: &Path) -> (Box<[u8]>, Box<[u8]>);
pub(crate) fn write(directory: &Path, program: &Path, bundle: &Path);

// tests/client-browser/src/forgery.rs (04.5)
pub(crate) fn request() -> Box<[u8]>;

// Used, from skein 01: skein_fake_person_browser::harness::Step::Address(Box<[u8]>), a path with its query,
// compared exactly; Step::KeyWithShift(skein_browser::boundary::Key), sent as the kit's Request::Key { page, op,
// key, shift: true }; stories::back_and_forward(setup, first, second, loaded: Find) -> Outcome.
```

## Questions to raise when reached

- **04.1:** a jig or smith crate that fails to build against skein's `main`, past the Size: report which and ask
  (decision 5).
- **04.1:** `temper serve` refusing an absolute path, or reading its working directory elsewhere: report where.
- **04.2:** the first chat not `Chat 1`: report the number and what made the tasks before it.
- **04.3:** the second serve not showing the first's chat, or refusing its sign-in: report the start's words.
- **04.5:** the engine refusing the forged chat from the page's own origin (a same-origin run of `pages::forging`, by
  hand, once): report its answer, since the stories' `Gone` would then prove nothing.

## Done when

Each in a worktree of temper at `origin/main`, after `git fetch` there and in `~/src/rust/skein` and `~/src/rust/jig`:
- `git show origin/main:tests/client-browser/Cargo.toml | grep -q -x 'name = "temper-client-browser-world"'`
- `git show origin/main:tests/client-browser/src/lib.rs | grep -q -F 'TEMPER_TEST_BUNDLE'`
- `git show origin/main:tests/client-browser/src/configuration.rs | grep -q -F '"127.0.0.1:9"'`
- `sh -c 'test -n "$(git ls-tree -r --name-only origin/main tests/client-browser)" && ! git grep -q -F "#[ignore]"
  origin/main -- tests/client-browser'`
- `git show origin/main:.config/nextest.toml | sed -n '/^\[profile\.browser\]/,/^\[/p' | grep -q -F
  'default-filter = "package(temper-client-browser-world) & binary(/^browser_/)"'`
- `git show origin/main:.config/nextest.toml | sed -n '/^\[profile\.default\]/,/^default-filter/p' | grep -q -F
  'not binary(/^browser_/)'`
- `git show origin/main:scripts/check.sh | awk '/^scripts\/bundle\.sh /{b=NR} /^export TEMPER_TEST_BUNDLE/{e=NR}
  /--profile browser/{p=NR} END{exit !(b && e && p && b < e && e < p)}'`
- `sh -c 'rev=$(git show origin/main:Cargo.lock | grep -o -E "fabrig-dev/skein\.git#[0-9a-f]{40}" | head -n 1 |
  cut -d"#" -f2); test -n "$rev" && git -C ~/src/rust/skein show "$rev:crates/skein-ui-web/src/page.rs" | grep -q -E
  "fn pressed\("'`: temper's skein holds skein 01.4, and so 01.1 to 01.3 before it.
- `sh -c 'rev=$(git show origin/main:Cargo.lock | grep -o -E "fabrig-dev/jig\.git#[0-9a-f]{40}" | head -n 1 |
  cut -d"#" -f2); test -n "$rev" && git -C ~/src/rust/jig merge-base --is-ancestor 8cc7e79f "$rev"'`
- `sh -c 'test "$(cargo nextest list -p temper-client-browser-world --profile browser --message-format oneline |
  wc -l)" -eq 10'`
- `scripts/check.sh` exits 0, with `SKEIN_TEST_CHROMIUM` set.
