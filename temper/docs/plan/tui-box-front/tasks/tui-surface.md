+++
id = "tui-surface"
part = "tui-front"
kind = "interface"
needs = []
files = [
  "temper/crates/temper-client-tui/src/box_line.rs",
  "temper/crates/temper-client-tui/src/lib.rs",
  "temper/crates/temper-client-tui/src/state.rs",
  "temper/crates/temper-client-tui/src/event.rs",
  "temper/crates/temper-client-tui/src/limits.rs",
  "temper/crates/temper-client-tui/src/words.rs",
  "temper/crates/temper-client-tui/src/look.rs",
  "temper/crates/temper-client-tui/src/theme.rs",
  "temper/crates/temper-client-tui/src/view.rs",
  "temper/crates/temper-client-tui/src/view/work.rs",
  "temper/crates/temper-client-tui/src/view/welcome.rs",
  "temper/crates/temper-client-tui/src/view/front.rs",
  "temper/crates/temper-client-tui/src/tests.rs",
  "temper/crates/temper-client-tui/src/tests/box_line.rs",
  "temper/crates/temper-client-tui/src/tests/work.rs",
  "temper/crates/temper-client-tui/src/tests/welcome.rs",
  "temper/crates/temper-client-tui/src/tests/front.rs",
  "temper/crates/temper-client-tui/src/tests/state.rs",
  "temper/crates/temper-client-tui/src/tests/limits.rs",
  "temper/crates/temper-client-tui/src/tests/theme.rs",
  "temper/crates/temper-client-tui/src/tests/launchpad.rs",
  "temper/crates/temper-client-tui/src/tests/foot.rs",
  "temper/crates/temper-client-shell-tui/src/host.rs",
  "temper/docs/design/client/README.status.md",
]
effort = "high"
minutes = 120
+++

# The TUI's surface in front of a box: the box's lines, the front's state, events, words and tokens

## Goal

The interface step of the TUI in front of a box (`temper/docs/design/client/README.md`, 6.17): everything of `temper-client-tui` that the host's seam (`tui-seam`) and the two view tasks (`tui-welcome`, `tui-front-views`) build on, so that they touch disjoint files. No screen draws anything new here: the views come after.

Acceptance, by `notes/contracts.md`, sections 6 and 7, and 6.17 (*The box's lines*, *Quitting*, *Its state, events, words and tokens*):

1. **`box_line.rs`,** re-exported at the crate root as `BoxLine`, exactly the contracts' section 7: `pub enum BoxLine { Link { url: skein_lib::Secret }, Log { line: Box<[u8]> }, Restarting { step: Box<[u8]> }, Running { step: Box<[u8]> }, GaveUp { step: Box<[u8]> } }`, deriving `Clone, PartialEq, Eq, Debug` (the `Secret`'s `Debug` shows its length alone); `pub const MOST: usize = 8192`; `pub fn read(line: &[u8]) -> Option<BoxLine>`: the word up to the first space (`link`, `log`, `restarting`, `running`, `gave-up`), the field everything after that space, at least one byte (the box never writes an empty one); `None` for an unknown word, no space after the word, an empty field, or a line of `MOST` bytes or more (the bound counts the line feed `read` never sees); `pub fn print(&self) -> Box<[u8]>`: the word, one space, the field, which `read` takes back.
2. **`state.rs`:** `Told` gains, after `Kept`, `Front`, `BoxLine { line: BoxLine }`, `Quitting`, `Stay` and `Log` (`BoxLine`, not `Boxed`: `tui-launchpad-2` adds a unit `Told::Boxed`); `pub enum BoxState<'a> { Running, Restarting { step: &'a [u8] }, GaveUp { step: &'a [u8] } }` (`Clone, Copy, PartialEq, Eq, Debug`), re-exported at the crate root; `Tui` gains the contracts' accessors, `front()`, `printed()`, `box_log()` (oldest first), `box_state()`, `quitting()`, `log_shown()`. `Tui::step`:
   - `Front` sets front; `BoxLine`: `Link` replaces the printed link, `Log` appends and drops the oldest past `limits::BOX_LOG`, `Restarting` sets that step restarting, `Running` clears it when it is that step, `GaveUp` sets the step given up once, for good; `Quitting` raises the question, `Stay` lowers it; `Log` raises `/log`'s note; and the existing `Unknown { .. }` and `Went` also lower `/log`'s note (it stands "until the next send or screen", and the host tells `Unknown { words: None }` at each send and `Went` at each screen).
   - `box_state()`: `GaveUp` once given up, whatever restarts; else `Restarting` while a step is; else `Running`.
   - each changes the revision exactly when it changes the state (a `Log` line always does).
