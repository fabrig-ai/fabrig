# The cut's decisions for component-kit-2

Where the draft (`skein/docs/design/draft/component-kit.md`) leaves a
choice to the plan, the choice and why. The specs bind them; this file is
the why. Survey on the trunk's base, `9b74022d7` (code as at `6de0cd271`).

## Scope

- **skein only.** The draft's counts and sites are skein's (the review's
  scope), and `skein/scripts/old_forms.py` counts skein's tracked files.
  The copies in smith, jig and temper (their own `io::resume` loops,
  `earliest`s, dispatchers) are a later run.
- **Additive public changes.** lib, io, shell and world gain items and
  provided methods; nothing existing changes signature. The other trees'
  141 `Host` impls and their referees compile unchanged. The one new enum
  variant other trees may match is `Request::Abort` in the two forge
  components; its tasks pre-approve the one-arm edit in readers.

## The counts (draft, 4)

Added by `counts`, counted over skein's tracked Rust files from `skein/`,
comments excluded. Path exclusions are prefixes from `skein/`.

| Form | Counts | Baseline | Target | What the target keeps |
|---|---|---|---|---|
| `io_resume` | `(skein_)?io::resume(` outside `crates/skein-io/` | 39 | 3 | `tests/io/src/{output,processes,world}.rs`: io's own bound tests, which measure each step's submissions against its `MAX_OUT` |
| `earliest` | `fn earliest` / `fn earlier` outside `crates/skein-lib/` | 4 | 1 | skein-oauth's `documents::earliest`, which reads a document's tokens |
| `lifecycle` | `enum Lifecycle` outside `crates/skein-lib/` | 3 | 1 | `tests/io/src/processes.rs`'s, a child's run (`Running`, `Exited`, `Closed`) |
| `growth_floor` | `const GROWTH_FLOOR` | 5 | 1 | lib's one floor |
| `pop_loop` | a `for _ in 0..` loop whose whole body is one discarded `pop()` | 21 | 0 | |
| `hex_decimal` | `fn hex` / `fn decimal` outside `crates/skein-lib/` | 26 | 5 | `tests/json/src/reference.rs` (the independent reference parser), skein-fake-forge-github `pages::hex` (random digits), skein-forge-setup `reconcile::hex` (a whole secret), skein-fake-llm-protocol's test `hex` (a whole text), skein-json `string::hex` (appends a digit to a `\u` unit; its digit comes from lib) |
| `host_dispatch` | an `fn completions(&mut self)` whose body opens with `match self` or `self.host_mut()` | 9 | 0 | |

A site task that leaves a site by its family note's defaults names it in
its report; the end report adds it to the target with that reason.

## 6.1 The forwarding `Host`: a `Delegate` trait (option 2), measured

Decided, as the brief asks (the recommended option, measured where cheap).

- **Measured on two dispatchers**, at the survey:
  - `tests/oauth-accounts/src/world.rs`: its `impl Host for Process` is
    52 lines today. Option 1 (`impl<H: Host + ?Sized> Host for Box<H>`)
    leaves it at 52: the world's referees match `Process::Client` and
    `Process::Issuer` (34 uses), so the world cannot hold a
    `Box<dyn Host>`. Option 2 leaves a 14-line `impl Delegate for
    Process` (two accessors, a two-arm match each). Option 3 leaves 52.
  - `tests/tls/tests/suite/fake_peer.rs`: 26 lines of `impl Host` plus
    the 14-line accessor pair it already has. Option 1: 40 (its `Judge`
    matches `Process::Sender`). Option 2: 14. Option 3: 40.
  - Over the nine dispatchers (44 to 70 lines each, 401 in all), option 2
    leaves about 126; options 1 and 3 save nothing where a referee needs
    the variants, which is every one of the nine.
- **The form:** in skein-shell, `pub trait Delegate { fn host(&self) ->
  &dyn Host; fn host_mut(&mut self) -> &mut dyn Host; }` and one `impl<D:
  Delegate> Host for D` forwarding every `Host` method, provided ones
  included. It ends the drift by construction: today 2 of the 9 forward
  `exit` and 1 forwards `state_digest`.
- **Why the generic item is allowed:** skein-shell is shell code, not
  step code, so programming-model.md 10's subset does not bind it; the
  item is one short impl, documented in place as the exception to
  section 4's preference for concrete composition; and a concrete
  delegate type cannot hold the typed variants the referees read.

## 6.2 A component-kit role: proposed, not landed

The user's (question `component-kit-role` in `plan.toml`). Proposed
wording for `skein/docs/foundation/programming-model.md`, 4, after lib's
role:

> **The component kit.** Between lib and the components sits concrete
> mechanism that components of more than one protocol share: step pieces
> a component holds by value or calls as free functions, never a trait it
> implements. Its first member is io's stage (`skein_io::stage`); the
> shared transport over io and tls is the next. It names no protocol and
> no domain, holds no policy, and depends only on lib, io and tls.

Nothing in this run waits for it: `skein_io::stage` lives in io, as the
draft places it.

## 6.3 Anything hard to reverse

None. No stored format or protocol changes. Every public change is
additive. `Abort` adds a request variant and no event or error variant:
temper matches the components' `Event` and errors exhaustively, so a new
terminal would change a public type it uses. `Abort` ends what is open as
`Close` does, without waiting, and `Closed` stays the one terminal
(programming-model.md, 5.2; forge-connection.md, 8.5; forge-setup.md,
6.3).

## Smaller choices the specs bind

- `skein_lib::earliest(Option<Time>, Option<Time>) -> Option<Time>`, one
  free function; the `&mut` shape becomes `*next = earliest(*next,
  Some(at))`.
- Hex is `skein_lib::hex::{digit, value}`; decimal parsing is
  `Decimal::parse` beside `Decimal::of`. A writer of decimal digits uses
  `Decimal::of(n).as_bytes()`, which exists.
- Slab iteration is a cursor, `Slab::first` and `Slab::after`, over the
  live (unretired) entities in slot order: a borrowing iterator could
  not be held across the `&mut self` steps oauth-accounts' walks take.
- The growth rule is one crate-private function and floor in lib
  (`growth.rs`), used by the five containers.
- `Host::next_mechanism_deadline` defaults to `None`, so the other trees'
  hosts, which all state `next_deadline` themselves, are unchanged; a
  host that stops stating `next_deadline` must state it.
- The stage's `up` keeps the echo model's `now` argument and order.
- The stage sites also move their hosts onto the provided deadline,
  `Delegate` and the `Referee` defaults, in the files they own: it is the
  same code, and a second pass would order another task after each.

## Ordering and the critical path

- No contract task waits for another but `kit-slab` and `kit-shell`,
  which build on `kit-lib` (lib's growth rule shares its files;
  the provided deadline uses `earliest`).
- `counts` first for every task that lowers a count.
- `echo-stage` before every other stage site (draft, 3.2).
- `abort-*` before the forge tasks that share their crates' files (the
  review's D9: before any refactor).
- **Critical path:** `kit-io` (40) → `echo-stage` (30) → `stage-oauth`
  (50): about 120 implementer-minutes. Beside it: `kit-lib` (35) →
  `kit-shell` (35), 70, which the stage sites also wait for; `kit-lib` →
  `lib-oauth` (40) → `lib-accounts-slab` (40), 115; `abort-connection`
  (45) → `lib-forge-connection` (35), 80.
