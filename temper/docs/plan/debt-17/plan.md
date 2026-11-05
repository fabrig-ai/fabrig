# debt-17: the seventeenth debt round

Ten small items (`inputs/request.md`), one part each. This file grows with each push. Every item was re-measured at this
plan's base and is still open. D003 reads 86, not the item's 97.

## Parts and tasks

| Item | Part | Task | Needs | Effort | Min | What it does |
|---|---|---|---|---|---:|---|
| hearth D008 | `hearth-path` | `hearth-path` | | medium | 15 | the fixtures move into `suite/memory/` |
| skein D034 | `codec-once` | `codec-once` | | medium | 15 | `build.rs`, `names.rs`, the `#[path]` go |
| skein D044 | `terminal-namespace` | `terminal-namespace` | | medium | 20 | namespace 4 of `Token::tagged`, the same bit |
| skein D003 | `env-with` | `env-with` | | medium | 40 | 75 sites onto `Env::with` and `Now::with` |
| skein D005 | `civil-date` | `civil-date` | | high | 60 | `Civil` and `Wall::civil` in lib; ui, http and forge on it |
| skein D006 | `lru` | `lru` | | high | 50 | `skein_lib::Lru`; both dialects' tables on it |

Still to come in later pushes: skein D008, D019, D026 and jig D027, each surveyed first.

## Critical path

Nothing here waits on anything yet: `civil-date`, 60 minutes.

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

## Overlaps

`hearth-memory-3`, a cut in flight beside this plan, may touch the config memory suite. `hearth-path` goes first and is a
small move, so whichever lands second merges it.
