# ui-walk-3: how the client is walked

For `walk`, `additions` and `walk-again`.

## The seven steps

The second walk's, which `driver/walk.py` holds:

1. **First run:** the welcome, Forge (Forgejo on this machine, empty `work/hello`), Model (each of ChatGPT and Claude offered; sign in with ChatGPT), Ready, Home with To begin.
2. **Everyday:** Settings, How work runs: turn on drafts, Goals, Changes and Away. The rail changes. Configure temper opens and reads as its look request says.
3. **A chat that changes the code:** new chat, tool lines, a change, Changes' queue and landed list, Home's brief.
4. **A draft to a goal:** + Draft, two document turns, Split, Draft, Start goal…, the sheet, the goal's page (report, tasks, Landing, How it runs), Pause, Resume, Stop, Draft back to the chat. The rail's menu.
5. **Needs you:** a question with choices, Later, Reply, Undo, one held task, a proposal; the rail's count, Home's card.
6. **Away:** turn it on, a decision the goal takes, "decided for you" on Home, overturn.
7. **Settings' other tabs:** Repositories, Forges, Accounts, Builders; adding a project; the project switcher; Sign out, reload, back in.

Where the fake model can't reach a state, say so and show it some other way (gallery, ctl) rather than skipping it silently.

## A person's path

- **A fresh `temper box --fake-backends`:** `temper/docs/help/box.md`, *Before you start* and 2. `driver/README.md` has the build and the run. The driver was last walked at `07f8081aff`: a name or a control that has moved since is the driver's to follow, and a state it now can't reach is what you look at, not what you route around.
- **Headless Chromium through CDP,** from `SKEIN_TEST_CHROMIUM` or the builder's `chromium`, at 1440 wide and, on another fresh box, at 420, set by `Emulation.setDeviceMetricsOverride`. Handle `Page.javascriptDialogOpening`.
- **No shortcuts:** the driver presses, types and toggles what a person sees at that width, finding each control by its role and name. It goes to an address only where the person is told to: the printed link, a reload the step asks for. Any address it had to type is named in that row of the table.
- **The mock, at the same width, beside the client:** `temper/docs/design/client/greenfield/*.html` from `file://`, by the same clicks in the same headless Chromium, as the driver does. The mock's data differs on purpose: compare the look, the words that aren't data, the order and what each control does. `phone.html` isn't the target: at 420 use the mock's own pages at 420. Where a state came from a look request, the design that request wrote is the reference (`temper/docs/design/client/README.md`, 3.5 and 3.6), and the row says so.
- **Compare by eye and by number:** a screenshot pair for each state, and, where a pair looks off, `getComputedStyle` and the box of the part in question on both sides. A finding says the numbers.

## The fake model

Its scripts are `model_document` in `temper/crates/temper-fake/src/backends.rs`. Its notes name literal numbers, so a script holds on a box's first chat and first goal: use as many fresh boxes as they need, and name the box on each row.

| Sent, or the cue | Reaches |
|---|---|
| "Show drafts and goals for this project." typed into Configure temper | a settings proposal of project 1's ways (W3). Its `against` is a fresh box's newest revision, so it goes **before** any setting changes |
| "+ Draft" pressed, then "Add what happens to a running goal." | the document and its second revision |
| "Keep only working notes for this chat.", "Restore the Away mode document after the working notes." | a chat with only working notes, and its document back |
| "Land a local hello status change." | a change on `work/hello`, coder then reviewer, landing |
| "Start a local away goal." | Away's decision, the question after an overturn, "Which local implementation while here?" with its choices |
| "Start a local goal and write its report." | a goal from a chat, with its report |
| Anything else | "Hello from the local fake model." |

A held task of your own: `temper ctl` stop places an owner's hold. What the fake still can't reach, show by ctl or the gallery (`heavy cargo run -p temper-client-gallery -- OUT`; its states are `temper/tools/gallery/names.txt`), and say so on the row. `backends.rs` has one writer in the first wave: `walk`. A script is added only where a state truly needs one.

## The screenshots

- **Under `~/src/fabrig/ui-walk-3/<your task>/`:** `NN-state-WIDTH-mock.png`, `-client.png` and a side-by-side `-pair.png`.
- **`index.html` beside them:** one line per state, at both widths, linking the three, with its verdict, finding and the state's way (box, ctl, gallery). No build tool, no outside address.

## The table

One row per state at each width, in the second walk's form (`walk-2/report.md`):

```
| step | page | state | width | verdict | why | box or way | pair |
```

- **verdict:** `matches`; `differs`, its why a decision (`D4`, `Ruling 9`, `By page, Home`) or a finding (`N7`); or `unreachable`, its why what stops it and how the state was shown instead.
- **Above the table:** the commit walked, the builder, the folder, how many boxes and which steps each walked, the counts of each verdict, against the second walk's (22 matches, 90 differs, 30 unreachable, of 142).
- **Below it:** whatever a table can't say: a flow that felt wrong, a word that reads badly though the mock has it, a listed decision the walk made you doubt. These stay as notes for `fabrig/ui`.
- **A state that got worse since the second walk is a finding,** whatever its verdict. Compare row by row with `walk-2/report.md`.
