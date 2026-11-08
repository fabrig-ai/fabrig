+++
id = "repair"
part = "repair"
kind = "feature"
needs = []
files = ["jig/crates/jig-protocol-engine-clients/src/**", "jig/tests/engine-clients/**", "jig/testing/jig-engine-clients-kit/**", "temper/tests/client-browser/src/**"]
effort = "high"
minutes = 180
+++

# A cross-site authorization isn't refused as busy while temper starts

## Goal

red-72, after red-71: `temper-client-browser-world`'s story `browser_authorize::a_cross_site_authorization_returns_a_code_and_the_links_state` (`temper/tests/client-browser/tests/browser_all/browser_authorize.rs`) fails at its last step under load. After the server's restart, the Authorize page draws the client's busy state, "temper couldn't sign you in just now" (`Word::AuthorizeBusy`), and the forge's callback never reads "Authorization received". On 2026-11-08 it failed at 00:09Z, 00:21Z and 01:23Z and passed at 00:32Z, 00:44Z and 00:55Z. A failing run takes about 12 s, a passing one about 2 s, and at 00:09Z a rerun alone failed too. So it's load and time, not order.

Find the answer that draws the busy page and the bound behind it. Hold that bound in a focused test that fails first. Fix it at its lowest place, so a sign-in through temper isn't refused as busy while the engine is merely slow.

Acceptance:

- **The cause, in the report, in a few lines:** which of jig's answers drew the page (*not ready*, *busy* or *lost*), the code that gave it, and why, with the restarted server's log lines that show it (filtered, Steps 1). Say where the run's 10 s went (Inputs, finding 1).
- **The decision, in one line of the report,** with the place of the fix:
  - **The product's**, if a person could meet it: a box just restarted or under load refuses the sign-in. Then the authorization waits for the engine within its own bound. It answers *not ready* only while the engine stops, or once that bound has passed, and never refuses as busy while the engine is merely slow.
  - **The story's**, only if a person's box never meets it: the story starts the authorization before something that only its world lacks. Then the story waits for the readiness that a person's client waits for.

  The lead's reading leans to the product's (Inputs, finding 3). A person whose box has just restarted can press "Sign in with temper" on Forgejo at once, as the story does.
- **A focused test, red first,** in the package that holds the cause. The authorization arrives while the bound is held, or while its answer is slow past it, and today gets the busy page's answer. After the fix it gets *go*, or the page the design gives. Make it a row of an existing table test where one covers the behaviour; the sketch below names one.
- **The story's checks stay as they are:** `git diff origin/main -- temper/tests/client-browser/tests/browser_all/browser_authorize.rs` changes no `see(...)`, `welcome()`, `Step::...`, `Find::...` or `assert...` line, nor `WITHIN` or the expected-error list. The one exception is the story-only option's waiting steps (Owns and may change), which add and remove no check.
- **The story under load, by name and alone,** with the command in Steps 1 and the load described there:
  - before the fix, up to five runs, stopping at the first failure. Report how many passed.
  - after the fix, at least five runs, all passing.

  For each run, report its `date -u` start, its outcome, its wall time, how many tests it selected (exactly one) and the load.
- **The focused tests of every package the fix changes pass,** and so do their direct dependents' in every tree (Steps 5). The fix changes what the provider's *not ready* means, so the tests of the packages that read it run too.

## Inputs

- **The request:** `temper/docs/plan/ui-authorize-busy/inputs/request.md`, read whole (it is short). Its `For every task of this plan` binds this task and stands in for a `notes/common.md`.
- **The story and its world:** the story's file. Its fixture is `temper/tests/client-browser/src/authorize.rs` (`Tier::authorize`, `Tier::authorize_callback`); `Tier::reserved`, `Tier::setup` and `WITHIN` are in `temper/tests/client-browser/src/lib.rs`. The harness's restart is `Step::StopServer` and `Step::StartServer` (`skein/testing/skein-fake-person-browser/src/harness/script.rs`).
- `temper/docs/design/protocol/sign-in.md`, 9.3: the authorization, jig's answers and their pages.
- `jig/docs/design/protocol/client.md`, 18: jig's OpenID provider. `Authorize` is *not ready* "while the engine stops", and *busy* when the codes are full.
- `jig/docs/design/domain/people.md`, 3: `Whose`, a read that writes nothing.
- `temper/docs/design/protocol/sign-in.status.md`: its 9.3 **Not yet** entry, which records red-71 and an exploratory busy page drawn without an authorize request.
- **The last repair,** `ui-authorize-red`, ran the story alone twice on an unloaded machine. It passed both times, and nothing was changed. A pass alone without load proves nothing.

