# Status

2026-11-11. Where skein's code lags or departs from its design, and its
design from its drafts, from the status files below. A job's log reads
by its start as well as its end, in both dialects (`forge/README.status.md`,
3.2). The full-screen
shell's renderer, keys, drawing and simulated terminal are built, with
panic restoration and simulated resizes. Rules, border-end titles, bands,
options menus, marks, unlabelled fields, a token's focused value's
paint and paging by the visible body are present, with end following and Escape released after the background
wait. A side region beside the body is built
(`ui/screen.md`, 5.2). Log-edge reporting and the other rendering and input extensions
remain deferred (`ui/screen.md`, section 10; `ui/screen.status.md`). The kernel's records of a tree's metadata,
the filesystem's status and btrfs's
subvolumes, and cgroups with contained spawn, are not built, nor their
simulation, faults and fake machine (`runtime/kernel.md`, `runtime/io/README.md`,
`testing/simulator.md`, `runtime/shell.md`, `testing/README.md`). The forge kit lacks the calls
no dialect has yet (the labels, `UpdateRef`) and GitHub's Actions
routes, `Artifact` and `Commits` among them (`forge/README.md`,
`forge/github.md`); seven of its artifact facts are unchecked at the
pinned Forgejo (`forge/README.md`); the App account's `GET /app` fact still has
no live observation (`forge/github.md`, `forge/setup/README.md`). A copy's facts
remain unchecked at the pinned Forgejo, one of them not
recorded at all, and no fact records a site admin's read of a repository
by its path (`forge/setup/README.md`), nor a comparison's `removed` (`forge/README.md`); the fake Forgejo leaves no repository
at a refused copy, asks a delete the wrong scope and answers a read by
path `403` to a caller with no grant, a public repository's too, and the
fake GitHub refuses no push as `GH013` (`testing/fakes/forge.md`).
The forge kit's targeted reads are designed and not built: `Range` for
containment and emptiness, `Item`'s `since`, `PullFor`'s `head` and `base`
filters and branch names bounded by bytes
(`forge/README.md`, `forge/github.md`, `forge/client.md`, `testing/fakes/forge.md`).
No macOS runner runs skein's suites, no build of the shell's `posix` for macOS is on record, and its two systems each keep eight functions alike but for a word (`runtime/shell.md`). json, tls and http's collector have no fuzz target
(`data/json.md`, `net/tls.md`, `http.md`). A few items lag in `runtime/lib.md`,
`examples.md`, `forge/setup/README.md`, `data/kv.md` and `ui/README.md`, and the LLM live
suite's logins have no design (`skein-live`), five of its provider
stories being manual, run by the owner (`llm/README.md`). Most carriers still
print the bytes they hold, credentials among them on known paths
(`runtime/lib.md`, `oauth/README.md`), as does a server's written head (`http.md`). The code departs
in `skein-playground`'s manifest and its session log's version (`playground.md`). An owner may ask a call
to keep its rendered body (`llm/README.md`, `llm/connection.md`), and a world the
fake LLM peer to keep what it read (`testing/fakes/llm.md`).
Provider-list parsing, the standalone exchange and fake list replies are
built; later pages and sharing the exchange shape remain deferred
(`llm/README.md`, 4.9 and 6; `llm/connection.md`, 4.3 and 9; `testing/fakes/llm.md`, 3.2).
`README.md`, `data/codec/README.md`, `testing/world.md`, `testing/fakes/checkout.md`,
`oauth/issuer.md` and `ui/text.md` have no gap. Updates recover for both
styles in the worlds; a merge update already holding its base and a
rebase overtaking a later push still lack real-Forgejo conformance
(`forge/client.md`, `forge/README.md`). The viewport story awaits its runtime
check after landing (`testing/browser.md`). The kit reads a toggle's pressed
state, and its harness awaits that state by a named target within the
step's bound (`testing/browser.md`, section 5; `ui/web/testing.md`, 2).
The three folds-away stories await the fixture's first apply before
acting or reading, retaining their folding, focus and outside-press
checks and their deadlines (`ui/web/testing.md`, 2).
Markdown flows have their own owner, bounded reader and deterministic
layout; scenes may state their height and the web puts that SVG in its
figure's flow, and their width, the web scaling that SVG to the
figure's width and a press's point back to the scene's pixels. The text
shell prints once a group's `NAME:` that its nearest such ancestor
printed. The case scenes and text fallback are held by host tests
(`ui/README.md`, `ui/web/README.md`, `ui/text.md`). Node hints are bounded text carried
by attribute patches, shown as `title` on the web and omitted by the text
shell (`ui/README.md`, 3.2; `ui/web/elements.md`; `ui/text.md`, section 5).
Underscore emphasis and strong keep word-internal underscores literal;
a field's options keep focus while arrows move, Enter presses and Tab
completes. The kept store and full-screen reports with Escape-only
locking are present; real-key checks belong to consuming applications
after landing (`ui/README.md`, 3.2 and 4.2 and `ui/markdown.md`; `ui/web/elements.md` and `ui/web/README.md`, 5, 8, 9 and 12).
Branches, merges, more shapes and
sequence diagrams remain later work (`ui/README.md`, section 15); the practice
and home of draft acceptance cases remain the user's open decision
(`draft/diagrams.md`, section 10).
The focused suite's budget is 40 CPU s, the user's of 11-08, and the
merge gate's full reads that day put the suite at 28.5 CPU s: 11.5 below
it, but 1.5 below the 30 CPU s limit under which a 2.5 CPU s margin was
requested. The focused cost repairs and shared world helpers establish no
whole-suite saving of their own. Selected instruction ratios
estimate 5.311441 CPU s saved by the focused repairs, without proving
their at-least-6 aim; process-world instruction pairs remain unknown.
App key admission parses without signing, fake JWT checks verify the
public half, and the row kit holds the new folds, but individual
instruction pairs and process-floor estimates establish no whole-suite
saving (`net/tls.md`, section 4; `forge/connection.md`, sections 3.5 and 5.2;
`testing/fakes/github.md`, section 1.1; `testing/README.md`, sections 6, 7 and 9).
Remaining allocator, fixture, product and startup cost work, and a
profiler supporting the process operations, are deferred (`testing/README.md`,
sections 5, 7 and 9). The codec's step tests count a reused pass's
allocations, but one of skein-lib's and one of skein-ui's still show a
table kept by its address (`testing/README.md`, section 5). The test-only fork
kit holds exhaustive crash sweeps' prefix once; checked allocation layout and native fork costs remain
deferred there, with their measurements in temper's deployment-world account.
The state machines of forge-connection, Forgejo's plans, the fake forge's
connection, oauth and its accounts, llm and
llm-connection depart from
`../foundation/programming-model.md`, 5.4; io's file store and private opening
and lib's `RequestTable` hold their states in enums, but keep some cells outside
its small handlers (`runtime/io/README.md`, `runtime/lib.md`); and the fakes share their
product's types against `../foundation/testing-strategy.md`, 4
(`testing/fakes/forge.md`, `testing/fakes/llm.md`); Forgejo's plans end a match over a decoded
kind in `_` against `../foundation/programming-model.md`, 10.3, for the user's
later review (`forge/README.md`); single departures stand in
`net/channel.md`, `testing/fakes/forge.md`, `forge/README.md`, `forge/connection.md`, `runtime/io/README.md`,
`data/json.md`, `data/kv.md`, `runtime/lib.md`, `net/link.md`, `llm/README.md`, `oauth/README.md`, `runtime/shell.md`,
`testing/README.md`, `ui/README.md` and `ui/web/README.md`. The bundler's loader draws no words for a
stall before its module's answer, its compressor's crates build optimized in
development with no sign-off, and no run of the browser tier on record holds the
loader, or a log's reading stories, within its budget. A read that failed at a log's head
is not asked for again on a move back within its reach (`ui/web/README.md`). The foundation's rules (`../foundation/`) have no status file: of `testing-strategy.md`, 6, no
tree runs transition coverage (`cargo llvm-cov`) or fuzzes a machine or
a decoder (`cargo fuzz`); of `cli.md`, what a tree doesn't follow yet is
its checks' exceptions and the `expect`s naming a round or milestone
(`../foundation/cli.md`, section 9), and no binary has a table yet, so none has checks 6 and 7.
Check 5 counts a declared variable as read or set wherever a tracked
file outside `docs/` names it, its own table's row among them, so a
declared variable nothing reads or sets passes it. `runtime/shell.status.md` and
`tools/lints/design.status.md` hold where its mechanism departs. Of the drafts,
`component-kit.md` leaves the forwarding `Host` and component-kit role open,
the design departs from `redb.md`: the store is `data/kv.md`'s own machine, and `progress-value.md`'s
status holds a departure the user has not answered, a progress's text
as runs of its own.

