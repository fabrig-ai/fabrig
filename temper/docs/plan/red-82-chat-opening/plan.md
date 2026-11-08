# red-82-chat-opening: the plan

An after-landing red (`inputs/request.md`): since `1e6e2388ee` a chat's
opening reads follow the older cursor to the window's bound, so a long
chat loads its whole window at open and the browser story
`browser_pages::a_chat_opens_at_its_end_and_reads_back_keeping_its_place`
never sees "Loading earlier messages". The repair keeps the rule 5.15
already states for a chat that opens: a long one with a screen and a
half, a short one whole.

Design written by this cut:

- `temper/docs/design/client/README.md`, 5.15, *Opening history*: the
  opening's reads stop at whichever comes first of the history's end and
  a screen and a half held, besides the window's bounds and the stops
  `1e6e2388ee` added; the rest of the section is untouched.
- Its status, `README.status.md`: a **Not yet** 5.15, *Opening history*,
  which the task removes.
- The draft's status, `draft/chat-reading.status.md`: the departure
  *The same rule opens a chat* now says what a shell with no view does.

## Decided

- **A screen and a half is a weight in the domain.** The domain measures
  no screen: skein's page part measures the browser's, and the TUI and the
  terminal measure none. So the opening's reads stop once the held lines
  weigh 2,880 bytes by jig's weight of a line (`row.weight()`: bytes, and
  `LINE_ROOM` for the line and each message, block, object and part), a
  screen and a half of an 80 by 24 terminal, the size the design's golden
  frames use. In a browser the log's measure reads on from there while
  its head is near, so a long chat still opens with at least a measured
  screen and a half; the domain's stop only has to come before the whole
  window. A fast short reply (finding-465) weighs a few hundred bytes and
  still reaches its opening; the story's position chat, whose newest reply
  alone is several kilobytes, stops after its first page as it did before
  `1e6e2388ee`.
- **A consequence for the TUI:** a long chat there now opens with that
  weight of history, where `1e6e2388ee` gave it the whole window; the TUI
  reads no further page (`chat-reading.status.md`, *Design lags*, 2 and
  5), as before that commit. The request asks for this rule; reading
  older pages in the TUI stays section 11's.
- **Tests:** one table test in the domain, made from
  `a_fast_completed_chat_reads_back_to_its_opening`: short, one byte under
  the weight, and at the weight. The browser story is the oracle and runs
  after landing.

## Parts and tasks

| Part | Task | Crates (files) | Needs | Minutes |
|---|---|---|---|---|
| `chat-opening` | `opening-weight` | `temper-client-domain` (`pages/chat.rs`, `tests/pages/chat.rs`); `README.status.md`'s entry | | 50 |

Slice 1 is the part; `walk = false`.

## Critical path

`opening-weight` alone, released with the first push.

## Overlaps with drafts beside this one

- `temper-client-domain`'s `pages/chat.rs` and `tests/pages/chat.rs`:
  `ui-drafter/design-session-web` (merged into its trunk) changed both.
  The request fixes the files; the task's hunks are in
  `accept_activity`, beside `thread_limits`, and in the test that opens
  the file, away from that task's.
- `temper/docs/design/client/README.status.md`: many plans add and close
  entries there (`ui-reaches`, `look-round-25`, `tui-whole-t1`,
  `ui-p4a-team`). This plan's entry sits after 5.15's *While a read is
  out* departure, among entries none of them names.
- `temper/docs/design/client/README.md` and
  `draft/chat-reading.status.md`: written by this cut, in 5.15's
  *Opening history* bullet and one departure's line.
