# A component kit, phase 1: lib's primitives and the io stage

Draft, 2026-10-31. The first slice of the skein code-quality review of
2026-10-31 (source `3371027`; `fab/review-2026-10-31/` in fab's home):
the primitives each component writes again for itself, moved into lib
and io, and the call sites moved onto them. It changes no rule of the
programming model or the testing strategy; section 6 asks whether one
should change. Sections 1 and 2 are the goal; section 3 is the parts,
in order; section 4 the counts that may only fall; section 5 what is
left for later runs; section 6 the questions the plan raises and does
not settle. Read it with [the programming
model](../../foundation/programming-model.md) and [the testing
strategy](../../foundation/testing-strategy.md).

## 1. In one page

- **Shared mechanism is written once.** A deadline helper, a lifecycle,
  a growth rule, hex and decimal parsing and the io stage are each
  written privately in 2 to 39 places today, and the copies have
  drifted. Each moves to the crate whose role it fits: the generic
  pieces to lib, the io stage to io.
- **The model already allows it.** These are concrete step pieces that a
  component holds by value or calls as free functions: no trait, no
  generic item, no `dyn` (programming-model.md, 4 and 10). Only the
  forwarding `Host` of section 3.5 is in question (6.1).
- **Contract first.** lib's and io's new public items land alone, as a
  skeleton, before any call site moves. Every later part uses that
  surface and adds to it only by amending it.
- **One root end to end first.** The echo example's host moves onto the
  shared stage before the other 38 sites do, and its story test stays
  green on main.
- **Then crate by crate.** Each remaining site moves in crate-graph
  order, one crate per task: expand (the new form beside the old),
  move, contract (the old form removed).
- **Done is counted.** Each copy removed lowers a count of section 4; the
  run is done when each count reaches its target, zero where a part
  removes every copy.

## 2. Why

From the review (themes 1b, 1c and 1d, decisions D6 and D9):

- **Economy scores 2 of 5 in 11 of 12 areas.** The same mechanism is
  hand-written in each component, each dialect, each fake face and each
  test harness.
- **The io stage is written in every host.** 39 `io::resume` call sites
  sit outside skein-io. Only some check the kernel's submission room
  before taking work, which testing-strategy.md 6 requires of every one.
  The same ordering test exists twice, each `#[path]`-including another
  crate's private source.
- **lib's gaps are filled privately.** `earliest(Option<Time>, Time)`
  four times, plus inline copies in forge-connection; `for _ in 0..len {
  pop }` loops where `clear` is missing; the growth rule five times in
  lib itself, each with its own `GROWTH_FLOOR`; the same `enum Lifecycle
  { Live, Closing, Aborting, Closed }` in llm-connection and
  oauth-accounts; no slab iteration, so oauth-accounts walks a linked
  list over its slab nine times; `hex` four times and more, and decimal
  parsing three times (HTTP's client and server `decimal` are
  identical).
- **The loop surfaces cost every test world.** `Referee::act`,
  `next_deadline` and `overdue` have no defaults
  (`testing/skein-world/src/referee.rs:49-60`), so about 20 referees
  return `None` or do nothing. About 10 test `enum Process` dispatchers
  forward `Host` method by method (44 to 70 lines each), and they have
  drifted: `exit` and `state_digest` are forwarded in some and not in
  others. Every host merges its mechanism and policy deadlines by hand.
- **Two components miss a rule.** skein-forge-connection and
  skein-forge-setup's host take no `Abort` request, which
  programming-model.md 5.2 requires.

## 3. The parts

| Part | Kind | What |
|---|---|---|
| kit-contract | contract | lib's missing primitives and the io stage, as a skeleton that lands alone |
| echo-first | migration | the echo host on the shared stage; slice 1 |
| stage-sites | migration | the other io stage sites, one crate per task |
| lib-sites | migration | the private copies of lib's primitives, one crate per task |
| world-surfaces | feature | `Referee` defaults, a provided `Host::next_deadline`, a forwarding `Host` |
| abort | fix | `Abort` in both forge components |

### 3.1 kit-contract

The public items, with signatures and short contracts; no call site
moves in this part.

- **lib:**
  - an earliest-deadline helper, in place of the four private
    `earliest` and the inline copies;
  - `Queue::clear` and `Stack::clear`;
  - one `Lifecycle` enum, in place of the copies in llm-connection and
    oauth-accounts;
  - one growth rule with one floor, used by lib's five growing
    containers;
  - slab iteration;
  - hex and decimal parsing.
- **io:** `skein_io::stage::{up, down}` as free functions, modelled on
  `examples/echo/service/src/io_stage.rs`, each checking the kernel's
  submission room (`io::MAX_OUT_*`, the kernel's edge) before it takes
  work. One ordering test in `tests/io` replaces the two
  `#[path]`-included copies.

### 3.2 echo-first

The echo example's host runs on the shared stage, and its story test is
green on main. This is slice 1: the first root to run end to end on the
contract.

### 3.3 stage-sites

The remaining `io::resume` sites move to the shared stage, one crate per
task, in crate-graph order. After the move every site checks the
kernel's submission room before taking work.

### 3.4 lib-sites

The private copies of section 3.1's lib primitives move to lib, one
crate per task, in crate-graph order.

### 3.5 world-surfaces

- **`Referee`:** `act`, `next_deadline` and `overdue` get defaults, as
  `starved` and `last_word` have.
- **`Host::next_deadline`:** provided, the earlier of the mechanism
  deadline and the policy deadline. `next_policy_deadline` stays
  required: `drive.rs` says that is deliberate, and it guards teardown.
- **Forwarding:** a forwarding `Host` lets the test dispatchers shrink to
  one accessor each. Its form is question 6.1.

### 3.6 abort

skein-forge-connection and skein-forge-setup's host take an `Abort`
request (programming-model.md, 5.2). The TLS `close_notify` is not part
of this run; it comes with the shared transport (section 5).

## 4. Counts that may only fall

Each is counted on main before the first task and recorded as the
baseline. The plan sets each one's target, zero where a part removes
every copy. The plan may add counts; it removes none of these.

- `io::resume` sites outside skein-io (39 by the review);
- private `earliest` and `earlier` helpers outside lib;
- `enum Lifecycle` definitions outside lib;
- `GROWTH_FLOOR` definitions;
- `for _ in 0..` pop loops over lib's containers;
- `fn decimal` and `fn hex` definitions outside lib;
- test `enum Process` dispatchers that forward `Host` method by method.

They sit beside the old forms in `scripts/old-forms.txt` and are lowered
the same way. The four kernel submission bounds are already exempt from
the `max_out` form, so the room checks section 3.3 adds raise nothing.

**Size.** About 25 to 35 tasks, and a net fall of a few thousand lines;
the plan sets the real numbers.

## 5. Later, each its own run

- kv's store as a 5.4 machine;
- the shared transport (theme 1a), after a short design note of its own;
- JSON parse and navigation (1e);
- the forge family's dialect and fake consolidation (theme 3);
- test consolidation (theme 4);
- the layout and lint decisions (D1, D2, D4, D5).

## 6. Questions

The plan raises these with a recommendation; it does not settle them.

### 6.1 The forwarding `Host`

`impl<H: Host + ?Sized> Host for Box<H>` is a generic item, and the
programming model favours concrete composition over traits and generic
items. The options:

1. the generic impl, as an explicit and documented exception;
2. a concrete delegate type;
3. leave the dispatchers, taking only the defaults and the provided
   deadline.

The plan recommends one, measured where it can be: the line count of
each option on two of the dispatchers.

### 6.2 A component-kit role

Shared mechanism between lib and the components (the review's S1) has no
role in section 4 of the programming model: lib holds the generic
containers, and nothing between it and the components holds mechanism.
A "component kit" role, whose first members would be the io stage and
later the shared transport, changes the code's rules, and so is the
user's to decide. The plan proposes the wording and does not land it.

### 6.3 Anything hard to reverse

A public interface other trees use, or a stored format, is the user's.
