# component-kit-4: how a site moves onto the kit

For the migration tasks (echo-first, stage-sites, lib-sites, and the world
moves of world-surfaces). Read `notes/common.md` first; this adds the
recipes. Your spec lists your sites; find each with `rg` (line numbers
drift) and read the kit item's doc before you move a site onto it.

## The io stage (`skein_io::stage`)

- A host's up stage (its `resume` loop, its completions loop, its `fire`
  loop, in that order) becomes one `skein_io::stage::up(&mut io, &env,
  &mut completions, &mut events, &mut submissions)`. The stage reads the
  time from `env.now`: set the env's `now` (and `wall`) for the iteration
  first, if the site kept `now` apart.
- Its down loop (`io::down` for each request, while `Io::takes` and the
  room holds) becomes `skein_io::stage::down(&mut io, &env, &mut requests,
  &mut submissions)`. Where a site calls `io::down` one request at a time
  from somewhere other than a queue, leave those calls as they are (they
  are not counted), unless a queue is already there.
- The private room helper (`fn room`, `fn io_room`) goes when nothing uses
  it.
- A test that drives io by hand (`while io.is_ready() { io::resume(..) }`,
  then reaps) moves to `stage::up` with its completions queue, as long as
  the test's assertions hold: the stage also takes completions and fires
  due deadlines, and checks room. If a test needs `resume` alone at that
  point (it meters each entry point, or asserts what one resume does), it
  keeps `io::resume` and the report names it with its reason.
- A changed outcome in a world is a finding, not a reason to change the
  world's expectations: before the stage, did the site take work without
  room (which testing-strategy.md, 6 forbids)? Then the new outcome is the
  fix: say so. Otherwise keep the site's order and report it.

## The world surfaces (`skein_shell::Forward`, `Host`, `Referee`)

- A world's `enum Process` whose `impl Host` forwards every method to the
  host it holds becomes `impl Forward for Process` with its two accessors
  (`host`, `host_mut`); the enum stays.
- A `Host` impl whose `next_deadline` is the earlier of a mechanism deadline
  (io's, a scripted timer's) and exactly what its `next_policy_deadline`
  returns implements `next_mechanism_deadline` instead and drops
  `next_deadline`. If its `next_deadline` is anything else (a `Some(ZERO)`
  for pending work, a deadline the policy one leaves out), leave it.
- A `Referee` impl's `act` that does nothing, `next_deadline` that returns
  `None` and `overdue` that returns `None` go: the trait's defaults are
  those.

## lib's primitives

- `fn earlier`/`fn earliest` over `Option<Time>`: each caller writes
  `skein_lib::earliest(a, b)` (an accumulator `earlier(&mut next, t)`
  becomes `next = skein_lib::earliest(next, Some(t))`). Inline hand merges
  of two `Option<Time>` (a `match` on both, `.min` inside `Some`) in the
  files you own go the same way.
- `for _ in 0..q.len() { q.pop(); }` (or `drop(q.pop())`) becomes
  `q.clear()` on a `Queue` or `Stack`; a `List` already has `clear`.
- `enum Lifecycle { Live, Closing, Aborting, Closed }`: use
  `skein_lib::Lifecycle`; delete the private enum.
- Hex: a nibble to its lowercase digit is `skein_lib::hex::digit`; an ASCII
  digit to its value is `hex::value` (either case) or `hex::lower_value`
  (lowercase only). Uppercase output, if a site has it, stays the site's.
- Decimal: parsing ASCII digits to a `u64` is `skein_lib::Decimal::parse`
  (`None` on empty, a non-digit, overflow); writing a number's digits is
  `Decimal::of(n)` and its bytes.
- A private `fn hex`/`fn decimal` whose body becomes one call goes, its
  callers calling lib. One that does more (hex-encodes a whole secret,
  draws random digits, folds a `\u` escape) may stay, its body using lib,
  and is named in the report.
