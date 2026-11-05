# debt-20: classifiers on the owning type (jig D015)

Every `classify-*` task follows this note. The item is `jig/docs/development/debt/D015-exhaustive-lists.md`.

## The rule

A question asked of a vocabulary in several places is a method on the type, and its callers match the answer. Routing stays: a list whose arms each do something different isn't a question. Nor is a list of one use: a method would only move it.

- **The method holds the type's one exhaustive match.** No `_`, no `matches!`.
- **The answer is a small enum or an `Option`,** defined beside the type and `Copy` where it can be. It carries `Copy` values or an `Option<&T>`. Rust lifetimes on an answer type are out (`skein/docs/foundation/programming-model.md`, 10), and so are traits and generics (`jig/AGENTS.md`). `if let` on an own enum is out too, so `let .. else` doesn't shorten a list.
- **A sub-enum of `Ask` or `Outcome` isn't this plan's:** the item suggests one, but it would touch 542 production and 1,596 test mentions, and `Event` is priced by `size_of`. An answer enum asks the same question and touches nothing else.
- **No record, wire or stored form changes.** A record's codec maps by hand, and no answer type reaches one.

## Pin first

No list has an exhaustive pin today: a classifier can't drop a variant, since its match is exhaustive, but it can misfile one. So:

1. **First commit, against today's code:** a table test over **every** variant of each question you move, whose expected answer you read from today's lists. It goes through the behaviour the callers have (what `permanent` says, which status an outcome maps to, which scope an ask has). It passes before you change anything.
2. **Then the classifier,** with that test unchanged but for adding a row that calls the new method. An assertion that needs changing is a misfile: fix the method.
3. **By hand:** file one variant under another answer in each classifier; the table goes red. Restore. Your report names each.

A big enum means a big table. Tests aren't in the size cap.

## The run

An added method changes no dependent, so `heavy cargo check -p <the crate's direct dependents> --tests` stands in for their tests. This departs from the implementer's text for that reason. A public item stays public, under its name, unless your spec says otherwise.

`cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p PKG --all-targets -- -D warnings`; your spec's tests. A `match_same_arms` or `too_many_lines` expectation that no longer fires comes off (clippy says so). The citations check.

Your report gives the item's **Measure** before and after on your crate, the net production lines, each question moved with how many lines it removed, and each list you left and why.
