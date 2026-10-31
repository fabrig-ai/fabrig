# Decisions for component-kit

The cut's choices where the draft (`skein/docs/design/draft/component-kit.md`)
leaves them to the plan, each with why. The specs bind them.

## Scope

- **skein only.** The draft's counts and sites are skein's (39 `io::resume`
  sites, the review's scope); `skein/scripts/old_forms.py` counts skein's
  tracked files. The copies in smith, jig and temper (their `earliest`,
  `hex`, `decimal`, `io::resume`) are a later run. Public changes to
  skein-lib, skein-io and skein-shell are additive, so the other trees'
  96 `Host` impls and their lib users keep compiling unchanged.
- **Crate-graph order** is the listing order of the tasks in each family:
  no site task builds on another's code, so they wait only for the
  contract tasks and for tasks that share their files.

## The counts (draft, 4)

Added to `skein/scripts/old_forms.py` by `ratchets`, counted over skein's
tracked Rust files, comments excluded. Baselines are those `ratchets`
records on the trunk (the survey's numbers in brackets); targets:

| Form | What it counts | Survey | Target |
|---|---|---|---|
| `io_resume` | `io::resume(` outside `crates/skein-io/` | 39 | 0, less sites left by `stage-common.md`'s defaults, each named |
| `earliest` | `fn earliest` / `fn earlier` outside `crates/skein-lib/` | 4 | 1: skein-oauth's `documents::earliest` reads a document's tokens, not a copy |
| `lifecycle` | `enum Lifecycle` outside `crates/skein-lib/` | 3 | 1: `tests/io/src/processes.rs`'s is a process's run (Running, Exited, Closed), not a copy |
| `growth_floor` | `const GROWTH_FLOOR` | 5 | 1, lib's one floor |
| `pop_loop` | `for _ in 0..` loops whose body only pops and drops | 21 | 0 |
| `hex_decimal` | `fn hex` / `fn decimal` outside `crates/skein-lib/` | 26 | 4: `tests/json/src/reference.rs` (the reference parser stays independent), skein-fake-forge-github's `pages::hex` (generates random digits), skein-forge-setup's `reconcile::hex` (encodes a whole secret) and skein-fake-llm-protocol's test `hex` (decodes a whole text) |
| `host_dispatch` | test `Host` impls that forward `completions` by `match self` | 9 | 0 |

The inline earliest-merges in skein-forge-connection are removed by
`lib-forge` and checked in its review, not counted.

## Ordering and the critical path

- `ratchets` comes first and every other task needs it, so no branch
  predates the counts it must not raise.
- Contracts first, small and parallel: `kit-lib`, `kit-io`,
  `shell-surfaces`; `kit-slab` after `kit-lib` (they share lib's
  containers and `lib.rs`), needed only by `lib-accounts-slab`.
- `echo-stage` before every other stage site (draft, 3.2).
- The stage tasks also move their hosts' hand-merged `next_deadline`
  onto the provided one: it is the same host code, and doing it apart
  would order a second task after each of them.
- `abort` comes before the forge lib sites (review D9: before any
  refactor), which share its crates' files.
- `referee-defaults` and `forwarding-sites` wait for the stage tasks
  whose files they share.
- **Critical path:** ratchets (15) → kit-io (35) → echo-stage (25) →
  stage-services (45) → forwarding-sites (40): about 160
  implementer-minutes, against the longest task (45) plus its contract.
  Others: ratchets → kit-lib (40) → lib-accounts (40) →
  lib-accounts-slab (40), 135; ratchets → abort-connection (45) →
  lib-forge (40), 100.

## Questions of the draft's section 6

### 6.1 The forwarding `Host`: the shorter of options 1 and 2, measured first

Decided (the brief: pick the recommended option, measured where cheap).
`forwarding-shell` writes options 1 and 2 on two dispatchers
(`tests/echo/src/proc.rs` and `tests/oauth-accounts/src/world.rs`) in
throwaway code, records each dispatcher's line count under each option in
its report, and lands the shorter (a tie goes to option 1):

1. `impl<H: Host + ?Sized> Host for Box<H>` in skein-shell, so a world
   whose referee needs no typed access holds `Box<dyn Host>` and the
   dispatcher goes; one whose referee does keeps its enum with one
   accessor pair (`fn host(&self) -> &dyn Host`, `fn host_mut(&mut self)
   -> &mut dyn Host`, a match each).
2. A delegate in skein-shell: the dispatcher states only that accessor
   pair, and `Host` is forwarded through it once, in skein-shell, for
   every dispatcher.

Recommended and expected: whichever leaves each dispatcher at one
accessor pair; the review's own fix names both. Why either is allowed:
skein-shell is shell code, not step code (programming-model 10's subset
does not bind it), the item is short and documented as the exception to
4's preference for concrete composition, and it ends the drift by
construction (`exit`, `state_digest` and `drain` are forwarded once,
where today 2 of 9 dispatchers forward `exit` and 1 `state_digest`).
Option 3, leaving the dispatchers, is the fallback only if neither
compiles under skein's lints; the task then asks.

### 6.2 A component-kit role: the user's, proposed here, not landed

Proposed wording for `skein/docs/foundation/programming-model.md`, 4,
after lib's role:

> **The component kit.** Between lib and the components sits concrete
> mechanism that components of more than one protocol share: step pieces
> a component holds by value or calls as free functions, never a trait it
> implements. Its first member is io's stage (`skein_io::stage`); the
> shared transport over io and tls is the next. It names no protocol and
> no domain, holds no policy, and depends only on lib, io and tls.

Nothing in this run waits for it: `skein_io::stage` lives in io, as the
draft places it.

### 6.3 Anything hard to reverse

None in this plan: every public change is additive (new lib, io and
shell items; a provided `Host` method with a defaulted companion;
`Referee` defaults; an `Abort` request variant that the components
match and their owners only construct). No stored format or protocol
changes.

## Smaller choices bound in the specs

- `earliest` is one free function over two options; the `&mut` shape
  callers use it as `*next = earliest(*next, Some(at))`.
- Decimal parsing is `Decimal::parse`, beside `Decimal::of`, so lib's
  exports gain only `hex`, `earliest` and `Lifecycle`.
- The growth rule is one private function and floor in lib
  (`growth.rs`), used by the five containers.
- `Host::next_mechanism_deadline` defaults to `None` so the other trees'
  hosts, which all override `next_deadline`, are unchanged.
- The stage's `up` keeps the echo model's `now` argument.
