+++
id = "close-words"
part = "close-words"
kind = "feature"
needs = ["chat-pause"]
files = [
  "temper/crates/temper-client-domain/src/pages.rs",
  "temper/crates/temper-client-domain/src/pages/task.rs",
  "temper/crates/temper-client-domain/src/pages/goal.rs",
  "temper/crates/temper-client-domain/src/pages/goals.rs",
  "temper/crates/temper-client-domain/src/tests/pages/goal.rs",
  "temper/crates/temper-client-domain/src/tests/pages/task.rs",
  "temper/crates/temper-client-view/src/confirm.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/pages/agent.rs",
  "temper/crates/temper-client-view/src/pages/goal.rs",
  "temper/crates/temper-client-view/src/pages/goals.rs",
  "temper/crates/temper-client-view/src/pages/task.rs",
  "temper/crates/temper-client-view/src/tests/**",
  "temper/crates/temper-client-shell-terminal/src/tests/**",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/tests/client/tests/suite/task.rs",
  "temper/tools/gallery/**",
  "temper/docs/design/client/journeys/**",
  "temper/docs/design/client/README.status.md",
]
effort = "high"
minutes = 100
+++

# A goal's and an agent's task's Close, and closed's words

## Goal

"Stop" goes as a person's word: a goal's and an agent's task's end says "Close", on the chat's one sheet, and an ended goal says
closed's words on rows, pages, Home and filters. An agent's task keeps what its "Stop" did, as "Pause" and "Resume", and gains
"Close", so its acts are a goal's. The journeys that print any of this are re-recorded.

The request says an agent's task's "Stop task" becomes "Close task", but that Stop is jig's hold, not an end. This plan's call:
"Stop task" takes its place as a real close (`Ask::Cancel`), and the hold it was stays, as Pause.

Acceptance:

- **A goal:** "Amend", "Pause" or "Resume", "Close"; its sheet "Close this goal?", "You can still read it, but it can't be
  reopened.", "What landed on its branch stays there.", "Keep it", "Close goal"; `Ask::Cancel` with the reason "Closed from its
  page".
- **An agent's task:** "Amend", "Pause" or "Resume", "Close". Pause and Resume sent at once, no sheet; the state "Paused"; no own
  held card on the person's own pause, and its card as today on any other hold. Close's sheet "Close this task?", "You can still
  read it, but it can't be reopened.", what it changes, "Keep it", "Close task"; `Ask::Cancel` with that same reason.
- **Closed's words:** a goal's state "closed", "Closed by you.", "Closed by you. What landed on the goal branch stays there.",
  "Closed before it landed." everywhere an ended goal says them. Ended goals keep their meaning; the filter "Ended" keeps its name.
- **No person's "Stop" left in `temper-client-view`:** `rg` over its `src` finds none but what `notes/common.md` keeps.
- **The gallery** follows `notes/contracts.md`'s names, `names.txt` sorted.
- **The journeys** that print a changed page re-recorded and reprinted, their tests green.
- **The status:** this request's **Not yet** entry gone from `temper/docs/design/client/README.status.md`.

## Inputs

- `temper/docs/plan/ui-pause-close/notes/common.md`.
- `temper/docs/plan/ui-pause-close/notes/contracts.md`: all of it. It binds. `chat::paused` is on the trunk from `chat-pause`.
- `temper/docs/design/client/README.md`, 3.5 (*Goals*: its states and headlines), 3.6 (*The sheet*), 5.8, 5.11.
- `temper/docs/design/client/browser.md`, 7.2.
- `temper/docs/design/client/journeys.md`, 8 and 8.1.

## Owns and may change

**Owns:** the `files` above. **A departure from your text:** `docs/design/` isn't otherwise yours, but you remove this request's
**Not yet** entry from `temper/docs/design/client/README.status.md` (it begins "3.5, *A person's pause*"), and the prints under
`temper/docs/design/client/journeys/**` are generated files. Nothing else under `docs/design/`.

**Readers, each edited only as said:**

- `goals::State::Stopped`, `goal::Fallback::Stopped` and `goal::Landing::Stopped` become `Closed`. Compiler-forced arms wherever
  they are matched, in any crate. `temper/tools/gallery/src/tests.rs`' table follows.
- `look::GOALS_STATE_STOPPED` becomes `GOALS_STATE_CLOSED`, its class `temper-goals-state-closed` in `temper.css`'s rule in place
  (about line 1792, and any other naming it). `tests/look.rs` follows.
- `temper/crates/temper-client-shell-terminal/src/tests/fixtures/goals.rs` and `golden/goals.txt` follow; regenerate the golden.
- `temper-client-domain/src/tests/pages/home.rs` reads a hold's words, not these: leave it unless a compiler arm asks.
- `temper-client-view/src/tests/`: `goal.rs`, `goals.rs`, `home.rs`, `toast.rs`, `task.rs`, `confirm.rs`, `chats.rs`, `look.rs` read the
  old words: each follows, and `press(&mut root, Act::Stop)` on a task's page reads `Act::Pause`.
- A declared count of the word table moves with the words, as a counted baseline.

**Pre-approved options:**

