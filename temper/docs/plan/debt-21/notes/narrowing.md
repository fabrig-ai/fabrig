# debt-21: `unreachable!` that re-narrows (jig D017)

Binding on the `narrow-*` tasks. Line numbers are at the plan's base: find a site by its message and neighbours, and re-measure first.

## What counts

- **A site goes when a type or the control flow carries the pairing:** one match instead of two, a helper taking the variant's fields, a private enum built at the one match, a lookup returning the narrow type.
- **Renaming the abort isn't a fix.** No `panic!`, `expect`, `let else` return or swallowed `_` takes an `unreachable!`'s place, unless your spec's list says so for that site. An arm a broken pairing can't reach today may read as absent where the code beside it already does.
- **An `unreachable!` your list doesn't name stays:** most guard a state machine or another step's promise on a shared enum. Your file's count after is in your spec. A site that turns out to need a public type, a stored or a wire change stays too, and your report says why. The file's count rises by that one, and that isn't a question.
- **No public type or function changes,** in any crate. `pub(crate)` and private are yours. Stored and wire forms don't change.
- **Every `match` lists every variant** (`AGENTS.md`, 2): a helper's `None` arm names the rest.
- **A test may change its spelling only** (the common note). No new test is wanted for a refactor that removes a site: the existing tests pin the behaviour. A new rule you add is pinned as the common note says.
- **Keep it small:** each site's fix is local. A helper earns its place by replacing two or more sites. A spec's line estimate is the cut's.

## What this plan doesn't pay

The item stays open: **Paid when** can't hold in this plan, since `jig-client-view` is off limits and some sites need a public type's change. `d017-record` writes what's left. So don't widen your task to reach a site outside your list.
