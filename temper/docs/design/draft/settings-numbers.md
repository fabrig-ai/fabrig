# Settings numbers: the stock's and a deployment's

Draft, 2026-11-18, from the user's words that day: "start the template numbers draft", on what a deployment's own template did to the stock's (section 2). It gives every numbered settings kind that has stock items two ranges: the low numbers are the stock's, and the high ones are a deployment's own. Then a later build can't bring a stock item onto a number a deployment holds. The user chose one line for every kind ("ok for 1"). They chose to move what's stored already by hand at the first redeploy, rather than keep code for it ("can we handle it by hand at the relevant redeployment"), without waiting for live tasks to drain.

## 1. In one page

- **Two ranges** (section 3). In templates, charters, policies and models, numbers 1 to 32 are the stock's, and 33 to 64 a deployment's own.
- **One rule, at every set and every start** (section 4). An item below 33 that the stock doesn't have is refused, naming the range: a new one when it's set, a stored one when a start reads it. An owner still changes or removes a stock item at its number, as today.
- **What's stored already moves by hand, once** (sections 5 and 7). dev.fabrig.ai's four such items move to 33 and above before its first redeploy with the rule, and the upgrade check says if any is left. No code is kept for them.
- **Then nothing can collide** (section 6). A stock item new in a build can only land on a number no deployment can hold.
- **Nothing stored or sent changes form** (section 8).

## 2. What happened

- On 11-15, dev.fabrig.ai set its own template 8, "goal for a lean design", when the stock had no 8. A later build added the stock's milestone template as 8. A stored change at a number replaces the stock's item there, so dev had no milestone template, and the client, which finds it by number 8, listed ordinary goals as milestones. ops moved dev's own template to 16 by hand on 11-17.
- Nothing warned. A start saves a new stock item under a stored change at its number and keeps the change in force (jig's `../../../../jig/docs/design/domain/engine/settings.md`, section 7). Nothing records whether a change was made over a stock item or added at a free number, except the bounded history.
- **Still exposed:** dev's own templates 9, "land a docs branch", on the number the stock most likely takes next, and 16; and its charters 17 and 19 (`temper ctl settings show`, revision 56).
- **Models already work this way.** The binary's models take low numbers, and one a person or a provider's list adds takes the highest free number, "so a release's new default never meets one" (jig's `domain/engine/settings.md`, section 11). This draft makes that a rule for the other numbered kinds, and a refusal rather than a habit.

## 3. The kinds and their ranges

| Kind | The stock's numbers today | Today's bound | Numbers |
|---|---|---|---|
| templates | 1, 2, 4 to 8 | `core.templates` 16, a count | 1 to 32 the stock's, 33 to 64 a deployment's |
| charters | 1, 2, 4, 5, 9 to 16 | `tasks.charters` 48, a count | the same |
| policies | 1 | `authority.projects` 8, a count | the same |
| models | 1 to 8 | `MODELS` 64, a number | the same; a deployment's model already takes the highest free number |

- **Why 32:** room for the stock to grow fourfold in charters and templates, and 32 own items per kind, far above dev's two templates and two charters. Staying within 64 keeps models' numbers as bits of an authority's model set. Wider ranges would cost nothing in templates, charters and policies, but one line for every kind reads more simply.
- **The count bounds stay counts.** A kind's numbers run 1 to 64, while its count bound still caps how many items it holds.
- **Companions follow their item.** temper's template-goal kind (11) is numbered by its template, and its charter-words kind (9) by its charter. Both take 1 to 64, up from 16 and 48 (`../protocol/configuration/settings.md`).
- **Out of scope:**
  - workers, whose only stock item is the box's `local` at 1;
  - temper's other application kinds, whose numbers are fixed slots or a project's own;
  - labels, which aren't settings.

## 4. The rule

- **A deployment's own item is one at a number where this build has no stock item.** Below 33 it's refused:
  - **at a set:** "template 9 is among the stock's numbers, 1 to 32; your own templates take 33 to 64";
  - **at a proposal's decision,** the same way;
  - **at a start,** as the start refuses any stored setting out of shape, naming each such item. `temper-engine upgrade --check` and `temper-box upgrade --check` (`../cli/temper-engine-args.md`) say so before a redeploy.
- **Past 64** it's refused as past the kind's numbers.
- **Not refused:** a removal at such a number, which offers nothing.
- **A change of a stock item stays at its number.** Setting, removing or resetting a number where the stock has an item works as today, whatever its number.
- **jig keeps the rule.** It's jig's settings mechanism, for its core kinds, with the line a constant of jig's. temper's companion kinds follow their items.
- **Help and Ask temper say it.** `../../help/template.md` and `../../help/charter.md` say that a deployment's own items take 33 to 64. Ask temper proposes the lowest free number from 33 for a new template, charter or policy. A model keeps taking the highest free number.

## 5. What's stored already

- **On dev.fabrig.ai, four items sit below 33:** templates 9 and 16, and charters 17 and 19. Staging follows dev, and no other deployment exists yet, so nothing else is affected.
- **They move by hand, once,** on the build dev runs now, before its first redeploy with the rule (section 7). A start with the rule refuses any that's left, and the upgrade check names it first.
- **No live task waits for it.** A task keeps the settings revision it was made under until it ends. One whose charter the newest revision doesn't offer finishes on its own version (jig's `domain/engine/settings.md`, sections 3 and 4). Everything made after the move reads the newest, where the moved numbers are offered. A live agent that delegates to an old number by itself is refused as not offered, and adapts.

## 6. A collision

- **Once section 5's items have moved, none can happen.** No deployment holds an item below 33 other than a change of a stock item. A stock item new in a build can then only land on a number nobody else uses.
- **A deliberate change of a stock item** stays the owner's choice, as today, at the stock's number.
- **Nothing in code detects collisions.** No mark, check or display is kept for them.

## 7. Moving an item

By hand, with `temper ctl settings`, as ops moved template 8 to 16 on 11-17. The steps for dev.fabrig.ai are in the execution file:

1. Set the same value at the new number.
2. Point each reference at it.
   - For a template: its template-goal (kind 11), each project's ways (kind 7, `way`), and the template overrides in policies, both the policy settings and each project's copy.
   - For a charter: the templates' executors, the rules' and policies' delegations (both the policy settings and each project's copy), its charter words (kind 9), the forge's producer and reviewer (kind 3), the areas' and triager's owners, and follows' producers.
3. Remove the old number.

## 8. Upgrades

- **No stored format and no wire changes.** The rule is a check, at a set and at a start, and the ranges add no field.
- **dev.fabrig.ai's step:** section 7, before the first redeploy with the rule, checked by `upgrade --check`.

## 9. Not covered

- **Finding stock items by what they are** rather than by number. The client finds the milestone template by 8, the chat by 1, the standing-role template by 7, and others likewise. With the ranges, a number keeps its stock meaning unless an owner changes that item on purpose.
- **Labels,** which templates copy into tasks, and the lists built from a template's current label.
- **A tool that moves an item** with its references.
