# ui-model-line

A chat says the model and effort it really uses. The request: `inputs/request.md`.

## The request's first question

Read from the code, not run.

**Does a run's record keep both?** Two records answer differently:

- **jig's task record: no.** `TaskRecord` holds the task's `llm`, which is its choice and may be none, and `kept_account`. A run's checked policy goes to the worker in `RunCharter` and isn't kept: `RunProof` and the fleet's `Assignment` don't hold it. `task.llm` in the client's document is `document_llm`, what the *next* run would take, so it can't answer this either.
- **smith's transcript: yes, per turn.** `smith/docs/design/protocol/transcript.md`, section 2: "the model its completion was made with and the effort it asked for". It is `model: text 128` and `effort: option text 32` in `v6`.

**Is the client given them?** The bytes, yes, and it throws them away: a turn crosses the wire whole (`w::TurnBytes::Carried`), and `translate` in `jig/crates/jig-protocol-transcript/src/lib.rs` keeps the messages, usage and spend and drops `model()` and `effort()`.

So carrying them is this plan's, and it takes a decode, not a stored format.

## Decided here

- **From the turn, not a new stored field.** A field on `TaskRecord` set at the claim was the alternative, and the request allows it. It was rejected for three reasons:
  - A field on the client's `Task` changes the wire, which makes the journey recordings stale, since they hold exchange bytes (`temper/docs/design/client/journeys.md`, 8). Re-recording takes a browser, which this plan's limits rule out.
  - It lands on `tasks-v9`, `client-v19`, their generated code and goldens, which `ui-model-next-turn` and `model-catalogue-2` are regenerating.
  - It would say what a run was *given*. A turn says what it *used*, which is what the user asked for, and holds through any policy fallback.
- **What that costs,** in `jig/docs/design/client/domain/client.md`, section 18:
  - A run that committed no turn names no model, and such a chat reads as one that hasn't run.
  - A tail with no turn in its first page reads the same.
  - Pages without a thread can't use it. None needs to.
- **The number alone until the tail is read,** so that a closed chat never names a default and then takes it back.
- **The same line in both shells.** The terminal's closed chat prints "T2 · GPT-6 Astra · high" although its heading holds the number: the request says the same words, and it takes no shell branch.
- **A new word is the old one.** `Word::BylineTask` comes back as it stood before `50e7171ce1`.
- **A turn's effort isn't filled in.** A turn that asked for none shows none, since its effort isn't known. A *choice* that names none takes the offer's default, which is what its run will ask for.
- **A goal's page draws no model line,** so the request's rule has nothing to change there. `temper/docs/design/client/README.md`, 3.5 says elsewhere that a goal's page shows its model, effort and account: that gap was there before this plan and is left alone.
- **A phone's bar hides the byline** today, the number included, and still does. The request asks for the bar's form and doesn't name the phone; 3.5 says so.

## Parts and tasks

One part, `model-line`, one slice, no walk.

| Task | Tree | Needs | Makes true |
|---|---|---|---|
| `turn-llm` | jig, with temper's literal lines | none | `Turn.llm`, `Thread.latest_llm`, `Thread::caught_up` |
| `model-line` | temper | `turn-llm` | The closed chat's bar, the closed agent's taskbar, the composer's line, effort, the terminal, and the meeting story |

**Critical path:** `turn-llm`, then `model-line`, about 3 implementer hours. The story that meets both sides is an existing one, extended by a line: `a_new_chat_lands_its_delegated_change_and_closes_in_your_chats`, in `temper-client-world`, takes real smith bytes through the real decode to the closed chat's bar, in the text shell.

## temper/migration's seam

**This plan writes no file in `temper/crates/temper-engine-domain`,** and no store schema. `tasks::TaskRecord` is unchanged, so `row_bound` in `temper/crates/temper-engine-domain/src/limits.rs`, which prices it by `size_of`, is too. There is nothing to meet there.

If section 18's open question is ever taken, the seam is `jig-domain-tasks`' `TaskDefinition` beside `kept_account`, its codec in `jig/crates/jig-protocol-store/src/codec/tasks/record.rs`, and that `row_bound`, which grows if the field holds names and not indices.

## Overlaps

Whichever lands second catches up; none of these is waited for.

- **`ui-model-next-turn`:** no shared file, meeting at catch-up as the request says. Its rule makes the line truer: a new choice starts a new run at the next message, whose first turn names it.
- **`look-round-13/look-proof` and `ui-proposal-2`:** `temper/docs/design/client/README.md` and `README.status.md`. This plan's edits are in 3.5's closed-line bullet and a new one after it, 5.5's *MODEL* and *Unsettled*, and 5.9's taskbar.
- **`limits-fixtures/view-words`, `ui-proposal-2/card-and-latest`, `look-round-13`'s `forge-cards`:** `temper-client-view/src/words.rs`, four lines for `BylineTask`. `limits-fixtures` also moves `temper-client-view`'s test fixtures, where `turn-llm` adds one line each.
- **`model-catalogue-2`:** `temper-client-view/src/choice.rs` (`provider_line`'s visibility and an `Address::Models` arm), far from this plan's lines; `model-act` changes `jig-client-domain/src/contract/llm.rs`, which this plan only reads. It also adds fields to `MeModel`, which `model-line` reads by name.
- **Prints and goldens:** the terminal's goldens and the journeys' prints, which many plans change. Regenerate on a catch-up.

The early work keeps off these where it can: `turn-llm` touches no file the plans beside it own, but for single lines in test fixtures.