What the lead's reading found. Nothing below was run; it narrows the search and settles nothing.

1. **The 10 s is likely the story's own wait.** `WITHIN` is 10 s, and it bounds the last step, `see(b"Authorization received")`. A failing run's 12 s is a passing run's 2 s plus that wait. So the busy page may come at once after the press, and the step waits out `WITHIN` for a heading that never comes. Check the log's times before you hunt for a 10 s deadline in the product. If the busy page came about 10 s after the press, there is a product deadline after all: find it.
2. **The page.** It is `temper_client_domain::authorize::State::Busy` (`temper/crates/temper-client-domain/src/authorize.rs`, `answered`), set for jig's `Authorized::NotReady`, `Busy` and `Lost` alike, and drawn as `Word::AuthorizeBusy` (`temper/crates/temper-client-view/src/sign_in.rs`). The client sends `Authorize` only once its link is `Live` (`progress`). The service's own 503 with the same words is a different path: Google's return refused at the mint (`temper/crates/temper-service/src/tests/sign_in.rs`, `a_signed_member_with_a_bad_then_returns_home_and_mint_refused_is_busy`). The story never takes it.
3. **Not ready is the likeliest under load.** In jig's provider (`jig/crates/jig-protocol-engine-clients/src/provider.rs`), `authorize_checked` asks people `Whose`, and `authorize_reply` answers *not ready* for every people reply but `Signer`. People answers `Whose` with `Again::NotReady` until its start opens: loading, linked, sweeping, or failed (`jig/crates/jig-domain-people/src/domain.rs`, `Event::Whose` and `ready`). Its test `whose_reads_a_service_and_answers_not_ready_before_restoration` holds that. The exchange's credential is checked against the secrets store, not people (`jig/crates/jig-protocol-engine-clients/src/services.rs`, `SecretsAnswer::Got`). So a client's link can be live while people is still restoring.

   After the story's restart, a loaded machine stretches people's start. The authorization arrives inside it and is answered *not ready* at once. jig's design gives *not ready* only "while the engine stops" (`jig/docs/design/protocol/client.md`, 18): here the code departs from it. Whether the link can be live before people opens depends on what its `me` delivery waits for. Check that in the log.
4. **Lost** comes from:
   - any non-200 answer to the exchange (`jig/crates/jig-client-protocol-http/src/call.rs`, `answered`);
   - a failed fetch (`temper/crates/temper-client-shell-web/src/host.rs`, `Heard::Failed`);
   - the mount's 503 refusal (`jig/crates/jig-protocol-engine-clients-http/src/lib.rs`, `refusal`);
   - temper-service's admission of a claimed head (`temper/crates/temper-service/src/clients.rs`, `server_requests` and `admit_waiting`);
   - a sign-in that ends with the authorization in flight (`jig/crates/jig-client-protocol-link/src/component.rs`, `end_sign_in`).
5. **Busy** comes from the client link, with an authorization already in flight or a full attempt table (`jig/crates/jig-client-protocol-link/src/authorize.rs`). It also comes from the provider, with its code or token table full (`provider.rs`, `authorize_person`).

Departures from the implementer's text, and why:

- **You run one browser story, by name and alone, under load you make.** The lab's rules forbid a plan's runs the slow tiers. They allow a red's repair to run the flagged test by name, alone, to reproduce it and to prove the fix, and this task is red-72's repair. Never run the browser tier or another story.
- **The load is a throwaway CPU burner, not a second build.** The request's example, a parallel focused build, would put a second heavy command beside the story's, and the lab's rules allow one at a time. The burner is throwaway code: untracked, running only for the length of each story run, stopped after it, and gone before the push (Steps 1).
- **One design sentence and two status entries are yours** (Owns and may change). The request allows a small design edit that the task needs.
- **The fix may sit in jig.** The red is temper's, but the cause is fixed where it sits (`AGENTS.md`, 5).

## Owns and may change

Owns:

- `jig/crates/jig-protocol-engine-clients/src/**`: the provider's wait, the default fix.
- `jig/tests/engine-clients/**` and `jig/testing/jig-engine-clients-kit/**`: the focused test, and any peer helper it needs.
- `temper/tests/client-browser/src/**`: the browser world's fixtures.

Reader and neighbour files, and the edits allowed there:

