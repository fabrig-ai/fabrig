# Opening the store: owner sets and frames' sites

## 1. Owner sets and frames' sites

- **Owner sets are data,** handed to the component's open by its shell
  beside the limits and its build's name, a `&'static str` the floor row
  records (`records.md`, 3): for each owner whose records it keeps, its
  families, their value types, its jobs and its level, and for a
  frames' family its frames' texts and patches, the generated modules'
  constants (skein's `data/codec/command.md`, 3), no function. A set
  carries no map patch, no older text of a map family, no fresh mark
  and no effects in flight (temper's `draft/store-floor.md`, 4.1 and
  4.2): no map record is rewritten but by a job or the seal, and a
  format outside the registry is fixed (temper's `draft/store-floor.md`, section 5).

```rust
// jig_protocol_store
pub enum Owner { Skein, Smith, Jig, Application }   // upstream first
pub struct OwnerSet {
    pub owner: Owner,
    pub name: &'static str,               // the owner, as its frames' patches' headers spell it
    pub families: &'static [KeptFamily],
    pub jobs: &'static [Job],             // its jobs at open, in the order they run (`jobs.md`, 1)
    pub level: Option<u32>,               // its level (`records.md`, 3): none for an owner the row records no level of
}
pub enum KeptMap { Store, Secrets }       // the map its records live in (`secrets.md`)
pub struct KeptFamily {
    pub name: &'static str,               // the family, as its texts spell it
    pub map: KeptMap,
    pub keys: &'static [Keys],            // its records' keys, and no other's
    pub frames: Option<Frames>,           // the frames' family's (`transcripts.md`, 6); none for every other
    pub current: &'static str,            // the binary's text of the family (`layouts.md`, 2)
    pub types: &'static [ValueType],      // a registered family's value types; empty otherwise (`layouts.md`, 2)
    pub retired: &'static [Retired],      // its retired value types (`layouts.md`, 2)
}
pub struct Frames {
    pub root: &'static str,               // the declaration it is in transcripts' frames, where their machine starts
    pub texts: &'static [&'static str],   // its frames' kept texts, in version order, the last at `current`'s version
    pub patches: &'static [&'static str], // the patches the frames' machine plans, its owner's, in number order
}
pub struct Keys {
    pub from: &'static [u8],              // inclusive
    pub to: &'static [u8],                // exclusive
    pub root: &'static str,               // the declaration its records are
}
pub struct OwnerSets {
    pub sets: &'static [OwnerSet],        // at most one per owner
}
impl OwnerSets {
    pub const NONE: OwnerSets = OwnerSets { sets: &[] };
}
```

- **Handed in by the shell:** `Store::owner_sets` takes `OwnerSets`.
  This crate's `owners` module declares `SKEIN_SET` and `SMITH_SET`;
  `JIG_STORE_SET` is its store-owned part of jig's set.
  `jig-protocol-accounts` appends its family to make `JIG_SET`.
  temper's `temper_protocol_store::OWNER_SET` and the shell's
  `OWNER_SETS` (`temper-engine-service-vocabulary`) complete the
  hand-over. `jig-test-store-files` takes fixture sets through
  `Store::with_owner_sets`. The `upgrade` module runs the copy (2).
- **Keys.** A family's records are its `map`'s, committed there
  (`secrets.md`). A family's ranges hold records of different
  declarations (the core's tasks beside their turns), so each range
  names its records' own root. Keys' bytes: the application's are its
  own keys, relative to its tag, as its rows are (`keys.md`, 1.2: the
  component adds the tag), and in the secrets' map relative to its
  secrets' prefix `[2, 0]` (`secrets.md`); every other owner's are the
  map's whole keys. A family's `keys` hold its own records and leave
  out what lies among them but is another's (the installation's
  identifier at `01 04 03` among the accounts', the settings' records
  among the core's): jig's from the component's own table (`keys.md`,
  1.4), the application's from its keys' encoders (`README.md`,
  section 7).
- **The frames' families** (`transcripts.md`, 6) are the two whose
  `frames` is set: in jig's set, the core's, whose `TurnRecord` is a
  frame's body, its `texts` the core's `current` alone and its
  `patches` none; in smith's, smith-transcript, whose `Turn` the body's
  `transcript` holds, with no `keys`, since none of its records is a
  map's, its `texts` and `patches` smith-transcript's generated
  module's (smith's `protocol/transcript.md`, section 12). Every other
  family's is `None`. A set with a frames' family in another slot, or
  two in one, is refused, `Set`; so is a body whose held turn names a
  family other than smith's frames' one, judged from the frames'
  texts, at every open before its machines are made (`transcripts.md`,
  6). A frames' family's `texts` must keep one version after another,
  each spelling the family's name, its last at `current`'s version.
- **Levels.** A set's level is its `level`, held explicitly: jig's 6,
  temper's 14, skein's and smith's 0, which no binary moves
  (`records.md`, 3). Every writer and reader of a level (the first
  commit, the seal, the comparison at open, the manifest, the rehearsal
  and the shell's versions line) reads `level`. temper's store shell
  hands the four real sets, `SKEIN_SET` and `SMITH_SET` of this crate,
  `JIG_SET` of `jig-protocol-accounts` and temper's `OWNER_SET`
  (temper's `protocol/floor.md`, 3). This crate owns
  `STORE_FAMILIES` and `JIG_STORE_SET`: its families' metadata,
  registered value types and written jobs. Accounts builds `JIG_SET`
  from that set, appending its own `jig-accounts` family; the store
  names no accounts consumer. The store's real-family layout and job
  tests use its own set. The component's other tests hand it fixture
  owners' sets: `jig-test-layouts`' fixture families for the registry's
  tests, and `jig-test-patches-jig` in jig's slot and
  `jig-test-patches-smith` in smith's, fixture frames' families whose
  frames are patched as read. No test owner names a production family.
  A fixture body family's last text is the core's `TurnRecord` at the
  binary's version under its own name, its `transcript` holding the
  fixture turn, since a patched body is decoded by the binary's codec;
  the fixture turn's last is its own, since the component hands a
  turn's bytes on undecoded. The names, texts and keys are `&'static`,
  the binary's own constants, never state: the only references in
  fields the component holds (skein/docs/foundation/programming-model.md,
  10.3), beside optional immutable parsed binary texts (below), read by each
  open's check and by each serving open, which makes
  the frames' machines from them (`transcripts.md`, 6).
