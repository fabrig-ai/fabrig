# ui-walk-2: the plan

Two findings of the second whole walk: A4, an answer that didn't send, and R1, a command that doesn't run. The request: `temper/docs/plan/ui-walk-2/inputs/request.md`.

## The tasks

| # | Task | Needs | Effort, minutes | What it gives |
|---|---|---|---|---|
| 1 | `needs-press` | | high, 90 | A4: a press survives a refresh of the inbox; a bound's refusal says so |
| 2 | `login-command` | | high, 150 | R1: the owner's command carries the box's directories, proved offline and on a real box |

Not chained, as the request asks: they share only `temper/docs/design/client/README.status.md`, one line each, already written apart, and `notes/common.md`.

## The critical path

`login-command` (150), beside `needs-press` (90), then the trunk's end check.

## What the cut decided

**A4**

- **Its cause, from the code, to be proved red first:** `Inbox::refresh` blanks each card's page, and empties the cards at a filtered address, which Needs you usually is, for the round trips of its reads, on every change of the inbox. A4's box had a goal running and a proposal arriving. The choices leave the page and come back as new elements, and `needs::answer` returns with nothing shown. It fits an intermittent failure and the walk's report (count 1, the question open, ctl unanswered: no line was ever made). The task proves or refutes it, and says what it ruled out: the hold's timer reads as sound, and it tests that too.
- **The fix keeps what's drawn, rather than only making the loss visible:** a lost click never reaches the domain, so no message could report it. The rule is `temper/docs/design/client/README.md`, 3.5, *A press stays under the finger*. What a bound still refuses takes a toast.

**R1**

- **The smaller way, as the request asks, in place of the three paths on the deployment's answer:** one string, `TEMPER_CONFIG=… TEMPER_DATA=… TEMPER_CACHE=…` or empty, composed in the binary where the directories are. Not the whole command, since NAME is the page's. Why it's smaller: the client gets no directory fact, no quoting and no notion of a default, and only the view changes by one run.
- **A read of its own, not a field of `DeploymentAnswer`:** that answer is made in `family::to_root` before any party is known, the same for every visitor, so it can't be gated without moving the whole read behind the people child, which the first page's welcome depends on. The new read takes the existing party path, `kinds::read_refusal`, with the owner alone permitted; this is the request's "only to a person who can see the command".
- **The owner at any time, not only during set up:** Connect a model draws the same command after set up, and needs the same words.
- **Asked when the address names a program's way,** not at the link's start: it reaches no one who won't see the command, the owner's role is settled by then, and no journey goes stale, since no recorded print holds the command.
- **The default is decided by comparing paths, not by `origins`:** `temper box` runs its child with `--config --data --cache`, so the child's origins always say `Flag`, and an origin rule would make every box's command long.
- **Left plain, in the design's Open questions:** Accounts' "On each host, run: …", since each host has its own directories, and anyone who isn't the owner. A directory that isn't UTF-8 isn't drawn as it would run. A fake box's sandbox directories make its page show the long form, which is true of it.

**Both**

- **No compatibility:** there is no supported deployment. `Reading` gains a variant, `ProgramEnvironmentAnswer` is new, and the client wire's family number moves if the generator asks.
- **The design was written by this cut:** `temper/docs/design/client/README.md`, 3.5, 4.6 and 11, with its **Not yet** entries in `README.status.md`, which each task removes as it lands. No draft's status changes: neither finding comes from a draft.

## Overlaps not avoided

- `temper-client-domain/src/pages/inbox.rs`, `pages/needs.rs`, `tests/pages/inbox.rs`, with `ui-walk-large/needs-proposal`: the fault is there, and it can't wait.
- `temper-client-view/src/words.rs` and `temper/docs/design/client/README.status.md`, with several plans: one addition each.
- `temper-client-view/src/pages/connect_model.rs`, `pages/welcome_model.rs`, with `look-round-7/sign-in-wait-page`: the one `command` line.
- `temper/crates/temper-client-shell-web/temper.css`, with several: under its marker.
- `temper-service/src/family.rs`, and the wire's schema and generated code: regenerate on a catch-up, as the request says of generated files.
