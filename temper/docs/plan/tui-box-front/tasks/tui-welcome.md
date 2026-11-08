+++
id = "tui-welcome"
part = "tui-front"
kind = "feature"
needs = ["tui-surface", "domain-front"]
files = [
  "temper/crates/temper-client-tui/src/view.rs",
  "temper/crates/temper-client-tui/src/view/welcome.rs",
  "temper/crates/temper-client-tui/src/tests/welcome.rs",
  "temper/crates/temper-client-tui/src/tests/golden/welcome-80x24-dark.txt",
  "temper/crates/temper-client-tui/src/tests/golden/welcome-steps-80x24-dark.txt",
  "temper/docs/design/client/README.status.md",
]
effort = "high"
minutes = 120
+++

# The TUI's welcome in front of a box: the printed link, then the browser's steps followed

## Goal

The welcome of the request's frames 01 and 02 (`temper/docs/plan/tui-box-front/inputs/box-front-01.txt`, `-02.txt`; `temper/docs/design/client/README.md`, 6.17, *The welcome*): in front of a box, while `me` names no project, the TUI draws the welcome in the no-project launchpad's place, from the first frame: before the person signed in by the printed link, the link whole, what it does and the four steps none started; after, "Signed in as the owner" and the steps marked as the browser's welcome reaches them, the forge under way named, read-only.

Acceptance, by 6.17 (*Its pages*, *The welcome*, *The place*) and `notes/contracts.md`, sections 3 and 7:

1. **The dispatch** (`view.rs`' `build`, its `Body` and `signed`): in the signed-in branch (not the sign-in screen), `welcome::shown(root, tui)` picks `Body::Welcome(welcome::read(root, tui, words, wall))` before the address' match; `signed()` draws it with `welcome::draw`; its tick is none. `shown`: `tui.front()`, the root's address not `Chat { .. }`, and `me` unread or naming no project (`root.pages().me(root.client())`, read as `launchpad::read` reads it). Nothing else in `view.rs` changes.
2. **`view/welcome.rs`**, data then drawing, as `view/launchpad.rs` is (`read` builds a `Welcome` from the root and the TUI's state; `draw` lays it):
   - *The header,* region `look::TOP`: the level-1 heading "temper" (`TITLE`) and the place at the border's end, `tui.address()` (`ADDRESS`), with no project; then temper box's line, one paragraph: the view's `WelcomeBoxName` (`look::NAME`), `Word::Separator` (`FAINT`), `Word::BoxPurpose` (`HINT`); on an out-of-date link the view's `OutOfDateMain` in its place; the TUI's `NotKept` alert after it when `tui.unkept()`, as the launchpad's.
   - *Before the person signed in* (`!temper_client_domain::pages::welcome::signed_in(root)`), the sections (`SECTIONS`, `SPACED`), laid as frame 01: `Word::WelcomeOpen`; two columns in, the printed link (`tui.printed()`'s bytes, made UTF-8 by `forge::words::lossy`, drawn whole, no style); two columns in, `Word::LinkOnce` (`HINT`); then `Word::InTheBrowser` (`HEAD`) and, two columns in, the four steps on one line (`HINT`): `Word::Step` with 1 to 4 and the view's labels (`WelcomeStep`, `WelcomeForgeStep`, `WelcomeModelStep`, `WelcomeReadyStep`), joined by `Word::Separator`. With no printed link yet, the link and `LinkOnce` are left out.
   - *After,* laid as frame 02: `Word::Following` (`HEAD`), then a list, each an unnumbered item two columns in: `Word::SignedInOwner` (`STEP_DONE`), then the four steps, `Word::Step` with their numbers and labels, each `STEP_DONE` if ticked, `STEP_NOW` if current, else `STEP_AHEAD`, by `welcome::step_marks(at, welcome::forge_kept(root), welcome::model_usable(root) == Some(true))`, `at` from `welcome::decision(root)` (`Empty`: `Step::Welcome`; `Unfinished { step }`: `step`; `Whole`: `Step::Ready`); with no decision (facts not in), every step `STEP_AHEAD`. On the Forge step while current, after its words, `Word::Separator` and the forge under way (`HINT`, below). Then `Word::StepsFollowed` (`HINT`).
   - *The forge under way:* the first row of `root.forges()` (`State::Rows`, in number order) whose mode isn't `Mode::Fake` and whose number isn't the playground's (`root.forge().playground()`'s `forge`): when `forge::connect::underway(row)`: `SettingUp { step: Some(Step::Fetch) }` on Forgejo `Word::Fetching`; any other `SettingUp` on Forgejo `Word::SettingUp`; `Connecting` `Word::ConnectingForgejo` or `Word::ConnectingGitHub` by its kind, `SettingUp` on GitHub too; `Connected` on GitHub not accepted (`forges::accepted`) `Word::ConnectingGitHub`; `Connected` with its first repository `Making` `Word::FirstRepository`. Else, when `forge::connect::failed(row)` or its connection is `SetUpFailed { .. }`, `Word::SetUpStopped`. Else nothing, and no separator.
   - *Its notes,* after the sections, through `foot::read_notes` and `foot::draw_notes`, as the launchpad's (`tui-front-views` gives them `/log` and the give up: call them, don't change them).
   - The TUI never opens the link: the welcome binds nothing.
3. **Tests,** a failing test first for each (`notes/common.md`, *Tests stay cheap*), in `src/tests/welcome.rs`:
   - one table test, `the_welcome_says_each_state`, rows named, each the screen's text (as `tests/launchpad.rs`' `body` reads it) and the tokens where they matter: not in front, no project (the launchpad's "No project yet", no welcome); in front, connecting (`Local::Signing`, link signed out: the welcome, state "Connecting"); in front, no `link` line yet (no link, no `LinkOnce`); the link (drawn whole, compared by equality with the fixture's); a second `link` line (the latest drawn); signed in by `linked` with the facts not in (every step "·"); an empty store (Welcome current); a GitHub row connecting (Forge current, "connecting GitHub…"), a Forgejo fetching, setting up, its first repository making, a forge failed, and the playground's fake row alone (Welcome current, no forge words); a forge connected with no model (Model current, Welcome and Forge ticked); Ready current; `me` naming a project (the launchpad, not the welcome); out of date (the view's line in the header); a chat address in front (the chat).
   - one golden test, `the_welcomes_are_their_goldens`, rows `welcome-80x24-dark` (frame 01: the fixture's link wrapped at 80 columns, the steps none started, "● Live") and `welcome-steps-80x24-dark` (frame 02: signed in, Welcome ticked, Forge current with "connecting GitHub…", Model and Ready ahead), written by hand from the frames (no regenerate command: `tests/goldens.rs`' `check`), the place `127.0.0.1:8367` until `place` is on main (*Options*). The fixture's printed link has a placeholder secret, 32 zero bytes (`AAAA…`), so no golden holds a real secret.
   - in the same file, each welcome tree fits `limits()` (`super::limits::fits`), the link at 4,096 bytes among them.
   - A fixture at the bottom of the file, `welcome_root(..)`, answering the root's watches and reads through public deliveries as `tests/launchpad.rs`' `home_cards` does: `me` naming no project, the deployment read (welcome shown), the playground read (not served, or running), the accounts document (a usable account or none), the `forges` document with its rows and `linked`.
