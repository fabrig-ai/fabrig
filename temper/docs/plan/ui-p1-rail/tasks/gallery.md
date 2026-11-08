+++
id = "gallery"
part = "checks"
kind = "feature"
needs = ["rail", "work-page", "deployment"]
files = ["temper/tools/gallery/src/samples.rs", "temper/tools/gallery/src/tests.rs", "temper/tools/gallery/src/worlds.rs", "temper/tools/gallery/src/worlds/*.rs", "temper/tools/gallery/names.txt", "temper/docs/design/client/browser.status.md"]
effort = "high"
minutes = 150
+++

# The gallery's states for the rail, Work and Deployment

## Goal

The gallery prints the states `temper/docs/design/client/browser.md`, 7.2, now
names for the rail, Work and Deployment, from `rail-new-change` to `night-limits`,
each a world stepped to its address as 7.2 says, never set by hand, each holding
what its name says by its test row. A laptop's and a phone's width are the same
tree, one print, the shell's style choosing; day and night are the `night-*`
states.

Acceptance:

1. `names.txt` holds, sorted as it is, the fifteen new names: `rail-new-change`,
   `rail-all-projects`, `rail-across`, `work`, `work-pair`, `work-drafts`,
   `work-goals`, `work-changes`, `work-empty`, `members`, `members-refused`,
   `limits`, `limits-not-owner`, `night-work`, `night-limits`; and `rail-new-menu`
   draws "+ New draft or change" with its menu open.
2. Each new state's row in `each_state_holds_what_its_name_says` gives its page's
   heading and the one part it is about, found by role and name, as 7.2's table
   says it (the new box's link "New change"; "All projects" pressed; Work's In flight
   cards and their from lines; "Nothing in flight yet."; Members' sections; Limits'
   capacities; the night switch pressed). Each state's tab bar is Home, Needs you,
   Work and Ask, or To begin's three, so that a phone's width prints its four tabs.
3. `work` and `work-pair` are hand-made deliveries after
   `temper/docs/design/client/web/whole-story/work.html` and `pair/work.html`, their
   words where the client holds the facts; `work-pair` offers no drafts.
4. `cargo run -p temper-client-gallery -- OUT`'s test of the print still passes:
   every page written, the index linking each.
5. The gallery's tests' CPU time, `heavy cargo nextest run -p temper-client-gallery
   --lib`, measured before and after the change on the same builder; the report
   says both. The overrun `browser.status.md` records is not this task's, but the
   states add no more than they need: a state that costs more than 30 ms of its
   own is found out and made cheaper.
6. `browser.status.md` loses its entry "**Not yet:** 2.5, *The rail as the whole
   story draws it*, *Work's look* and *Deployment's frame*, and 7.2", once the
   rules it names are in `temper.css` (the tasks `rail`, `work-page` and
   `deployment` wrote them) and these states print; if a rule it names is missing,
   the entry stays with that rule alone and the report says which.

## Inputs

- `temper/docs/plan/ui-p1-rail/notes/common.md`.
- `temper/docs/design/client/browser.md`, 7 (7.1 to 7.3), the new rows of 7.2's
  table; 2.5, *The rail as the whole story draws it*, *Work's look*, *Deployment's
  frame*.
- `temper/docs/design/client/README.md`, 3.5, *Work* and *Settings and Deployment*;
  5.3.
- The look: `temper/docs/design/client/web/whole-story/work.html`, `pair/work.html`.
- The gallery's code: `samples.rs` (a sample's world, address and `then`; the
  `night-*` samples; `changes`' and `goals-*`' hand-made deliveries), `worlds.rs`
  and `worlds/*.rs`, `tests.rs`.

## Owns and may change

Owns: the gallery's samples, worlds, tests and `names.txt`; the one status entry.

Not this task's: the view, the domain, the shells. A state that can't be reached
by stepping the root from deliveries is a question, never a state set by hand.

Pre-approved options:

- **If two states share their deliveries** (`work`, `work-drafts`, `work-goals`,
  `work-changes`, `night-work`): build them once, as the `goals-*` states share
  theirs, so that the cost is one world's.
- **If `rail-across` needs a second project:** use `rail-all-projects`'s deliveries,
  at `Inbox` with no project.
- **If Limits' deliveries need a `Deployment` document:** take the one `builders`'
  states deliver, with the capacities 7.2 names added.

## Steps

1. Measure the gallery's tests' CPU time on the trunk (acceptance 5).
2. Build the deliveries and the samples, in 7.2's order, and add their names.
3. Write each row; break one row's rule by hand (the `work` row expecting a third
   Drafts card) and see it fail; restore.
4. Run the print's test; measure again.
5. **Check** as `notes/common.md` says. Remove or narrow the status entry
   (acceptance 6). Commit and push: the message says the gallery prints the rail,
   Work and Deployment as the whole story draws them.

## Report

Follow the implementer's text. Also give the CPU times before and after, each new
state's own cost where you measured it, and the row you broke and its failure.