3. **`event.rs`:** `Event` gains `Stop`, `Stay`, `Log`, after `Dismiss`; and `pub const LOG_COMMAND: &[u8] = b"/log";` re-exported at the crate root, the name of the TUI's own command (6.17, *`/log`*), which the host's send and the views' lists read.
4. **`work_runs`** (`view/work.rs`, a module of `view.rs`, re-exported at the crate root through `view.rs`), with its body (6.17, *`work_runs`*): `pub fn work_runs(root: &temper_client_domain::Domain, _tui: &Tui) -> bool` (the contract's signature; the TUI's state is not read yet) is `running(root).count > 0`, and `pub(crate) fn running(root: &Domain) -> Running`, `pub(crate) struct Running { pub count: u32, pub title: Option<Box<[u8]>> }`, which `tui-front-views` reads for the question's head. A chat works while it has not ended (`pages::chats::ended(&chip)` false, and for the open chat no `result`) and its `jig_client::facts::chat_status(chip, waits_on, run_wait, project, me)` is `Starting`, `Writing`, `Tool { .. }`, `Working { .. }` or `Retrying { .. }`, or its chip's `details.live_delegates > 0`. Read: the open chat (`root.pages().chat()` and its `task(root.client())`, `run_wait` its own) and Home's chats (`root.pages().chats()` → `live(root.client())`, `run_wait` none), each in `root.pages().project()`, a task counted once. `title`: the open chat's when it works, else the working chat of the latest `last_activity`, its `chip.title` bytes. With no `me`, no project or nothing read, none.
5. **Words** (`words.rs`), every new word of 6.17, the TUI's own under its owner, numbered 60 on in this order, English exactly as here (`…` is U+2026; apostrophes ASCII), each added to `ALL`, with the phrasebook's capacities (`List::with_capacity(60)`, `PhraseLimits.phrases: 60`) following `ALL`'s length:

   | Word | English |
   |---|---|
   | `BoxPurpose` | `on this machine, to try it` |
   | `WelcomeOpen` | `Welcome. Set temper up in your browser: open this link on this machine.` |
   | `LinkOnce` | `It signs you in as this temper's owner, once, within 2 hours.` |
   | `InTheBrowser` | `In the browser` |
   | `Step` | NUMBER ` ` LABEL (holes `Slot::Number`, `Slot::Label`) |
   | `Following` | `Setting temper up in your browser` |
   | `SignedInOwner` | `Signed in as the owner` |
   | `StepsFollowed` | `The steps are the browser's; this screen follows them.` |
   | `Fetching` | `fetching Forgejo…` |
   | `SettingUp` | `setting up Forgejo…` |
   | `ConnectingForgejo` | `connecting Forgejo…` |
   | `ConnectingGitHub` | `connecting GitHub…` |
   | `FirstRepository` | `making its first repository…` |
   | `SetUpStopped` | `stopped: the browser says why` |
   | `Playground` | `playground` |
   | `PlaygroundBanner` | `This is the playground: a scripted model and in-memory forge. Set temper up in your browser when you're ready to leave it.` |
   | `QuitOne` | `Stop the box? ` TITLE `'s running task stops with it.` (hole `Slot::Title`) |
   | `QuitMany` | `Stop the box? ` NUMBER ` running tasks stop with it.` (hole `Slot::Number`) |
   | `QuitNone` | `Stop the box?` |
   | `StopBox` | `Stop the box` |
   | `DontQuit` | `Don't quit` |
   | `Restarting` | STEP ` restarting` (hole `Slot::Step`) |
   | `GaveUp` | STEP ` gave up` (hole `Slot::Step`) |
   | `GaveUpNote` | `The box gave up: ` STEP `. /log shows why; Ctrl+C closes.` (hole `Slot::Step`) |
   | `LogCommand` | `The box's recent log` |
   | `LogHead` | `/log` |
   | `LogEmpty` | `Nothing yet.` |
   | `ExitFront` | `Close temper and stop the box` |
   | `KeyCloseFront` | `Ctrl+C closes temper and stops the box` |

   New `Slot`s, each `HoleKind::Name`: `Step`, `limits::STEP` (64) bytes; `Label`, 32 bytes (the view's step labels); `Title`, `domain.client.domain.text` bytes (a chat's title; `Slot::bytes` already takes the domain's limits).
6. **Look tokens** (`look.rs`), appended after `COMMAND` in `NAMES`' order, each a `pub const`, and their looks in `theme.rs`'s `styles`, in the same order, as 6.17's table says: `STEP_DONE` `tui-step-done` (no paint; mark `✓`, good), `STEP_NOW` `tui-step-now` (bold; mark `▸`, accent), `STEP_AHEAD` `tui-step-ahead` (dim; mark `·`, faint), `PLAYGROUND` `tui-playground` (accent, `Align::End`), `END_FAINT` `tui-end-faint` (faint, `Align::End`).
7. **Limits** (`limits.rs`): `pub const BOX_LOG: u32 = 50`, `pub const LOG_LINE: u32 = 512` (a log line drawn, cut there), `pub const STEP: u32 = 64` (a step's name drawn, cut there); and `limits()` grows for the trees the views will build: `text_per_node` at least `BoxLine::MOST` (the printed link drawn whole), `nodes` by at least 192 (the welcome's lines, the question, `/log`'s note of `BOX_LOG + 1` lines and the give up's note), `text_bytes` by at least `BOX_LOG * LOG_LINE + MOST + 4096`, `bindings` by one (`/log` among the commands). The new words' capacities join `own` through `ALL`, as today.
8. **Stubs for the views,** so that `tui-welcome` and `tui-front-views` share no file: `view.rs` gains `pub(crate) mod front;`, `pub(crate) mod welcome;` and `mod work; pub use work::work_runs;` beside its other modules (four lines, nothing else in `view.rs`); `src/view/welcome.rs` and `src/view/front.rs` hold one `//!` line each (6.17's *The welcome*; *The playground*, *Quitting* and *The box's state and `/log`*); `src/tests.rs` gains `mod box_line; mod front; mod welcome; mod work;`, and `src/tests/welcome.rs` and `src/tests/front.rs` hold one `//!` line each. `tests/limits.rs`' `fits` becomes `pub(super)`, and `tests/launchpad.rs`' `home` and `line` and `tests/foot.rs`' `root_state` and `TaskState` become `pub(super)`, so that the view tasks' rows and `tests/work.rs` reuse them.
9. **The host's forced arms** (`temper-client-shell-tui/src/host.rs`): `fn event`'s match gains `tui::Event::Stop | tui::Event::Stay | tui::Event::Log => {}`, nothing else; `tui-seam` gives them their bodies.
10. **Tests,** a failing test first for each:
    - `src/tests/box_line.rs`, one table test, `each_box_line_reads_and_prints_back`: rows for each of the five words, a field with spaces, an empty field (dropped), an unknown word (`stop forgejo-3`), a word alone (`log`), a word with no space (`logx`), a line of `MOST - 1` bytes read and of `MOST` dropped, non-UTF-8 field bytes kept as they are; each read line's `print` equals it. The link's row names its URL in words and compares by equality, never printing it (`notes/common.md`, *Never shown*).
    - `src/tests/state.rs`, one new test, `the_front_states_steps_are_its_table` (rows as `the_surface_state_changes_only_when_its_facts_change`'s, named): front; a link, the same link (no revision), another link; 51 logs keeping the last 50 oldest first; restarting, running for another step (stays), running for it (cleared); gave up winning over restarting and a second `GaveUp` changing nothing; quitting, the same, stay; `Log`, then `Unknown { words: None }` lowering it, then `Log` and `Went` lowering it.
    - `src/tests/work.rs`, one new table test, `work_runs_reads_the_projects_working_chats`: rows none read; a Home chat idle waiting on the person (false); a Home chat working (`ChipPhase::Active(RunStage::Due)`, `WaitsOn::Nothing`: true, its title); one with a live delegate (true); one ended (false); one in another project (false); two working (count 2, the latest's title); the open chat working (true, its title), built with `tests/foot.rs`' `root_state(.., TaskState::Working)` and `tests/launchpad.rs`' `home`/`line` (their chip and `waits_on` changed in the row).
    - `src/tests/limits.rs`' `the_trees_fit_their_limits`: a row per new word with its holes and its English (the hole rows at their bound: a 64-byte step, a `domain.client.domain.text`-byte title, `u64::MAX` for the numbers).
    - `src/tests/theme.rs`' `the_themes_are_valid_and_name_every_look`: a row per new token in both schemes.
11. **Status:** delete the **Not yet** bullet naming `tui-surface` in `temper/docs/design/client/README.status.md`. No other design line.
12. **Runs,** through `heavy`, counts in the report: `cargo nextest run -p temper-client-tui`; `cargo nextest run -p temper-client-shell-tui` (the forced arms only); `cargo check -p temper --tests` and `cargo check -p temper-client-world --tests` (readers of the crate, unchanged).

## Inputs

- `temper/docs/plan/tui-box-front/notes/common.md`; `notes/contracts.md`, sections 6 and 7 (and 8, to see how `tui-seam` uses this).
- `temper/docs/design/client/README.md`, 6.17, whole; 6.14 (*The TUI's state*, *Its words*); 6.15 (the tokens' table, which 6.17's extends).
- The code: `temper-client-tui/src/state.rs` (`Tui` 7-26, `Told` 53-92, `step` 255-354, the existing `Unknown` and `Went` arms); `event.rs`; `lib.rs`; `words.rs` (`Word` 12-70, `ALL` 71-129, `Slot` 131-163, `number` 166-228, `english` 232-290, `Words::english` 322-395); `look.rs`; `theme.rs` (`styles` 103-138); `limits.rs` (`limits`); `view.rs` (modules 14-16); `view/launchpad.rs` (`left_off` 79-98, how Home's chats are read); `temper-client-view/src/pages/terminal.rs:220-252` (`running_words`, the browser's reading of the open chat's work); `jig/crates/jig-client/src/facts.rs:20-45` (`ChatStatus`, `chat_status`); `temper-client-domain/src/pages/chats.rs:586` (`live`) and `:643` (`ended`); tests: `src/tests/state.rs` (`the_surface_state_changes_only_when_its_facts_change` 264), `limits.rs` (`fits`, `the_trees_fit_their_limits`), `theme.rs` (`the_themes_are_valid_and_name_every_look`), `launchpad.rs` (`home` 107, `line` 69), `foot.rs` (`TaskState` 213, `root_state` 68, `set_state` 224); `temper-client-shell-tui/src/host.rs:350-385` (`fn event`).

## Owns and may change

Owns everything listed in `files` but `host.rs`, where it adds the forced arms alone, and `tests/launchpad.rs` and `tests/foot.rs`, where it changes visibility alone. `view/welcome.rs` and `tests/welcome.rs` are `tui-welcome`'s after this; `view/front.rs` and `tests/front.rs` are `tui-front-views`'; leave them a doc line each. Not `view/launchpad.rs`, `view/foot.rs`, `view/foot/*`, `view/chat*`, or `build`'s body in `view.rs`.

Options, each decided:

- **`tui-launchpad-2` on your base:** its `look` task adds four tokens after `COMMAND` (`tui-bullets`, `tui-left-off`, `tui-picked`, `tui-rule-title`), a unit `Told::Boxed`, `Tui::boxed()` and `theme(scheme, background)`. If they are on your base: append this task's tokens after the last token there, give `theme`'s new looks whatever its signature is then, and add `Told::BoxLine` beside its `Told::Boxed`, each its own. Otherwise build to the contracts as written.
- **The log's container:** a `skein_lib::List<Box<[u8]>>` or `Queue` of at most `BOX_LOG`, whichever gives `box_log()` oldest first without indexing; say which.
- **`work_runs`' `_tui`:** unused, with the leading underscore; if clippy objects to the public function's unused parameter, a scoped `#[expect(.., reason = "the contract's signature keeps the TUI's state for the front's later reads")]`.
- **A word whose English collides** with the view's (`temper-client-view`'s words) is still the TUI's own: the TUI says the view's phrase only where 6.17 names the view's (`WelcomeBoxName`, the steps' labels), which the views read from `words.view()`.

## Steps

1. The failing tests of acceptance 10, seen failing (or not compiling) for the missing items; then `box_line.rs`, `state.rs`, `event.rs`, `view/work.rs`, `words.rs`, `look.rs`, `theme.rs`, `limits.rs`, the stubs and re-exports, the host's arms; the narrowest test after each, e.g. `heavy cargo nextest run -p temper-client-tui -E 'test(/box_line::/)'`.
2. The status edit; the runs of acceptance 12; `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-tui -p temper-client-shell-tui --all-targets -- -D warnings`; `python3 -I -B tools/citations/citations.py --base origin/main`.
3. Commit (why: the TUI's surface in front of a box, so the seam and the views build apart), with `Look-request: tui-box-front` directly above the other trailers; push.

## Report

As the implementer's text says, with each test added and the rule broken by hand for it (for instance `read` keeping a line of `MOST` bytes, or `Unknown` leaving `/log`'s note), the log's container chosen, and anything under `## Left`.
