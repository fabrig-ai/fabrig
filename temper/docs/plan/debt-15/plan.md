# debt-15: carry out the user's answers

Three independent parts (`inputs/request.md`). All are in this plan.

## Parts and tasks

| Part | Task | Needs | Effort | Min | What it does |
|---|---|---|---|---:|---|
| `temper-wrong-kind` | `temper-wrong-kind` | | medium | 30 | a wrong kind's `Corrupt` stops the engine; the one behaviour change in temper |
| `github-media-type` | `github-media-type` | | medium | 30 | `Server`'s charset switch goes; the three approved test changes |
| `d013-kept` | `d013-kept` | | medium | 5 | hearth's D013 `kept` |
| `d001-kept` | `d001-jig` | | medium | 30 | `config::clients::invalid`, `link_owner`, the read check's test moves to the store |
| `d001-kept` | `d001-temper` | `d001-jig` | medium | 25 | temper-service takes them; D001 `kept` |
| `d011-names` | `fake-names` | `github-media-type` | medium | 45 | the measure reads 0; a commit's bytes are the domain's |
| `d011-core` | `connection-core` | | high | 75 | `skein-fake-forge-http` and `connection`, no reader yet |
| `d011-route` | `route-limits` | `fake-names` | high | 90 | `git::Limits`, `Unrouted`, `status`; temper's call sites |
| `d011-route` | `route-move` | `route-limits`, `connection-core` | medium | 45 | `git`, `Hosted`, `Faults` move; GitHub drops the Forgejo fake |
| `d011-servers` | `forgejo-on-core` | `route-move` | high | 90 | Forgejo's `Server` on the core |
| `d011-servers` | `github-on-core` | `forgejo-on-core` | high | 90 | GitHub's `Server` on the core |
| `d011-rename` | `forgejo-rename` | `github-on-core` | medium | 60 | `skein-fake-forge-forgejo`, temper following; D011 goes |

`listener-off-loopback` has no task: its two edits are design text, written in this cut (below).

## Critical path

`github-media-type`, `fake-names`, `route-limits`, `route-move`, `forgejo-on-core`, `github-on-core`, `forgejo-rename`: 450
minutes, all on the two fakes' crates, so one writer at a time. `connection-core` joins at `route-move`. Parts 1 and 2 share
nothing with it but `github-media-type`.

## Overlaps with the drafts beside this one

- **`repository-copies-1` could not be kept off.** It hasn't landed, and it changes both fakes (42 files):
  - `P/lib.rs`, beside `Faults`: `route-move` takes that struct out.
  - `P/server.rs`, `P/server/git.rs`, `G/git.rs`: neighbouring lines. Its new `server.state = State::Waiting` can merge
    cleanly and fail to compile once `forgejo-on-core` has moved the field: the fix is the field's path.
  - New files under the Forgejo fake's directory (`setup/migrate.rs`, `tests/migrate.rs`). This is why the rename is the
    last task: the longer that draft has to land, the less it meets a moved directory. The rename's spec says what to do
    with files that have arrived.
- **`temper-fake`'s `forge.rs`** is touched by `repository-copies-1/fake-box` too: one type path here, and the crate's name.
- **`Cargo.lock`** is `fail-stop-names`' too: mechanical.
- **`fake-forge.status.md`:** `github-media-type` edits its long **Not yet** line, so D011's entry is a line of its own at
  the file's end, off that line.
- **debt-14 and hearth-memory-1:** none. D011 names no hearth or smith file, and `d013-kept` touches one item debt-14 isn't
  changing. `d013-kept` runs apart, on one file.

## Decisions

Parts 1 and 2:

- **`listener-off-loopback` is done in this cut's push, not in a task.** Both of its edits are design text: the **Departs**
  entry of `temper/docs/design/protocol/configuration.status.md` now says the user chose the check, and section 11's open
  question goes. Section 3's table stays as it is, as the request records the gap rather than moving the section.
- **`WRONG_KIND` isn't `Token::new(PREFIX)`,** as the held spec sketched. A source's restore reads under its own number,
  so exchange 0 could be a real read's owner. Bit 59 is above every exchange jig's accounts issue (their highest is bit
  39) and under the `tag` assert.
- **A wrong kind on any `secret()` read stops the engine,** not only a restore's: jig's accounts give the owner one
  `InvalidSecret`, and jig's reference stops on all of them too.
- **D001's sizing test moves to `jig-protocol-store`,** the crate that holds `check_read_page`. It's the test of the
  check, not of either service's composition, so this is the right tier and no budget moves.

D011, from two read-only surveys:

- **One neutral crate, `skein-fake-forge-http`,** for the core and the route together. The item's title is the fake GitHub
  standing on the fake Forgejo, and a neutral `Limits` alone would meet the letter of **Paid when** and leave that
  dependency. With `git`, `Hosted` and `Faults` moved, GitHub's fake drops the crate altogether. `-connection` was refused
  as a name: `skein-forge-connection` is something else.
- **The neutral crate doesn't reuse `skein-fake-forge-protocol`,** though the name would fit. A draft still writing to the
  old path would land in the wrong crate, silently.
- **`git::status` is a table of its own,** with a test pinning it to `errors::answer` on the git route. Going on calling
  Forgejo's table would keep a change to the Forgejo fake able to break the GitHub fake, which is why the item matters.
- **`git::Unrouted` is local,** one variant, so `git::name` keeps its shape and its assertions.
- **`Connection` reads `send` from the limits, per call,** as GitHub's does. Forgejo stores it at `new`; the hosts pass the
  same limits to both, and `forgejo-on-core` stops and asks if a test doesn't.
- **The servers move one after the other,** though they're different crates. Each may add to the core, so this keeps the
  core one writer's and asks no question; it costs 90 minutes on the critical path.
- **`routes::object` and `routes::commit` stay:** their readers run across skein and temper. They now stand on the domain's two functions.
- **The rename's sweep carries the design's crate names,** so the code and the words move in one commit, and the design
  needs no **Not yet** for each document. `fake-forge.md`, 12 names the target, and one status entry covers it until then.
- **"The protocol layer" stays the words of `fake-forge.md`, section 4:** the item asks the crate's name, and the section's
  title is cited.

**Proportion.** The core is about 170 production lines, and each server loses about 80: net code is about +10. The payoff is
one copy, not fewer lines. The item's "about 200 lines" holds: the survey found about 230.

**The item's figures:** its measure reads 28 at this plan's base, not 26. Its server sizes are stale too (922 and 812).

No `[[question]]`: the user's answers cover everything here.
