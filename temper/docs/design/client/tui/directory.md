# The TUI's directory

The TUI's directory and the engine's local link are split from [the TUI](README.md).

## 1. Its directory

- **Where:** `$XDG_STATE_HOME/temper/tui` where `XDG_STATE_HOME` is an
  absolute path, else `$HOME/.local/state/temper/tui` (skein-shell's
  `Base::State` for the tool `temper`, then `tui`); with neither, the
  start is refused. It is the TUI's alone, never `temper-box open`'s or
  `temper ctl`'s, and no argument names it.
- **Private,** as `temper-box open`'s (`../line.md`, 1): the binary makes it and its
  missing parents `0o700` and refuses it unless it is a directory of
  this user's with no group or other bits (skein's `runtime/io/files.md`, 3), then
  makes the origin's own directory beneath it the same way (*One
  directory per deployment*, below) and gives that directory alone to
  `jig-client-shell-native`, as `temper-box open` gives DIRECTORY.
- **What it holds:**
  - `address`, the origin the TUI last signed in to, as `Origin::print`
    prints it (1), `https://NAME[:PORT]` or `http://HOST:PORT`, and
    nothing else, `0o600`. The binary reads it once at start, at
    most 300 bytes; one absent, empty, unreadable or not an origin
    reads as none, and the default holds. Once the TUI has closed, if it
    signed in at an origin the file did not hold, the binary writes
    the file whole through skein's io (skein-shell's `store_file`, a
    synced temporary replacing it).
  - `at/HOST/PORT`, each origin's own directory (below).
  - `recording-TIME`, a run's recording, while `TEMPER_INTERNAL_RECORD`
    is set, each run's its own and the last ten kept (`journeys.md`,
    3.1), `0o600`.
