+++
id = "deadline-words"
part = "deadline-words"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-shell-cli/src/waited.rs",
  "temper/crates/temper-client-shell-cli/src/host.rs",
  "temper/crates/temper-client-shell-cli/src/recovery.rs",
  "temper/crates/temper-client-shell-cli/src/transport.rs",
  "temper/crates/temper-client-shell-cli/src/lib.rs",
  "temper/crates/temper-client-shell-cli/src/tests.rs",
  "temper/crates/temper-client-shell-cli/src/tests/deadline.rs",
  "temper/crates/temper-client-shell-cli/src/tests/fixtures.rs",
  "temper/docs/help/ctl.md",
]
effort = "high"
minutes = 60
+++

# The deadline's words say what it waited for

## Goal

`OneShot::link_progress` says `the engine at ADDRESS did not answer within 30 s` whenever `until` passes, whatever is
outstanding. Make it say which of four things it waited for, and, for an act, what is true of the act: the two tables in
`temper/docs/design/client/tools.md`, 6.3, **The deadline says what it waited for**, word for word.

Acceptance:

- **Each row of both tables** has a test: the pure mapping and words, and a `fixtures::Engine` scenario for each state below.
- **Unchanged:** the 30 s; `nothing listens at ADDRESS`; the busy line and the streams-full line, still ahead of these; a
  sign-in's and a sign-out's `the engine at ADDRESS did not answer within 30 s`; the exit status 1; what stays in the small store.
- **The bound:** every combination of the two tables, at a 64-byte ADDRESS (`limits::ADDRESS`), is at most 240 bytes with its
  newline. A test holds it.
- **No line holds a request's words or a secret:** the words are the tables' alone, with ADDRESS.
- **Help:** the one line in `temper/docs/help/ctl.md` below, and its tests pass.

## Inputs

- `temper/docs/plan/ctl-waits/notes/common.md`.
- `temper/docs/design/client/tools.md`, 6.3 (the tables), 7 (kept, drawn and carried requests), 8.2 (the steps, streams full).
- `jig/docs/design/client/domain/client.md`, 8.2: *starting*, *live*, *offline*.

## Owns and may change

**Owns:** the `files` above.

**Departure from the implementer's text:** `docs/help/` is open to this task for the one line named under Steps. It's read by
tests (`.config/gate/read-by-tests.txt`).

**The survey, on the trunk this spec was cut from:**

- The message is written only at `host.rs` `link_progress`. Tests that assert it, all in `src/tests.rs`: 209, 699, 715, 735,
  1033, 1210 and 1244. Nothing in `temper/crates/temper/tests`, `temper/tests`, the stock charters or the other trees matches it.
- Each test's new words follow from its state. Expected: 209 the stream; 699 the outcome, kept under `--key`; 735 the link; 1033
  unchanged (a sign-in); 1210 a read. 715 and 1244 end the stream with an engine stopping, so *offline*, the link. Change a test's
  expected line only to its row, and say in the report which row each took. A test whose state is not what's expected here is
  read, not rewritten to pass: if the row looks wrong, that's the report's.

**Pre-approved options:**

- If `fixtures::Engine` can't leave a connect uncompleted, add the smallest hook to `tests/fixtures.rs`.
- If telling a Save of the act's own record from another write needs more than one flag in `Recovery`, use `!writing &&
  writes.is_empty()` and say so in the report. The cost is a replayed record's erase in flight reading as *not sent*.

Nothing outside this crate changes, but the help line. `temper-client-tools` is read, not changed.

## Steps

**1. A pure module,** `src/waited.rs`, `mod waited;` in `lib.rs`:

```rust
//! What a call waited for when the link's deadline passed (client/tools.md, 6.3).
pub(crate) enum Waited { Link, Stream, Read, Outcome }
pub(crate) enum Act { None, NotSent, Kept { named: bool }, Once { named: bool } }
pub(crate) fn waited(link: jig_client::Link, connected: bool, heard: bool, acting: bool) -> Waited;
pub(crate) fn line(address: &str, waited: Waited, act: Act) -> String; // without `temper ctl: ` and the newline
```

`waited`, with every `Link` listed:

| `link` | `connected` | `heard` | `acting` | `Waited` |
|---|---|---|---|---|
| `Offline`, `SignedOut`, `OutOfDate` | any | any | any | `Link` |
| `Starting` | false | any | any | `Link` |
| `Starting` | true | false | any | `Stream` |
| `Starting` | true | true | any | `Read` |
| `Live` | any | any | true | `Outcome` |
| `Live` | any | any | false | `Read` |

`SignedOut` and `OutOfDate` already fail in `client_link` before a deadline; they take `Link` so the match is whole.

**2. The facts in `OneShot`:**

- `connected: Option<()>`, sticky for the call: set in `complete` on a native `Op::Connect` that succeeded, and from `Memory`,
  which keeps a sticky bit beside its `connected` set (that set forgets at `Closed`).
- `act: Act`, `NotSent` for `Class::Act`, else `None`. At the act's `root::step` (`client_progress`, the `Class::Act` branch),
  before `self.value.take()`: `Once { named }` if `self.value.is_some()` or the tool is `rows::EXPORT`'s, else `Kept { named }`;
  `named` is `call.key.is_some()`.
- At the deadline, `Kept` reads as `NotSent` while that call's record isn't stored yet: a `Save` still queued or in flight in
  `Recovery` (`saving()`, kept beside `writing`).
- `Request::Progress` sets `Act::None`: that act has its answer, and the call owns a watch.
- `acting` is `class == Class::Act && waiting == Waiting::Answer`.

**3. `link_progress`:** after the refused, busy and streams-full checks, as they are: a `SignIn` or `SignOut` class keeps its line;
any other fails with `waited::line`.

**4. Tests:** `src/tests/deadline.rs` (`mod deadline;` in `tests.rs`), failing first. The mapping and words by table, the 240-byte
bound, and these scenarios:

- a connect left uncompleted: the link;
- the stream unanswered: the stream (a read), and the stream with `; it was not sent` (an act);
- `me` delivered and a read unanswered: a read;
- an act under `--key`, unanswered on a live link: the outcome and `; it is kept, and the same --key may be tried again`;
- an act with a drawn name: the outcome and `; it is kept, and the next call that sends tries it again`;
- `account add-key` under `--key`: `; it is not kept, and the same --key and file may be tried again` (`tests/moves.rs` and its
  fixtures have the way in);
- `account export`: `; it is not kept, and calling again is a new request`;
- an act sent, then its stream dropped with connections refused: the link and the kept words.

**5. Help.** `temper/docs/help/ctl.md`, *Limits*, replace `- The link's deadline is 30 seconds: no engine answer by then fails the
call (exit 1).` with:

```
- The link's deadline is 30 seconds: no engine answer by then fails
  the call (exit 1), and its line says what it waited for: the link,
  the stream, a read or an act's outcome. An act's line says whether
  it is kept, and whether the same key may be tried again.
```

**6. Checks:** `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-shell-cli
--all-targets -- -D warnings`; `heavy cargo nextest run -p temper-client-shell-cli`; the pages' readers, `heavy cargo nextest run
-p temper-docs -p temper-client-tools --lib`. Commit and push.

## Report

Which row each existing test took; what `Recovery` tracks for *not yet stored*; each new test's bite (the rule broken, the failure
seen); and any state in which a row's words read false to you, as a finding.
