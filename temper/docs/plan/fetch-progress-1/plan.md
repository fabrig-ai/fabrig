# fetch-progress-1: the plan

From `inputs/request.md`: while an owned Forgejo's set up fetches its binaries, the person sees how far it has gone, in words in both shells, and in the browser a bar that fills.

## Parts and tasks

| Part | Task | Needs | Minutes | What it does |
|---|---|---|---|---|
| progress-value | `progress-value` | | 55 | skein-ui's fraction, a progress's own runs, the web's attributes, the stylesheet's 101 rules |
| fetch | `fetched-line` | | 20 | the line's type, its writer and its reader, for both sides |
| fetch | `fetch-count` | `fetched-line` | 65 | the size, the refusal, the lines on stdout |
| box | `box-relay` | `fetched-line` | 80 | the pipe's `fetched` line, none on a terminal, replies kept |
| row | `engine-fetched` | | 35 | the forge domain keeps the latest, the row carries it |
| row | `wire-fetched` | | 30 | `SettingUp.fetched`, generated, every literal `None` |
| row | `link-fetched` | `engine-fetched`, `wire-fetched` | 40 | the link reads the line, the family writes the wire |
| client | `client-fetched` | `wire-fetched` | 35 | the client's row and its words |
| client | `view-fetched` | `client-fetched`, `progress-value` | 45 | the progress node, the step list, the meter's rule |
| client | `fetched-story` | `link-fetched`, `view-fetched` | 35 | the client's world, through the real family and wire |
| look | `look-account` | `fetch-count`, `box-relay`, `fetched-story` | 60 | two gallery states, the render check, the status, the account |

Eleven tasks, 480 implementer-minutes. The request's scope, item by item: 1 is `fetch`, `box`, `row` and `client`; 2 is `progress-value`; 3 is `view-fetched`.

Slices, in the order they run: skein's value; the fetch and the box; the row; the clients and the look.

## Critical path

`wire-fetched` (30), `client-fetched` (35), `view-fetched` (45), `fetched-story` (35), `look-account` (60): about 205 implementer-minutes, then the end check. `progress-value` (55) and `link-fetched` (engine or wire, then 40) feed it and are shorter. `fetch-count` and `box-relay` run beside it from `fetched-line` and join only at `look-account`.

## What no test covers

**No test runs a real fetch through a real box to a real client.** The fetch's and the box's sides meet at a line each tests to the same form, the box's and serve's at another, and `fetched-story` runs from the engine's event to the view. A whole run needs an owned forge fetching from a mirror in the process tier, and the draft's Later keeps the fake fetch that would let a journey record it. So the two gallery states and the render check are the only look a person gets until a real fetch runs. `walk = false` on every slice, for that reason.

## What the cut found

- **`aria-valuetext` never reaches a page.** `skein-ui-web/src/mapping.rs` writes it from the progress node's own runs, and `Builder::run` refuses every role but text, so a built progress has no such runs. A text child does not stand in: `role="progressbar"` presents no children to assistive technology. The draft says the mapping writes it "today"; the code holds the line, and no tree reaches it.
- **The box's reply queue is short, and its `Say` drops on no room.** It holds the longest group plus two lines. A plain relay of a hundred lines could lose a `started` or a `failed`, and an ask would never end. So the box holds one line, the latest, and writes it only once its replies are.
- **A step's lines reach the terminal three ways:** `--echo steps`, a log not written, and a failure's tail. A plain relay would fill a failed fetch's tail with percentages and push out why it failed.
- **The machine moves on at the step's exit,** so a line left in the pipe could follow the next `step` line. It stays in the log.
- **`box.line_bytes`'s least is 64,** and `forgejo-runner` as the binary's word takes the longest line to 71. The word is `runner`, and the longest line is exactly 64.
- **skein-http delivers no byte past `Content-Length`,** so "a body past its size refused" can be shown at the machine alone.
- **`forge_fetch_command` asserts an empty stdout** on every row, so its `digest` row changes with the fetch.

## What the cut decided

