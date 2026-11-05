# The box's link, first and alone

Draft, 2026-11-05, from a session with the user that day. The
project's site tells a person to download the binary and run `./temper
box`, and shows the link it prints. The user asked that
`temper box` print the link at once, its port already open, and print
nothing else: "It should print nothing else, and only refreshed links.
Except for errors, those would be printed too, but that's it." The user
chose this version over a progress page. It changes `../protocol/setup.md`,
7.2 (the box's terminal and its sign-in block), and jig's
`protocol/client.md`, section 8 (the printed link's secret). The user
asked for it to be built (2026-11-05).

## 1. In one page

- **What the terminal shows** (section 2): the sign-in block, in the
  user's words, before anything starts; then only fresh links and
  errors.
- **The link first** (section 3): the box draws the first link's secret
  itself and prints it at once. It gives the secret to `temper serve`,
  whose sign-in component takes it as its first link instead of drawing
  one.
- **The port first** (section 4): the box binds the clients' listener
  before it starts anything, and hands the bound socket to `temper
  serve`. A browser that opens the link early waits on the connection
  until serve answers, then signs in.
- **The rest goes to the box's log** (section 5): the first run's line,
  `Logs:`, `Starting`, the groups' lifecycle, `Stopping` and `Stopped`,
  and `Ctrl-C stops temper.`

## 2. What the terminal shows

At start, before any step starts:

```
Open this link in a browser on this machine.

http://127.0.0.1:8367/jig/v1/sign-in/link?secret=...

It signs you in as this temper's owner, once and for a short while.
Opened again once spent, it prints a fresh one here.
```

- **The link alone on stdout,** the other lines on stderr, as today, in
  that order (`../protocol/setup.md`, 7.2). The link's address is the
  configured public origin and link route, as today.
- **A fresh link,** after a spent or expired one was opened or when a
  restarted serve draws one, is printed alone: a blank line, then the
  link, on stdout. The block's last line already says it comes.
- **Errors** are printed as today, in today's words, on stderr: the
  box's refusals and failures, a step's exit and its restart, and the
  box giving up. Each line after the block keeps its time of day,
  `HH:MM:SSZ `.
- **Nothing else:** no first run's line, no `Logs:`, no `Starting`, no
  group's starting, started or stopped, no `Stopping` or `Stopped`, no
  `Ctrl-C stops temper.`.
- **`--echo steps`** keeps adding every step's lines, for a person who
  asks for them.
- **`--fake-backends`** keeps its `box:` lines (the effective
  directories, the fakes' addresses, the model key and the GitHub App's
  key file): the help's later steps read them (`temper help box`,
  `temper help github`). They follow the block.

## 3. The link first

- **The box draws the secret,** as jig's component draws one today: one
  time, from the platform's randomness, before it starts any step.
- **It gives it to `temper serve`,** on serve's start, through a channel
  serve's only reader holds (its standard input, or an inherited
  descriptor), never its arguments or environment.
- **jig's sign-in component takes a given secret.** At start, where it
  draws the first link's secret (jig's `protocol/client.md`, section 8),
  it takes the given one instead, if any. Its expiry runs from when the
  component takes it. All else is section 8's: one link live at a time;
  a spent, expired or unknown secret draws a fresh one; not ready, busy
  and full spend nothing.
- **A restarted serve** draws its own first secret, as today, and the box
  prints it as a fresh link (section 2). The box gives a secret to serve's
  first start alone.
- **A link opened before serve is ready** is held on its connection
  (section 4), then signed in. One opened once the box has given up gets
  no answer, and the terminal has said why.

## 4. The port first

- **The box binds** the clients' listener, `application.clients.listener`,
  before any step starts. A taken address is refused then, before the
  block is printed, in today's words.
- **It hands the bound socket to `temper serve`** as an inherited
  descriptor. Serve takes it, rather than binding its own, when it is
  given one.
- **Across serve's restarts** the box keeps the socket, and gives it to
  each start. No other process can take the port while temper runs.
- **Until serve accepts,** a browser's connection waits in the socket's
  backlog, and the browser shows its own loading.

## 5. The box's log

What leaves the terminal is written to the box's own log,
`DATA/box/logs/box.log`, each line with its time of day, as the
terminal wrote it before. `temper help box` names the log where it names
`Logs:` today, and its "Expect:" lines become the block.

## 6. Open questions

- **A progress page:** a page the box would serve while it starts, then
  hand the port to serve. Left out for now. Today's start takes a few
  seconds (about 4 s on a laptop's debug build, first run included), and
  the browser's own loading covers it.
- **The time of day on error lines:** kept; the user may prefer none.