- **One directory per deployment** (`../../draft/tui-remote.md`, 6):
  `at/HOST/PORT`, HOST and PORT `Origin::authority`'s two parts, the
  origin's name or its IP address as Rust prints it (`127.0.0.1`,
  `::1`), and its port in decimal, 443 included. Each component stays
  within a file name's 255 bytes, and names and IP addresses never
  meet, so neither do the two schemes. The binary makes it and its
  missing parents `0o700` and refuses it as the TUI's own (above). It
  holds that deployment's:
  - `token`, the bearer credential, and `store`, the small store, both
    the native shell's files, kept as `../sign-in.md`, section 1 says;
  - `sign-in`, the note of how the TUI's last sign-in at the origin
    stands, one line, `0o600`: `AUTHORITY until NANOS`, signed in there
    until that wall time in nanoseconds, as the sign-in's answer said
    (jig's `client/domain/client/watches.md`, 2); or `AUTHORITY signed-out`,
    its sign-in there ended before that time, so that only a person's act
    signs it in there again (`pages/sign-in.md`, 2, *Expired is not signed out*);
    AUTHORITY `Origin::authority`'s, `127.0.0.1:8367` or
    `dev.fabrig.ai:443`. It is
    `temper-client-tui`'s `Note`, printed by `Note::print` and read by
    `Note::read`, at most 300 bytes, `Note::MOST`, room for a 253-byte
    name with its port. The binary reads it once at start: a
    line for another origin than the TUI's, or one absent, unreadable
    or past its bound, reads as none. The run writes it whole at each
    duty `note` (3), through skein-io's file layer on its own io, as
    the recorder saves (`journeys.md`, 3.1): a synced temporary
    replacing it. A write that fails says nothing, and the next note is
    written whole again.
- **No credential is offered to another engine:** the native shell has
  the origin's directory alone, so a credential and the requests kept
  meet only the engine they were made at; a token is never offered at
  another address, where it would hand one deployment's bearer to
  whatever listens at another's.
- **Switching signs nobody out:** a box and a deployment by name each
  keep their own sign-in, so going from one to the other and back asks
  nothing again while each sign-in lives.
- **The theme stays the person's:** it is a record of the small store
  (3, `Start.kept`; `theme.md`), so an origin's store, at its first
  start, when the binary makes its directory, takes the `theme` record
  of the store of the origin `address` names, the one used last, and
  nothing else of that store; the run saves it into the new store as a
  theme taken is saved. A store with no `theme` there, or none to read,
  starts at Auto.
- **Nothing new in a versioned format:** `address` and `sign-in` are
  lines of text, `token` the credential's bytes, and `store`
  `jig-client`'s, as before, so an upgrade of the laptop's build reads
  them as it did.
- **From the directory before it:** a build before this one reached an
  engine on loopback alone and kept `token`, `store` and `sign-in` in
  the TUI's directory itself, writing the token during its run but
  `address` only once it closed, so neither file alone says whose the
  token is. At its first start, a build with this moves the three into
  the directory of the address `sign-in` names, only where `address`
  names that same address (`http://HOST:PORT` and `HOST:PORT`);
  otherwise it removes them, and the TUI signs in afresh, which on a
  box takes nothing (`pages/sign-in.md`, 2). The move goes through
  `at/HOST/PORT.new`, made private, the three renamed into it, then it
  renamed whole into place; a start that finds a `.new` finishes that
  move, and one whose destination exists already removes the old
  files. The move runs before the origin's directory is opened, whatever
  the origin the TUI starts at, and a failure in it refuses the start
  as the directory's does (1).
- **The engine's link, kept at rest.** While the printed link is
  enabled (`../sign-in.md`, section 1), the running engine, `temper-box serve` alone or under
  `temper box`, keeps its live local link, a one-time secret of its own
  beside the printed link's (jig's `protocol/client/sign-in.md`, *The
  local link*), with the address, in a private file, so that a TUI on
  the same machine signs in with nothing to paste (`pages/README.md`). The file never
  holds the printed link's secret: a TUI's sign-in spends the file's
  link alone, and the printed link stays live for the person's browser,
  never touched, printed again or spent by a TUI. The file's secret is
  otherwise held only in jig's memory: never on serve's stdout or
  stderr, nor in a log; the file is never in the configuration
  directory, which holds no secret.
  - *Where:* on Linux, `$XDG_RUNTIME_DIR/temper/printed-link-KEY`, where
    `XDG_RUNTIME_DIR` is an absolute path (a tmpfs, never backed up);
    with none, no file is kept and the TUI reads none. On macOS,
    `DATA/printed-link`, in the deployment's data directory. KEY is the
    first 8 bytes of the SHA-256 of the data directory's absolute path,
    as 16 lowercase hex digits. One function, `temper-shell`'s
    `link_file::path`, makes it for the engine and for the TUI.
  - *Its deployment:* the engine keys it by the data directory it runs
    on, however it was named; the TUI by the deployment its environment
    names, resolved as `temper box` with no flag resolves it
    (`../../protocol/configuration/directories.md`): `TEMPER_DATA`, else the
    XDG default. No flag names another: the command line's flags are
    designed with the user later.
  - *Written* whole, `0o600`, a synced temporary replacing it, in a
    directory the engine makes `0o700` with its missing parents and
    refuses unless it is this user's with no group or other bits, as the
    TUI's own (above): the local link's URL, made as serve makes the
    printed link's, then a line feed. Serve writes it at each `LocalLink`
    its startup hears, once its clients' listener is up, and prints none
    of it: at its start, and each time the component draws the next,
    after a TUI spent the last or a sign-in found it expired. A `Link` is
    printed alone and never written, and `temper box`'s first secret,
    the printed link's (`../../protocol/setup/box-supervision.md`, 1), is never written
    either. A write that fails is one line on stderr, `temper-box serve: link
    file PATH: WHY`, naming no secret, and serve goes on.
  - *Removed* when serve exits, by any way out it has. An engine killed
    outright leaves it, and its link then grants nothing (below).
  - *Read* as a private file of this user's, at most 4,096 bytes
    (skein-shell's `read_private_file`): absent, unreadable, too large or
    not private reads as none. The binary reads it once at start, for
    bare `temper`'s address (1) and, whatever the `http://` address, for
    whether the TUI is on a box; the run reads it at each duty `local`
    (3), for the sign-in (`pages/README.md`). It names an engine of this
    machine, so at an `https://` origin the binary reads none and the
    TUI is never on a box: `Start.local` and `Start.boxed` are false.
  - *On a box:* the file read at start holds a link, as
    `jig-client-protocol-http`'s `printed` reads it, whose `HOST:PORT`
    is the TUI's address: the engine there is this machine's temper box,
    and the TUI names its place "box" (`pages/chat.md`, 1, *The rule above the box*).
    The binary keeps the answer alone, `run::Settings.boxed`, and drops
    the secret. `temper-box serve` run alone writes the same file and reads
    as a box too: the file does not tell them apart. With no file, or a
    link for another address, the TUI is not on a box. The binary opens a separate root for
    the link's parent; the run resolves only its filename beneath that
    root and closes the descriptor on shutdown. A parent that cannot be
    opened reads as no link, without refusing the TUI's state directory.
  - *A stale file grants nothing.* Its address and its secret come from
    one file, written by the engine that holds the listener. A restart
    forgets links, and a spent link is dead (jig's `protocol/client/sign-in.md`
    ): a link that another engine answers, or that none
    answers, signs nothing in, and the TUI falls back (`pages/README.md`).
  - *The risk, accepted by the user, 2026-11-07:* a live owner link,
    one-time and live at most 2 hours, at rest and readable by any
    process running as the person, the box's own agents among them.
    Those can already read the TUI's credential (`token`, above), so this
    adds little. The file's link is its own, the user's pick of
    2026-11-08, so it adds nothing to the printed link's exposure, and a
    `temper` started before the person opens the printed link leaves that
    link to the browser. *The way back:* the file holds the address
    alone, and the sign-in takes one step, the paste or the browser's
    approval.

## 2. Open questions

- **On a box, told by the box** (1, *On a box*; `pages/chat.md`, 1, *The rule
  above the box*): in front, the box tells the TUI it is one (`box.md`, *The
  place*); bare `temper` reads the engine's link file, which names an
  engine of this machine, so `temper-box serve` run alone reads as a box
  too, where the user's request of 2026-11-08 names "box" for a temper
  box. Default: the file, for bare `temper`. Owner: temper/tui.
- **The TUI's address kept at closing** (1): the binary writes it once
  the loop has ended, so a TUI killed outright keeps the address it
  read. Default: so, though the run now writes the note at once with
  file io of its own; moving the address there waits for a need.
