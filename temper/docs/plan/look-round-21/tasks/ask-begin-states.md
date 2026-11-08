+++
id = "ask-begin-states"
part = "begin"
kind = "feature"
needs = []
files = [
  "temper/tools/gallery/src/samples.rs",
  "temper/tools/gallery/src/worlds.rs",
  "temper/tools/gallery/src/tests.rs",
  "temper/docs/design/client/browser.status.md",
]
effort = "high"
minutes = 70
+++

# The gallery's first-run Ask states draw an unfinished To begin

## Goal

The request's task 2 (`temper/docs/plan/look-round-21/inputs/request.md`). `ask-first-run`, `ask-typing`, `ask-sending`, `ask-refused` and `night-ask` still draw the first run's whole rail, Needs you, Chats, "+ New chat" and More ways to start work, at 1440 and 1000, and no tabs at 420; live, Start gives the restricted rail and three tabs. Their world delivers no unfinished To begin, so `temper_client_domain::pages::home::beginning` is false on their page. Give the five states the facts of an unfinished To begin, the way of working left, as `home-begin-ask-left` gives Home, so that the view draws what a fresh project shows. Gallery code alone: the view and the domain are right live.

Acceptance, by `temper/docs/design/client/browser.md`, 7.2, the five rows:

1. **The facts:** for each of the five, `temper_client_domain::pages::home::beginning(&root, 1, wall)` holds on `Address::Ask { project: 1 }`, with `Facts::left()` counting the way of working (`temper/crates/temper-client-domain/src/pages/home.rs`, about lines 700 to 770: the person's dismissals delivered with no `ToBegin`, the forges known, the ways ended or delivered, the project's chats delivered with no Ask temper chat, the settings not yet followed). Each keeps what it holds today: template 5 listed, "Ways to begin", the words typed, the start out, the refusal.
2. **`night-ask`** is `ask-first-run` with the night switch pressed: its base in `samples::domain` (about line 1433), `"configure"`, which names no state and so takes the generic world, becomes `"ask-first-run"`.
3. **Nothing else moves:** `ask-reference` and `ask-reference-draft`, the same world at Ask's address, keep their facts (the render check passed them; their rail is folded), and every other state its tree. The facts go to the five alone: by name in `ask_domain`, or by a flag of the world builder that only they pass.
4. **The rows,** in the existing table `HOLDS` (`temper/tools/gallery/src/tests.rs`, read by `each_state_holds_what_its_name_says`): each of the five rows gains the parts `home-begin-ask-left`'s row has for the frame,

   ```rust
   Part::Links { style: temper_client_view::look::RAIL_NAV, names: &["Home", "Ask temper"] },
   Part::Links { style: temper_client_view::look::TABBAR, names: &["Home", "Ask temper", "Settings"] },
   Part::NoStyle { style: temper_client_view::look::RAIL_NEW },
   ```

   keeping its own. No new test function. For the bite: drop the facts once (step 1's change undone), see `each_state_holds_what_its_name_says` fail on these parts, restore; the report names the failure seen.
5. **The print:** the gallery's print is made by its tests and never committed; print it into a scratch directory under your `target/` before your change and into another after it, `heavy cargo run -p temper-client-gallery -- OUT`, and compare: the five states' pages alone differ. Delete both before the push; the report lists the pages that differed.
6. **Status:** delete the **Not yet** entry naming `ask-begin-states` in `temper/docs/design/client/browser.status.md`. No other design line.
7. Runs through `heavy`, counts in the report: `cargo nextest run -p temper-client-gallery`, within 1 s of the default suite together (`browser.md`, 7.3); `cargo clippy -p temper-client-gallery --all-targets -- -D warnings`.

One commit or more, each with the trailer `Look-request: to-begin-two-steps`; the last carries the account (below).

## Inputs

- `temper/docs/plan/look-round-21/notes/common.md`.
- `temper/docs/plan/look-round-21/inputs/request.md`, task 2.
- `temper/docs/design/client/browser.md`, 7.2: the rows of the five, of `home-begin-ask-left` and `home-begin-both-left`; 7.3.
- `temper/docs/design/client/README.md`, 5.3, *While To begin shows*.
- `temper/crates/temper-client-domain/src/pages/home.rs`: `Facts`, `to_begin`, `left`, `beginning`, `facts` (about lines 700 to 790).
- `temper/tools/gallery/src/samples.rs`: `domain`'s night bases (about line 1431), `ask_domain` (about line 1958), `home_domain` (about line 2209) and its deliveries (the `Dismissed`, `Forges`, `Settings` and `Tasks` answers, about lines 2320 to 2400), which give Home these facts.
- `temper/tools/gallery/src/worlds.rs`: `world`, `world_in_layout` (about line 395), its family watches (about lines 709 to 760).
- `temper/tools/gallery/src/tests.rs`: `HOLDS` (the five rows, about lines 150 and 2130 to 2200; `home-begin-ask-left`'s), `asking_gallery_states_list_the_template_and_send_readiness` (about line 3668).

## Owns and may change

Owns `ask_domain`, the night base for `night-ask`, and what the five states need in `worlds.rs`; the five `HOLDS` rows; the status entry. Not the view or the domain; not another state's row or deliveries; not `names.txt` (no name changes).

Options, each decided:

- **Where the facts are delivered:** after `worlds::world(...)` in `ask_domain`, deliver what the root still lacks to the watches it holds open, as `home_domain` delivers them; if `world_in_layout` answers or drops those watches before returning (the `Dismissed` watch among them), add a flag to it or a sibling of `world` in `worlds.rs` (`to_begin_world(address, then) -> Domain`, say) that answers them for the five, and say which.
- **A test reads the old frame:** `asking_gallery_states_list_the_template_and_send_readiness` or another gallery test that relied on the whole rail for these states keeps what it checks for every other state, its rows adjusted, named in the report.
- **More ways to start work** sits outside `RAIL_NEW`: add `Part::Absent` for it to the five rows, by its role and name as the view draws it (`temper/crates/temper-client-view/src/frame.rs`, about line 662).
- **The way of working is already counted done** in that world (a setting followed, or an Ask temper chat counted): deliver the settings `Default` and no Ask temper chat for these five, as `home-begin-ask-left`'s deliveries do.

## Steps

1. Read the request's task 2 and the Inputs; find which of `Facts`' inputs the world leaves unanswered at Ask's address.
2. Acceptance 1 to 3, then the rows (4) and their bite; then the print's comparison (5).
3. The status entry (6); the runs (7); `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `python3 -I -B tools/citations/citations.py --base origin/main`.
4. Commit with the trailer and the account; push with `git push origin HEAD:refs/heads/<your branch>`.

The account's rows, at least:

| Item | Result |
|---|---|
| `ask-first-run`, `ask-typing`, `ask-sending`, `ask-refused`: the restricted rail at 1440 and 1000, three tabs at 420 | landed (`ask_domain`, the facts named) |
| `night-ask`: the same, at night | landed (its base `ask-first-run`) |
| Their prints | made by the gallery's tests; the five pages alone differ |
| Render check | follows landing, temper/ui's (`temper/docs/design/draft/look-requests.md`, 4): compiling and focused tests establish no rendered equality |

## Report

The implementer's report: the facts each state lacked and how they're delivered, the options taken, the rows changed and their bite, the pages that differed in the print, the runs' counts, and the account as committed.

## Answers

2026-11-08. **The gallery's tests over their 1 s (`browser.md`, 7.3): a follow-up cost repair, or widen this task to optimize the whole gallery?** A follow-up; this task is not widened. The overrun is inherited (pristine base 2.909 CPU s, this change 2.644 s, `heavy cargo test -p temper-client-gallery --lib`, serial): the change lowers the cost, and acceptance 7's budget clause is met for this task by not raising it. Its cause lies in the all-state table and the print, beyond the five states this task owns. The commit `28bcd334e2` stands as pushed; keep its account row "differs: inherited overrun". In addition, so the overrun is carried by the design status and not by this request (`temper/docs/design/draft/look-requests.md`, 5), add one **Departs** entry to `temper/docs/design/client/browser.status.md`: 7.3's cost, the two measurements above with the command, the two costly tests (`each_state_holds_what_its_name_says`, the print check), and its owner, a gallery cost repair (testing-strategy 8.1, question 2: find and fix the cause; only failing that, measurements to the user). This entry is the one design line the task adds besides the deletion of acceptance 6. Rerun nothing but `python3 -I -B tools/citations/citations.py --base origin/main`; commit with the trailer and push.

## Rebase (2026-11-08, step merge, foreman look-round-21/ask-begin-states/foreman-2)

resolve browser.status.md by keeping both tasks’ completed-entry deletions and the gallery cost departure.
The merge conflicts only in `temper/docs/design/client/browser.status.md`.
Integration `f98a1d0a62` includes `tab-room` commit `85db8975cf`, which removes its **Not yet** entry.
Rebase onto the landing’s current integration base; preserve that deletion.
Also preserve `28bcd334e2`’s deletion of the `ask-begin-states` **Not yet** entry.
Keep `c1383ae592`’s **Departs:** 7.3 paragraph intact, including both CPU measurements, command, costly tests and gallery cost repair owner.
The resolved opening should contain neither task’s **Not yet** entry, followed by the gallery cost departure and the existing status prose.
Keep all other status text and gallery code unchanged.
Run `python3 -I -B tools/citations/citations.py --base origin/main`; the answer requires no other reruns.
