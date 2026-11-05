# debt-17: the seventeenth debt round

Ten small items (`inputs/request.md`), one part and one task each. Every item was re-measured at this plan's base and is
still open. D003 reads 86, not the item's 97.

## Parts and tasks

| Item | Part | Task | Needs | Effort | Min | What it does |
|---|---|---|---|---|---:|---|
| hearth D008 | `hearth-path` | `hearth-path` | | medium | 15 | the fixtures move into `suite/memory/` |
| skein D034 | `codec-once` | `codec-once` | | medium | 15 | `build.rs`, `names.rs`, the `#[path]` go |
| skein D044 | `terminal-namespace` | `terminal-namespace` | | medium | 20 | namespace 4 of `Token::tagged`, the same bit |
| skein D003 | `env-with` | `env-with` | | medium | 40 | 75 sites onto `Env::with` and `Now::with` |
| skein D005 | `civil-date` | `civil-date` | | high | 60 | `Civil` and `Wall::civil` in lib; ui, http and forge on it |
| skein D006 | `lru` | `lru` | | high | 50 | `skein_lib::Lru`; both dialects' tables on it |
| skein D008 | `http-header` | `http-header` | | high | 70 | one reading of a head; skein-llm on its predicates |
| skein D019 | `channel-states` | `channel-states` | | high | 120 | the machine's flags become states; one close path |
| skein D026 | `crate-tests-move` | `crate-tests-move` | `codec-once` | medium | 50 | three suites move under `skein/tests/` |
| jig D027 | `cursor-schema` | `cursor-schema` | | high | 120 | the family `jig-engine-cursor`, with goldens |

## Critical path

`channel-states` and `cursor-schema`, 120 minutes each, side by side. `codec-once` then `crate-tests-move` is 65. Nothing else
waits on anything.

## Decisions

- **hearth D008 goes now.** The item says it waits for hearth's memory migration, which holds both files; the request sets
  that aside unless a `#[path]` needs `hearth-service-builder/src/lib.rs` or a `MAX_OUT` form. Neither does.
- **The terminal keeps bit 62, as namespace 4.** Every raw token stays the same. `temper-client-shell-terminal` keeps its own
  hand tag at bit 61, and `temper-box`, `temper-client-shell-cli` and smith keep theirs: the item's measure is skein's, and
  this plan stays out of temper's client work.
- **smith D003 stops waiting on skein D044.** `Token::tagged` is the lane the smith item asked for; its text says so, and it
  stays open.

- **D003's measure can't read 0.** It matches 11 sites that stay: two definitions and nine whose clocks aren't one value's.
  Three in `skein-fake-forge-github/src/server.rs` must not convert, since tests move those clocks apart. The spec's narrower
  search reads 0 after, and `env-with` also takes the 15 `Now` rewraps the old measure counted, since `Now::with` is the same
  one line.

- **IMF-fixdate's read stays in skein-forge.** The item puts it in skein-http, but skein-forge, its one reader, takes only lib
  and json (`skein/docs/design/forge.md`, 2.1). Only the arithmetic and the month names move to lib. `skein-http` keeps the
  write, and `skein_ui::date` keeps its public names as delegates, since jig reads them and this plan stays in skein.
- **`Civil::days` is signed.** `skein_forge::time::read` accepts a date just outside `Wall` when its offset brings it in, and
  going through `Civil::wall` would have refused it without a test noticing. The spec adds the pins.
- **`Lru` is a pair of B-trees,** after `Deadlines`, not today's scan. The price is a few nodes in tables of at most 16 rows;
  the forge's memory ceilings stay unmoved, and the task stops and asks if one trips. Two unreachable panics in Forgejo's
  table go (capacity 0, clock exhaustion); the spec names them.
- **`users.rs` stays, as an alias.** `forge.md`, 4.4 and `forge-github.md` name the root's logins' table, and the latter is
  read by tests.
- **temper's own calendars stay.** Six in `temper-box`, `temper-client-domain`, `temper-client-view` and
  `temper-client-tools`: the item is skein's, and fabrig/ui has priority over those crates. They're worth an item of temper's.

- **jig D027 changes the cursor's bytes, in place, without asking.** The request stops and asks on a stored format or a breaking
  public interface; this is neither, by the design. A schema can't hold today's bytes (big-endian integers, a `u16` version,
  variants without gaps). `jig/docs/design/protocol/client.md`, 6 says a client gives the bytes back as opaque; no store keeps them
  (`jig/docs/design/client/domain/client.md`, 11: "everything shown is read again"; no store schema has a cursor); `jig/docs/design/protocol/README.md`, 7 changes
  encodings in place until the first release, and there's no upgrade in place. A cursor from before is refused, not misread, and its
  read is answered unknown. The one holder that can outlive a build is temper's printed `--after <hex>`, which gets the same answer.
  The item's "across releases" is what the check is for, not a promise about these bytes. The plan's owner can overrule this:
  it's one branch to revert.
- **D027 grows in generated lines.** Hand-written `cursor.rs` falls from 652 to at most 450, but the generated module runs to some
  thousands of lines and about 100 goldens. The item asks for them by name, so the request's "less code" holds for what's written
  by hand. Goldens over the hand format would keep the bytes and leave **Paid when** unmet.
- **D027 keeps a tree's cursor as it behaves.** `Tree.next` is written in a form `decode_window` refuses, so a tree's continuation
  looks unable to continue. Behaviour stays, and the task reports it.
- **D019 pins five cells before the recast.** Three look like faults against `channel.md`, 8.3: a credential past its limit closes
  silently, a framing error after `Finish` in two-pipe mode never reaches a terminal, and so does a grant settled after a failure in
  one-stream mode. The request holds behaviour still, so they stay, pinned plainly, and the task lists them for the plan's owner.
- **D019's measure is partly a rename.** `read_outstanding` becomes a two-state `Demand`. The acceptance also searches for the flags'
  names and counts the writers of `Closing` and `Closed`, which is what the item is after.
- **D008's head measure sits in `client`, not `header`:** it needs `client::Body`. skein-llm keeps its stricter value check (ASCII only),
  which no test pinned; the task adds one.
- **D026 keeps two `#[path]`s,** each saying why: a test crate reaching fixtures its step crate's unit tests share. D028 lists them.
  The moved suite joins `tests/forge`'s half of each suite (`skein/docs/design/testing.md`, 7), and the task measures it.

## Overlaps

- `hearth-memory-3`, a cut in flight beside this plan, may touch the config memory suite. `hearth-path` has landed; whichever
  lands second merges it.
- Inside this plan, `civil-date` and `lru` each close an entry on the same line of `lib.status.md` and of `forge.status.md`, and
  both run at once: the second to merge resolves it by keeping both removals. The design and these specs went out together, so
  it couldn't be cut apart without holding one back.
- `env-with` touched `skein-llm/src/client.rs` and `skein-fake-llm-protocol/src/provider.rs` ahead of `http-header`, on other lines.
- No overlap with `claude-always`, `ui-signin`, `look-round-4` or `reference-walks`: this plan stays out of temper but for a check
  of `temper-client-tools`, and `terminal-namespace` ran `temper-client-shell-terminal`'s tests without changing it.
