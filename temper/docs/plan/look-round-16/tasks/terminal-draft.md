+++
id = "terminal-draft"
part = "terminal-draft"
kind = "feature"
needs = ["terminal-frame", "md-underscore"]
files = [
  "temper/crates/temper-client-view/src/pages/document.rs",
  "temper/crates/temper-client-view/src/tests/document.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/docs/design/client/journeys/chat-draft/*.html",
]
effort = "high"
minutes = 80
+++

# A chat's terminal: its draft, a document beside it; a draft's emphasis

## Goal

Land the look request `chat-terminal-draft`: the draft beside the terminal reads as a document, in temper's own type and figure, on a ground of its own, through the shell's own document rules under the terminal's tokens, never restating them. And, from the same request, a draft's first paragraph reads its Markdown, so `_Draft, with the user._` shows as emphasis, in Split and in the terminal alike (skein's reader takes `_` since `md-underscore`).

Acceptance, by the tests below and by reading the rules. No browser runs, here or in any step:

1. The pane's first paragraph under the title, today's plain `DOC_MUTED` paragraph, is read by `skein_ui::markdown::read`, in the muted style: `_Draft, with the user._` gives one `Run::Emphasis` "Draft, with the user.", `**x**` strong and `` `x` `` code, in both shells; words with no markers draw as today's words.
2. In `temper.css`, beside the document's rules in the chats area (after `#app > .temper-session` `.temper-doc` rules, not after `terminal-frame`'s): the terminal's pane sets the shell's tokens (`--gf-ink`, `--gf-surface`, `--gf-paper`, `--gf-line-2`, `--gf-muted`, `--gf-faint`, `--gf-sans` and what the figure's and the tint's rules read) to the request's `--doc-*` values, dark, and light under `.temper-terminal-light`; its ground, padding, rule and type as the request's `.tt .tt-doc` and `.tt .tt-doc .temper-doc-path` rules say; a changed section's pale block and note in the terminal's accent. No rule of `terminal.css` that restates a shell document rule (`h2`, `h3`, `p`, `ul`, `li`, `strong`, `code`, `figure`) is copied: the shell's own apply.
3. On a phone, at 760 px or less, the terminal's Split shows the document in the conversation's place with the box at the foot, as the request's 760 px rules say; if `terminal-frame` landed those rules, they stay as they are.
4. The browser's look of the pane (Split, Draft) is unchanged but for acceptance 1.
5. Every commit carries `Look-request: chat-terminal-draft`.

## Inputs

- `temper/docs/plan/look-round-16/notes/common.md`: the standing rules, the requests and how to read them.
- `temper/docs/plan/look-round-16/notes/contracts.md`, 4 and 5: the terminal's tokens.
- `temper/docs/design/client/README.md`, 3.5, *A chat's terminal*, *The draft beside it*; *A chat's document*, *The pane*.
- `temper/docs/design/client/browser.md`, 2.5, *A chat's terminal*, *The draft beside it*, and *A chat's document*.
- `skein/docs/design/ui.md`, 10, *emphasis and strong*.
- The request `chat-terminal-draft`: its message and its `terminal.css` diff (`git show COMMIT -- '*terminal.css'`), page 03.

## Owns and may change

Owns the header's files; in `temper.css`, only the terminal's pane rules, placed as acceptance 2 says.

Reader edits allowed:

- A test elsewhere in temper whose fixture's first paragraph holds Markdown markers and so now draws runs (`temper-client-shell-terminal`'s goldens, `temper/tools/gallery`'s `HOLDS`, `temper/tests/client`): the expected words or runs alone, a terminal golden by its own means (`SKEIN_CODEGEN_WRITE=1` with the owning test). Name each.
- `chat-draft`'s pages, reprinted by `temper-client-journeys` with no arguments, if the paragraph's nodes change in them.
- `temper/crates/temper-client-shell-web/tests/suite/main.rs`: none; its document heading checks read the shell's rules, which stay.

Options, each decided:

- **The muted paragraph's node:** read the paragraph into a `Role::Slot` styled `DOC_MUTED`, its paragraph inside, if `markdown::read` can't take the style itself; then the muted rule (`#app > .temper-session :is(.temper-doc-sub, .temper-doc-muted)`) colours what it holds, and the gallery's and tests' checks of `DOC_MUTED` read the slot. Add nothing to skein.
- **The figure:** the shell's `figure` and flow rules, unchanged, read the tokens the pane sets; the request's `figure > svg { max-width: 100%; height: auto; }` only where the shell has no such rule.
- **The type:** the pane's `font: 400 15.5px/1.7 var(--gf-sans)` and `font-variant-ligatures: normal` on the pane, since the terminal's monospace is inherited otherwise; the path line in IBM Plex Mono at 12px, as the request has it.

Leave alone: the conversation, the list and the frame's rules (`terminal-frame`'s and `terminal-commands`'), `pages/terminal.rs`, the domain, the gallery (`terminal-commands` adds `chat-terminal-draft`), skein.

## Steps

### 1. The paragraph

`pages/document.rs`, `opening`: the introductory paragraph read by the Markdown reader in the muted style, in place of `b.text(Run::Plain, …)`.

### 2. The pane's look

`temper.css`, as acceptance 2.

### 3. The tests

Rows of what exists, in `temper-client-view/src/tests/document.rs`, no new test:

- `split_draws_the_pane_after_the_conversation`: its fixture's opening paragraph gains a row with `_Draft, with the user._`, checked for its `Run::Emphasis` "Draft, with the user." in the muted style; and one with plain words, as today.
- `the_terminal_prints_the_document_after_the_conversation`: the same emphasised row, printed as its words with no underscores.

Break the rule once by hand (the plain text back in `opening`), run the test, see it fail, restore, and name it in the report.

### 4. The checks

One command at a time:

- `heavy cargo nextest run -p temper-client-view --lib -E 'test(=NAME)'` for each test you changed.
- `cargo fmt`, then `python3 -I -B temper/scripts/old_forms.py`.
- `heavy cargo clippy -p temper-client-view -p temper-client-shell-web --all-targets -- -D warnings`.
- `heavy cargo run -p temper-client-journeys` to reprint, then `heavy cargo nextest run -p temper-client-view -p temper-client-shell-web -p temper-client-shell-terminal -p temper-client-gallery -p temper-client-journeys -p temper-client-world`.
- `python3 -I -B tools/citations/citations.py --base origin/main`.

### 5. Commits

- **Trailer:** `Look-request: chat-terminal-draft` on every commit, one line above the other trailers.
- Push with `git push origin HEAD:refs/heads/<your branch>`.

## Report

The implementer's report, and, for the account `terminal-close` writes:

- each rule of the request's `terminal.css` diff: landed (its selector here), or left as a restatement of the shell's, and which shell rule draws it;
- the paragraph's node, slot or paragraph, and why;
- each print or golden regenerated, and each reader file edited.
