# Loading the browser's bundle

Draft, 2026-11-06, from a session with the user that day: "can you mock a nice
loading placeholder for the client bundle? with progress. On slow connections
it can take a while. Probably we'll need it to be JS, for initial
responsiveness/speed, but if Rust could be equally fast, that'd be great". And,
of plain HTML and CSS for the paint, a script for the bar and a compressed
bundle: "ok for CSS+JS+compression". The look request `bundle-loading` draws
the pages and holds the script and the stylesheet
(`../client/journeys/bundle-loading/`). This draft carries what the look
cannot.

## 1. In one page

- **The first paint needs no script:** the page holds the placeholder as
  markup, with one small stylesheet of its own (section 2).
- **A small script moves the bar,** counting real bytes, and starts the client
  as today (section 3).
- **The bundle is served gzipped,** which shortens the wait itself (section 4).
- **A cached load shows nothing:** the placeholder appears only after 250 ms.
- **Rust cannot paint sooner** (section 5).

Measured at 2 Mb/s, on a debug bundle of 11.5 MB, under temper's policy:

| | With the placeholder | Today |
|---|---|---|
| First paint | 152 ms, 12 KB sent | 1,444 ms, 328 KB, blank |
| Something to read | 400 ms | 46.5 s |
| The client's first node | 46.46 s | 46.49 s |

## 2. The page

- **Its head** names the title `temper`, the placeholder's stylesheet and the
  module `boot.js`, and nothing else. A stylesheet in the head holds back the
  first paint and the module's start, which is why `temper.css`'s 250 KB moves
  out of it.
- **Its body** holds `#app` and, beside it, the placeholder: the mark as inline
  paths, a native `progress`, two lines, a hidden "Try again" and the noscript's
  words. With no font fetched, nothing swaps.
- **The policy doesn't change** (skein's `ui/web/bundle.md`, 4): nothing inline and no
  `style` attribute. A `progress`'s value and presentation attributes need
  neither.
- **Whose it is:** the bundler writes the page and the script (skein's
  `ui/web/bundle.md`, 3). It takes the placeholder's markup and stylesheet from the
  application, as it takes `--noscript`, and given none writes the page as
  today, so that skein names no application. The script's words are the
  markup's, as data attributes, since the phrasebook isn't there yet.
- **Sizes:** 3.6 KB for the page and stylesheet, of which about 1 KB is the word's
  paths, and 2.1 KB for the script; 2.9 KB gzipped together.

## 3. The script

- **The bundler writes** each file's name and size at its top, as the digest's
  directory fixes them.
- **The stylesheets** go in by script, in today's order, where they can't hold
  back a paint. Each counts whole when it has loaded.
- **The glue** is a dynamic `import()`, so that it can't hold the script back.
  It counts whole.
- **The module** is fetched by the script, its answer's clone counted as it
  arrives, and the answer itself given to the glue's `init`. So compiling while
  it downloads holds, as V8's trace shows it did.
- **The count is of decoded bytes** against decoded sizes, never against
  `Content-Length`, which section 4 would make wrong.
- **Handing over:** a `MutationObserver` on `#app`. On the first node, once the
  stylesheets are in, a 200 ms fade, then the placeholder is removed.
- **A failure** is a fetch's error, an answer that isn't OK or no WebAssembly. A
  stall isn't: it says it is waiting (the look request's message).

## 4. Compression

This takes up skein's `ui/web/README.md`, section 12, "the bundle served compressed", which
waits "for a bundle whose size over the network matters". It does:

| File | Raw | Gzipped |
|---|---|---|
| The module | 11,147,538 | 3,097,532 |
| `temper.css` | 250,882 | 37,857 |
| The glue | 54,030 | 9,344 |
| `task-pages.css` | 15,658 | 3,802 |
| `skein-ui.css` | 6,372 | 1,229 |

That is a debug build. At 2 Mb/s it would take the 46 s to about 13 s, by
arithmetic and not measured. A release build's sizes are for the plan to read.

- **The bundler writes** `NAME.gz` beside each `.wasm`, `.js`, `.css` and `.svg`.
  A `.woff2` or a picture is compressed already. The workspace has no compressor
  today, and the bundler, being a tool, takes one of its own.
- **Gzip alone:** every browser accepts it, on plain http too, where browsers
  don't offer Brotli. Brotli would save more, and can come later.
- **A server** answers `NAME.gz` with `Content-Encoding: gzip`, `Vary:
  Accept-Encoding` and `NAME`'s type, where the request accepts gzip, and `NAME`
  where not. The rest of skein's `ui/web/bundle.md`, 4 holds.
- **The figure** says what travels: where the module's answer says gzip, the
  script scales its total by the compressed sizes the bundler also wrote. The
  fraction is the same either way.
- **The bounds** grow to hold both forms: `clients.bundle_files` and
  `clients.bundle_bytes` (`../protocol/configuration.md`, section 7) and what the
  binary embeds (`../client/browser.md`, 2.6).

## 5. Why not Rust

A browser starts WebAssembly only from a script. A loader in Rust would still
need that script, then its own fetch, compile and glue before its first pixel, and
would reach for the page's nodes through them. The paint here takes neither, and
the bundler, in Rust, writes what the browser runs.

## 6. Steps

1. **Compression:** the bundler, the server's choice and the bounds. It needs
   nothing from the page, and is worth having alone.
2. **The page and the script:** the bundler's options and temper's markup and
   stylesheet.

## 7. Open questions

- **Other browsers:** tried in Chromium alone, so Firefox's and Safari's
  `progress` styles are unseen.
- **Both forms in memory:** a bundle held whole twice, or the plain form alone
  inflated for the rare client that refuses gzip.
- **The playground:** it would take the same page. Compressing it is its host's.
- **A box still starting:** a link opened before serve is ready gets no bytes
  until it is, and that wait stays the browser's own (`../protocol/setup.md`, 7.1).
