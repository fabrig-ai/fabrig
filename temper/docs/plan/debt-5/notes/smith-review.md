# debt-5: smith's review, checked at this plan's base

`temper/docs/plan/debt-5/inputs/smith-lifecycles-review.md` was checked against the code by the cut. No finding has moved on or
is skipped. Where the review's sketch does not fit, the task's sketch wins; the review's numbers
of lines are its estimates, and the check found about -190 in all, about half in tests.

- **Three crates, not two:** `host_io` is `smith-local-service`'s.
- **Finding 1's design** is `smith/docs/design/protocol/agent.md`, section 6.
- **Finding 3:** the deferred `let x;` form is in `reported` and `close_next`; `adopt` only pushes
  twice. A `left` counter would put the tokens outside the state, against
  `skein/docs/foundation/programming-model.md`, 5.4, and `Close::root_answered`'s in-place form
  does not fit a `Roots` that moves a `List` between variants. The form is one match that returns
  the state and the answer.
- **Finding 5:** the four forwards also have production callers, so those call the field directly
  as well. Tests already reach `child`, so `process_mut` serves `host_io.rs`'s three panicking
  matches, not the tests.
- **Finding 7:** two sites, in `chat/machine.rs`.
- **Finding 8:** two builders and a wrapper. Only `after` differs.
- **Finding 9:** the rule refines 5.4's "every transition uses the same idiom", which it does not
  replace: a `Copy` state needs no terminal placeholder, because `*self = match *self` moves
  nothing out. Non-`Copy` states keep `mem::replace`. A same-variant change matches `self` in
  place. `skein/docs/foundation/programming-model.md` and `smith/AGENTS.md` do not change; each
  task's commit says the rule, as `AGENTS.md`, 1 asks of a departure. `End` is outside debt-3's
  diff and stays.
- **Finding 10** gives two names for three items. `Close::root_answered` becomes `settle_root`.

Every renamed item is crate-private. Only `Teardown` and `Last` are `pub`, their change adds or
keeps only what readers use, and no design file or other tree names any changed item, so nothing
here touches `.config/gate/read-by-tests.txt`'s Markdown. `smith/scripts/old-forms.txt` counts none
of these forms. "No debt item closes here", so no task deletes one.

No machine's behaviour changes: every transition table, state and event stays as it is. A test's
assertion changes only where it names a renamed item or a removed panic message.
