+++
id = "bar-tag"
part = "bar-tag"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/docs/design/client/browser.status.md",
]
effort = "high"
minutes = 40
+++

# An Ask temper chat's bar shows " · Ask temper"

## Goal

The request's first miss (`temper/docs/plan/look-round-18/inputs/request.md`, task 1): an Ask temper chat's bar draws `CHATS_TAG`'s paragraph, " · Ask temper", but `temper.css` hides it at every width. Show it as `temper/docs/design/client/browser.md`, 2.5, *An Ask temper chat's tag*, says: after the number, in the number's look, so the two read "T9 · Ask temper" at 1440 and 1000; at 420, where the number goes, the tag alone without its " · ". A stylesheet change alone: the view already draws the tag.

Acceptance:

1. `temper.css`, under `/* ==== ui: chats ==== */`, where `#app > .temper-session .temper-session-right > p:not(.temper-byline) { display: none; }` stands (about line 1516): that rule goes, and in its place
   - `#app > .temper-session .temper-session-right > p.temper-chats-tag { margin: 0 0 0 -10px; font-size: 12px; white-space: pre; color: var(--gf-faint); }`: the bar's 10 px gap taken back, its leading space kept, the byline's size and colour (`.temper-byline` at about line 1490);
   - then `@media (max-width: 760px) { ... }` holding `#app > .temper-session .temper-session-right > p.temper-chats-tag { margin-left: 0; }` and `#app > .temper-session .temper-session-right > p.temper-chats-tag > :first-child { display: none; }`, the separator's span hidden where the byline is.
   No other rule changes; nothing is added at the file's end.
2. No other paragraph is brought back: the bar's right (`session::right`, the `SESSION_RIGHT` slot) holds paragraphs only from `session::number` and `session::closed_number` (`BYLINE`) and the tag (`pages/chat.rs`, about line 105). Confirm it by reading `temper/crates/temper-client-view/src/pages/chat.rs`, `new_chat.rs` and `configure.rs`, the three callers of `session::right`, and what each draws into the slot (`document::seg`, `document::add`, `session::to_terminal`, `chat_sheet::bar`, `bar_status`, `bar_actions`).
3. The tag's markup is two text spans, " · " then "Ask temper", inside `p.temper-chats-tag`: confirm it in the gallery's print of `configure-chat-document` (the print `tests::print_writes_every_page_and_the_bundle` writes, or `heavy cargo run -p temper-client-gallery -- OUT` into a scratch directory outside the slot's tree, deleted after).
4. The **Not yet** entry under 2.5, *An Ask temper chat's tag*, in `temper/docs/design/client/browser.status.md` (naming `look-round-18`) is deleted whole, and no other line.
5. One commit at least, `Look-request: ask-temper-place`, its last carrying the account (below).
6. `heavy cargo nextest run -p temper-client-shell-web` and `heavy cargo nextest run -p temper-client-gallery --lib` pass, with their counts in the report.

## Inputs

- `temper/docs/plan/look-round-18/notes/common.md`.
- `temper/docs/plan/look-round-18/inputs/request.md`, task 1.
- `temper/docs/design/client/browser.md`, 2.5, *An Ask temper chat's tag* (this cut's) and *The Chats page and every chat are the greenfield mock's*.
- The request: `git show 180f76e480`, its message's states in words ("its bar's \" · Configuring\" reads \" · Ask temper\"").
- The code, read only: `temper/crates/temper-client-view/src/pages/chat.rs` (`main`, the tag's paragraph after `session::number`), `pages/session.rs` (`right`, `number`, `closed_number`), `look.rs` (`CHATS_TAG` is `temper-chats-tag`, `BYLINE` `temper-byline`, `SESSION_RIGHT` `temper-session-right`).
- `temper.css`: the session bar's rules from about line 1484 (`.temper-byline`), its phone block after them (`:is(.temper-trail, .temper-byline, .temper-session-hint) { display: none; }`), and the block at about 1514 holding the rule this task replaces. `#app > .temper-chats-page .temper-chats-tag` (about 1435) is the Chats page's band and stays.

## Owns and may change

Owns the header's files: in `temper.css`, only the rule named in acceptance 1 and the rules that replace it; in `browser.status.md`, its one entry.

Options, each decided:

- **Another paragraph at the bar's right** (acceptance 2 finds one): keep the hiding rule as `#app > .temper-session .temper-session-right > p:not(.temper-byline, .temper-chats-tag) { display: none; }` where it stands, add the tag's rules after it, and name the paragraph in the report.
- **One span, not two** (acceptance 3 finds " · Ask temper" in one text node): leave out the phone's `> :first-child` rule, keep its `margin-left: 0`, so that a phone reads "· Ask temper", and say so in the report and in the account's 420 row, as differing; the view isn't this plan's to change (`notes/common.md`, *Leave alone*).
- **The tag wraps to its own row** at 1000 in a gallery print you read (the bar's right is `flex-wrap: wrap`): it shouldn't, since the number and the tag lead the row; if it does, say so in the report and leave the rule as acceptance 1 has it.
- **A test that pinned the hiding rule's text** (none is known: no test reads `temper.css`'s rules): repair its expectation, keeping what else it checks, and name it.

## Steps

1. Read the request's message and acceptance 2's code; record what the bar's right holds.
2. The stylesheet change (acceptance 1), then acceptance 3's read of the tag's markup.
3. Delete the status entry (acceptance 4).
4. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; the two runs of acceptance 6, one `heavy` command at a time; `python3 -I -B tools/citations/citations.py --base origin/main`. No clippy: no Rust changes.
5. Commit with the trailer and the account; push with `git push origin HEAD:refs/heads/<your branch>`.

The account's rows, at least:

| Item | Result |
|---|---|
| `.temper-session-right > p:not(.temper-byline)`, hiding the tag at every width | landed: removed where it stood, the bar's right holding no other paragraph (or narrowed, by the option) |
| 1440 and 1000: " · Ask temper" after the number, "T9 · Ask temper" on one line, 12 px and faint | landed: `p.temper-chats-tag`'s rule |
| 420: the number gone, "Ask temper" alone | landed: the phone's rules (or differs, by the option) |
| Gallery `configure-chat-document` and `-split`: the tag shown | landed through the stylesheet; the gallery's print is not committed |
| Render check | follows landing, temper/ui's (`temper/docs/design/draft/look-requests.md`, 4); compiling, prints and focused tests establish no rendered equality |

## Report

The implementer's report: what acceptance 2 and 3 found (the bar's paragraphs, the tag's spans), the option taken if any, the tests' counts, and the account as committed.
