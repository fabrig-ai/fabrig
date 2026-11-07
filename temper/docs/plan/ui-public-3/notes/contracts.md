# ui-public-3: the run's binding calls

Both tasks build to this note; neither needs the other's code.

## The words

One line, word for word, wherever they stand (the request, section 1; jig's `domain/people.md`, 13.3):

> Anything said in this project's chats may show on its public pages, in the titles of its goals and tasks. On a public repository, a change's pull request shows what its agent wrote too.

A Markdown page may wrap them. Nothing shortens or rewords them.

## ctl (`ctl-and-pages`; `temper/docs/design/client/tools.md`, 5.3 and 6.2)

| `project amend` | stdout | stderr |
|---|---|---|
| made public | as today | the words, then a newline |
| made public, `--json` | as today | `{"may_show":"<the words>"}`, one JSON object a line, as the `kept:` lines are |
| made private, either form | as today | nothing |
| refused | nothing | as today |

A key replay prints the first answer, these lines with it. stdout stays as it is because `browser_public`, which another plan is repairing, reads it and asserts nothing follows the page line.

`rows.rs`'s `PROJECT_AMEND`:

- `prints`: `` `project 1: public` and, made public, `page: /public/projects/3f2a9c01d4e5b6a7`, then what may show on stderr; made private, `project 1: private` ``
- `notes`: `Anything said in this project's chats may show on its public pages, in the titles of its goals and tasks. On a public repository, a change's pull request shows what its agent wrote too. Nothing asks first; made public, it prints these words on stderr. Only an owner changes visibility. A member and project 0 are refused, exit 2. The public page is a path under the deployment's address, from me's public identifier. A key replay prints the first answer.\n`

## The board's Landings link (`ctl-and-pages`; `temper/docs/design/client/browser.md`, 8.3)

The word stays "Landings", the tree stays a level-2 heading holding a link, and only the sheet changes, so no browser story's words change. The public section's `h2` rule in `temper.css`, under `/* ==== ui: public ==== */`, is replaced where it stands by the mock's `.link`:

```css
#app > .temper-public-page h2 { margin: 0; font: 400 16px/1.6 var(--gf-sans); letter-spacing: normal; color: var(--gf-ink); }
#app > .temper-public-page h2 > a { text-decoration: underline; text-decoration-color: var(--gf-line-2); text-underline-offset: 3px; }
#app > .temper-public-page h2 > a:hover { text-decoration-color: var(--gf-ink); }
```

## The configuring chat (`chat-and-help`; `temper/docs/design/domain/agent.md`, 8.1)

A paragraph of its own in `configuring.txt`, right after the one that starts "Say what settings cannot express", with a blank line either side:

```
Making the project public is no setting: never propose it. Asked, say its owner does it outside this chat, and say: "Anything said in this project's chats may show on its public pages, in the titles of its goals and tasks. On a public repository, a change's pull request shows what its agent wrote too."
```

That is 303 bytes and its two line ends 2, so 305 in all. charter 3 is the stock's largest, so the figures tied to its room move with it:

| Where | Now | Then |
|---|---|---|
| `temper-service/src/tests/stock.rs`, charter 3's instructions' length | 5957 | 6262 |
| `temper-service/src/tests/stock_brief.rs`, charter 3's row | `42_392, 357_743` | `42_697, 357_438` |
| `temper-service/src/tests/stock_brief.rs`, `room` | `357_743` | `357_438` |
| `temper-service/src/limits.rs`, `STANDARD_BRIEF`'s comment | 357,743 | 357,438 |
| `temper-service/src/limits.rs`, `STANDARD_BRIEF.brief_bytes` | `356_270` | `355_965` |
| `temper-service/src/tests/limits.rs`, `brief_bytes` and `result_bytes` | `356_270` | `355_965` |
| `temper/docs/design/protocol/configuration.md`, section 11, and its status, already written | | 355,965 |

These are the arithmetic of `the_stock_brief_with_a_whole_spec_fits_both_receiving_bounds`. Where a test measures otherwise, its figure wins; the task's spec says what to do then.
