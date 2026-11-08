# tui-whole-t1: the launchpad as drawn, and views printed, T1 of whole-story

The spec is `temper/docs/design/draft/surfaces.md`, sections 11.1 and 11.4, less the team lines, `/talk` and `/tell`, and section 11.5, views printed. It's carried out as `temper/docs/design/draft/whole-story.execution.md`, T1 says, with the mockup `temper/docs/design/client/tui/whole-story/launchpad.html` as its look (M0, on main) and `surfaces.md`, section 13, for the mockups' precedence. Read section 11 whole, then T1's entry, which is this plan's input. `surfaces.md`, 11 revises `temper/docs/design/draft/tui.md`, sections 3, 6 and 7 (`tui.status.md`), and makes this v0.1.0 work. The plan runs without the priority mark: only the migration and UI with looks carry it now. It designs and builds together and lands whole.

## What it builds, on today's launchpad (`temper/docs/design/client/README.md`, 6.13 to 6.16)

- **The brief, framed:** the status agent's, across the person's projects, with its byline, and `/brief` showing it whole. Home's brief across projects is P2's (temper/ui's); draw what the client root's Home gives. Until P2 lands, that's the project's brief as today, with a status entry.
- **Where you left off:** the conversations the person can go back to, newest first, numbered. Each shows its project, what it is (a chat, and later a design session or a talk with a role), when, and what is open in it. The person's newest goal belongs in this list too: it was deferred from `tui-launchpad` (`temper/docs/design/client/README.md`, 11), so build it here, reading what the client root holds. The arrows and Enter stay as `tui-launchpad-2` built them.
- **Needs you:** a line each, marked "?" for a question, "$" for money, "✓" for a sign-off. `n` walks them, one prompt after another (`tui.md`, 3, *Needs from elsewhere*). Draw the kinds the engine has; a kind N1 adds later (temper/ui's) takes its mark then.
- **Start:** the commands T1 has. `/change` is one change, run as the project's way says. `/ask` is Ask temper. `/draft` and `/talk` come with T2 and T3: leave them out of Start and the commands until then, with a status entry.
- **The box and the quiet line** as 11.1 words them: the project, the model and effort, the link's state, and "N need you · n to walk".
- **Commands added (11.4):** `/change`, `/ask`, `/brief`, `/needs` (as `n`), and `/open`, which opens the object in focus on the web. It opens the page's address alone, never a secret, through the opener as 4.2's tab is opened.

## Views printed (11.5)

**Printed, not drawn** (the user, 2026-11-08): a view in a reply or a chat's document (`temper/docs/design/client/README.md`, 5.16) prints in the conversation as lines of the terminal's own text, with no picture. Today the TUI's chat doesn't split a reply's view blocks: `temper-client-tui`'s `view/chat/read.rs` takes `Block::Text` straight to an entry, so a way shows as its raw address.
- **A path:** a stop a line, in the derivation's order. First the mark the web draws for its actor (a ring for a person, a four-point star for an agent, a small square for code), then its words. Code's stops are indented, as they stand below the line on the web. A branch's bar closes its part, with the branch's name. For example:

  ```
  You approve
  An agent makes each change; nothing reaches main until you approve it.

  ○ you ask
  ✦ produce
      ▪ check
  ✦ repair
      ▪ check
  ○ you approve
      ▪ update
  ✦ resolve
      ▪ merge
      ┃ main
  ```

- **The other views:** a way prints its name and line over its path; a proposal prints its two paths under "Before" and "After"; a charter prints its `who`, `name` and `facts` as terms; a `not_drawn` prints its words.
- **Print what the derivation gives.** Its words change with temper/ui's P3, so never spell a stop's words in the TUI.
- **Nothing to press:** a stop's card stays the web's.

## Tests and journeys

As T1's entry says: golden frames for each screen at two sizes, light and dark. `launchpad-chat` is recorded again (generation), and a new journey of the launchpad as drawn is added where a fake box reaches its states. The mockup is hand-made: say in the plan's report where the build departs from it and why.

## Overlaps

- **`temper-client-domain`** is shared with temper/ui, whose P tracks change it beside this (P1, P2): the smallest change the TUI needs, named in `plan.md`. I tell temper/ui before it starts.
- **This starts after `tui-box-front` and `client-binary` have landed**, so it builds on both. Layout faults the launchpad's walks found that this plan redraws are this plan's, and are named in its account.

## Walk

`walk = false`. The walk, each screen against its mockup in a real terminal, follows the landing.

## For every task of this plan

- **A plan is complete when it lands.** No walk, render check or throwaway driver in it. Recording a journey again is generation, which a task runs.
- **Only focused and fuzzy run in a plan.** Never `tools/tiers/run`, a browser, process or live tier, one of their stories, or a tier's CPU figure. A new story for those tiers is written and compiled (`cargo check --tests`), not run. A slow-tier test whose expected output the change makes stale is updated in the same task, compiled, and named in the report.
- **Tests stay cheap:** a new case of one behaviour is a row of an existing table test. Budgets and timeouts are the user's, never raised.
- **Decide, don't ask.** A stored format, a protocol or a public interface may change freely before v0.1.0: no compatibility, no migration, no `Question:`; note it in `plan.md`. A small edit a task needs just outside its file list is made and named in the report. A `Question:` is only for what truly blocks the task.
- **Recording flakes:** a re-record that stops on a moving screen is retried up to three times, each on a fresh fake box; if all fail, widen the mark's wait and say so.
- **Generated files** that a change makes stale are that task's to regenerate.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and the branch fails, it's the branch's.
