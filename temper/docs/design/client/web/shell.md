# The browser's shell

The browser's shell and its host are split from [the browser overview](README.md).

## 1. The shell

- **`temper-client-shell-web`** is a library, for
  `wasm32-unknown-unknown` and on every target for its host
  and its journeys (`journeys.md`, section 2), ordinary Rust as a shell is (skein's
  `ui/README.md`, section 2), copied from jig's reference browser shell,
  `jig-client-test-shell-web` (jig's `protocol/browser/README.md`, 3.1), with
  temper's client root, view and records in place of jig's test
  application's, and none of jig's look (`jig-client-shell-web`'s
  `look`, `jig.css`). Its modules:
  - `lib`: the start (the page's internal settings, below; skein's
    restart part; jig's sign-in part taking
    the link from the address before anything is sent; the small store;
    the page with temper's look; the DOM) and the turn of skein's
    `ui/web/README.md`, 4.2; and, as jig's reference shell has it, the test
    address `/test/panic`, at which the shell panics at start, for
    skein's `panic` story (`testing.md`, 1);
  - `host`: the client host: temper's client root and view, and the
    view's patch queue, reserved at start at the view's limit on
    patches and drained each turn (skein's `ui/web/README.md`, 4.2); the root's
    `Leave`, an authorization's departure or a removed forge's start
    over, as skein's
    transport's navigation away, in the turn that asked it (skein's
    `ui/web/effects.md`, section 1); the root's `Tab`, a sign-in's tab
    or GitHub's way's (`../pages/accounts.md`, 1, `ConnectModel`, *The
    sign-in's tab*; `../forge/deployment.md`, *GitHub's way, in a new
    tab*), as skein's
    platform's, the opening before the turn's transport, and a refusal
    given back as a report of that turn; jig's
    mapping under `/jig` and its stream; the root's requests to jig's
    adapter (`jig-client-shell-web`'s `adapter`) and skein's transport,
    and their reports back to the root, the root settled once per report
    before its requests are carried (jig's `client/domain/root.md`,
    section 3), a batch's answer read a piece at a time as the stream is
    (jig's `protocol/client/default-http.md`, section 6); and the wake, skein's timer
    set at the earlier of the root's deadline and the instant the
    view's words next change (`../view.md`, 6); its seam, a report
    given and the duties asked back, a turn's drawing and its deadline,
    the shell's turn and the journeys' replay call alike
    (`journeys.md`, section 2);
  - `journey`: a recording's records and file, its recorder and its
    replay (`journeys-recordings.md`, sections 1 and 2), on every target; the
    recorder's keys and save are in `lib`, installed only when the page
    records, by its `internal-record` or its `temper-record` meta (below);
  - `records`: the small store's records, `jig-client`'s and a sign-in's
    address (`../sign-in.md`, section 1); from the
    adoption form, a pending adoption is among them, a kept
    request whose request `temper-client-protocol-wire` encodes
    (`../forge/records.md`);
  - `limits`: the client's limits in a browser, derived as the
    terminal's are (`../README.md`, section 2), and their sum; and
    `RECORDING_BYTES`, 67,108,864, a recording's bound in memory
    (`journeys-recordings.md`, 1.1): two of `BODY_BYTES`, so that a recording
    holds a whole body's answer beside what came before it.
- **A page is an entry crate on the shell:** the library holds no wasm
  start, since a module has one and a page's crate links the library.
  `temper-client-web-page`, a `cdylib`, is the shipped page: its start
  calls the shell's, and the bundle is built from it (`bundle.md`, 1).
  `temper-playground-page` is the playground's (`../../playground.md`,
  3 and 6): skein's worker platform where the shipped page holds the
  browser's (skein's `ui/web/README.md`, 15), driving the shell's loop
  through its public seam (the shell, its platform's type and its
  report), the starter, the frame and the failure page with the
  session's log. The shipped bundle holds none of the playground's.
- **The page's internal settings** (`../../draft/cli.md`, section 6) are
  query parameters of the address the page loads at, read once at its
  start, before anything else reads the address, and taken out of it
  then, with the history's replace the sign-in part uses, so that no
  route, recording or link sees them. Each is on when its value is `1`,
  and off when absent or anything else:
  - `internal-record`: the browser's recorder runs (`journeys-recordings.md`, 1.1),
    on any origin, `http` or `https`;
  - `internal-small-window`: the client's limits name a conversation's
    window of 32,768 bytes, with `long_text` and `turn_text` 32,768
    (`testing.md`, 3, The tier's window), where they name none and
    131,072;
  - `internal-loopback-navigation`: skein's platform follows a
    navigation to plaintext loopback (skein's `ui/web/effects.md`, section 1).

  `internal-small-window` and `internal-loopback-navigation` are
  honoured only on a loopback origin, the page's host `127.0.0.1` or
  `[::1]`, as the browser tier and the journeys tool serve it, and
  ignored elsewhere: loopback navigation elsewhere would let a page
  send a person's browser to plaintext services on their own machine,
  the small window is the browser tier's. `internal-record` is
  honoured everywhere: the recorder adds no node, no style and no
  request, and a recording leaves the browser only by the person's own
  Alt+Shift+S, or the save before a departure, into their own downloads
  (`journeys-recordings.md`, 1.1), so a link that sets it on someone else's page
  takes nothing from them. Each of the three is taken out of the
  address wherever it is given, honoured or not. A reload is a load
  without them. The shell's limits are given at its host's start, which
  the replay gives the ordinary ones.
- **The page's `temper-record` meta:** a page made with `<meta
  name="temper-record" content="on">` (`serving.md`, 1) records as if its address
  had `internal-record=1`, on any origin, at every load: a reload, or a
  return from a departure, starts a new recording (`journeys-recordings.md`, 1.1).
  The shell reads it at start, beside `temper-sign-in`; a meta of any
  other content, or none, turns nothing on. The address's setting and
  the meta are read once each and joined: either turns the recorder on,
  and neither turns it off.
- **It decides nothing.** What a page shows and does is the domain's
  and the view's, step code under the subset; the shell orders the
  platform's work (jig's `protocol/browser/README.md`, 3.1).
- **The failure page:** past the restart part's limit, the page says
  "The application failed.", in temper's words, since it takes none of
  jig's look; it starts nothing and keeps the small store (jig's
  `protocol/browser/README.md`, 5.1).
- **The small store** is the browser's session storage, under the
  prefix `temper.`, holding `jig-client`'s records alone, as the
  terminal's file does (`../sign-in.md`, section 1); from the adoption form
  a pending adoption is among them, a kept request
  (`../forge/records.md`). Each load of the page is a client of its own, with
  its own stream (jig's `protocol/browser/README.md`, section 10). The root can
  step the client several times in one turn: each write the client asks
  for (`Save`, `Erase`) is answered once, after the turn's whole storage
  has finished, and every one is refused if any write was refused.
- **What the terminal says in lines of its own** (`../root.md`, 1.4, and `../line.md`, 6
  ): another party's kept work and kept work dropped have no place
  in the browser, whose small store lives as long as its tab, of one
  person (`README.md`, section 3). A service's credential is never shown: temper's
  client makes no service (`../README.md`, section 5).

The person's zone is one of the shell's inputs: `Date`'s offset, in minutes
east of universal time, read at start and recorded in a journey's facts, so a
replay says the same clocks (`../view.md`, 6).

## 2. Open questions

- **Kept work in the browser** (`../root.md`, 1.4, and `../line.md`, 6): another
  party's kept work, and kept work dropped at start, which the terminal
  says in lines of its own. Default: not shown; the small store lives
  as long as its tab, of one person.
