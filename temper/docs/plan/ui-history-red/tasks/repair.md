+++
id = "repair"
part = "repair"
kind = "feature"
needs = []
files = ["temper/tests/client-browser/tests/browser_all/browser_pages.rs", "temper/crates/temper-client-view/src/**", "temper/crates/temper-client-domain/src/**", "temper/crates/temper-client-shell-web/src/**", "temper/docs/plan/ui-history-red/plan.md"]
effort = "high"
minutes = 150
+++

# The history story finds the Needs page at an unknown address every time

## Goal

`temper-client-browser-world`'s story `browser_pages::history_keeps_the_chat_focus_and_filter_across_reload_and_back` (red-78) sometimes fails at step 107 with `history: person 0, step 107: keep selector main.temper-needs attribute class: no element`. Step 107 is the `needs_page()` keep after `Step::Person(PersonStep::Go { address: b"/nowhere" })` and `see(Find::exactly_named(Role::Main, b""))`, at `temper/tests/client-browser/tests/browser_all/browser_pages.rs:182-183` on the trunk. Find the race by running the story, then fix it at its lowest place. The story's checks stay as they are.

The story began failing about when `look-round-22` landed (2026-11-08, 14:37Z). On 11-08 it failed and then passed alone at main `8d2cdcdcc6`, `da3a06bf55` and `703cbe11ae`, passed at `423669a32a` and `c049c710fd`, and failed twice at `0f6c343237`.

Acceptance:

1. **The race is recorded.** The report says which page draws at step 107 and when, from at least one failing run (the harness's failure message holds the page's snapshot). It says whether that is the empty busy main, "No project yet", another page, or the Needs page drawn and then replaced. It also says how many runs it took to fail, and under what load.
2. **The fix sits at its lowest place:**
   - **Only the story is early** (the page at step 107 is the empty main marked busy that the client draws while `me` is read, then the Needs page): the story waits for a mark the person's client draws only once `me` is read, before its unchanged `see(Main "")` and `needs_page()` lines. It no longer relies on any `main`. No product code changes.
   - **A person could see the wrong page flash at `/nowhere`** (a page other than the empty busy main, then the Needs page; or the Needs page, then another): the fix is the product's. First add a focused test that fails for that reason, as a row of the existing table where one covers the draw (Owns, options). Then make the client draw one page for an address once `me` is read, never another first.
3. **The story passes every time after the fix:** at least 10 runs by name, alone, under the conditions that reproduced the failure, every one passing. If the failure rate before the fix was below one in five, run at least 20. The report lists each run's `date -u` time and outcome, and checks that each run selected exactly one test.
4. `git diff origin/main -- temper/tests/client-browser/tests/browser_all/browser_pages.rs` changes none of the story's checks: every `see(...)`, `needs_page()`, `Step::Keep`, `Step::Address`, `Step::Focused`, `pressed(...)` and `assert...` line on the trunk stays as it is, in its order. The diff only adds steps that wait.
5. `temper/docs/plan/ui-history-red/plan.md`, section *Found*, says which case it was and how many runs passed after the fix (Steps, 5).
6. If the fix is the product's, the focused tests of every package the fix changes pass, and so do their direct dependents' (Steps, 4).

## Inputs

- The request: `temper/docs/plan/ui-history-red/inputs/request.md`, read whole (it is short). Its `For every task of this plan` binds this task and stands in for a `notes/common.md`. Where it and this spec differ, it wins.
- The story: `temper/tests/client-browser/tests/browser_all/browser_pages.rs`, `history_keeps_the_chat_focus_and_filter_across_reload_and_back`; its world `temper/tests/client-browser/src/lib.rs` (`playground_ready`, `connected`, `Tier::states`).
- `temper/docs/design/client/browser.md`, 4.6, item 2: the story's design ("An address that is no page opens the inbox at `/`").
- `temper/docs/design/client/README.md`, 5.1, the item *while `me` is read*: the top bar alone and "an empty main marked busy" until `me` is read.
- `skein/docs/design/ui-web.md`, 14.2: the browser harness's steps. A `Step::Keep` reads its selector once and doesn't wait (`skein/testing/skein-fake-person-browser/src/face.rs`, `attribute_reported`). A `See` waits up to its `within`.

What the lead's reading found. Nothing below was run; it narrows the search and settles nothing.

