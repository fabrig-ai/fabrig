+++
id = "repair"
part = "repair"
kind = "feature"
needs = []
files = ["temper/tests/client-browser/src/**", "temper/crates/temper-client-domain/src/authorize.rs", "temper/crates/temper-service/src/**", "jig/crates/jig-protocol-engine-clients/src/**", "jig/crates/jig-domain-people/src/**"]
effort = "high"
minutes = 120
+++

# The cross-site authorization story passes again

## Goal

`temper-client-browser-world`'s story `browser_authorize::a_cross_site_authorization_returns_a_code_and_the_links_state` (red-71) fails at step 7, near `temper/tests/client-browser/tests/browser_all/browser_authorize.rs:106`. It finds no level-1 heading "Authorization received". The page reads "temper couldn't sign you in just now" in its region "LLM account". Find why from the server's log and fix the cause where it sits. The story's checks stay as they are.

The story passed in every after-landing run up to main `2ce016c348` (2026-11-07 23:56Z). It failed at `bf198e50bb` (00:09Z) and `0c00e03c9d` (00:21Z), twice at each. The only code between them is `post-landing-probes/finding-438`: charter instructions in `temper/docs/examples/reference/settings.json` and engine tests. The browser world reads neither. The failure began just after midnight UTC, so a clock-bound cause is likely: a fixed date, an expiry, a certificate, or a token minted at a fixed time.

Acceptance:

- The report gives the cause in a few lines, with the server log's lines that show it. Every line has `secret=` values filtered out (see Steps, 1). It names the file and line where the cause sits.
- The fix is in the lowest place that holds the cause (`AGENTS.md`, 5: a skein or jig fault seen in temper is fixed and tested in skein or jig):
  - **A fixture's clock** (the browser world, `temper_fixtures`, a fake issuer, a key or certificate the world mints): make the value relative to the run's clock, never a fixed date.
  - **Product code:** first add a focused test that fails for the cause's reason, in the package that holds the cause. Make it a row of that package's existing table test where one covers the behaviour. Then fix the code.
  - **finding-438's change after all:** say so, and fix it the same way.
- The story passes when run by name, alone, at least twice in a row after the fix. The report gives both runs' outcomes and the times they ran (`date -u`).
- `git diff origin/main -- temper/tests/client-browser/tests/browser_all/browser_authorize.rs` changes none of the story's checks: no `see(...)`, `welcome()`, `Step::...`, `Find::...` or `assert...` line. Its `WITHIN` and its expected-error list stay too. If the cause sits in the story's own setup lines, see the options below.
- The focused tests of every package the fix changes pass, and so do their direct dependents' (Steps, 4). If the fix changes what a shared item means, not only its signature, the focused tests of every package that reads that item pass too.

## Inputs

- The request: `temper/docs/plan/ui-authorize-red/inputs/request.md`, read whole (it is short). Its `For every task of this plan` binds this task. That section stands in for a `notes/common.md`.
- The story: `temper/tests/client-browser/tests/browser_all/browser_authorize.rs`, and its world's fixture `temper/tests/client-browser/src/authorize.rs` (`Tier::authorize`, `Tier::authorize_callback`) with `Tier::reserved` and `Tier::setup` in `temper/tests/client-browser/src/lib.rs`.
- `temper/docs/design/protocol/sign-in.md`, 9.3: the authorization page. Its *not ready* or *busy* row is the page the story finds.
- `temper/docs/design/client/browser.md`, 4.5: the story's design.
- `jig/docs/design/protocol/client.md`, 18: jig's OpenID provider, which answers `Authorize`.

What the lead's reading found: where the page the story sees comes from. Nothing below was run; it narrows the search and settles nothing.

- The heading is `Word::AuthorizeBusy` (`temper/crates/temper-client-view/src/words.rs`). It is drawn for `temper_client_domain::authorize::State::Busy` (`temper/crates/temper-client-view/src/sign_in.rs`, `authorize`). The client domain sets that state for jig's `Authorized::NotReady`, `Authorized::Busy` and `Authorized::Lost` alike (`temper/crates/temper-client-domain/src/authorize.rs`, `answered`).
- `Lost` and `Busy` can come from the client's own link: `jig/crates/jig-client-protocol-link/src/authorize.rs` (an authorization already in flight, a full attempt table, an answer that doesn't decode) and `component.rs` (a failed exchange).
- `NotReady` and `Busy` come from the server, in `jig/crates/jig-protocol-engine-clients/src/provider.rs`:
  - `authorize_checked` answers `NotReady` while the component is stopping.
  - `authorize_reply` answers `NotReady` for every people reply but `Signer`.
  - `authorize_person` answers `NotReady` while stopping, and `Busy` when the code or token tables are full.
- jig's people answers `Whose` with `Again::NotReady` while its start is not open: loading, linked, sweeping, or failed (`jig/crates/jig-domain-people/src/domain.rs`, `Event::Whose` and `ready`). Its `person(sign_in, now, wall)` reads a sign-in's saved wall expiry. An expired sign-in answers *signed out*, not *busy*. The restart in the story's middle (`Step::StopServer`, `Step::StartServer`) restores sign-ins, and a restore with an expired sign-in to erase sweeps.
- None of these is known to be the cause. Read the log first.

Departures from the implementer's text, and why:

- **You run one browser story, by name and alone.** The lab's rules forbid a plan's runs the slow tiers, but allow a red's repair to run the flagged test by name, alone, to reproduce it and to prove the fix. This task is that repair (red-71). Run only this story, never the browser tier or another story. The command is in Steps, 1.
- **The fix may sit in jig or skein,** not only temper. The red is temper's, but the cause goes where it sits (`AGENTS.md`, 5). The branch then changes the tree that holds the cause, plus only the temper lines the fix forces (compiler-forced arms and call sites).

