# ui-final-walk: how the client is walked

For `walk` and `walk-again`. The steps themselves are the request's *The walk*, 1 to 7: walk every state it names.

## A person's path

- **A fresh `temper box --fake-backends`:** `temper/docs/help/box.md`, *Before you start* and 2. Built by `heavy cargo build -p temper -p smith-fake-claude` and `heavy temper/scripts/bundle.sh --loopback BUNDLE`, then copied (`notes/common.md`, Artefacts). The link follows the output's `Open this link in a browser on this machine.`
- **Headless Chromium through CDP,** from `SKEIN_TEST_CHROMIUM` or the builder's `chromium`, at 1440 wide and, on another fresh box, at 420, set by `Emulation.setDeviceMetricsOverride`. Handle `Page.javascriptDialogOpening`, or a pending request's `onbeforeunload` hangs the driver.
- **No shortcuts:** the driver presses, types and toggles what a person sees at that width, finding each control by its role and name. It goes to an address only where the person is told to: the printed link, a reload the step asks for. At 420 the way there differs (tabs, the switcher: `notes/decisions.md`, Phone), and that way is what's walked. Any address it had to type is named in that row of the table.
- **The mock, at the same width:** `temper/docs/design/client/greenfield/*.html` from `file://`, by the same clicks in the same headless Chromium. Its scenarios are its `Mockup` button's, kept in `localStorage`: `temper-greenfield-mode`, then `temper-greenfield-v2` (everyday) or `temper-greenfield-first-run` (first run). The mock's data differs from the box's on purpose: compare the look, the words that aren't data, the order and what each control does. `phone.html` isn't the target (`notes/decisions.md`): at 420 use the mock's own pages at 420.
- **Step 7's Sign out comes last on its box.** Back in is what the signed-out page offers or a fresh printed link, whichever a person would have; say which.
- **Compare by eye and by number:** a screenshot pair for each state, and, where a pair looks off, `getComputedStyle` and the box of the part in question on both sides. A finding says the numbers.

## The fake model

Its scripts are `model_document` in `temper/crates/temper-fake/src/backends.rs`. What it answers, by what is sent:

| Sent, or the cue | Reaches |
|---|---|
| "+ Draft" pressed ("Write this up as a draft beside us.") | writes the document (`chat-draft`'s README, `temper/docs/design/client/journeys/chat-draft/README.md`) |
| "Add what happens to a running goal." | the document's second revision |
| any goal ("You are a goal: ...") | a report note, then finishes |
| "Start a local goal and write its report." | a goal started from a chat, with its report |
| "Land a local status change." | a `change` on `work/widget`, coder then reviewer, landing; `temper/crates/temper/tests/fake_backends/walk/status.rs` |
| "Start a local away goal." | Away's decision, the question after an overturn, and "Which local implementation while here?" with its choices; `temper/crates/temper/tests/fake_backends/walk/away.rs` |
| Anything else | "Hello from the local fake model." |

What binds the walk, and the call already made on each:

- **The scripts' notes name literal numbers:** the document's scope is goal 1, its second revision `name` 2 and `recalled` 1; a goal's report and the away goal's note are goal 2. So they hold on a box's first chat and its first goal only. **Default:** use as many fresh boxes as the literals need, each walking steps 1 and 2 again, and name the box on each row of the table: for instance one for steps 1, 2, 4 and 7, one for 3, one for 5 and 6. Replacing a literal with more scripts in `model_document` is also allowed, where it spares a box; keep what's there, since the walks and `chat-draft`'s recording read it.
- **`work/widget`, not `work/hello`:** "Land a local status change." names a repository a person's fresh path doesn't have, whose README starts `# widget`. **Default, step 3:** add scripts to `model_document` for a change on `work/hello`, an ask and its cued coder, as the `work/widget` ones are, and walk that. If an empty repository can't take a change, that's a finding of its own (what a person sees is the point); then connect the fake forge's `work/widget` by `temper ctl` as `walk/status.rs` does, adopt it, and say so on the row.
- **Home's brief waits for time:** a new sitting is 30 minutes away, with something landed and no brief within the hour (`temper/docs/design/domain/forge.md`, 5.6 and 5.7; `SITTING` in `temper/crates/temper-engine-domain-forge/src/status.rs`). **Default:** after step 3's landing close the page, leave that box running and go on to other boxes, then reopen Home after 31 minutes by the clock. Once, at 1440; at 420 use the same box's state at 420 and say so.
- **What the fake can't reach:** a held task of temper's own and a proposal have no script. **Default:** try `temper ctl` and the configuring chat for a real one (a stop places a hold: `temper/docs/help/hold.md`) and otherwise show the gallery's state, `heavy cargo run -p temper-client-gallery -- OUT`: `needs-held`, `needs-funds`, `needs-proposal`, `needs-later`, `needs-reply-open`, `needs-saw-open`, `chat-held-task`, `configure-chat-proposal` (`temper/tools/gallery/names.txt`). Adding a script is allowed where it's small. Say on the row how the state was shown. Nothing is skipped silently.
- **Configure temper's look request** is `configure-chat-draft`, and its pages are `git show 727f3523ee` (`temper/docs/design/client/journeys/configure-chat/`, its README and overlay), not landed as files: compare against them too. Its drawing and figure wait on `skein/docs/design/draft/diagrams.md`: not a finding.

## The screenshots

- **Under `~/src/fabrig/ui-final-walk/<your task>/`:** `NN-state-WIDTH-mock.png`, `-client.png` and a side-by-side `-pair.png`, NN the step.
- **`index.html` beside them:** one line per state, at both widths, linking the three, with its verdict, finding and the state's way (box, ctl, gallery). No build tool, no outside address.

## The table

`report.md`, one row per state at each width:

```
| step | page | state | width | verdict | why | box or way | pair |
```

- **verdict:** `matches`; `differs`, its why a decision (`D4`, `Ruling 9`, `By page, Home`) or a finding (`W7`); or `unreachable`, its why what stops it and how the state was shown instead.
- **Above the table:** the commit walked, the builder, the folder, how many boxes and which steps each walked, the counts of each verdict.
- **Below it:** whatever a table can't say: a flow that felt wrong, a word that reads badly though the mock has it, a listed decision the walk made you doubt. These stay as notes for `fabrig/ui`.
