# Fetch progress

Draft, 2026-11-04, from a session with the user that day. While an
owned Forgejo's set up fetches its binaries, the person sees how far
the download has gone, as a percentage, in the browser and the
terminal, where today they see "Setting up: fetching" for as long as
it lasts. The bar that fills with it is skein's part, a draft of its
own after this one (section 6).

## 1. In one page

- **The fetch counts:** `temper forge fetch` knows each binary's size
  from its answer and the bytes it has written, and says both on its
  stdout as it goes (section 2).
- **The box passes them on:** for the forge it was asked to set up, it
  turns each into a line on its pipe to serve (section 3).
- **serve keeps the latest in memory,** with the forge's setting up,
  never in the store, and the forge's row carries it to every client
  (section 4).
- **The clients say it:** "Setting up: fetching Forgejo · 42%", in the
  browser and the terminal alike, from the client's domain (section 5).
- **Changed in place:** until v0.1.0 there is no supported deployment,
  so the pipe, the client protocol, its codecs and goldens change in
  place, with no version kept for an older side (the user, 2026-11-04).
- **The look in parallel:** a look request draws the target meanwhile,
  a percentage written in by hand beside the moving bar
  (`../client/journeys.md`, 7); this draft makes it true.

## 2. The fetch

- **Per binary, in order:** Forgejo, then its runner, as
  `../protocol/setup.md`, 6.6, fetches them.
- **The size is the answer's `Content-Length`:** a body longer than it
  is refused, as one past `setup.fetch_bytes` is. An answer without one
  has no size, and its progress is its bytes alone.
- **A line on stdout** at the binary's start, at each whole percent it
  passes, and at its end: the binary, the bytes written and the size,
  or none. With no size, a line at each MiB. A machine reads them; a
  person running the command by hand reads them too, so they stay
  plain.
- **Nothing else changes:** the digest, the part file, the rename and
  every refusal are 6.6's.

## 3. The box

- **A fetch line becomes a pipe line** to serve, for the asked forge's
  fetch step alone: `fetched FORGE BINARY DONE TOTAL`, within
  `box.line_bytes`, between that forge's `step FORGE fetch` and its next
  step (`../protocol/setup.md`, 7.1).
- **Its terminal shows none of them:** the step's log keeps them, as it
  keeps the step's other lines.
- **A line it cannot read,** or one for no asked forge, is named on its
  stderr and dropped, as a pipe line is.

## 4. serve and the client protocol

- **The box's link keeps the latest** `fetched` line of the forge
  setting up, in memory alone, as it keeps the ask's answer
  (`../protocol/forge.md`, 2.8): a restart of serve forgets it, and the
  next line brings it back.
- **The forge's row carries it:** its setting up gains the binary, the
  bytes and the size, present while the step is the fetch and a line
  has been heard, gone at the next step, at `started` and at `failed`.
- **Each line is a change of the row,** sent as any change is: at most a
  hundred or so per binary.

## 5. The clients

- **The row's words:** "Setting up: fetching Forgejo · 42%", then
  "Setting up: fetching its runner · 80%"; with no size, "Setting up:
  fetching Forgejo · 41 MB". The set up's step list says the same on
  its first step, "Fetching Forgejo and its runner · 42%"
  (`../client/forge.md`, 5.4).
- **Both shells, one domain:** the words are the view's from the
  domain's row, so the terminal says what the browser does.
- **The look** is the look requests' (the forge's pane beside the Model
  step, its moving bar, and the percentage beside it), landed as
  `../client/journeys.md`, 7, says.

## 6. The bar that fills: skein's

skein-ui's progress node carries text alone: "how far a piece of work
has gone, its text saying it" (`skein/docs/design/ui/README.md`, 3.1). A bar that fills with the percentage needs a value it maps to
the platform's (`aria-valuenow` on the web) and a width the stylesheet
can draw. That is skein's, in a draft of its own, after this one. Until
it lands, the bar stays the moving one, and the percentage says how
far.

## 7. Its tests

- **The fetch's,** at its host: the lines in order for an answer with a
  size and for one without; a body past its size refused.
- **The box's:** a fetch line relayed for the asked forge, one dropped
  for another, none on its terminal.
- **serve's:** the row's progress set from a line, and gone at the next
  step, at `started` and at `failed`.
- **The client's worlds:** the words in the browser's view and the
  terminal's print, with a size and without.

## 8. Later

- **One percentage for both binaries,** from sizes pinned beside their
  digests (`skein/docs/design/forge/README.md`, 4.3), known before the first
  byte, so that the bar does not start again for the runner.
- **The other steps' progress:** preparing and starting take seconds,
  and keep the moving bar.
- **A fake fetch for the fake box,** so that a journey records the
  percentage: the fake box fetches nothing a walk can see today.

## 9. Open questions

None at its writing.
