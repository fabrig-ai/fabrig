# look-round-15: a chat's heading is its first sentence, short

The request: `inputs/request.md`. It lands the look request `chat-title` (`27b6052708` on `origin/look-requests` when cut), by `temper/docs/design/draft/look-requests.md`, sections 3 and 5. The two held requests above it, `chat-terminal` and `chat-terminal-font`, are left alone.

## Parts and tasks

| Part | Task | Needs | What it makes run |
|---|---|---|---|
| `node-hint` | `tasks/node-hint.md` | | skein-ui's node hint: `Builder::hint`, its patch, its bytes, `title` on the web, nothing in the text shell |
| `chat-title` | `tasks/chat-title.md` | `node-hint` | A chat's heading shows its short form in both shells, its whole title as name and, where cut, as hint; the crumbs yield to the controls; the account |

Critical path: `node-hint`, then `chat-title`, about 45 and 75 minutes by their estimates.

## What was decided

- **Two tasks, not one.** The request asks for one, and for the whole prompt to show on hover. skein-ui has no words for hover on a node: its only hint is a scene mark's (`skein/docs/design/ui.md`, 11.2), and the web shell writes no `title` anywhere. A missing generic mechanism goes into skein before its consumer (`AGENTS.md`, 1), and a branch stays in its tree, so the hover is a small skein task before the temper one. Dropping the hover was the other way, and the request names it as a page change; split-off work is never dropped (`look-requests.md`, 6).
- **The hint takes skein's light path** (`ui.md`, 4.3): it maps one to one onto HTML's `title` and Android's tooltip text, so this cut wrote its rows in `ui.md` (3.2, 5, 7, 15, 16.1 to 16.3), `ui-web.md` (6) and `ui-text.md` (5, not printed), with **Not yet** entries in `ui.status.md` and `ui-web.status.md` that `node-hint` deletes. A browser reads a `title` as the description of a node with none, so a screen reader hears the chat's whole title twice: in the heading's name and as its description. `ui-web.md`, 6 says so; the request keeps the whole prompt in the label.
- **A hint only where the heading is cut.** The request's page draws `title` on its long prompt, and a short prompt "shown whole as today". So a title shown whole has no hint, and no recorded journey's print changes: `chat-draft`'s "Let us design Away mode." is its whole first sentence.
- **The rules are written to the letter** in `temper/docs/design/client/README.md`, 3.5, *A chat's heading is short*: what ends a sentence (`.`, `!`, `?` followed by white space or the end, so "3.5" and "lib.rs" end none), white space first and last, characters as Unicode scalar values, and the cut. The cut keeps the words before the last space within the first 72 characters, drops a trailing `,`, `;` or `:`, and adds "…". A first word of more than 71 characters is cut after its 71st character. The heading is at most 72 characters, its ellipsis included.
- **One function, `pages::chat::short_title`**, where `pages/chat.rs` passes the chip's title to `Heading::Chat`, as the request says. `Heading::Chat` carries the name, the browser's words, the terminal's "TN · SHORT" and the hint, all made in `chat.rs`, so `session.rs` only draws them. `notes/contracts.md` fixes both surfaces.
- **Only a chat's heading is short.** An agent's page also heads with "TN · TITLE", and the request says only a chat's heading changes, so it stays whole.
- **The overlay's rule joins the rule of the same selector,** `#app > .temper-session .temper-crumbs`, where it stands in the chats area. It replaces nothing. No stylesheet test is added for one declaration: the shell-web tables there are other areas', and the render check reads it after landing.
- **Tests:** one new table test for `short_title`, as the request asks, with a row for each case its message and this design name. The heading's forms in both shells are rows of the existing `the_session_heading_keeps_its_name_and_shows_the_title_in_the_browser`, a title at its `long_text` bound among them, so the view's per-node text bound is checked rather than assumed. In skein, two small byte tests sit beside the description's twins, and the rest are rows of the placeholder's tables.
- **Not the look's:** a model-written title, which the request names as a later request of the user's own. The account says so.

No stored format, protocol or public interface changes: the hint is an additive field of skein-ui's tree. No journey's recording goes stale. No browser story is added. temper/ui's render check follows landing.

## Overlaps

- **`ui-wait-words/wait-words`** (integration, not merged) changes `temper-client-view/src/pages/session.rs` (`over`), `words.rs` and a row of `tests/chat.rs`. `chat-title` must change `session.rs`'s `Heading` and `top`, and two other tests of `tests/chat.rs`, and it keeps off `words.rs`. The hunks are apart; whichever merges second takes a small merge.
- No other draft beside this one lists a file of `node-hint`'s.