- **Since `b5d1bc6daf` ("Keep an empty busy main until me is read", look-round-22), the client draws `<main aria-busy>` with no name and no class while `me` is read** (`temper/crates/temper-client-view/src/root.rs`, `build`, the `reading` arm; `crate::outside::reading_me`). Before it, `c0fa375fe9` drew no main at all then. So `see(Find::exactly_named(Role::Main, b""))` met nothing until the page drew, and the keep after it found `main.temper-needs`. Now the busy main meets that `see` at once after a page load.
- **`/nowhere` is a page load** (`PersonStep::Go`), so `me` is read again. Nothing between the `Go` and the `see(Main "")` waits for `me`. If the keep runs before `me` arrives, `main.temper-needs` doesn't exist yet: the failure seen. The `/` step at lines 162-166 has the same window, behind `see(tier.states().live)` and `connected()`, which don't wait for `me` either.
- **The rail comes with the page.** `crate::frame::rail_project` (`temper/crates/temper-client-view/src/frame.rs`) needs `me` and a project. While `me` is read, `root.rs` draws `crate::outside::top` instead. The rail's "New chat" link (`frame.rs`, `Word::NewChat`, drawn unless the project's Home is `beginning`) is drawn in the same build as the Needs page's `main.temper-needs` (`temper/crates/temper-client-view/src/pages/needs.rs`, `main`). The story presses that link right after the `/` step's keep, so it is there in this story's state.
- **"No project yet"** (`crate::outside::no_project_main`) is drawn when `me` is read, no project is chosen and the deployment has answered. `crate::pages::project_changed` (`temper/crates/temper-client-domain/src/pages.rs`) picks `me`'s first project in the same handling as `me`'s arrival, so it shouldn't draw here. The run says whether it does.
- **A missing place** (`temper_client_domain::address::parse` gives `Place::Missing`) leaves the pages (`temper/crates/temper-client-domain/src/route.rs`, `went`). The story's last step expects the address `/`. Where the web shell replaces it, and whether any draw comes between, is the run's to show.

Departures from the implementer's text, and why:

- **You run one browser story, by name and alone, as often as you need.** The lab's rules forbid a plan's runs the slow tiers. They allow a red's repair to run the flagged test by name, alone, to reproduce it and to prove the fix. This task is that repair (red-78). Run only this story, never the browser tier or another story. The command is in Steps, 1.
- **You may make a load,** and add temporary instrumentation (Steps, 1 and 2), to see the race. Neither is committed. Neither drives the product as a walk or render check does: they're throwaway code, not a throwaway driver.
- **You write one section of the plan's `plan.md`** (Steps, 5): the request asks it to say which case it was.

## Owns and may change

Owns:

- `temper/tests/client-browser/tests/browser_all/browser_pages.rs`: the story's steps that wait. Never its checks (acceptance, 4).
- For the product case only: `temper/crates/temper-client-view/src/**` (`root.rs`, `outside.rs`, `frame.rs`, `pages/needs.rs` and their tests); `temper/crates/temper-client-domain/src/**` (`pages.rs`, `route.rs`, `address.rs` and their tests); `temper/crates/temper-client-shell-web/src/**` (where the shell replaces a missing address, and `focus.rs`).
- `temper/docs/plan/ui-history-red/plan.md`, its section *Found* alone.

Change only the files the cause and its fix need, and name them in the report. Name each file outside this list that you change, with the reason.

Reader and neighbour files:

- `temper/tests/client-browser/src/lib.rs`: only if the wait reads better as a helper beside `connected()`, used by this story alone. Then add that one function and nothing else.
- `temper/crates/temper-client-shell-web/tests/suite/host.rs` and `temper/crates/temper-client-view/src/tests/**`: rows and assertions a product fix makes stale, kept checking what they check for everything else.
- `temper/docs/design/client/README.status.md`: one **Departs** or **Not yet** line under 5.1, only if a product fix leaves the code otherwise than 5.1 says. Leave the design text itself alone.
- Prints and goldens a product fix makes stale (`temper-client-view`'s, the web shell's), regenerated by their own tool; a journey recording, by `temper-client-journeys again`.

Pre-approved options:

- **If the page at step 107 is the empty busy main (the default case):** add before line 182's `see(Find::exactly_named(Role::Main, b""))`, after the `/nowhere` `Go`:

  ```rust
  // The rail comes with the page once `me` is read; until then the main is empty and busy
  // (temper/docs/design/client/README.md, 5.1).
  see(Find::exactly_named(Role::Link, b"New chat")),
  ```

  Add the same `see` before line 165's `see(Find::exactly_named(Role::Main, b""))` in the `/` step, which has the same window. The comment goes once, on the first of the two in the file (the `/` step's); the `/nowhere` one is the bare `see`. If the run shows the rail's link isn't drawn there (a `beginning` Home, a phone viewport), wait instead for the first mark the Needs page itself draws once `me` is read, as an existing `Find`, and say which in the report.