Design documents:
- [`README.md`](README.status.md)
- [`runtime/lib.md`](runtime/lib.status.md)
- [`runtime/kernel.md`](runtime/kernel.status.md)
- [`runtime/io/README.md`](runtime/io/README.status.md)
- [`runtime/shell.md`](runtime/shell.status.md)
- [`testing/simulator.md`](testing/simulator.status.md)
- [`testing/world.md`](testing/world.status.md)
- [`testing/README.md`](testing/README.status.md)
- [`examples.md`](examples.status.md)
- [`data/codec/README.md`](data/codec/README.status.md)
- [`data/json.md`](data/json.status.md)
- [`data/kv.md`](data/kv.status.md)
- [`http.md`](http.status.md)
- [`net/tls.md`](net/tls.status.md)
- [`net/link.md`](net/link.status.md)
- [`net/channel.md`](net/channel.status.md)
- [`llm/README.md`](llm/README.status.md)
- [`llm/connection.md`](llm/connection.status.md)
- [`testing/fakes/llm.md`](testing/fakes/llm.status.md)
- [`oauth/README.md`](oauth/README.status.md)
- [`oauth/issuer.md`](oauth/issuer.status.md)
- [`forge/README.md`](forge/README.status.md)
- [`forge/client.md`](forge/client.status.md)
- [`forge/connection.md`](forge/connection.status.md)
- [`forge/github.md`](forge/github.status.md)
- [`forge/setup/README.md`](forge/setup/README.status.md)
- [`testing/fakes/forge.md`](testing/fakes/forge.status.md)
- [`testing/fakes/checkout.md`](testing/fakes/checkout.status.md)
- [`ui/README.md`](ui/README.status.md)
- [`ui/text.md`](ui/text.status.md)
- [`ui/screen.md`](ui/screen.status.md)
- [`ui/web/README.md`](ui/web/README.status.md)
- [`testing/browser.md`](testing/browser.status.md)
- [`playground.md`](playground.status.md)

Drafts:
- [`draft/component-kit.md`](draft/component-kit.status.md)
- [`draft/diagrams.md`](draft/diagrams.status.md)
- [`draft/progress-value.md`](draft/progress-value.status.md)
- [`draft/redb.md`](draft/redb.status.md)