- **The story's file:** only its setup lines (before `let mut person`), and the story-only option's waiting steps. Never its checks (Goal).
- **`jig/docs/design/protocol/client.md`, 18:** in the `Authorize` bullet, the sentence "*Not ready* while the engine stops, and *busy* when the codes are full (below), each changing nothing." Extend it with what an authorization does while people starts: it waits up to its bound, asking again, then is *not ready*. Change nothing else in the document.
- **`temper/docs/design/protocol/sign-in.status.md`, 9.3's Not yet entry,** and the matching sentence about red-71 in `temper/docs/design/STATUS.md`. red-71's sentences give way to the cause found and where it was fixed. The exploratory busy page drawn without an authorize request stays, unless the cause explains it too; then say so in the entry.
- **`jig/docs/design/protocol/client.status.md`:** a line only if the fix leaves a gap against section 18.
- **Compiler-forced arms** for `Waiting::Authorize`'s new fields, wherever the crate matches it (`closing.rs`, `component.rs`, `reads.rs`, `services.rs`, `watches.rs`, `public/me.rs`).

Pre-approved options. The default comes first.

- **If people's start answers `Whose` *not ready* (finding 3), it's the product's. Fix it in the provider,** as sketched below. People keeps answering *not ready* before its start opens, as its design and test hold. The provider waits for people instead of passing its *not ready* on.
- **If the start itself is slow for a cause of its own** (a load or a sweep that waits on something only the restart leaves behind), fix that too where it sits, under a focused test, beside the provider's wait. If it is in `jig/crates/jig-domain-people/src/**`, its tests run whole. Name the files.
- **If the page is *lost*: a non-200 or failed exchange** (finding 4). It's the product's if a person's page can meet it under load. Fix it at the place that drops or refuses a call it could hold within its connection's bound, under a focused test in that package:
  - temper-service's admission: `temper/crates/temper-service/src/clients.rs`, with its tests in `clients/admission_tests.rs`;
  - or jig's mount: `jig/crates/jig-protocol-engine-clients-http/src/**`.

  `ui-ask-drop/stock` changes `temper/crates/temper-service/**` at the same time. Make the smallest edit there, and name the overlap in the report.
- **If the page is the client link's own *busy* or *lost*** (findings 4 and 5), fix it in `jig/crates/jig-client-protocol-link/src/authorize.rs` under a focused test in that package. `tui-link-colours/jig-expiry` changes that crate's `component.rs` and `sign_in.rs` and `jig/tests/client/tests/suite/authorize.rs`. If the fix needs one of those, make the smallest edit and name the overlap. A double send from the client domain is fixed in `temper/crates/temper-client-domain/src/authorize.rs`, under a row of its `every_answer_has_its_page_and_only_go_replaces_the_address` or a test beside it.
- **If the provider's code or token table is full** (finding 5), find what fills it, and fix that where it sits.
- **If only the story's world meets it:** the story waits for the readiness that a person's client waits for. Add to `after`, between `Step::StartServer` and the forge's `Step::Visit`, a visit to temper's page and `welcome()`, as the story's `before` does. Or fix the world's fixture in `temper/tests/client-browser/src/**`. Add no check and remove none, and say in the report why a person's box never meets it.
- **If the fix makes a generated file stale** (a golden, a print, a journey recording), regenerate it with its own tool in this task, and name it in the report.
- **If an oracle or helper trips on the provider's correct wait** (a fuzzy model, or a world that expects *not ready* at people's first `Again`), repair it in the same task. Keep what it checks for everything else, and name it in the report.

The provider's wait, sketched. This binds the names and what crosses the component's boundary; inside it, the choices are yours.

- **`provider.rs`:** two constants beside `CODE_LIFE` and `TOKEN_LIFE`:
  - `AUTHORIZE_WAIT: Duration = Duration::from_secs(20)`: the longest an authorization waits for people's start. It stays below the server's 30 s connection stall bound (`temper/crates/temper-service/src/limits.rs`, `server`), so the page is answered before its connection is dropped.
  - `AUTHORIZE_AGAIN: Duration = Duration::from_millis(250)`: the time between one ask and the next.
- **`Waiting::Authorize`** (`sign_in.rs`) gains `since: Time`, when the authorization was checked, and `again: Option<Time>`, when it next asks people. `again` is `None` while a `Whose` is out.
- **`authorize_reply`** handles `people::Reply::Again(people::Again::NotReady)` this way: while the component isn't stopping and `env.now` is before `since + AUTHORIZE_WAIT`, it keeps the exchange with `again` at `env.now + AUTHORIZE_AGAIN` and answers nothing yet. Past the wait, it answers *not ready*, as today. Every other reply is answered as today.
- **`Component::next_deadline`** includes the earliest `again`. **`Component::fire`** sends `Whose` again for each authorization whose `again` has passed.
- **A held authorization when the component stops** is answered *not ready*, as one with its `Whose` out is today.

