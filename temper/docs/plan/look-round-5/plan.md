# look-round-5: the plan

From `inputs/request.md`: land the look request `configure-temper-name` (`cea9fdb5d2`), by
`temper/docs/design/draft/look-requests.md`, sections 3 to 5. It sits directly over
`configure-chat-in-settings`, which `look-round-4` landed. The trunk is `look-round-5/integration`.

**The check before the cut:** on `origin/look-requests`, above `configure-chat-in-settings`, stand
`configure-temper-name`, released, then `new-menu`, `configure-chat-draft` and
`configure-temper-own-way`, each with a `Look-hold`. No other released request; none of the held is
landed.

The design is cut ahead of the tasks: `temper/docs/design/client/README.md`, 3.5 (*Settings*, *The
first chat*, *The configuring chat's page*), 5.13, the pages' table and section 11; its status;
`draft/configuring.status.md`; `domain/testing.status.md`.

## Parts and tasks

| # | Task | Needs | Effort, minutes | What it lands |
|---|---|---|---|---|
| 1 | `configure-opening` | | high, 130 | the name, the opening, the cards, the rules, the terminal, the gallery's words |
| 2 | `configure-entry` | 1 | high, 130 | Settings' link and its choice of chat, the render check, the screenshots, the account |

## Critical path

`configure-opening`, `configure-entry`: ~260 implementer-minutes, then the trunk's end check. Both
write `temper-client-view`, so they chain.

## What the cut settled

- **The chats page's offer** keeps "Tell temper how you work". The message names To begin's step as
  keeping its words because "a step says what the person does", and says nothing of this offer, which
  only shows before a project's first chat; it is the same kind of thing and opens the same page.
- **The Settings page's entry doesn't exist yet,** though the message speaks of it, so this landing
  makes it: the first link of `Settings`' list. How work runs' entry waits for `HowWorkRuns`.
- **How the entry chooses:** `Settings` reads the person's chats in the project, as `Configure` does,
  and the link's address is chosen from what is delivered: until then, the page. Choosing at the page
  (`Configure` sending the person on to a chat) would break "Start" and a later "Start over", which
  must open the page.
- **"Newest" is the highest task number** among the person's configuring chats not closed, not the
  latest activity, and it reads the live watch's window: a chat past that window isn't seen, and the
  link opens the page. The account names it.
- **A card's button has no name in the browser:** skein-ui's web shell writes a button's name as its
  text, so a button with the request's two spans has only its children to name it, and the
  accessible name comes from them. No skein change. The terminal prints one line, "TITLE: LINE".
- **The turns share the column by one rule the overlay hasn't,** since its page draws no turns and
  the message asks for it in words.
- **"Start over"** isn't built: `configure-chat-draft`, held, draws it. The design holds it in section
  11, as an open question with its default.
- **Rules that change move too:** every rule the overlay adds or changes goes under `/* ==== ui:
  settings ==== */`, the old one going from where it stands, and a moved base rule takes its phone
  rule with it, or the later base would override it.

No stored format, protocol or public interface changes, beyond `temper-client-domain` gaining
`Chats::open_configuring` and `pages::chats::ended`.

## Overlaps with other work

- **`claude-always/fetching-words`** changes `temper/docs/design/client/README.status.md`, which both
  tasks edit a line of: unavoidable, and small.
- **`ui-chats`,** landing soon, takes `.temper-session` to every chat. It meets the rules this round
  moves in `temper.css`, and `pages/chat.rs` isn't touched here. The rules keep their plain names, and
  the column's rule isn't the configuring page's alone, so it can.
- **`configuring-walks`,** queued, may read this page. Its starters' acts don't change; its words and
  names do.

## What the survey found, and the specs settle

- No golden or recorded journey holds this page's words: only `tests/configure.rs`,
  `tools/gallery/src/tests.rs` and the design. `configure-chat/` lives on the person's branch alone.
- `Word::FirstChat` named the page and the chats page's offer; they part here.
- `temper/tests/client/tests/suite/configuring_chat.rs` presses `Act::Starter` with its own words, so
  it stands, and takes the entry's meeting point over the real engine.
