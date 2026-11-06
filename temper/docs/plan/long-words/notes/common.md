# long-words: the rules every task names

The request is `temper/docs/plan/long-words/inputs/request.md`: read it, it is short. `temper/docs/plan/long-words/notes/contracts.md` holds the shapes that cross tasks: they bind.

## Departures from the implementer's text

- **Status lines:** the design for this plan is written, in the sections your spec cites. You change no design text, but your task removes the **Not yet** line or clause your spec names from its status file, once its code holds.
- **Failing test first,** in the lowest tier that shows the rule: run it red, and your report says what it showed. If it passes before your change, the request's chain is wrong there: say so in the report and go on.
- **A bound that moves is named:** your report names each client protocol or wire bound your task moves, with its value before and after, for temper/ui.

## Not this plan

- **How a conversation is read:** `chat-reading-c`'s, as this trunk stands on it: no "Older" on a chat, "Older" on an agent's page, `Word::LoadingEarlierFailed`, a window by size. A read at the foot says nothing, but what `temper/docs/design/client/README.md`, 5.15 now adds.
- **No new browser story:** the browser tier is over its budget, and budgets and timeouts are the user's (`skein/docs/foundation/testing-strategy.md`, 8.1). The proof by hand is after landing.
- **`docs/help/`:** nothing here changes a limit it names.
- **Stored formats:** none moves. People's `AskSay.words` and `AskAnswerQuestion.words` stay 8,192.

## Beside this plan

Other drafts are changing `temper-client-domain`'s `pages/chat.rs`, `pages/task.rs`, `pages/chats.rs` and `pages.rs`, and `temper-client-view`'s `pages/chat.rs`, `pages/agent.rs`, `pages/goal.rs` and `words.rs`. Keep hunks small, don't reformat, and don't move code you don't need to change. If your merge meets another draft in a file your spec names, keep their change, and say so in your report. A regenerated file a merge makes stale is regenerated, never edited by hand.

**Regenerating the client wire:** `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p jig-store-codecs-world --test drift`, which holds `codec_families_match_their_schemas_and_goldens` (`jig/tests/store-codecs/tests/suite/drift.rs`, family `client`). A diff there beyond your task's schema lines is a red, not stale.

## Limits

No budget or timeout is raised. A limit this plan changes is one your spec names. A heap ceiling that moves is named in your report, with its measurement before and after.