- **A progress's text is runs of its own** (`skein/docs/design/ui.md`, 3.4 and 5): `Builder::run` takes a progress as well as a text node, so "its text as `aria-valuetext`" (`skein/docs/design/ui-web.md`, section 6) comes true, and `close` can assert that a fraction has its words. It widens a public builder call by one role, in the direction the draft already asks for, so it is decided here and not asked. A text child stays allowed with no fraction, so no temper fixture moves. Recorded under **Design departs** in `progress-value.status.md`.
- **The forge domain keeps the latest line, not the box's link** (draft 4 says the link): the link decides nothing and keeps no state a row reads (`temper/docs/design/protocol/forge.md`, 2.8). Still in memory alone, gone at the next step, at `started` and at `failed`. Recorded under **Design departs** in `fetch-progress.status.md`.
- **The progress sits beside the connection,** `Row.fetched` in the engine, as `failed` does, and not in `Connection::SettingUp { step }`: forty-odd literals and patterns stay as they are, and that keeps this plan off the lines `repository-copies-1` and `look-round-3` are on.
- **A new pipe word's form:** `fetched FORGE BINARY DONE TOTAL`, `BINARY` `forgejo` or `runner`, `TOTAL` `none` with no size. On stdout, `fetched forgejo DONE of TOTAL bytes` and `fetched runner DONE bytes`, which a person can read. The draft left the forms open.
- **A zero `Content-Length` is no size,** since a fraction's total is above zero.
- **A binary already there says nothing,** and on macOS the bytes are the archive's, its extraction saying nothing.
- **"41 MB" is `done / 1000000`,** rounded down, true to its name, though the fetch's lines fall at each MiB: the fetch counts as a machine does and the words as a person reads.
- **The browser's muted detail says "Setting up: fetching Forgejo",** and the bar's end says the figure, as the request's pages draw it. The paragraph's name, the terminal and every other page say the whole line.
- **With no size the node is still `temper-flow-meter`,** with no `data-progress`, and the stylesheet draws the sliding segment on it, so the bytes take the same place the percentage does ("the bytes in place of the percentage" in the request). `temper-flow-busy` stays for the steps with no figure.
- **While the runner fetches the node is named "Fetching its runner":** the draft names Forgejo's alone.
- **`temper forge list` stays as it is:** it prints a state and a step from the domain, not the row's words, and the draft does not name it.
- **The look's commits carry the trailer:** `client-fetched`, `view-fetched` and `look-account`. `look-account` writes the account.
- **The request's pages are copied into `notes/request/`,** since no ref holds the request's commit and a slot would not have it. They leave with the plan.
- **The drafts stay:** their status now has no lag, and retiring or archiving them is for when the plan has landed.

## The question

`progress-runs`, in `plan.toml`: the request makes a public interface beyond the drafts a question, and `Builder::run` taking a progress is one, however small. No task waits: the default is what `progress-value` builds, and another answer is a follow-up task.

## skein

`skein-ui`, `skein-ui-web`, `skein-ui-bundle`, and `skein-ui-text`'s fixture. **skein-ui isn't on hearth's skein surface:** hearth builds nothing of this and nothing of hearth changes.

## Overlaps with the drafts beside this one

Early work keeps off them: `progress-value`, `fetched-line`, `fetch-count` and `box-relay` touch no file another draft is changing. The rest could not avoid it:

- **`client-wire-v1.schema`, `generated/v1.rs`, its goldens:** `repository-copies-1/making-wire` and `to-begin-store/dismiss-wire`. Their hunks are elsewhere in the schema; the generated file and the goldens are regenerated on a conflict, never merged by hand.
- **`temper-service/src/family.rs` and `src/tests/family.rs`:** `repository-copies-1/making-run`. `link-fetched` keeps to `connection` and its call.
- **`temper-engine-domain-forge/src/boundary.rs`, `domain.rs`:** `repository-copies-1/making-*` and `to-begin-store`. `engine-fetched` adds lines and moves none; keeping `Connection::SettingUp` as it is keeps this small.
- **`temper-client-domain/src/forge/family.rs`, `forges.rs`:** `repository-copies-1/making-facts`.
- **`temper-client-view`'s tests, `temper.css`, `temper/tests/client/**`, the gallery:** `look-round-3`. No `Word` is added, since any new one meets `words.rs`'s counts, and the meter's lines in `temper.css` are not theirs. The gallery's `samples.rs`, `tests.rs` and `names.txt` are the likeliest catch-up.
- **Literals another trunk adds:** a `SettingUpParts` written beside this plan will miss `fetched`, a catch-up the compiler names.
