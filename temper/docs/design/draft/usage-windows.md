# Usage windows

Draft, 2026-11-18, from a session with the user that day. A Claude
subscription's usage, its 5-hour and weekly windows, comes from the runs
already using it, and from a cheap read when none is. The user, asking
for it on the Accounts page: "can we also show usage information for
subscription accounts (percentage, possibly with a bar, and info like
'resets in 4 d 14 h · 2.0%/h · full in 1 d 11 h' ...)", and on how:
"maybe we can (also) do it opportunistically, essentially if some agent
is running at all it will send us usage from time to time (not all of
them, just one is enough), if there's no such agent and the system is
idle, we spawn one from time to time".

## 1. In one page

- **The place is there:** jig keeps an account's facts, and kind 7 is a
  usage window: its length, its share used, when it resets and its state
  (jig's `domain/accounts.md`, 3.7, the facts' registry). Nothing writes
  it yet.
- **Runs report it, for free:** every Claude Code run already streams
  `rate_limit_event`s (smith's `protocol/programs.md`, 5.6), and smith
  reads only the one that says the limit is hit. Each one becomes the
  account's window, whatever its status. One run of an account is enough:
  a newer report replaces the fact, so many runs cost nothing more.
- **A read when no run speaks:** an account none of whose runs has
  reported for 30 minutes is read once by its program's own usage
  command, which makes no model call, as Claude Code's `/usage` prints
  each window's share. No agent is started for it: a read spends no
  budget, no slot and no turn.
- **Drawn on Accounts:** the user's look request `accounts-usage` draws
  the windows wherever kind 7 holds one (`../client/pages/accounts.md`).
  Where none is held, nothing is drawn and nothing is guessed.

## 2. From the runs

- **Every `rate_limit_event`,** not only a rejected one, reports its
  window. Its `rateLimitType` names the window (`five_hour` as "5 h",
  `seven_day` as "week", any other by its own word). Its status gives the
  state: allowed as open, a warning as warned, rejected as limited. Its
  `resetsAt` is the reset, and its share used is taken where the event
  carries one. An event with no share keeps the share already held and
  changes the state and the reset.
- **Relayed as a fact:** the agent reports the window with the run's
  account and source, origin 5, a program's event. The core keeps it as
  any fact (jig's `domain/accounts.md`, 3.7, *A fact is its kind and
  label*), replacing the same kind and label. A report equal to the one
  held changes nothing and marks nothing, so a busy account doesn't churn
  its readers.
- **A limit stays as today:** a rejected event still ends the run failed
  for its account (smith's `protocol/programs.md`, 5.6). The window is
  reported as limited beside it.

## 3. A read when no run speaks

- **When it is due:** for an account whose source is a program and is
  usable, when no fact of kind 7 has come from a run, or from a read, for
  30 minutes; and once at a limited window's reset. A source not usable
  holds what is due until it is, and an account is never read twice at
  once, as the model list's read (jig's `domain/accounts.md`, 3.7, *When it
  is due*).
- **How:** the core gives the read to a worker whose host keeps the
  program's login (jig's `domain/accounts.md`, 3.4, the host rule), and the
  worker's program adapter runs the program's own usage command. The read
  is no run: no budget, no slot, no turn, no model call. Its answer gives
  each window's share and reset, and becomes the facts, origin 6, a
  provider's usage read. A failed read keeps the facts held and is tried
  at the next due time.
- **An idle deployment** reads each such account at most every 30
  minutes, and a busy one hardly ever, since its runs report first.

## 4. What it isn't

- **No agent spawned for usage:** a read makes no model call, so it costs
  nothing against the window it measures.
- **No ChatGPT:** a ChatGPT plan exposes no windows to an app
  (`../client/pages/accounts.md`: "temper reads no plan's windows").
- **No key:** an API key's limits are rates, not windows, and aren't
  drawn.
- **No history:** the pace on Accounts comes from the fact alone, its
  share over the time since the window opened. A recent rate needs an
  earlier figure kept, and is section 6's.

## 5. What changes where

- smith's `protocol/programs.md`, 5.6: every `rate_limit_event` is
  reported, its window read as section 2 says.
- jig's `domain/accounts.md`, 3.7: kind 7 is said by a program's events
  and a program's usage read, and the read's due times and host are
  section 3's.
- jig's hosts' design: a worker's program adapter offers a usage read for
  a program source.

## 6. Open questions

- **The read's interval.** Default: 30 minutes since the account's last
  report.
- **A recent rate for the pace,** from an earlier figure kept beside the
  fact. Default: none; the pace is the fact's own.
- **The window's share when an event carries none.** Default: the share
  held is kept.