- **`Act::Stop` on a task's page** goes: `pages.rs`' `Act::Stop => task_confirmation(...)` and its `ActKind::Stop` branches in
  `confirmation` and `task_confirmation` are removed, since no page draws it; `Act::Stop` stays a chat's. If a reader needs the arm
  for exhaustiveness, it does nothing.
- **The domain still reads an own card** for a held task: leave it. `pages/agent.rs` skips `controls` on the person's own pause.
- **`task::Sending::Cancel`'s reason:** "Closed from its page" for a goal and an agent's task, replacing "Stopped from its page" and
  "Cancelled from its page". If a procedure's page could send it, keep its words the same: no page draws Close there today.
- **A sheet's confirm branch for `Shape::Procedure`:** it isn't reached: one `Close this task?` serves, and `CancelTaskTitle`'s old
  words go.
- **A journey that stops on "a key mark caught a moving page"** is retried up to three times, each on a fresh fake box; if all fail,
  widen the mark's wait in the recording driver and say so.

**Not yours:** a chat's acts and status, `temper-client-tools`, jig's words, `docs/help/`.

## Steps

1. **The domain's failing tests,** `src/tests/pages/task.rs` and `goal.rs`:
   `an_agent_s_pause_and_resume_are_sent_at_once`, `an_agent_s_resume_is_its_person_s_own_pause_alone`,
   `an_agent_s_close_asks_first_then_cancels`, and `goal.rs`'s reason, about line 692, "Closed from its page".
2. **The domain:** in `pages.rs`, `Act::Pause | Act::Resume` reaches a page of `Shape::Tracked | Shape::Agent`, and `Act::Stop` as
   above. In `task.rs`, `Sending::Resume` on `Shape::Agent` asks `chat::paused(&task.chip, me)`, on `Shape::Tracked` what it asks
   today; `me` is read as `pages/chat.rs` reads it; `header_state` says a new `State::Paused` where `chat::paused`, compiler-forced
   arms following. `act`'s and `task_confirmation`'s Close reach Cancel, as a goal's does.
3. **The view's failing tests,** both shells: the acts on an agent's task, running, paused and held otherwise; no "Applied" after
   its accepted Pause or Resume, and a refusal's why; each sheet's words, in `confirm.rs`'s table (about line 387); closed's words
   on a goal's row, page, Home and filter.
4. **The view:** the words, as the contract's table; in `pages/goal.rs`, `Word::Close` on Close; in `pages/agent.rs`, the taskbar
   as a goal's acts (`Act::Pause` and `Act::Resume`, by `chat::paused`, then `ActKind::Cancel` with `Word::Close` and `Act::Close`),
   and `controls` skipped on its person's own pause; in `pages/task.rs`, the "Paused" of `task::state`; in `confirm.rs`, one Close
   sheet each, `lead` of `Word::CloseCannotReopen` first, `Word::KeepIt`, and `open.act == ActKind::Stop`'s branch gone.
5. **The world,** `temper/tests/client/tests/suite/task.rs`: `a_delegate_stopped_from_its_page_is_held_then_released` becomes
   Pause at once, "Paused", "Resume" at once, its run again; a new `a_delegate_closed_from_its_page_ends`, through its sheet's words.
   Other tests there reading "Stop" follow, keeping what they check.
6. **The gallery:** as the contract's table. `Confirmation::StopTask` goes; `Confirmation::CancelTask` is `CloseTask`; `task-paused`
   joins with its person's placed stop.
7. **The journeys:** find the stale ones with `rg -l 'Stop|Applied'` under `temper/docs/design/client/journeys/` and the journeys'
   tests. Record each again (`heavy cargo run -p temper-client-journeys -- again NAME`, `journeys.md`, 8.1, in `temper/tools/journeys`)
   and reprint, as `journeys.md`, 8 has it. Since `chat-pause` left a chat's words stale, `chat-draft` is among them.
8. **The render:** the gallery's `goal-closing`, `goal-closed`, `task-closing` and `task-paused`, at 1440 and 420, checked by eye,
   screenshots in `~/src/fabrig/ui-pause-close/` on your builder, outside any worktree, never committed.
9. **The status** entry goes (above).
10. **Checks, once:** `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `python3 -I -B tools/citations/citations.py --base
    origin/main`; `heavy cargo clippy -p temper-client-domain -p temper-client-view -p temper-client-shell-terminal -p
    temper-client-world -p temper-client-gallery -p temper-client-journeys --all-targets -- -D warnings`; `heavy cargo nextest run -p
    temper-client-domain -p temper-client-view -p temper-client-shell-terminal -p temper-client-shell-web -p temper-client-gallery -p
    temper-client-journeys`; `heavy cargo nextest run -p temper-client-world --test suite -E 'test(task) | test(goal)'`. This changes
    what an agent's task's Stop does, so `heavy cargo check -p temper-client-tools -p temper-real-loop-world --tests` too, and `temper
    ctl`'s tests if it builds on any of it.
11. Commit in small steps that say why, and push.

## Report

As your text says, and: which builder holds the screenshots; which journeys were re-recorded, and any mark widened; every person's
"Stop" `rg` still finds, and why it stays.