- **If a busy main should not meet `See` for any story** (a reason you find in skein's `ui-web.md` or the fake person's design, not only this story): stop. That changes a shared item's meaning across every tree's browser stories. Fix this story by the default above, and name the wider point under `## Left`.
- **If "No project yet", another page, or a second page shows at `/nowhere` once `me` is read:** it's the product's. Pin it first with a focused test that fails for that reason. If the wrong draw is the view's, make it a row of `temper/crates/temper-client-view/src/tests/frame.rs`'s `the_top_bar_says_the_links_state_and_the_persons_name`. If it's the domain's (the project or the address after `Place::Missing`), use `temper/crates/temper-client-domain/src/tests/`. If it's the shell's replacement of the address, use `temper/crates/temper-client-shell-web/src/host_tests.rs`. Then fix it where the test points. The story's wait above is still added if the story is also early. Say so.
- **If the fix changes what a shared item means** (what `reading_me`, `no_project`, `rail_project` or `project_changed` return, an address the shell writes): also run the focused tests of every package that reads it (`temper-client-view`, `temper-client-shell-web`, `temper-client-shell-tui`, `temper-client-tui`, `temper-client-world`'s focused suite). Name those packages in the report.
- **If the story passes 20 runs alone without failing, loaded and unloaded:** widen the window with a temporary edit, never committed, that holds back `me`'s first delivery (for example a delay where the client dispatches the `me` watch's answer). Show the story fails at step 107 with it, and that the fix passes with it. Then remove it and run the 10 runs of acceptance 3 without it.
- **If neither the load nor the widening reproduces it:** change nothing, report `Done: not reproduced`, and give the runs' logs and times, the machine, and what you ruled out. After-landing runs the story again, and a red comes back as a repair.

## Steps

1. **Reproduce.** Run the story by name, alone, through heavy:

   ```sh
   TIERS_WRAPPER=heavy tools/tiers/run temper browser -- -E 'test(=browser_pages::history_keeps_the_chat_focus_and_filter_across_reload_and_back)'
   ```

   The runner builds the browser bundle, the playground bundle and the `temper` binary, then runs nextest's `browser-temper` profile with the filter (`tools/tiers/design.md`, 1 and 3). Check that it selected exactly one test. Record `date -u` before each run.

   If it won't fail in 10 runs unloaded, run it under a load you make on the machine the story runs on: CPU busy loops or `stress-ng --cpu N` beside it, stopped when the run ends. Say in the report what you used. Never stop heavy's units or touch its state.

2. **Record which page draws at step 107 and when.** A failing run's message holds the page's snapshot. Read it with `grep` and `tail` around `step 107`, never whole. For passing runs too, you may add a temporary, uncommitted `Step::Keep` right after line 182's `see(Main "")`, with a selector and attribute that tell the busy main from the Needs page (`main[aria-busy]`, `main`'s `class`). Print it from `harness.kept()`. Remove it before you commit.

3. **Fix it at its lowest place,** following acceptance 2 and the options. For a product fix, write the focused test first and see it fail for the cause's reason, then fix and see it pass: the implementer's text's "a new test must bite". Run the narrowest test first: `heavy cargo nextest run -p PKG --lib -E 'test(=NAME)'`, or `--test BIN`.

4. **Check.**
   - The story's runs of acceptance 3, with the command in step 1, under the conditions that reproduced it.
   - `cargo fmt`.
   - `python3 -I -B temper/scripts/old_forms.py`, and for each other tree the branch changes.
   - `heavy cargo clippy -p temper-client-browser-world --all-targets -- -D warnings`, and every other package you changed.
   - `heavy cargo check -p temper-client-browser-world --tests`.
   - For a product fix: the focused tests of the packages you changed and of their direct dependents (`heavy cargo nextest run -p PKG ...`). Add `--profile fuzzy` where the package has fuzzy tests that touch the change.
   - Commit, then `python3 -I -B tools/citations/citations.py --base origin/main` (it reads `HEAD`).

5. **Write `plan.md`'s *Found*.** Replace the section's one placeholder line with three to six lines. Say which case it was (the story early, or the product's flash), and the page seen at step 107 and when. Say the fix and its file, and how many runs passed after the fix, under which load. Change nothing else in `plan.md`.

6. **Commit and push.** Name the cause in the message: `ui-history-red: <the cause, in a few words>`. Its body says why the story began failing with look-round-22, or that it didn't reproduce, and where the fix sits.

## Report

Follow the implementer's text. Also give:

- the page at step 107, when it drew, and the failure's lines that show it;
- each run of the story: its `date -u` time, its outcome, how many tests it selected, and the load;
- for a product fix, the focused test you added, the rule you broke by hand, and the failure you saw;
- every file you changed outside `files`, and why;
- what you ruled out.

## Answers

2026-11-08. *May the run execute the flagged browser story, given the request's "only focused and fuzzy run" rule?* Yes. The user's own text grants the exception: the request's *One task*, item 1, asks for exactly these runs ("by running the story by name, alone, through the build wrapper, as a red's repair may, as often as needed, under a load you make"), and `AGENTS.md`, 5, under the same 11-06 rule, says "That repair may run the test it repairs, by name, alone." The blanket line in *For every task of this plan* governs every other slow-tier test and every other task; it does not take back the item that names this one. So run `browser_pages::history_keeps_the_chat_focus_and_filter_across_reload_and_back` alone, by its exact name, with Steps, 1's command (`TIERS_WRAPPER=heavy tools/tiers/run temper browser -- -E 'test(=...)'`), as often as acceptance 1 and 3 need, loaded as Steps, 1 says. Check that each run selected exactly one test. Never run the browser tier unfiltered, another story, or another slow tier. Acceptance stays as written.