- **A set is checked where it is admitted,** at every open, before the
  levels are compared, so a start at equal levels proves its sets too:
  one set per owner; the manifest's bounds, at most 64 families in all
  and each name at most 64 bytes, since the manifest counts every kept
  family by its name (`README.md`, section 2); each family's key ranges
  ordered and non-empty, disjoint from every other family's in the same
  map (and from the installation's identifier and the other rows the
  component keeps), within its owner's tag (the application's relative
  to it), and each range's root a declaration of the family's
  `current`; its value types (`layouts.md`, 2) and its jobs (`jobs.md`,
  1); the frames' families (above); and the holds: the plan refuses
  sets whose texts' held families (each family's `current` and each
  frames' family's `texts`) close a cycle. A set that fails is refused,
  `Set` with its owner, before anything is written. That each record of
  the map lies in one family's range takes a walk, which a rehearsal
  makes (5), not every open.
- **One check per open.** An open checks its sets once. By default each
  text it reads is parsed for it alone. A shell may supply immutable parsed
  binary schemas through `Store::prepared_schemas` before open. Admission
  borrows a supplied schema only when its entire text equals the text being
  checked; a missing or unmatched text is parsed by that open. These schemas
  contain no stored data, admission verdict or layout verdict: every admission
  and layout check still runs. jig's test store keeps only schemas derived
  from its constant Skein, Smith and jig owner-set texts across tests, never a store or an
  open's state; first starts and cold restarts remain real opens. The
  component owns no cache that outlives an open. Whichever step needs the check first
  runs it: a first start before anything is written (`opening.md`, 1.2), the
  rehearsal (5), the registry's (`layouts.md`, 5) or readiness before
  the open serves. Its verdict, passed or the `Set` it refuses with its
  owner, is that open's: kept through the copy, its check and the
  reopen (2), and read by each later step in place of a fresh check, so
  each step refuses where it did, with the same `Set`. The verdict is a
  few bytes beside the open's state, not a parsed text: dropped when the
  open settles, served (`Opened`) or refused, and whenever the shell
  hands new sets (`Store::owner_sets`) or prepared schemas, so an open's start finds none
  and no verdict outlives the open it was made in. The jobs' check the
  open makes alone at its first step (`jobs.md`, 1), before reading the
  format row, is cheap and stays where it is.
- **A frame's written site goes up.** Step code holds no function
  (skein's `data/codec/patches.md`, 4; skein/docs/foundation/programming-model.md,
  10.3), so where a frame read while serving meets `Step::Written`
  (`transcripts.md`, 6), the component answers `Site` to the read's
  token with the owner, the frame's key, the site, the node and the
  frames' `Patcher` itself, moved, never shared. The shell sets the key
  in a `Default` `Context` of that owner's for each frame
  (skein's `data/codec/patches.md`, 5), runs the owner's generated
  `written` over `parts()`, and hands the `Patcher` back in a `Site`
  request with the result; the component then calls `next()`. One
  frame waits on at most one site at a time. No map record goes up: the
  open runs jobs and seals, with no eager map patching walk.

```rust
// up                                      // down
Event::Site {                              Request::Site {
    owner: Token,                              owner: Token,
    of: Owner,                                 patcher: Box<Patcher>,
    key: Box<[u8]>,                            result: Result<(), Halt>,
    site: Site,                            }
    at: Node,
    patcher: Box<Patcher>,
}
```

- **A job's other stored inputs:** while a job's page is outstanding
  (`jobs.md`, 3), `Store::patch_page` reads a bounded page of public
  application rows from the copy's map, with relative keys and an
  exclusive cursor, for the shell answering the page. No secrets are
  read, no serving codec is selected by the component, and no input is
  retained for a later page. Outside a job's page the method refuses. It
  reads at most the store's admitted operations per page and reports the
  same data failure as the job's page.
- **The shell that answers** is the store's shell, the one that names
  the owners' crates and drives the store's open: the application's
  (temper's `temper-service`, its `Store`); jig has no production shell
  of its own. In jig's tests the store world's (`tests/store`) answers
  the fixture owners' frames' sites and jobs. `jig-test-service`'s
  hands no sets. `jig-test-store-files`' hands none either, unless a
  core world gives it fixture sets (`Store::with_owner_sets`, and
  `owner_sets` for the next open): then it opens over a fake machine's
  directory, so that the copy, the jobs and the switch run. A site its
  shell has no function for is answered `Halt::NoSite` (skein's
  `data/codec/patches.md`, 4), as their routes already do for any token.
- **A shell with no sets.** The component holds `OwnerSets::NONE`
  until a shell hands it sets, by a setter beside the constructors
  (`Store::owner_sets`); with none it never answers `Site`, never wants
  `Request::Site`, and `Opened`'s `pending` is false. A root that sees
  `pending` true asks for `Request::Switch` before its runs are adopted
  (2, step 7).