4. **Status:** delete the **Not yet** bullet naming `tui-welcome` in `temper/docs/design/client/README.status.md`. No other design line.
5. **Runs,** through `heavy`, counts in the report: `cargo nextest run -p temper-client-tui`; `cargo check -p temper-client-shell-tui --tests`.

## Inputs

- `temper/docs/plan/tui-box-front/notes/common.md`; `notes/contracts.md`, sections 3 and 7; the request's frames `inputs/box-front-01.txt` and `-02.txt` and `inputs/box-front-README.md`.
- `temper/docs/design/client/README.md`, 6.17 (*The welcome*, *The place*, *Its state, events, words and tokens*); 6.14 (*The launchpad*, *No project yet*); 3.5 (*The welcome*: *Its steps*, *The store's state*).
- The code: `view.rs` (`Body` 32, `build` 236-286, `signed` 288-305); `view/launchpad.rs` (the shape to follow: `Launchpad`, `read`, `sections`, `draw`); `view/foot/notes.rs` (`read_notes`, `draw_notes`); `temper-client-domain/src/pages/welcome.rs` (`decision`, `step_marks`, `forge_kept`, `model_usable`, and `domain-front`'s `signed_in`); `forge/connect.rs` (`underway` 278, `failed` 312); `forge/forges.rs` (`Connection`, `Step`, `FirstState`, `accepted`); `temper-client-view/src/words.rs` (`WelcomeBoxName`, `WelcomeStep`..`WelcomeReadyStep`, `OutOfDateMain`); tests: `tests/launchpad.rs` (`home_cards`, `body`, `the_launchpads_are_their_goldens`), `tests/goldens.rs` (`Golden`, `check`), `tests/limits.rs` (`fits`), golden `launchpad-none-80x24-dark.txt` (the nearest frame today).
- `tui-surface`'s items: the words, `STEP_*` tokens, `Tui::{front, printed}`, the stubs `view/welcome.rs` and `tests/welcome.rs`.

## Owns and may change

Owns `view/welcome.rs`, `tests/welcome.rs`, its two goldens, the dispatch in `view.rs` (`Body`, `build`, `signed`) and the status bullet. In the same wave `tui-front-views` changes `view/launchpad.rs`, `view/foot.rs`, `view/foot/lists.rs`, `view/foot/notes.rs`, `view/front.rs`, `tests/front.rs`, `tests/foot.rs`, `tests/launchpad.rs` and their goldens: not this task's; read them, change none. Not `temper-client-shell-tui` (`tui-seam`'s).

Options, each decided:

- **The place:** if `temper_client_tui::place` is on your base (`tui-launchpad-2`), draw `place(tui.address(), true)` and the goldens say `box`, and say so in the report; else `tui.address()` and the goldens say `127.0.0.1:8367`; leave the README.status bullet on the place as it is either way (it names no task of this plan).
- **The layout's rows:** match frames 01 and 02's rows and blank rows as skein lays your tree; where a blank row differs from the frame and no tree reaches it, keep skein's and name the difference in the report.
- **The step labels** are the view's words (`words.view().say(..)`), never copied into the TUI's phrasebook.
- **A link that isn't UTF-8** is drawn through `lossy`; one longer than the tree's bound can't come (`BoxLine::MOST` is the bound `tui-surface` gave `text_per_node`).

## Steps

1. The failing tests of acceptance 3 (the table first, against the stubs), seen failing; then `view/welcome.rs` and the dispatch; the narrowest test after each, `heavy cargo nextest run -p temper-client-tui -E 'test(/welcome::/)'`.
2. The goldens by hand, from the frames, then the test green.
3. The status edit; the runs of acceptance 5; `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-tui --all-targets -- -D warnings`; `python3 -I -B tools/citations/citations.py --base origin/main`.
4. Commit (why: the box's first frame is the welcome, with the link to open and the browser's steps followed), with `Look-request: tui-box-front` directly above the other trailers; push.

## Report

As the implementer's text says, with each test added and the rule broken by hand for it (for instance the welcome drawn out of front, or the step current taken from the browser's address), the place drawn, and each difference from the frames.
