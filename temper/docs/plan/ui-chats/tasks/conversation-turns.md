+++
id = "conversation-turns"
part = "conversation-turns"
kind = "feature"
needs = ["chat-byline"]
files = [
  "temper/crates/temper-client-view/src/pages/chat.rs",
  "temper/crates/temper-client-view/src/pages/chats.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/limits.rs",
  "temper/crates/temper-client-view/src/tests/chat.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/tools/gallery/src/samples.rs",
  "temper/tools/gallery/src/worlds.rs",
  "temper/tools/gallery/src/tests.rs",
  "temper/tools/gallery/names.txt",
]
effort = "high"
minutes = 85
+++

# The conversation's turns

## Goal

The conversation looks like the mock's: a 720 px column, bubbles for the person, plain text for the agent, three dots while it
works, and its scroll kept at the end.

Acceptance, the request's section 4 but its tool lines and cards (`tool-lines`):

- **The log scrolls,** as `notes/contracts.md`, *The conversation*: on a chat `[role=log]` is the mock's `.thread`, its one child
  `THREAD_INNER`, 720 px centred, gap 20. "Older", the turns, the unanchored cards, the end's line and the acts' status are in it,
  in today's order, so that all of it scrolls together. It keeps `look-round-5`'s scrollbar gutter on both edges; on a phone no
  gutter, both 16 px in. `pages/agent.rs`, by `Drawn::Lines`, keeps its log.
- **The scroll:** the log's end stays in view while the person was at the end, by skein's own rule; a test in a real browser or
  the journeys' replay shows it on a chat.
- **The person,** `.turn-you`; **the agent,** `.turn-agent`, Markdown as today, first and last margins zero. `.temper-who` and its
  clock are hidden from sight but stay for readers and the terminal, but "Being sent" and "Sent": 12 px faint, right aligned under
  the bubble. `.temper-day` is hidden. `.temper-marker` is `.meta-line`.
- **A delegate's words:** `BY_DELEGATE`, as `.turn-agent`, "TN" kept visible in 12 px faint above it.
- **While it works:** `TYPING`, three dots at the end of the log, in the browser alone (`w.shell`), named `ChatWorking`, while the
  chat's status token is `STATUS_ACTIVE` and the window's last line is not `Line::Provisional`. On a configuring chat too. Not on an
  agent's task's page. A pure function in `chats.rs` answers it. The terminal's prints don't change for it.
- **Provisional text:** `.turn-agent`, its "writing" label hidden.
- **"Older":** `.btn.quiet.small`, centred, at the head. **Ended:** its end line in `.meta-line`'s look, and no composer.
- `.temper-session` leaves every `:is(.temper-agent-page, .temper-session)`, and each such rule is `.temper-agent-page`'s alone, or goes.
- **Gallery:** `chat-working`. `chat-solo-T6`, `chat-reference-T1`, `chat-first-run-*`, `chat-closing`, `chat-rejecting` hold.
- `README.status.md`: 3.5's conversation's look leaves the **Not yet**, but what `tool-lines` closes.

## Inputs

- `temper/docs/plan/ui-chats/notes/common.md`, `notes/contracts.md`.
- `temper/docs/plan/ui-chats/inputs/request.md`, section 4.
- `temper/docs/design/client/README.md`, 3.5, *A chat's conversation*, its look.
- `skein/docs/design/ui-web.md`, 7, *A log*; `skein/crates/skein-ui-web/src/page.rs`, `follow_log`.

## Owns and may change

Owns, as `files` lists them.

May also change:

- `temper/crates/temper-client-view/src/pages/agent.rs`: `conversation`'s signature alone; its print and look stay.
- `temper/crates/temper-client-view/src/tests/{delegation,playground,task,times,confirm}.rs`,
  `temper/crates/temper-client-shell-terminal/src/tests/**`, `temper/tools/gallery/src/**`: nodes moved inside the log.
- `temper/tests/client/**`, `temper/tests/client-browser/**`, `temper/tests/one-process/**`, `temper/tests/real-loop/**`: a walker
  that finds "Older" or the end's line by its place. **A change of meaning runs its readers' tests.**
- The terminal's goldens: indentation under "Conversation:" may change. The journeys' prints.

Options, each pre-approved:

- Three dots drawn by three child slots or by one node and its `::before` and `::after`; whichever matches the mock's pixels.
- A slot with a name, if skein-ui maps it to `aria-label`; if not, `Role::Status`, in the browser alone.
- Skein's `Log` needing a change to scroll this way is a `Question:`, with what you measured. Don't work around it in temper.

## Steps

1. The log's child, the tail inside it, `conversation_with_times`' shape.
2. The rules, replacing where they stand. `TYPING` and its function.
3. Renders beside the mock's `chat.html?id=T3` in Hello, `T4`, `T5` at 1440, 1000 and 420. Long conversations at the end, and scrolled.
4. **Tests,** `tests/chat.rs`: the dots for each status, none with provisional text, none in the terminal; a delegate's "TN";
   "Being sent" kept; the order. Each bites. Gallery, goldens, prints.
5. **Check,** `notes/common.md`, clippy, push.

## Report

As `notes/common.md`, *Your report*.

## Rebase (2026-11-05, step merge, foreman ui-chats/conversation-turns/foreman-1)

combine conversation turns with landed Needs you styles, vocabulary and gallery samples during rebase
The merge failed on `temper/crates/temper-client-view/src/{look,words}.rs` and `temper/tools/gallery/src/samples.rs`.
Rebase onto the updated integration containing `670f3aefab` (“Merge main to keep chats compatible with Needs you”), which incorporates main’s Needs you changes.
In `look.rs`, preserve the incoming names through token 320. Tokens 295–297 now name `temper-chat`, `temper-chatbar`, and `temper-sidebar`; move `THREAD_INNER`, `TYPING`, and `BY_DELEGATE` to unused tokens 321–323 and append their names in that order.
Keep both sides’ style-name tests in `src/tests/look.rs`.
In `words.rs`, retain the incoming Needs/Home vocabulary and bounds, plus `ChatWorking` in the enum, `ALL`, identifier match and English match.
Combine counts: `PHRASES = 1476`, `SEGMENTS = 2592`, and `ALL` length 880 against the inspected integration. `ChatWorking` identifier 3287 remains unused there.
In gallery `samples.rs`, preserve incoming Home/Needs samples and fixture behavior, plus `Then::Working`, its exhaustive match arms and `chat-working`; combine `N` to 147.
Preserve both sides of the automatically merged CSS, gallery names and tests.
Run the focused view style/vocabulary/chat checks and gallery checks, including `the_chat_log_follows_its_end_until_the_person_scrolls_away`, through heavy.
Regenerate and report any combined-code prints, goldens or counted baselines made stale by the rebase.