The test, sketched, in `jig/tests/engine-clients/tests/suite/authorize.rs`. `authorization_while_stopping_is_not_ready` becomes one table test, `authorization_is_not_ready_only_while_stopping_or_past_its_wait`, with these rows:

| Row | What happens | Answer |
|---|---|---|
| stopping | today's case | *not ready* at once, with no core event |
| people starting, then open | the peer answers the first `Whose` with `Again(NotReady)`; nothing is answered yet; after `AUTHORIZE_AGAIN` (`advance_time`, then the component's fire) a second `Whose` goes out, answered `Signer` | *go* |
| people starting past the wait | every `Whose` is answered `Again(NotReady)` until `AUTHORIZE_WAIT` has passed | *not ready* |

Today the second row is answered *not ready* at the first `Again`: that is the red. `fixtures::settings()` configures no provider. Take the provider world's settings (`jig/tests/engine-clients/tests/suite/provider.rs`), or put the rows in that file if its world answers people's replies by hand. Name the choice in the report. For the bite, answer *not ready* at the first `Again` again, and see the second row fail.

## Steps

1. **Reproduce, under load.** Record `nproc` and `date -u`. The story's one command, through heavy:

   ```sh
   TIERS_WRAPPER=heavy tools/tiers/run temper browser -- -E 'test(=browser_authorize::a_cross_site_authorization_returns_a_code_and_the_links_state)'
   ```

   The runner builds the browser bundle, the playground bundle and the `temper` binary, then runs nextest's `browser-temper` profile with the filter (`tools/tiers/design.md`, 2.1, 2.2 and 3). Check that it selected exactly one test.
   - Run it once without load first: that run builds.
   - Then make the load for each later run. Start `nproc` busy loops as a throwaway shell job just before the run, for example `for i in $(seq "$(nproc)"); do sh -c 'while :; do :; done' & done`, and keep their PIDs. Stop exactly those PIDs right after the run, never by name. Record `uptime` during the run.
   - Run up to five times like that, stopping at the first failure.

   **Reading a failure:** the harness's message holds the page's snapshot, Chromium's stderr and the server's stderr (`Server stderr:`, cleared at each restart, so it holds the restarted server's log). Read it through a filter, never whole and never raw: `grep -v 'secret='` or `sed 's/secret=[^ &"]*/secret=…/g'`, then `grep` and `tail` around the authorization. Nothing with `secret=` in it goes into the report, a commit or a test. Look for:
   - whether `POST /jig/v1/authorize` arrived, and what it was answered;
   - the times of people's start (loading, sweeping, open) beside the authorization's;
   - the time from the press to the busy page.

   **If five loaded runs all pass,** say so with their times, and go on from the reading. The focused test needs no reproduction.
2. **Find the cause and decide** (Goal), using the findings in Inputs. Write down what you ruled out, and why.
3. **The focused test, red first.** Write it, run it, and see it fail for the cause's reason: `heavy cargo nextest run -p jig-engine-clients-world --test suite -E 'test(=NAME)'`, or the package that holds the cause.
4. **Fix it,** and see the test pass. Then do the bite: break the rule by hand, see the test fail for the rule's reason, and restore.
5. **Check:**
   - Run the story at least five times under load (Steps 1), after the fix. All must pass.
   - `cargo fmt`.
   - `python3 -I -B <tree>/scripts/old_forms.py` for each tree the branch changes.
   - `heavy cargo clippy -p PKG ... --all-targets -- -D warnings`, naming every package you changed.
   - The focused tests of `jig-protocol-engine-clients` and `jig-engine-clients-world`, with `--profile fuzzy` where they have fuzzy tests. Then `temper-service`'s, since it composes the component and its tests drive authorizations; this is the change of meaning's readers' run. For any other package you change, its tests and its direct dependents' too. Check cross-tree dependents of a public change with `heavy cargo check -p <dependent> --tests`.
   - `python3 -I -B tools/citations/citations.py --base origin/main`.
6. **Commit and push.** Name the cause in the commit message: `ui-authorize-busy: <the cause, in a few words>`. Its body says why load made the story fail and where the fix sits.

## Report

Follow the implementer's text. Also give:

- the cause, its file and line, and the filtered log lines that show it, with their times;
- the decision (the product's or the story's) and why, in one line;
- each run of the story: its `date -u` start, outcome, wall time, test count and load (`nproc`, the loops, `uptime`), and how many passed before and after the fix;
- the focused test, the rule you broke by hand, and the failure you saw;
- every file you changed outside `files`, and why, with any overlap with `ui-ask-drop/stock` or `tui-link-colours/jig-expiry`;
- what you ruled out.
