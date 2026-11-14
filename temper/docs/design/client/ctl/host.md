# The host's tools

Provisional, 2026-11-14. The tools `temper ctl` answers on its host, sending no
request: signing in and out, what its store keeps, and its help. The reads and
acts are `tools.md`'s.

| Tool | What it does |
|---|---|
| `sign-in FILE\|-` | signs in by the link `temper-box serve` printed, read from FILE, or from stdin for `-`; writes `token` and `address` (`README.md`, 6.1) |
| `sign-in --service FILE` | signs in as the service whose FILE `party make` wrote, jig's `SignIn { with: Service(credential) }` at the address FILE names; writes `token` and `address` as `sign-in` does, so every later call acts as that party, within its role's authority and pool (`README.md`, section 7). FILE readable by others than its owner, or not in `party make`'s form, is refused before anything is sent, exit 1 |
| `sign-out` | ends the sign-in and leaves `token` empty, zero bytes, read as signed out (`README.md`, 6.1) |
| `pending` | lists what DIRECTORY's store keeps: each request's operation, its party and when it was first sent, and how many records these tools cannot read |
| `help [GROUP [VERB]]` | the page `ctl`, then the tools (`testing.md`, 5); with GROUP, its rows; with GROUP VERB, its record of the table |

- **`pending` reads the store's file only:** it takes no lock, starts
  no client and sends nothing (`README.md`, 6.3).
- **A `pending` row** is a line each: the request's operation, as a
  `kept:` line names it (`results.md`, 1); its party, `party N`; and when it was
  first sent, `first sent HH:MM UTC`, the documents' words for a wall
  time (skein's `runtime/lib.md`, section 12), or `not sent`. No name and no
  key, as a `kept:` line prints none. Under `--json`: `pending`, rows
  of `operation`, `party` and `first_sent`, a wall time as jig's
  integer, or `null`; and `unread`.
- **A record these tools cannot read,** of a store version, or of a
  family's owner or version, that they do not declare, is counted, never
  listed: jig's client reads nothing of such a record, drops it at start
  and says how many (jig's `protocol/client/README.md`, section 7), and
  `pending` says what a call's client will. Its line, `N kept records
  these tools cannot read; a call that sends drops them`, follows the
  rows, and the call exits 0. A file that is not a small store's at all
  exits 1, "DIRECTORY's store cannot be read".
- **A TEXT of `-`** is read from stdin, to its end. A call takes at
  most one `-`, and none with `--call -`; otherwise it is a usage
  error.

## 1. Open questions

None.
