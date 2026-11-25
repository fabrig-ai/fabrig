# layouts-both-ways L5: the wires' translation, counted

2026-11-24, on the goal branch at `0bfdbcb710`, before L5's changes. The
count `temper/docs/design/draft/codec-shape.md`, 4.1 asks of goal L5,
owner by owner: each declaration of each wire's schema, read against
each crate's hand-written translation and its domain type. Read-only
surveys, declaration by declaration, with the kinds of
`codec-shape-k5-count.md`, 1; lines approximate. *Direct* is a
declaration a crate uses as its own vocabulary, with no type of its own.

## 1. The kinds

| Family | Crate | Decls | Copy | Scalar | Owner's shape | None |
|---|---|---:|---:|---:|---:|---:|
| jig worker wire | `jig-protocol-engine-workers` | 146 | 43 | 6 | 21 | 76 |
| | `jig-protocol-worker-link` | 146 | 42 | 5 | 23 | 76 |
| temper worker wire | `temper-worker-service` | 38 | 16 | 0 | 5 | 17 |
| | `temper-engine-service-vocabulary` | 38 | 8 | 0 | 7 | 23 |
| smith charter | `smith-protocol-translate` | 22 | 14 | 0 | 7 | 1 |
| | `jig-protocol-charter` | 22 | 13 | 0 | 7 | 2 |
| smith turns | `smith-protocol-translate` | 68 | 15 | 0 | 12 | 41 |
| | `jig-protocol-transcript` | 68 | 0 | 0 | 8 | 60 |
| smith channel | `smith-host-protocol` | 118 | 31 | 2 | 19 | 66 |
| | `smith-protocol-translate` | 118 | 21 | 1 | 13 | 83 |
| | `smith-protocol-channel` | 118 | 10 | 1 | 6 | 101 |
| | four others, one to three each | | 6 | 1 | 3 | |
| smith pieces | `jig-protocol-worker-agents`, `jig-protocol-charter`, together | 9 | 4 | 0 | 5 | 9 |
| jig client wire | `jig-protocol-engine-clients` | 1,108 | 215 | 49 | 196 | 648 |
| | `jig-client-protocol-link` | 1,108 | 272 | 49 | 158 | 629 |
| jig public, cursor, views | engine clients, `jig-client-view` | 58 | 18 | 3 | 10 | 27 |
| temper client wire | engine, five crates | 500 | 202 | 17 | 69 | 204 |
| | client, six crates | 500 | 54 | 4 | 29 | 99 |
| temper public wire | `temper-engine-domain-forge` | 28 | 6 | 0 | 18 | 4 |

temper's client end uses 314 of its wire's declarations directly, its
views drawing the records; every client reads both public wires' records
directly. jig's engine end translates about 10,800 lines by hand, its
client end about 11,250; temper's engine family about 3,500; each other
crate under 1,200.

## 2. What the owner's shape is

- **Framing and budgets:** rows and lists packed as pieces, a room
  charged as names are written, configured counts checked before
  allocation, a page's continuation.
- **State and lookups:** public indexes for names, model names and
  catalogues, the link's sign-in state, a run's parts in order, the
  forge configuration behind an address.
- **Mismatched sides:** a record split, joined or flattened, a field
  supplied from the enclosing ask, a variant one side lacks, a domain
  type with a checked constructor (jig's client `Text`, smith's names).
- **Checks:** unique names, bounds across fields, a head's length, which
  failures carry which facts.

## 3. What earned a form

- **One direction:** the largest recurring shape, at every end. About
  890 of jig's engine end's 1,020 translated declarations go one way,
  most of them out; 371 of its client end's 479, most of them in; 255 at
  temper's engine; 71 of 87 at temper's client; a worker wire's each
  message one way at each end. Generating both directions would leave
  half the wires' generated functions uncalled. Hence the directions
  (`skein/docs/design/data/codec/maps.md`, 2 and 4.1).
- **A type for each direction:** jig's engine end translates at least
  30 declarations against two domain types, among them some sent from
  one and received into another (an account's use from accounts out,
  into people in; a task's contract document out, a task's contract
  in). Hence a line per direction. Two types translated the same way
  stay the owner's.
- **`slice` on `stands for`:** six records of one list held as a boxed
  slice, across four owners (jig's worker wire at both ends, smith's
  channel twice, jig's client wire at each end).
- **Left by hand:** positional tuples, sixteen sites across four owners,
  most joined with a check or a split of the owner's, a handful pure; a
  variant's payload held in a named field, two; values from mutable
  state, recurring but each a different state under the owner's own
  checks.

## 4. Converters

Shared: a `Wall`'s nanoseconds on about 250 fields across the client
wires, a `Token`'s number on about 70, mostly the worker wires, and
UTF-8 text on about ten. The owners' own: bit sets (jig's engine, nine
fields), checked names and bounded text (jig's worker and client wires,
about 80), millisecond durations, job numbers and kinds, a signed
number's bits, nonzero numbers, commits and cursors as bytes (temper).
