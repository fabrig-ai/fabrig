# Settings numbers: the stock's and a deployment's

Draft, 2026-11-18, from the user's word that day: "start the template numbers draft", on what a deployment's own template did to the stock's (section 2). It gives every numbered settings kind that has stock items two ranges: the low numbers are the stock's, and the high ones are a deployment's own. Then a later build can't bring a stock item onto a number a deployment already holds, and one that does is caught before it hides anything.

## 1. In one page

- **Two ranges** (section 3). In templates, charters, policies and models, numbers 1 to 32 are the stock's, and 33 to 64 a deployment's own.
- **A deployment adds its own items from 33** (section 4). A set at a number below 33 where the stock has no item is refused, naming the range. An owner still changes or removes a stock item at its number, as today.
- **What's stored already stays** (section 5). An item a deployment added below 33 before this rule keeps working. The settings page and ctl mark it as sitting among the stock's numbers.
- **A collision is caught** (section 6). It happens when a build brings a new stock item onto a number where a deployment's item sits. The upgrade check refuses that build, naming both. If a start goes ahead anyway, it keeps the deployment's item, as today, logs both, and shows the hidden stock item beside it on the settings page and in ctl.
- **Moving an item is a recipe in help, not a tool** (section 7).
- **Nothing stored or sent changes form** (section 8). dev.fabrig.ai needs no step.

## 2. What happened

- On 11-15, dev.fabrig.ai set its own template 8, "goal for a lean design", when the stock had no 8. A later build added the stock's milestone template as 8. A stored change at a number replaces the stock's item there, so dev had no milestone template, and the client, which finds it by number 8, listed ordinary goals as milestones. ops moved dev's own template to 16 by hand on 11-17.
- Nothing warned. A start saves a new stock item under a stored change at its number and keeps the change in force (jig's `../../../../jig/docs/design/domain/engine/settings.md`, section 7). Nothing records whether a change was made over a stock item or added at a free number, except the bounded history.
- **Still exposed:** dev's own template 9, "land a docs branch", on the number the stock most likely takes next; its template 16; and its charters 17 and 19 (`temper ctl settings show`, revision 55).
- **Models already work this way.** The binary's models take low numbers, and one a person or a provider's list adds takes the highest free number, "so a release's new default never meets one" (jig's `domain/engine/settings.md`, section 11). This draft makes that a rule for the other numbered kinds, and a refusal rather than a habit.

## 3. The kinds and their ranges

| Kind | The stock's numbers today | Today's bound | Numbers |
|---|---|---|---|
| templates | 1, 2, 4 to 8 | `core.templates` 16, a count | 1 to 32 the stock's, 33 to 64 a deployment's |
| charters | 1, 2, 4, 5, 9 to 16 | `tasks.charters` 48, a count | the same |
| policies | 1 | `authority.projects` 8, a count | the same |
| models | 1 to 8 | `MODELS` 64, a number | the same; a deployment's model already takes the highest free number |

- **Why 32:** room for the stock to grow fourfold in charters and templates, and 32 own items per kind, far above dev's two templates and two charters. Staying within 64 keeps models' numbers as bits of an authority's model set.
- **The count bounds stay counts.** A kind's numbers run 1 to 64, while its count bound still caps how many items it holds.
- **Companions follow their item.** temper's template-goal kind (11) is numbered by its template, and its charter-words kind (9) by its charter. Both take 1 to 64, up from 16 and 48 (`../protocol/configuration/settings.md`).
- **Out of scope:**
  - workers, whose only stock item is the box's `local` at 1;
  - temper's other application kinds, whose numbers are fixed slots or a project's own;
  - labels, which aren't settings.

## 4. The rule

- **An addition is a set at a number where this build has no stock item.** Below 33 it's refused: "template 9 is among the stock's numbers, 1 to 32; your own templates take 33 to 64". Past 64 it's refused as past the kind's numbers. A proposal is judged the same way when it's decided.
- **A change of a stock item stays at its number.** Setting, removing or resetting a number where the stock has an item works as today, whatever its number.
- **jig keeps the rule.** It's jig's settings mechanism, for its core kinds, with the line a constant of jig's. temper's companion kinds follow their items.
- **Help and Ask temper say it.** `../../help/template.md` and `../../help/charter.md` say that a deployment's own items take 33 to 64. Ask temper proposes the lowest free number from 33 for a new template, charter or policy. A model keeps taking the highest free number.

## 5. What's stored already

- **An earlier addition below 33 keeps working.** A start accepts a stored change at a number without a stock item, as today, since the rule refuses new additions only. The settings page and `temper ctl settings show` mark it "among the stock's numbers: a later build may bring a stock item here".
- **When to move one:** when the upgrade check names it (section 6), or before. On dev.fabrig.ai that's templates 9 and 16 and charters 17 and 19. Template 9 sits where the stock's next template most likely goes, so moving it first is advised, not required.

## 6. A collision

- **What it is:** a stock item new in this build lands on a number where the store holds a change, and the stored defaults have no item at that number. A start already finds the new stock items, the ones its stored defaults lack (jig's `domain/engine/settings.md`, section 7). A collision is one of those with a stored change at its number. A change the owner made over a stock item that was already there is no collision.
- **The upgrade check refuses it.** `temper-engine upgrade --check` and `temper-box upgrade --check` (`../cli/temper-engine-args.md`) name each collision: "the stock's template 9 "milestone" arrives on your template 9 "land a docs branch": move yours to 33 to 64 first (help: settings, moving an item)". The check fails until there's none.
- **A start doesn't refuse.** A deployment started without the check keeps running, with the owner's item in force, as today. The start logs one line per collision. The settings page and `temper ctl settings show` show the hidden stock item beside the owner's: "hides the stock's template 9 "milestone"".

## 7. Moving an item

`../../help/settings.md` gains a recipe, as ops moved template 8 to 16:

1. Set the same value at the new number.
2. Point each reference at it.
   - For a template: its template-goal (kind 11), each project's ways (kind 7, `way`), and the template overrides in policies, both the policy settings and each project's copy.
   - For a charter: the templates' executors, the rules' and policies' delegations, its charter words (kind 9), the forge's producer and reviewer (kind 3), the areas' and triager's owners, and follows' producers.
3. Remove the old number. A running task keeps the charter revision it holds.

A tool comes only if moves become common.

## 8. Upgrades

- **No stored format and no wire changes.** The rule is a check on additions, and the collision a comparison a start already makes. The ranges add no field.
- **Each milestone is compatible with dev.fabrig.ai.** Its existing additions below 33 stay and get marked. Moving template 9 first is advised.

## 9. Not covered

- **Finding stock items by what they are** rather than by number. The client finds the milestone template by 8, the chat by 1, the standing-role template by 7, and others likewise. With the ranges, a number keeps its stock meaning unless an owner changes that item on purpose.
- **Labels,** which templates copy into tasks, and the lists built from a template's current label.
- **A tool that moves an item** with its references.
