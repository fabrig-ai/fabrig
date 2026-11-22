# The gallery

`temper-client-gallery` (`bin/temper-client-gallery`, calling
`crates/temper-client-gallery-tool`), a program of temper's workspace, prints the browser's pages in named states into a
directory, as static documents, for screenshots beside the mockups
and a person's eye. It is a tool, not a test of the pages: what
the pages do is the worlds' (`testing.md`), and the gallery draws what the
view draws, never changing a view to suit a sample. It stays for the
states no journey reaches (`journeys.md`): a journey is its walk,
printed in a world from the engine's own answers by
`temper-client-walk` (`../journey-walks.md`), where the look is tuned.

## 1. What it prints

- **One command:** `cargo run -p temper-client-gallery -- OUT`. It
  reads no configuration, environment, clock or network, and starts no
  server.
  `OUT` is replaced whole when it holds the gallery's `index.html`,
  made when it is missing, and refused otherwise.
- **Each state's page** as `OUT/NAME.html`: the view's tree for the
  state, made a page by skein's `Page` with the shell's look (`Look`
  with `look::NAMES`, primary input `Control`) and its prefix, its
  links given the hrefs the shell gives them (`addresses.md`, 1), by the view's one
  href function, which the shell's host calls too. The page's root is
  the document's `#app`, as skein's `Dom` mounts it; dialogs carry
  `open`, since a static document has no top layer. The title is the
  shell's: the page's heading, then "temper". A state whose root's kept
  look is night, its switch pressed, prints `data-night` on its `html`, as
  the shell sets it (`look.md`, *Day and night*).
- **The bundle's stylesheets and files,** under `OUT/bundle/`, made by
  skein's bundler from the stylesheets and files `scripts/bundle.sh`
  passes, in its order (`look.md`), with no module: each page links them as
  the bundle's page does, so `temper.css`'s files resolve as they do
  in the bundle. The bundle's own page and its empty module are unused.
- **An index,** `OUT/index.html`: each state's name, in the gallery's
  order, linking its page, with its mockup's address beside it where a
  mockup draws it.
- **Static:** no script runs. The opening focus is an `autofocus`
  attribute, and a named sample may hold one disclosure open; caret,
  scroll-following and live updates are not replayed.
- **A path printed for both widths:** a static page measures no box, so
  each path's drawing (`../pages/chat/conversation.md`, 3) is printed twice in its figure:
  across, drawn in a box 720 wide, and down the page, drawn in a box 388
  wide, the box at 420 pixels with a phone's 16 pixels each side
  (`../pages/README.md`). The page's own style, in its head, shows the second
  at 760 pixels and under, the shell's phone width, and the first above
  it, so that the print at 420 runs the path down the page as the shell
  does.

## 2. Its states

- **A state** is a world at a page's address, then what is done there:
  the client's root stepped from start by the documents a world
  delivers, then by the person's acts or the engine's answers that the
  state names (a close pressed, a reject's reason box, a refusal), as a
  page would be. No page state is set by hand.
- **The worlds are the mockups':** reference, solo and the first run at
  its steps, their words copied from `mockups/world-*.js` where temper's
  model holds them, so that a picture differs from its mockup by look,
  not content. A value the model has no place for is left out, never
  invented. The gallery's own documents carry what the domain's
  fixtures don't: the providers, accounts and their holders by name.
  A spend is the contract's whole number (jig's owner numbers), so a
  mockup's fraction is rounded to the nearest (0.8 is 1) until the
  contract holds one. A project lists the templates its box would offer:
  the first run's worlds the stock's chat (1), goal (2) and Ask temper
  (5, charter 11), as a fresh box does, so that their pages draw Ask
  temper's rail row and tab (the look request `ask-temper-place`); the
  reference world Ask temper at 5 too, and its own ways at numbers the
  stock leaves free; the solo world Ask temper in its Ask temper states
  alone.
- **Its time** is each world's, fixed: the mockup's now, its
  `world.now`, at which the view builds, and each record's time the
  mockup's offset from it, so that a time reads as its mockup's
  ("12 min ago"). The program still reads no clock (1).
- **Its bounds** are the browser's shipped client limits as they are
  (`temper-client-shell-terminal`'s `native()`, which the browser's
  shell takes), never widened, and the view's bounds are derived from
  them as the shell derives its own: a page shows what a person's
  browser would, a window's rows and a chat's lines included (`../view.md`,
  4), so that its outline is the live page's. A world holds more than a window shows; the page shows the
  window, and the state's test row says what it shows.
- **Names,** binding: a state a mockup draws is named by
  its address, `<page>-<world>[-<step>][-<task>]` (`chat-solo-T10` for
  `chat.html?world=solo&task=T10`, `chats-first-run-1` for
  `chats.html?world=first-run&step=1`); one no mockup draws takes a
  plain name (`signed-out`, `chats-empty`, `adoption-refused`). A name
  is never reused for another state.

### 2.1 Project pages and settings

Moved to [`gallery-project-states.md`](gallery-project-states.md), section 1.

### 2.2 Journeys and interactions

Moved to [`gallery-journey-states.md`](gallery-journey-states.md), section 1.

## 3. Its tests

In `crates/temper-client-gallery-tool`'s own tests, in the view's world: each state's tree
read by skein's fake person (`../../testing/fakes.md`, 4), as the
gallery builds it.

- **The names:** the states' names, in order, are `crates/temper-client-gallery-tool/names.txt`,
  kept by hand, so a name gone, changed or added fails until the file
  says so.
- **Each state holds what its name says:** a table of rows, each a
  name, its page's heading and the one part the state is about, found
  by role and name (the proposal in `chat-solo-T10`, the three
  suggested requests in `chat-new-first-run-1`, the alert in
  `adoption-refused`).
- **The print:** the program run into a scratch directory writes every
  name's page, the index linking each, and the bundle's stylesheets in
  `scripts/bundle.sh`'s order, read from that script.
- **Their cost:** together at most 1 s (0.5 s before the user's doubling of
  10-22, provisional) of the default suite; no fuzzy test.

## 4. Open questions

None.
