# The lead's answers to questions foreseen after release

For answer calls: a question these cover is answered so, unless the
asker shows the answer would break its acceptance.

## cl-wire: the playground's read bridged to no record

The wire has no `not_started`, so the bridge answers today's not-started
playground as no record, and the welcome's "Try the playground" (which
needs a held answer) and `route::playground` (which selects the held
answer's project) stop working. Answer, inside cl-wire's crates:

1. the welcome offers the playground once the read has answered (any
   answer, none included) and no real forge row is connected, its other
   conditions as today;
2. `route::playground` sends the start without a held answer, opens
   Chats, and selects the playground's project when the running read
   arrives.

`cl-welcome` makes this the lasting rule (offered while no real forge is
kept, draft 6.3). The `welcome-playground` journey re-recorded on it is
right.

## A fresh box is empty

`temper box --fake-backends` seeds no kept forge at its start: a fresh
deployment under it is empty (draft 6.3), so the welcome offers the
playground and both committed journeys stand. Its fake GitHub is the
machine's GitHub endpoint (`settings::Machine.github`, the stock GitHub
definition's addresses), so the welcome's GitHub choice makes a GitHub
record that talks to the fake. Its managed Forgejo is seeded only for a
test that asks for it (a flag or the test's own seed).
