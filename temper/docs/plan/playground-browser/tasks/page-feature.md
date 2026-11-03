+++
id = "page-feature"
part = "page"
kind = "feature"
needs = ["shell-turn-split"]
files = [
  "temper/crates/temper-client-shell-web/Cargo.toml",
  "temper/crates/temper-client-shell-web/src/lib.rs",
  "temper/crates/temper-client-shell-web/src/playground.rs",
  "temper/crates/temper-client-shell-web/src/playground/**",
  "temper/crates/temper-client-shell-web/playground/index.html",
  "temper/crates/temper-client-shell-web/playground/playground.css",
  "temper/docs/design/client/browser.status.md",
  "temper/docs/design/playground.status.md",
  "Cargo.lock",
]
effort = "high"
minutes = 150
+++

# The page: temper's shell with the feature `playground`, its starter, frame and failure page

## Goal

"Built from the shipped shell, `temper-client-shell-web` with its feature `playground`, which
holds skein's worker platform where the shipped shell holds the browser's: the shell's platform
is a type the feature chooses, and nothing else of its loop changes" (`temper/docs/design/
playground.md`, 6). Give the shell that feature: the starter that starts the worker and the
client at the printed link, the frame around the client, the failure page, and the page's own
HTML (`notes/contracts.md`, C5).

Acceptance:

- C5 exactly: the wasm32 `Shell` generic over its platform; `pub mod playground` (portable) with
  `web` (wasm32) built with or without the feature; the feature choosing the entry; `Report`'s
  `Frame` and `World` variants; `playground::{Control, STORIES}` and the shared element ids;
  `playground/index.html` and `playground/playground.css`.
- **The starter** (`playground.md`, 6): shows the frame and a loading line; draws the seed
  (`platform::facts().seed`) and reads the wall; starts the worker with `Worker::start` at
  `worker_address()`, the story chosen (`a` first), the seed and the wall, the bundle's digest
  directory's name as the log's build, a log bound in bytes the implementer sets (`LOG_BYTES`,
  with a comment); waits for the first `Heard::Printed` from `temper serve` on standard output;
  puts the link's path and query in the address (`history.replaceState`); then starts the client
  as the shipped `start` does from there (`sign_in::web::take_from_page`, the host, the page),
  with no sign-in `meta` (`Ways::NONE`, no forge sign-in) and no `Restart`.
- **The frame** (`playground.md`, 2 and 6): what this is ("temper's playground: a scripted model
  and forge, running in your browser"); the story's description (`STORIES`); pause, ½×, 1×, 10×
  through `Worker::pace`; reset and the story's choice, each ending the worker and starting a
  fresh world and client in the same page (one life per page; no reload, nothing fetched);
  the world's time from `Worker::clocks`; the memory's high water and the longest slice from
  `Worker::high_water`, in MiB and milliseconds; temper's version (`CARGO_PKG_VERSION`); the
  download as words with no link until the site's design names its address (plan question
  `download-address`); "Download the session log" (C5). A hidden tab pauses the world and the
  shown tab resumes it (`Worker::visibility` on `visibilitychange`).
- **The failure page** (`playground.md`, 6; `skein/docs/design/playground.md`, 7): a
  `Heard::Failed` or a panic of the page's module ends the worker, hides the client, and shows
  that the playground failed, its words, the session log's download and "Start over"; it never
  reloads by itself.
- Native unit tests of the portable half (`src/playground/tests.rs`): the frame's figures' words
  (time, MiB, ms, speeds), the printed line read as a link's path and query (only `temper
  serve`'s standard output, only `/jig/v1/sign-in/link?secret=`), each control's report, and
  `STORIES` naming `a` with its description non-empty.
- `heavy cargo clippy --target wasm32-unknown-unknown -p temper-client-shell-web -- -D warnings`
  passes, and the same with `--features playground` and with `--features record`; `heavy cargo
  build --target wasm32-unknown-unknown -p temper-client-shell-web --features playground`
  passes; natively `heavy cargo nextest run -p temper-client-shell-web` passes with the trunk's
  count plus yours, `heavy cargo clippy -p temper-client-shell-web --all-targets -- -D warnings`
  passes, and `heavy cargo check -p temper-client-journeys` (temper/tools/journeys)
  builds.
- The shipped shell's behaviour is unchanged (`Restart`, meta tags, browser platform and timer).
- Status edits below.

## Inputs

- `temper/docs/plan/playground-browser/notes/common.md`;
  `temper/docs/plan/playground-browser/notes/contracts.md`, C4 (the worker's name) and C5.
- `temper/docs/design/playground.md`, 2, 6 and 11 (the frame's mockup and the hidden tab are
  Open: a plain frame in temper's look, paused when hidden, are the defaults).
- `skein/docs/design/ui-web.md`, 18 (the worker's platform); `skein/docs/design/playground.md`,
  5 (the bridge, the speed in thousandths, the worker's ready), 6 (the log, offered as a file)
  and 7 (fail-stop, the page finding `boot-worker.js`).
- skein's API: `skein/crates/skein-ui-web/src/platform/worker.rs` (`Heard`, `Page`) and
  `platform/worker/web.rs` (`Worker::{start, pace, visibility, timer, clocks, high_water, log,
  end}`, `worker_address`); `skein/crates/skein-ui-web/src/bridge.rs`.
- The bundler's `--page` (`skein/docs/design/ui-web.md`, 4.3; `skein/crates/skein-ui-bundle/
  src/page.rs`): the page is copied with each `{bundle}` replaced by the digest directory's
  address, so `playground/index.html` holds its own stylesheet links (`{bundle}skein-ui.css`,
  `temper.css`, `task-pages.css`, `playground.css`), `<script type="module"
  src="{bundle}boot.js">`, and `<div id="app">`.

## Owns and may change

- The files in `files`. `Cargo.toml`: the feature `playground` (empty or naming web-sys
  features); web-sys features the playground's web half needs, added to the wasm32 dependency's
  base list since that half builds without the feature; `js-sys` made a plain wasm32 dependency
  if the log's download needs it, `record`'s `dep:js-sys` then dropped.
- `src/lib.rs`: `Shell` made generic, `Report`'s variants, the entry's `cfg_attr`; `start`'s
  body may move into helpers the playground's starter shares, its behaviour kept.
- Not yours: `src/host.rs` and the journeys (a host change is a question), skein's crates, the
  bundle scripts (`bundle-playground`), the `headers` template (`page-policy`).
- Status, edited in place, never reflowed:
  - `temper/docs/design/client/browser.status.md`, **Not yet**: delete the words from "2.1:"
    through "`loopback-navigation` alone." (its lines 5-7 at this plan's cut), leaving "-
    **Not yet:**" at the start of its line and the following "4.3: ..." line as it is.
  - `temper/docs/design/playground.status.md`, **Not yet**: delete "3: `temper-client-shell-web`'s
    feature" and "`playground`. ", leaving "- **Not yet:**" and "4: the" as they stand; replace
    "6: the page, its starter" by "6: the playground forge's links" and "and its frame." by
    "shown as text.". The other words stay.
  - Each file's date line today's.
- Pre-approved options:
  - **The private trait** over the two platforms (clock, timer, `end`) is yours to name; if the
    browser's `timer::web::Timer` cannot sit behind it without changing the shipped loop's order,
    keep the shipped loop's calls and put the worker's beside them under the trait, saying so.
  - **Reset and a new story** may rebuild the inbox and the `Shell`; if skein's `Inbox` cannot be
    started twice, make a new one per life and drop the old after `Worker::end`.
  - **A panic hook** of the page's module: install your own (the shipped shell's `Restart` is
    not installed in the playground); it must not reload.
  - If a web-sys feature or the worker's platform makes the shipped shell's bundle change size by
    more than 1 % (the dev-profile `.wasm` with no feature, before and after), report both sizes.

## Steps

1. `Shell` generic over its platform, the shipped shell unchanged; its native tests and wasm32
   clippy.
2. `playground.rs` (portable): ids, `Control`, `STORIES`, the figures' words, the link's reading;
   its tests.
3. `playground/web.rs`: the starter, the frame, the failure page, the entry; `index.html` and
   `playground.css` in temper's look (`temper.css`'s colours and type).
4. The status edits; the citations check.
5. `cargo fmt`, old-form count, the clippy commands, the native tests; commit per step; push.

## Report

As the implementer's text says, and also: each clippy command's outcome; the shipped `.wasm`'s
size before and after and the playground's; the log bound chosen; what the frame shows, listed;
anything of 6 or 7 you could not meet.
