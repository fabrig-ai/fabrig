# debt-29: the rules every task names

Read this note, then your spec. Where this note and the implementer's text differ, this note wins. The request is `temper/docs/plan/debt-29/inputs/request.md`; a spec quotes what you need of it.

## Departures from the implementer's text

- **Design text.** A task may edit the `docs/design/` lines its `Owns and may change` lists, and those alone. Anything else of `docs/design/` is not yours: what the code taught goes in the report.
- **A question about the request's own words,** or about a size cap, goes to fabrig/debt, the plan's owner: write it as a `Question:`, and fab carries it. A question on a budget or a timeout is the user's.

## What holds in every task

- **Read the item file whole,** and the design text it cites. Items may have changed since they were written: your spec gives what the cut measured; measure again first. What is already paid goes with a word in your report.
- **Behaviour stays unchanged.** If a test's visible events or outcome change, stop and ask. Don't retime a test, or rewrite an assertion to make a refactor pass. A stored format or a breaking public interface stops and asks too; additions are fine, and a removal your spec names is agreed. A test that only reached for a removed helper may change how it reaches, keeping what it asserts.
- **A measure reading 0 doesn't pay an item.** A renamed flag, a tuple that only beats a lint, or `Some(_) | None` passes a search and pays nothing. If the honest shape isn't reachable, keep the item open or record the departure, and say so in the report.
- **An item you pay is deleted in your branch.** You update any status entry it names as closed by it and any citation that breaks: `python3 -I -B tools/citations/citations.py --all` stays at 0, and `python3 -I -B tools/debt/debt.py` exits 0. An item that keeps anything it named stays open, narrowed to what's left; the remainder never lives only in a plan or a report.
- **Acceptance searches are narrow.** Code that uses the same word for something else is exempted, not changed.
- **Never pin an exact HEAD.** fab merges the trunk in at each run's start: a debt item's `Checked:` line gives the date alone, and a line number in your spec is the one the cut saw, so find the site by its text.
- **New tests earn their lines:** a table or a helper, not copies. A table written to pin a refactor is reduced, before the task ends, to the rows that still bite. A test that pins a rule goes red when the rule breaks: break it by hand, watch it, restore, and report what you broke and what failed.
- **These tasks remove code.** Each ends with fewer lines than it started, unless its spec says otherwise; your report gives the net, with inline `#[cfg(test)]` items split out and generated code counted apart. If yours turns out bigger than its spec's minutes and cap say, stop and ask fabrig/debt rather than grow it.
- **A size cap in a spec is the cut's estimate,** no user budget. Compact first. If a cap still can't be met without losing wanted behaviour, ask: an answer call may raise it by up to 25%, stating the measured size and why. Beyond that it's fabrig/debt's.
- **A size or memory pin** that shrinks because of your change is updated, and the report says so. One that grows stops and asks fabrig/debt.
- **Stay out of fabrig/ui's files:** `temper/crates/temper-client-view/**`, `temper/crates/temper-client-shell-web/temper.css`, `jig/crates/jig-client-view/**`. A caller there that your change would break is a question, never an edit.
- **The old-forms counts run from the tree:** `(cd TREE && python3 -I -B scripts/old_forms.py)`, for each tree your branch touches.