## Owns and may change

Owns, for the cause it finds:

- the browser world's fixtures, `temper/tests/client-browser/src/**`;
- `temper/crates/temper-client-domain/src/authorize.rs` and its tests;
- `temper/crates/temper-service/src/**` (the clients' composition, the provider's configuration);
- `jig/crates/jig-protocol-engine-clients/src/**` (the provider);
- `jig/crates/jig-domain-people/src/**` (sign-ins, `Whose`, the start).

Change only the files the cause and its fix need, and name them in the report. Name each file outside this list that you change, with the reason.

Reader and neighbour files:

- `temper/tests/client-browser/tests/browser_all/browser_authorize.rs`: only its setup lines (before `let mut person`), and only if the cause sits there. Never its checks (Goal, acceptance).
- A design status line, if the fix makes one stale: a **Departs** or **Not yet** line in the status file beside the section the fix touches. Edit that one line and name it in the report. Leave the design text itself alone.
- `.config/gate/read-by-tests.txt` names Markdown that tests read. Changing any of it means running the tests that read it.

Pre-approved options:

- **If the story passes alone on the pristine base:** the clock-bound cause may hold only part of the day. Run it once more alone, to have two passing logs. Then look for the cause in the paths above, aiming at a value bound to the UTC day or to a fixed time. If you find it, pin it with a focused test at a fixed `Wall` that shows it (for example, just after midnight), and fix it. If you find no cause, change nothing and report `Done: not reproduced`. Give both logs with their `date -u` times, the machine, and the paths you ruled out and why. After-landing runs the story again, and a red comes back as a repair.
- **If the cause is in the browser world or `temper_fixtures`:** fix the fixture there. No product test is needed. The story run by name is the check.
- **If the cause is a fixed time in the story's own setup** (its fake forge page, its query): make that line relative to the run's clock, and keep every check.
- **If the cause is in skein:** fix it in the skein crate that holds it, under a focused test there. Add that crate's files to the report's list of changed files.
- **If the cause is in a jig file `ui-account-again` changes** (`jig/crates/jig-protocol-engine-clients/src/translate/accounts.rs`, `jig/crates/jig-domain-accounts/**`, `jig/crates/jig-domain-core/src/routing/party/account_requests.rs`): make the smallest edit the fix needs and name the overlap in the report.
- **If the cause is the client's link** (`jig/crates/jig-client-protocol-link/**`, `jig/crates/jig-client/**`): fix it there under a focused test in that package, though the file list above doesn't name it, and say so in the report.
- **If the fix changes what a shared item means** (a reply, a readiness state, an expiry, not only a signature): also run the focused tests of every package that reads that item. Name those packages in the report.
- **If the fix makes a generated file stale** (a golden, a print, a journey recording): regenerate it with its own tool in this task, and name it in the report.

## Steps

1. **Reproduce.** Run the story by name, alone, through heavy:

   ```sh
   TIERS_WRAPPER=heavy tools/tiers/run temper browser -- -E 'test(=browser_authorize::a_cross_site_authorization_returns_a_code_and_the_links_state)'
   ```

   The runner builds the browser bundle, the playground bundle and the `temper` binary, then runs nextest's `browser-temper` profile with the filter (`tools/tiers/design.md`, 2.1, 2.2 and 3). Check that it selected exactly one test. Record `date -u` before the run.

   On failure, the harness's message holds the page's snapshot, Chromium's stderr and the server's stderr (`Server stderr:`, cleared at each restart, so it holds the restarted server's log). Read it through a filter, never whole and never raw: `grep -v 'secret='`, or `sed 's/secret=[^ &"]*/secret=…/g'`, then `grep` and `tail` around the authorize exchange. Nothing with `secret=` in it goes into the report, a commit or a test.

2. **Find the cause.** Follow the log to the answer the server gave (`NotReady`, `Busy`, `Lost`) and why, using the leads in Inputs. Compare against what changes at midnight UTC: a fixed date or epoch in a fixture, a `Wall` against a day bound, a minted key's, token's or certificate's validity. Say in the report what you ruled out.

3. **Fix it at its lowest place,** following the acceptance's rule for the cause's kind. For product code, write the focused test first and see it fail for the cause's reason. Then fix, and see it pass. This is the implementer's text's "a new test must bite".

   Run the narrowest test first: `heavy cargo nextest run -p PKG --lib -E 'test(=NAME)'`, or `--test BIN`.

4. **Check.**
   - Run the story twice more, alone, with the command in step 1. Both runs must pass. Record `date -u` before each.
   - `cargo fmt`.
   - `python3 -I -B <tree>/scripts/old_forms.py` for each tree the branch changes.
   - `heavy cargo clippy -p PKG ... --all-targets -- -D warnings`, naming every package you changed.
   - The focused tests of those packages and of their direct dependents, in every tree: `heavy cargo nextest run -p PKG ...`. Add `--profile fuzzy` if the package has fuzzy tests that touch the change. Check cross-tree dependents of a public change with `heavy cargo check -p <dependent> --tests`.
   - `python3 -I -B tools/citations/citations.py --base origin/main`.

5. **Commit and push.** Name the cause in the commit message: `ui-authorize-red: <the cause, in a few words>`. Its body says why the story began failing at midnight UTC on 11-08 (or that it didn't reproduce), and where the fix sits.

## Report

Follow the implementer's text. Also give:

- the cause, its file and line, and the filtered log lines that show it;
- each run of the story: its `date -u` time, its outcome, and how many tests it selected;
- the focused test you added, the rule you broke by hand, and the failure you saw;
- every file you changed outside `files`, and why;
- what you ruled out.
